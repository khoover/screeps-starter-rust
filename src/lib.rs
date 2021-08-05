use wasm_bindgen::{JsCast, prelude::*};
use web_sys::console;

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (console::log_1(&format_args!($($t)*).to_string().into()))
}

use std::time::Duration;
use std::convert::{TryFrom, TryInto};

#[inline]
fn to_u64(bytes: &[u8]) -> u64 {
    u64::from_ne_bytes(bytes[..8].try_into().unwrap())
}

#[inline]
fn write_u64(num: u64, buf: &mut [u8]) {
    buf[..8].copy_from_slice(&u64::to_ne_bytes(num));
}

const LOW_FIFTEEN_BITS: u64 = 0x7FFF;
const LOW_FOUR_BITS: u64 = 0xF;
const INPUT_BLOCK_SIZE: usize = 120;
const OUTPUT_BLOCK_SIZE: usize = 64;

#[inline]
fn write_byte_chunk(bytes: &[u8], extras: &mut u64, output_chunk: &mut [u16]) {
    let byte_u64 = to_u64(bytes);
    *extras = (*extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

    output_chunk[0] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[1] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[2] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[3] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;
}

// borrowed parts from the base64 encoder
#[inline]
fn alt_bytestring_to_utf15(bytes: &[u8]) -> Vec<u16> {
    if bytes.is_empty() { return Vec::new(); }

    // each block is 15 u64s, so 15*8=120 u8s
    let (encoded_len, has_remainder): (usize, bool) = {
        let len = bytes.len();
        let remainder = len % INPUT_BLOCK_SIZE;
        let num_blocks = len / INPUT_BLOCK_SIZE + (if remainder != 0 { 1 } else { 0 });
        (OUTPUT_BLOCK_SIZE*num_blocks + 1, remainder != 0)
    };
    let mut output: Vec<u16> = vec![0; encoded_len];
    if has_remainder { output[0] = 0x0031; } else { output[0] = 0x0030; }
    let mut output_index: usize = 1;
    let mut input_index: usize = 0;
    let last_fast_index = bytes.len().saturating_sub(120);
    
    if last_fast_index > 0 {
        while input_index <= last_fast_index {
            let bytes_chunk = &bytes[input_index..input_index + INPUT_BLOCK_SIZE];
            let output_chunk = &mut output[output_index..output_index + OUTPUT_BLOCK_SIZE];
            let mut extras: u64 = 0;

            for (bytes_index, output_chunk_index) in (0..120).step_by(8).zip((0..60).step_by(4)) {
                write_byte_chunk(&bytes_chunk[bytes_index..], &mut extras, &mut output_chunk[output_chunk_index..]);
            }
            /*let byte_u64 = to_u64(&bytes_chunk[0..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[0] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[1] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[2] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[3] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[8..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[4] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[5] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[6] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[7] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[16..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[8] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[9] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[10] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[11] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[24..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[12] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[13] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[14] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[15] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[32..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[16] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[17] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[18] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[19] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[40..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[20] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[21] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[22] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[23] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[48..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[24] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[25] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[26] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[27] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[56..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[28] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[29] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[30] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[31] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[64..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[32] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[33] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[34] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[35] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[72..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[36] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[37] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[38] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[39] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[80..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[40] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[41] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[42] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[43] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[88..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[44] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[45] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[46] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[47] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[96..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[48] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[49] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[50] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[51] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[104..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

            output_chunk[52] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[53] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[54] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[55] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

            let byte_u64 = to_u64(&bytes_chunk[112..]);
            extras = (extras | (byte_u64 & LOW_FOUR_BITS));

            output_chunk[56] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[57] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[58] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[59] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;*/

            output_chunk[60] = ((extras >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[61] = ((extras >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[62] = ((extras >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[63] = ((extras >> 49) & LOW_FIFTEEN_BITS) as u16;

            input_index += INPUT_BLOCK_SIZE;
            output_index += OUTPUT_BLOCK_SIZE;
        }
    }

    if has_remainder {
        let mut bytes_chunk: [u8; 120] = [0; 120];
        let len = bytes.len();
        bytes_chunk[0] = (len - input_index) as u8;
        bytes_chunk[1..1 + len - input_index].copy_from_slice(&bytes[input_index..]);
        let bytes_chunk = bytes_chunk;
        let output_chunk = &mut output[output_index..output_index + 64];
        let mut extras: u64 = 0;

        let byte_u64 = to_u64(&bytes_chunk[0..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[0] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[1] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[2] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[3] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[8..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[4] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[5] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[6] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[7] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[16..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[8] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[9] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[10] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[11] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[24..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[12] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[13] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[14] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[15] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[32..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[16] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[17] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[18] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[19] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[40..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[20] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[21] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[22] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[23] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[48..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[24] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[25] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[26] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[27] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[56..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[28] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[29] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[30] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[31] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[64..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[32] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[33] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[34] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[35] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[72..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[36] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[37] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[38] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[39] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[80..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[40] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[41] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[42] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[43] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[88..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[44] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[45] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[46] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[47] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[96..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[48] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[49] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[50] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[51] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[104..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

        output_chunk[52] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[53] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[54] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[55] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        let byte_u64 = to_u64(&bytes_chunk[112..]);
        extras = (extras | (byte_u64 & LOW_FOUR_BITS));

        output_chunk[56] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[57] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[58] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[59] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;

        output_chunk[60] = ((extras) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[61] = ((extras >> 15) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[62] = ((extras >> 30) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[63] = ((extras >> 45) & LOW_FIFTEEN_BITS) as u16;
    }

    output
}

#[inline]
fn full_bytes_to_utf15(bytes: &[u8; 30], output: &mut [u16; 16]) {
    for i in 0 .. 15 {
        let temp = u16::from_ne_bytes([bytes[2*i], bytes[2*i+1]]);
        output[15] = (output[15] | (temp & 0x8000)) >> 1;
        output[i] = (temp & 0x7FFF) + 0x0030;
    }
    output[15] += 0x0030;
}

#[inline]
fn partial_bytes_to_utf15(bytes: &[u8], output: &mut [u16; 16]) {
    assert!(bytes.len() < 30);
    let mut full_bytes: [u8; 30] = [0; 30];
    full_bytes[0] = bytes.len() as u8;
    (full_bytes[1..1+bytes.len()]).copy_from_slice(bytes);
    full_bytes_to_utf15(&full_bytes, output)
}

#[inline]
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

#[wasm_bindgen]
pub fn bytestring_to_jsstring(bytes: &[u8]) -> JsString {
    let utf15 = bytestring_to_utf15(bytes);
    unsafe { utf16_to_jsstring(Uint16Array::view(utf15.as_slice()).as_ref()) }
}

#[wasm_bindgen]
pub fn alt_bytestring_to_jsstring(bytes: &[u8]) -> JsString {
    let utf15 = alt_bytestring_to_utf15(bytes);
    unsafe { utf16_to_jsstring(Uint16Array::view(utf15.as_slice()).as_ref()) }
}

#[wasm_bindgen(inline_js = "const fromCharCode = String.fromCharCode; export function utf16_to_jsstring(utf16) { let ret = ''; let i = 0; for (; i < utf16.length - 32; i += 32) { ret += fromCharCode(
    utf16[i+0], utf16[i+1], utf16[i+2], utf16[i+3], utf16[i+4], utf16[i+5], utf16[i+6], utf16[i+7], utf16[i+8], utf16[i+9],
    utf16[i+10], utf16[i+11], utf16[i+12], utf16[i+13], utf16[i+14], utf16[i+15], utf16[i+16], utf16[i+17], utf16[i+18], utf16[i+19],
    utf16[i+20], utf16[i+21], utf16[i+22], utf16[i+23], utf16[i+24], utf16[i+25], utf16[i+26], utf16[i+27], utf16[i+28], utf16[i+29],
    utf16[i+30], utf16[i+31]
); } if (i != utf16.length) ret += fromCharCode.apply(String, utf16.subarray(i, i+32)); return ret; }")]
extern "C" {
    fn utf16_to_jsstring(utf16: &Uint16Array) -> JsString;
}

fn utf15_to_full_bytes(utf15: &[u16; 16], output: &mut [u8; 30]) {
    let mut remainder = (utf15[15] - 0x0030) << 1;
    for i in (0..15).rev() {
        let temp = u16::to_ne_bytes((utf15[i] - 0x0030) | (remainder & 0x8000));
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

#[inline]
fn decode_block(input: &[u16], remainder: u64, output: &mut [u8]) {
    let mut accum = remainder;
    
    accum |= (input[0] as u64) << 4;
    accum |= (input[1] as u64) << 19;
    accum |= (input[2] as u64) << 34;
    accum |= (input[3] as u64) << 49;

    write_u64(accum, output);
}

#[inline]
fn decode_extra(input: &[u16]) -> u64 {
    let mut accum: u64 = input[0] as u64;
    accum |= (input[1] as u64) << 15;
    accum |= (input[2] as u64) << 30;
    accum |= (input[3] as u64) << 45;
    accum
}

#[wasm_bindgen]
pub fn alt_jsstring_to_bytestring(utf15: &JsString) -> Vec<u8> {
    if utf15.length() == 0 { return Vec::new(); }

    let mut utf15_iter = utf15.iter();
    let first = utf15_iter.next().unwrap();
    assert!(first == 0x0030 || first == 0x0031);
    let has_remainder = first == 0x0031;
    let utf15_vec: Vec<u16> = utf15_iter.collect();
    assert_eq!(utf15_vec.iter().max().unwrap() & 0x8000, 0);
    assert_eq!(utf15_vec.len() % OUTPUT_BLOCK_SIZE, 0);

    let (fast_blocks, remainder_size): (usize, usize) = if has_remainder {
        let mut buf: [u8; 8] = [0; 8];
        let remainder_start = utf15_vec.len() - OUTPUT_BLOCK_SIZE;
        let extra = (utf15_vec[remainder_start + 63] >> 11) as u64 & LOW_FOUR_BITS;
        decode_block(&utf15_vec[remainder_start..], extra, &mut buf);
        (utf15_vec.len() / OUTPUT_BLOCK_SIZE - 1, buf[0] as usize)
    } else {
        (utf15_vec.len() / OUTPUT_BLOCK_SIZE, 0)
    };
    let mut output: Vec<u8> = vec![0; INPUT_BLOCK_SIZE * fast_blocks + remainder_size];
    let mut utf15_index: usize = 0;
    let mut output_index: usize = 0;
    for _ in 0 .. fast_blocks {
        let utf15_chunk = &utf15_vec[utf15_index..utf15_index + OUTPUT_BLOCK_SIZE];
        let output_chunk = &mut output[output_index..output_index + INPUT_BLOCK_SIZE];
        let extra = decode_extra(&utf15_chunk[60..]);

        decode_block(utf15_chunk, (extra >> 56) & LOW_FOUR_BITS, output_chunk);

        decode_block(&utf15_chunk[4..], (extra >> 52) & LOW_FOUR_BITS, &mut output_chunk[8..]);

        decode_block(&utf15_chunk[8..], (extra >> 48) & LOW_FOUR_BITS, &mut output_chunk[16..]);

        decode_block(&utf15_chunk[12..], (extra >> 44) & LOW_FOUR_BITS, &mut output_chunk[24..]);

        decode_block(&utf15_chunk[16..], (extra >> 40) & LOW_FOUR_BITS, &mut output_chunk[32..]);

        decode_block(&utf15_chunk[20..], (extra >> 36) & LOW_FOUR_BITS, &mut output_chunk[40..]);

        decode_block(&utf15_chunk[24..], (extra >> 32) & LOW_FOUR_BITS, &mut output_chunk[48..]);

        decode_block(&utf15_chunk[28..], (extra >> 28) & LOW_FOUR_BITS, &mut output_chunk[56..]);

        decode_block(&utf15_chunk[32..], (extra >> 24) & LOW_FOUR_BITS, &mut output_chunk[64..]);

        decode_block(&utf15_chunk[36..], (extra >> 20) & LOW_FOUR_BITS, &mut output_chunk[72..]);

        decode_block(&utf15_chunk[40..], (extra >> 16) & LOW_FOUR_BITS, &mut output_chunk[80..]);

        decode_block(&utf15_chunk[44..], (extra >> 12) & LOW_FOUR_BITS, &mut output_chunk[88..]);

        decode_block(&utf15_chunk[48..], (extra >> 8) & LOW_FOUR_BITS, &mut output_chunk[96..]);

        decode_block(&utf15_chunk[52..], (extra >> 4) & LOW_FOUR_BITS, &mut output_chunk[104..]);

        decode_block(&utf15_chunk[56..], extra & LOW_FOUR_BITS, &mut output_chunk[112..]);

        output_index += INPUT_BLOCK_SIZE;
        utf15_index += OUTPUT_BLOCK_SIZE;
    }
    if has_remainder {
        let utf15_chunk = &utf15_vec[utf15_index..];
        let output_chunk = &mut output[output_index..];
        let mut tmp: [u8; 8] = [0; 8];
        let extra = decode_extra(&utf15_chunk[60..]);

        decode_block(utf15_chunk, (extra >> 56) & LOW_FOUR_BITS, &mut tmp);
        output_chunk[..7.min(remainder_size)].copy_from_slice(&tmp[1..8.min(remainder_size+1)]);
        if remainder_size <= 7 { return output; }

        let output_chunk = &mut output_chunk[7..];
        let utf15_chunk = &utf15_chunk[4..];
        let semi_fast_blocks = remainder_size.saturating_sub(7) / 8;
        let annoying_bytes: usize = remainder_size.saturating_sub(7) % 8;
        if semi_fast_blocks > 0 {
            let mut extra_shift: u8 = 52;
            let mut inner_utf_index: usize = 0;
            let mut inner_output_index: usize = 0;

            for _ in 0 .. semi_fast_blocks {
                decode_block(&utf15_chunk[inner_utf_index..], (extra >> extra_shift) & LOW_FOUR_BITS, &mut output_chunk[inner_output_index..]);
                extra_shift -= 4;
                inner_utf_index += 4;
                inner_output_index += 8;
            }

            decode_block(&utf15_chunk[inner_utf_index..], (extra >> extra_shift) & LOW_FOUR_BITS, &mut tmp);
            output_chunk[inner_output_index..inner_output_index + annoying_bytes].copy_from_slice(&tmp[..annoying_bytes]);
        } else if annoying_bytes > 0 {
            decode_block(utf15_chunk, (extra >> 52) & LOW_FOUR_BITS, &mut tmp);
            output_chunk[..annoying_bytes].copy_from_slice(&tmp[..annoying_bytes]);
        }
    }

    output
}

use js_sys::{Date, JsString, Uint16Array};
use getrandom::getrandom;
use base64::{encode, decode, encode_config_slice, STANDARD};

#[wasm_bindgen]
pub fn make_call() {
    const RUNS: usize = 100;
    const BYTES: usize = 3*1024*1024;

    const NINE_NINE_INDEX: usize = (RUNS / 100) * 99 - 1;

    console_error_panic_hook::set_once();
    let mut bytes = vec![0; BYTES];
    let mut ser_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut de_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut b64_ser_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut b64_de_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut b64_slice_ser_times: Vec<Duration> = Vec::with_capacity(RUNS);
    for _ in 1 ..= RUNS {
        getrandom(&mut bytes).expect("Didn't get the bytes.");
        let start = Date::now();
        let s = alt_bytestring_to_jsstring(&bytes);
        let mid = Date::now();
        let back = alt_jsstring_to_bytestring(&s);
        let end = Date::now();
        ser_times.push(Duration::from_secs_f64((mid - start)/1000f64));
        de_times.push(Duration::from_secs_f64((end - mid) / 1000f64));
        assert_eq!(back, bytes);

        let start = Date::now();
        let s2: JsString = encode(&bytes).into();
        let mid = Date::now();
        let back = decode(String::from(s2.clone())).expect("Should have been good bytes");
        let end = Date::now();
        b64_ser_times.push(Duration::from_secs_f64((mid - start)/1000f64));
        b64_de_times.push(Duration::from_secs_f64((end - mid) / 1000f64));
        assert_eq!(back, bytes);

        let mut dest = vec![0; 8*1024*1024];
        let start = Date::now();
        let s2: JsString = { let len = encode_config_slice(&bytes, STANDARD, &mut dest); dest.truncate(len); unsafe { String::from_utf8_unchecked(dest) }.into() };
        let mid = Date::now();
        let back = decode(String::from(s2.clone())).expect("Should have been good bytes");
        let end = Date::now();
        b64_slice_ser_times.push(Duration::from_secs_f64((mid - start)/1000f64));
        assert_eq!(back, bytes);
    }
    ser_times.sort();
    let (ser_max, ser_min, ser_99) = (ser_times[RUNS-1], ser_times[0], ser_times[NINE_NINE_INDEX]);
    let ser_avg: Duration = ser_times.iter().sum::<Duration>() / (RUNS as u32);
    de_times.sort();
    let (de_max, de_min, de_99) = (de_times[RUNS-1], de_times[0], de_times[NINE_NINE_INDEX]);
    let de_avg: Duration = de_times.iter().sum::<Duration>() / (RUNS as u32);
    console_log!("ALT SERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", ser_max, ser_min, ser_avg, ser_99);
    console_log!("ALT DESERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", de_max, de_min, de_avg, de_99);
    b64_ser_times.sort();
    let (b64_ser_max, b64_ser_min, b64_ser_99) = (b64_ser_times[RUNS-1], b64_ser_times[0], b64_ser_times[NINE_NINE_INDEX]);
    let b64_ser_avg: Duration = b64_ser_times.iter().sum::<Duration>() / (RUNS as u32);
    b64_de_times.sort();
    let (b64_de_max, b64_de_min, b64_de_99) = (b64_de_times[RUNS-1], b64_de_times[0], b64_de_times[NINE_NINE_INDEX]);
    let b64_de_avg: Duration = b64_de_times.iter().sum::<Duration>() / (RUNS as u32);
    console_log!("BASE64 SERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", b64_ser_max, b64_ser_min, b64_ser_avg, b64_ser_99);
    console_log!("BASE64 DESERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", b64_de_max, b64_de_min, b64_de_avg, b64_de_99);
    b64_slice_ser_times.sort();
    let (b64_slice_ser_max, b64_slice_ser_min, b64_slice_ser_99) = (b64_slice_ser_times[RUNS-1], b64_slice_ser_times[0], b64_slice_ser_times[NINE_NINE_INDEX]);
    let b64_slice_ser_avg: Duration = b64_slice_ser_times.iter().sum::<Duration>() / (RUNS as u32);
    console_log!("BASE64 SLICE SERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", b64_slice_ser_max, b64_slice_ser_min, b64_slice_ser_avg, b64_slice_ser_99);
}
