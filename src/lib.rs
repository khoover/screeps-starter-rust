use wasm_bindgen::{JsCast, prelude::*};
use web_sys::console;

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (console::log_1(&format_args!($($t)*).to_string().into()))
}

use std::time::Duration;
use std::convert::{TryFrom, TryInto};

fn full_bytes_to_utf15(bytes: &[u8; 30], output: &mut [u16; 16]) {
    for i in 0 .. 15 {
        let temp = u16::from_be_bytes([bytes[2*i], bytes[2*i+1]]);
        output[15] = (output[15] | (temp & 0x8000)) >> 1;
        output[i] = (temp & 0x7FFF) + 0x0030;
    }
    output[15] += 0x0030;
}

fn partial_bytes_to_utf15(bytes: &[u8], output: &mut [u16; 16]) {
    assert!(bytes.len() < 30);
    let mut full_bytes: [u8; 30] = [0; 30];
    full_bytes[0] = bytes.len() as u8;
    (full_bytes[1..1+bytes.len()]).copy_from_slice(bytes);
    full_bytes_to_utf15(&full_bytes, output)
}

fn bytestring_to_utf15(bytes: &[u8]) -> Vec<u16> {
    if bytes.is_empty() { return Vec::new(); }

    let blocks = bytes.chunks_exact(30);
    let remainder = blocks.remainder();
    let mut output: Vec<u16> = Vec::with_capacity(1 + 16 * (blocks.len() + (if remainder.len() == 0 { 0 } else { 1 })));
    output.resize(output.capacity(), 0);

    if remainder.len() == 0 {
        output[0] = 0x0030;
    } else {
        output[0] = 0x0031;
        let len = output.len();
        partial_bytes_to_utf15(remainder, (&mut output[len - 16..]).try_into().unwrap());
    }
    let mut unfilled = &mut output[1..];
    for block in blocks {
        full_bytes_to_utf15(block.try_into().unwrap(), (&mut unfilled[0..16]).try_into().unwrap());
        unfilled = &mut unfilled[16..];
    }
    output
}

fn bytestring_to_jsstring(bytes: &[u8], decoder: &MyTextDecoder) -> JsString {
    let utf15 = bytestring_to_utf15(bytes);
    unsafe { decoder.decode_from_u16(Uint16Array::view(utf15.as_slice()).as_ref()) }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "TextDecoder")]
    type MyTextDecoder;

    #[wasm_bindgen(constructor, js_class = "TextDecoder")]
    fn new_with_label(s: &str) -> MyTextDecoder;

    #[wasm_bindgen(method, final, js_name = "decode", js_class = "TextDecoder")]
    fn decode_from_u16(this: &MyTextDecoder, array: &Uint16Array) -> JsString;
}

fn utf15_to_full_bytes(utf15: &[u16; 16], output: &mut [u8; 30]) {
    let mut remainder = (utf15[15] - 0x0030) << 1;
    for i in (0..15).rev() {
        let temp = u16::to_be_bytes((utf15[i] - 0x0030) | (remainder & 0x8000));
        remainder = remainder << 1;
        output[2*i] = temp[0];
        output[2*i+1] = temp[1];
    }
}

fn utf15_to_partial_bytes(utf15: &[u16; 16]) -> Vec<u8> {
    let mut bytes: [u8; 30] = [0; 30];
    utf15_to_full_bytes(utf15, &mut bytes);
    let len = bytes[0] as usize;
    bytes[1..1+len].to_vec()
}

fn jsstring_to_bytestring(utf15: &JsString) -> Vec<u8> {
    if utf15.length() == 0 { return Vec::new(); }

    let mut utf15_ints = utf15.iter();
    let first = utf15_ints.next().unwrap();
    assert!(first == 0x0030 || first == 0x0031);
    assert_eq!(utf15_ints.len() & 0xF, 0);

    let mut utf15_block: [u16; 16] = [0; 16];
    let mut output: Vec<u8> = Vec::with_capacity((utf15_ints.len() / 16) * 30);
    output.resize(output.capacity(), 0);
    let mut unfilled: &mut [u8] = &mut output[0..];
    if first & 0x0001 == 0 {
        // There's no remainder.
        while utf15_ints.len() != 0 {
            for i in 0..16 {
                utf15_block[i] = utf15_ints.next().unwrap();
            }
            utf15_to_full_bytes(&utf15_block, (&mut unfilled[0..30]).try_into().unwrap());
            unfilled = &mut unfilled[30..];
        }
    } else {
        // There is a remainder.
        let mut remainder: [u16; 16] = [0; 16];
        for i in (0..16).rev() {
            remainder[i] = utf15_ints.next_back().unwrap();
        }
        while utf15_ints.len() != 0 {
            for i in 0..16 {
                utf15_block[i] = utf15_ints.next().unwrap();
            }
            utf15_to_full_bytes(&utf15_block, (&mut unfilled[0..30]).try_into().unwrap());
            unfilled = &mut unfilled[30..];
        }
        drop(unfilled);
        output.truncate(output.len() - 30);
        output.append(&mut utf15_to_partial_bytes(&remainder));
    }
    output
}

use js_sys::{Date, JsString, Uint16Array};
use getrandom::getrandom;

#[wasm_bindgen]
pub fn make_call() {
    console_error_panic_hook::set_once();
    let mut bytes = vec![0; 3*1024*1024];
    let decoder: MyTextDecoder = MyTextDecoder::new_with_label(&"utf-16");
    for _ in 1 ..= 100 {
        getrandom(&mut bytes).expect("Didn't get the bytes.");
        let start = Date::now();
        let s: JsString = bytestring_to_jsstring(&bytes, &decoder);
        let mid = Date::now();
        let back = jsstring_to_bytestring(&s);
        let end = Date::now();
        console_log!("serialize: {:?}", Duration::from_secs_f64((mid - start)/1000f64));
        console_log!("deserialize: {:?}", Duration::from_secs_f64((end - mid) / 1000f64));
        console_log!("{} bytes vs {} js length", bytes.len(), s.length());
        assert_eq!(back, bytes);
    }
}
