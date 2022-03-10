#![feature(test)]
#![feature(bench_black_box)]
#![feature(slice_as_chunks)]

#[macro_use]
extern crate lazy_static;

use wasm_bindgen::prelude::*;
use web_sys::console;
use js_sys::JsString;
use std::convert::{TryInto, TryFrom};

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (console::log_1(&format_args!($($t)*).to_string().into()))
}

#[cfg_attr(not(feature = "no_web"), wasm_bindgen(inline_js = "const fromCharCode = String.fromCharCode; export function utf16_to_jsstring(utf16) { let ret = ''; let i = 0; for (; i < utf16.length - 32; i += 32) { ret += fromCharCode(
    utf16[i+0], utf16[i+1], utf16[i+2], utf16[i+3], utf16[i+4], utf16[i+5], utf16[i+6], utf16[i+7], utf16[i+8], utf16[i+9],
    utf16[i+10], utf16[i+11], utf16[i+12], utf16[i+13], utf16[i+14], utf16[i+15], utf16[i+16], utf16[i+17], utf16[i+18], utf16[i+19],
    utf16[i+20], utf16[i+21], utf16[i+22], utf16[i+23], utf16[i+24], utf16[i+25], utf16[i+26], utf16[i+27], utf16[i+28], utf16[i+29],
    utf16[i+30], utf16[i+31]
); } if (i != utf16.length) ret += fromCharCode.apply(String, utf16.subarray(i, i+32)); return ret; }

export function jsstring_to_utf16(s, buf) {
    if (buf.length < s.length) throw new Error('Insufficient buffer space to store string.');
    for (let i = 0; i < buf.length; ++i) {
        buf[i] = s.charCodeAt(i);
    }
}"))]
#[cfg(not(feature = "no_web"))]
extern "C" {
    fn utf16_to_jsstring(utf16: &[u16]) -> JsString;

    #[wasm_bindgen(catch)]
    fn jsstring_to_utf16(s: &JsString, utf16: &mut [u16]) -> Result<(), JsValue>;
}

#[cfg(feature = "no_web")]
use js_sys::{Function, Uint16Array};
#[cfg(feature = "no_web")]
use wasm_bindgen::JsCast;

#[cfg(feature = "no_web")]
fn utf16_to_jsstring(utf16: &[u16]) -> JsString {
    thread_local!(static INNER_FUNC: Function = Function::new_with_args(&"utf16", &"const fromCharCode = String.fromCharCode; let ret = ''; let i = 0; for (; i < utf16.length - 32; i += 32) { ret += fromCharCode(
    utf16[i+0], utf16[i+1], utf16[i+2], utf16[i+3], utf16[i+4], utf16[i+5], utf16[i+6], utf16[i+7], utf16[i+8], utf16[i+9],
    utf16[i+10], utf16[i+11], utf16[i+12], utf16[i+13], utf16[i+14], utf16[i+15], utf16[i+16], utf16[i+17], utf16[i+18], utf16[i+19],
    utf16[i+20], utf16[i+21], utf16[i+22], utf16[i+23], utf16[i+24], utf16[i+25], utf16[i+26], utf16[i+27], utf16[i+28], utf16[i+29],
    utf16[i+30], utf16[i+31]
); } if (i != utf16.length) ret += fromCharCode.apply(String, utf16.subarray(i, i+32)); return ret;"));
    INNER_FUNC.with(move |f| {
        let buf = unsafe { Uint16Array::view(utf16) };
        f.call1(&JsValue::NULL, &buf).expect("Unexpected error while copying UTF-15 to JsString.").unchecked_into()
    })
}

#[cfg(feature = "no_web")]
fn jsstring_to_utf16(s: &JsString, utf16: &mut [u16]) -> Result<(), JsValue> {
    thread_local!(static INNER_FUNC: Function = Function::new_with_args(&"s, buf", &"if (buf.length < s.length) throw new Error('Insufficient buffer space to store string.');
    for (let i = 0; i < buf.length; ++i) {
        buf[i] = s.charCodeAt(i);
    }"));
    INNER_FUNC.with(move |f| {
        let buf = unsafe { Uint16Array::view(utf16) };
        f.call2(&JsValue::NULL, s, &buf).map(|_| ())
    })
}

const LOW_FIFTEEN_BITS: u64 = 0x7FFF;
const LOW_FOUR_BITS: u64 = 0xF;
const INPUT_BLOCK_SIZE: usize = 120;
const OUTPUT_BLOCK_SIZE: usize = 64;

#[inline(always)]
fn to_u64(bytes: &[u8]) -> u64 {
    u64::from_ne_bytes(bytes[..8].try_into().unwrap())
}

#[inline(always)]
fn write_u64(num: u64, buf: &mut [u8]) {
    buf[..8].copy_from_slice(&u64::to_ne_bytes(num));
}

#[inline(always)]
fn write_byte_chunk(bytes: &[u8], extras: &mut u64, output_chunk: &mut [u16]) {
    let byte_u64 = to_u64(bytes);
    *extras = (*extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

    output_chunk[0] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[1] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[2] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[3] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;
}

// borrowed parts from the base64 encoder
fn bytestring_to_utf15(bytes: &[u8]) -> Vec<u16> {
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

            for (bytes_index, output_chunk_index) in (0..INPUT_BLOCK_SIZE).step_by(8).zip((0..OUTPUT_BLOCK_SIZE-4).step_by(4)) {
                write_byte_chunk(&bytes_chunk[bytes_index..], &mut extras, &mut output_chunk[output_chunk_index..]);
            }
            output_chunk[60] = ((extras >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[61] = ((extras >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[62] = ((extras >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[63] = ((extras >> 49) & LOW_FIFTEEN_BITS) as u16;

            input_index += INPUT_BLOCK_SIZE;
            output_index += OUTPUT_BLOCK_SIZE;
        }
    }

    if has_remainder {
        let mut bytes_chunk: [u8; INPUT_BLOCK_SIZE] = [0; INPUT_BLOCK_SIZE];
        let len = bytes.len();
        bytes_chunk[0] = (len - input_index) as u8;
        bytes_chunk[1..1 + len - input_index].copy_from_slice(&bytes[input_index..]);
        let bytes_chunk = bytes_chunk;
        let output_chunk = &mut output[output_index..output_index + OUTPUT_BLOCK_SIZE];
        let mut extras: u64 = 0;

        for (bytes_index, output_chunk_index) in (0..INPUT_BLOCK_SIZE).step_by(8).zip((0..OUTPUT_BLOCK_SIZE-4).step_by(4)) {
            write_byte_chunk(&bytes_chunk[bytes_index..], &mut extras, &mut output_chunk[output_chunk_index..]);
        }
        output_chunk[60] = ((extras >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[61] = ((extras >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[62] = ((extras >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[63] = ((extras >> 49) & LOW_FIFTEEN_BITS) as u16;
    }

    output
}

#[inline(always)]
fn faster_write_chunk(bytes: &[u8; 8], extras: u64, output_chunk: &mut [u16; 4]) -> u64 {
    let byte_u64 = u64::from_ne_bytes(*bytes);
    output_chunk[0] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[1] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[2] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[3] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;
    (extras | (byte_u64 & LOW_FOUR_BITS)) << 4
}

fn alt_bytestring_to_utf15(bytes: &[u8]) -> Vec<u16> {
    if bytes.is_empty() { return Vec::new(); }
            
    let (fast_input_blocks, remainder): (&[[u8; INPUT_BLOCK_SIZE]], &[u8]) = bytes.as_chunks();
    let has_remainder = !remainder.is_empty();
    let encoded_len = 1+OUTPUT_BLOCK_SIZE*(fast_input_blocks.len() + if has_remainder { 1 } else { 0 });
    
    let mut output: Vec<u16> = vec![0; encoded_len];
    if has_remainder { output[0] = 0x0031; } else { output[0] = 0x0030; }
    // SAFETY: output.len is 1 plus a multiple of OUTPUT_BLOCK_SIZE.
    let output_blocks: &mut [[u16; OUTPUT_BLOCK_SIZE]] = unsafe { output[1..].as_chunks_unchecked_mut() };
    
    let maybe_remainder: Option<[u8; INPUT_BLOCK_SIZE]> = if has_remainder {
        let mut fixed_remainder = [0u8; INPUT_BLOCK_SIZE];
        let len = remainder.len();                                                                                                                                                                                                                                                                                                       
        fixed_remainder[0] = remainder.len() as u8;
        fixed_remainder[1..1+len].copy_from_slice(remainder);
        Some(fixed_remainder)
    } else {
        None
    };
    
    output_blocks.iter_mut().zip(fast_input_blocks.iter().chain(&maybe_remainder)).for_each(|(output_block, input_block)| {
        //SAFETY: INPUT_BLOCK_SIZE is a multiple of 8.
        let input_groups: &[[u8; 8]] = unsafe { input_block.as_chunks_unchecked() };
        //SAFETY: OUTPUT_BLOCK_SIZE is a multiple of 4.
        let output_groups: &mut [[u16; 4]] = unsafe { output_block.as_chunks_unchecked_mut() };
        let extras = output_groups.iter_mut().zip(input_groups).fold(0u64, |extras, (output_group, input_group)| {
            faster_write_chunk(input_group, extras, output_group)
        });

        output_block[60] = ((extras >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_block[61] = ((extras >> 19) & LOW_FIFTEEN_BITS) as u16;                                                                                                                                                                                                                                                      
        output_block[62] = ((extras >> 34) & LOW_FIFTEEN_BITS) as u16;                                                                                                                                                                                                                                                               output_block[63] = ((extras >> 49) & LOW_FIFTEEN_BITS) as u16;
    });
    
    output
}

#[wasm_bindgen]
pub fn bytestring_to_jsstring(bytes: &[u8]) -> JsString {
    let utf15 = bytestring_to_utf15(bytes);
    utf16_to_jsstring(utf15.as_slice())
}

#[wasm_bindgen]
pub fn alt_bytestring_to_jsstring(bytes: &[u8]) -> JsString {
    let utf15 = alt_bytestring_to_utf15(bytes);
    utf16_to_jsstring(utf15.as_slice())
}

#[inline(always)]
fn decode_block(input: &[u16], remainder: u64, output: &mut [u8]) {
    let mut accum = remainder;

    accum |= (input[0] as u64) << 4;
    accum |= (input[1] as u64) << 19;
    accum |= (input[2] as u64) << 34;
    accum |= (input[3] as u64) << 49;

    write_u64(accum, output);
}

#[inline(always)]
fn decode_extra(input: &[u16]) -> u64 {
    let mut accum: u64 = input[0] as u64;
    accum |= (input[1] as u64) << 15;
    accum |= (input[2] as u64) << 30;
    accum |= (input[3] as u64) << 45;
    accum
}

#[wasm_bindgen]
pub fn jsstring_to_bytestring(utf15: &JsString) -> Vec<u8> {
    if utf15.length() == 0 { return Vec::new(); }
    assert_eq!(utf15.length() % OUTPUT_BLOCK_SIZE as u32, 1);

    let mut utf15_vec: Vec<u16> = vec![0; utf15.length().try_into().unwrap()];
    jsstring_to_utf16(&utf15, utf15_vec.as_mut_slice()).unwrap();
    let first = utf15_vec[0];
    assert!(first == 0x0030 || first == 0x0031);
    let utf15_vec = &utf15_vec[1..];
//    assert_eq!(utf15_vec.iter().max().unwrap() & 0x8000, 0);

    let has_remainder = first == 0x0031;
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

#[inline(always)]
fn alt_write_u64(num: u64, buf: &mut [u8; 8]) {
    let bytes = u64::to_ne_bytes(num);
    for i in 0..8 {
        buf[i] = bytes[i];
    }
}

#[inline(always)]
fn alt_decode_block(input: &[u16; 4], remainder: u64, output: &mut [u8; 8]) {
    let mut accum = remainder;

    accum |= (input[0] as u64) << 4;
    accum |= (input[1] as u64) << 19;
    accum |= (input[2] as u64) << 34;
    accum |= (input[3] as u64) << 49;

    alt_write_u64(accum, output);
}

#[inline(always)]
fn alt_decode_extra(input: &[u16; 4]) -> u64 {
    let mut accum: u64 = input[0] as u64;
    accum |= (input[1] as u64) << 15;
    accum |= (input[2] as u64) << 30;
    accum |= (input[3] as u64) << 45;
    accum
}

#[wasm_bindgen]
pub fn alt_jsstring_to_bytestring(utf15: &JsString) -> Vec<u8> {
    if utf15.length() == 0 { return Vec::new(); }
    assert_eq!(utf15.length() % OUTPUT_BLOCK_SIZE as u32, 1);

    // SAFETY: TryFrom<u32> for usize is always Ok.
    let mut utf15_vec: Vec<u16> = vec![0; unsafe { utf15.length().try_into().unwrap_unchecked() }];
    jsstring_to_utf16(&utf15, utf15_vec.as_mut_slice()).unwrap();
    let first = utf15_vec[0];
    assert!(first == 0x0030 || first == 0x0031);
    let utf15_vec = &utf15_vec[1..];
    debug_assert!(utf15_vec.iter().max().unwrap() & 0x8000 == 0);

    let has_remainder = first == 0x0031;
    let (fast_blocks, maybe_remainder, remainder_size): (&[[u16; OUTPUT_BLOCK_SIZE]], Option<&[u16; OUTPUT_BLOCK_SIZE]>, usize) = {
        // SAFETY: we removed one element of utf15_vec, and checked that the original length was a
        // multiple of OUTPUT_BLOCK_SIZE plus 1.
        let chunks: &[[u16; OUTPUT_BLOCK_SIZE]] = unsafe { utf15_vec.as_chunks_unchecked() };
        if has_remainder {
            let remainder = &chunks[chunks.len() - 1];
            let mut buf: [u8; 8] = [0; 8];
            let extra = (remainder[63] >> 11) as u64 & LOW_FOUR_BITS;
            // SAFETY: Immediate.
            alt_decode_block(unsafe { <&[u16; 4]>::try_from(&remainder[0..4]).unwrap_unchecked() }, extra, &mut buf);
            (&chunks[..chunks.len() - 1], Some(remainder), buf[0] as usize)
        } else {
            (chunks, None, 0)
        }
    };
    let mut output: Vec<u8> = vec![0; INPUT_BLOCK_SIZE * fast_blocks.len() + remainder_size];
    let (output_blocks, output_remainder): (&mut [[u8; INPUT_BLOCK_SIZE]], &mut[u8]) = output.as_chunks_mut();

    fast_blocks.iter().zip(output_blocks.iter_mut()).for_each(|(utf15_block, byte_block)| {
        // SAFETY: INPUT_BLOCK_SIZE % 8 == 0
        let byte_groups: &mut [[u8; 8]] = unsafe { byte_block.as_chunks_unchecked_mut() };
        // SAFETY: OUTPUT_BLOCK_SIZE % 4 == 0
        let utf15_groups: &[[u16; 4]] = unsafe { utf15_block.as_chunks_unchecked() };
        let mut utf15_iter = utf15_groups.iter();
        // SAFETY: utf15_groups is never empty;
        let extra = alt_decode_extra(unsafe { utf15_iter.next_back().unwrap_unchecked() });
        utf15_iter.zip(byte_groups.iter_mut()).rfold(extra, |extra, (utf15_group, byte_group)| {
            alt_decode_block(utf15_group, extra & LOW_FOUR_BITS, byte_group);
            extra >> 4
        });
    });

    if let Some(input_remainder) = maybe_remainder {
        // SAFETY: OUTPUT_BLOCK_SIZE % 4 == 0.
        let input_groups: &[[u16; 4]] = unsafe { input_remainder.as_chunks_unchecked() };
        let mut tmp: [u8; 8] = [0; 8];
        let extra = alt_decode_extra(&input_groups[input_groups.len() - 1]);

        alt_decode_block(&input_groups[0], (extra >> 56) & LOW_FOUR_BITS, &mut tmp);
        output_remainder[..7.min(remainder_size)].copy_from_slice(&tmp[1..8.min(remainder_size+1)]);
        if remainder_size <= 7 { return output; }

        let (semi_fast_bytes, annoying_bytes): (&mut [[u8; 8]], &mut [u8]) = output_remainder[7..].as_chunks_mut();
        let input_groups = &input_groups[1..];
        if !semi_fast_bytes.is_empty() {
            let extra_shift = input_groups.iter().zip(semi_fast_bytes.iter_mut()).fold(52u8, |extra_shift, (utf15_group, byte_group)| {
                alt_decode_block(utf15_group, (extra >> extra_shift) & LOW_FOUR_BITS, byte_group);
                extra_shift - 4
            });
            // SAFETY: We know output_remainder.len() < INPUT_BLOCK_SIZE, so in particular:
            // semi_fast_bytes.len() = (output_reminder.len() - 7) / 8 < INPUT_BLOCK_SIZE / 8 - 1 <
            // OUTPUT_BLOCK_SIZE / 4 = input_groups.len().
            let annoying_input_group = unsafe { input_groups.get_unchecked(semi_fast_bytes.len()) };
            alt_decode_block(annoying_input_group, (extra >> extra_shift) & LOW_FOUR_BITS, &mut tmp);
            for i in 0..annoying_bytes.len() {
                annoying_bytes[i] = tmp[i];
            }
        } else if !annoying_bytes.is_empty() {
            alt_decode_block(&input_groups[0], (extra >> 52) & LOW_FOUR_BITS, &mut tmp);
            for i in 0..annoying_bytes.len() {
                annoying_bytes[i] = tmp[i];
            }
        }
    }

    output
}

use std::{cell::RefCell, sync::RwLock, collections::HashMap};
thread_local! {
    static TL_CELL: RefCell<HashMap<u8, u8>> = RefCell::new(HashMap::new());
}

lazy_static! {
    static ref RWLOCK: RwLock<HashMap<u8, u8>> = RwLock::new(HashMap::new());
}

#[cfg_attr(not(feature = "no_web"), wasm_bindgen(inline_js = "function getRandomInt(min, max) {
    min = Math.ceil(min);
    max = Math.floor(max);
    return Math.floor(Math.random() * (max - min) + min); //The maximum is exclusive and the minimum is inclusive
}

export function fill_random(bytes) {
    let i = 0;
    for (; i < bytes.length - 4; i += 4) {
        const randU32 = getRandomInt(0, 0x100000000);
        bytes[i] = randU32 & 0xFF;
        bytes[i+1] = (randU32 >> 4) & 0xFF;
        bytes[i+2] = (randU32 >> 8) & 0xFF;
        bytes[i+3] = (randU32 >> 12) & 0xFF;
    }
    for (; i < bytes.length; ++i) {
        bytes[i] = getRandomInt(0, 256);
    }
}"))]
#[cfg(not(feature = "no_web"))]
extern "C" {
    fn fill_random(bytes: &mut [u8]);
}

#[cfg(feature = "no_web")]
fn fill_random(bytes: &mut [u8]) {
    thread_local!(static INNER_FUNC: Function = Function::new_with_args(&"bytes", &"let i = 0;
    for (; i < bytes.length - 4; i += 4) {
        const randU32 = getRandomInt(0, 0x100000000);
        bytes[i] = randU32 & 0xFF;
        bytes[i+1] = (randU32 >> 4) & 0xFF;
        bytes[i+2] = (randU32 >> 8) & 0xFF;
        bytes[i+3] = (randU32 >> 12) & 0xFF;
    }
    for (; i < bytes.length; ++i) {
        bytes[i] = getRandomInt(0, 256);
    }
}"));
    INNER_FUNC.with(move |f| {
        let buf = unsafe { Uint8Array::view(bytes) };
        f.call1(&JsValue::NULL, &buf).expect("Unexpected error while filling bytes.");
    });
}

fn replacement_getrandom(bytes: &mut [u8]) -> Result<(), getrandom::Error> {
    fill_random(bytes);
    Ok(())
}

use getrandom::register_custom_getrandom;

register_custom_getrandom!(replacement_getrandom);

#[wasm_bindgen]
pub fn make_call() {
    console_error_panic_hook::set_once();

    use getrandom::getrandom;
    use base64::{decode, encode_config_slice, STANDARD};
    use std::time::Duration;
    use js_sys::Date;

    const RUNS: usize = 1000;
    const NINE_NINE_INDEX: usize = (RUNS / 100) * 99 - 1;
    const BYTES: usize = 1024*1024;

    let start = Date::now();
    for _ in 1 ..= 100_000_000 {
        std::hint::black_box(TL_CELL.with(|refcell| { std::hint::black_box(refcell.borrow()); }));
    }
    let end = Date::now();
    console_log!("ThreadLocal took {:?}.", Duration::from_secs_f64((end - start)/1000f64));

    let start = Date::now();
    for _ in 1 ..= 100_000_000 {
        std::hint::black_box(RWLOCK.read().unwrap());
    }
    let end = Date::now();
    console_log!("RwLock took {:?}.", Duration::from_secs_f64((end - start)/1000f64));

    let mut bytes = vec![0; BYTES];
    let mut ser_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut de_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut alt_ser_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut alt_de_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut b64_de_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut b64_slice_ser_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut dumb_ser_times: Vec<Duration> = Vec::with_capacity(RUNS);
    let mut dumb_de_times: Vec<Duration> = Vec::with_capacity(RUNS);

    let mut u16s = vec![0; BYTES];

    for _ in 1 ..= RUNS {
        getrandom(&mut bytes).expect("Didn't get the bytes.");

        let start = Date::now();
        let s = bytestring_to_jsstring(&bytes);
        let mid = Date::now();
        let back = jsstring_to_bytestring(&s);
        let end = Date::now();
        ser_times.push(Duration::from_secs_f64((mid - start)/1000f64));
        de_times.push(Duration::from_secs_f64((end - mid) / 1000f64));
        assert_eq!(back, bytes);

        let start = Date::now();
        let s = alt_bytestring_to_jsstring(&bytes);
        let mid = Date::now();
        let back = alt_jsstring_to_bytestring(&s);
        let end = Date::now();
        alt_ser_times.push(Duration::from_secs_f64((mid - start)/1000f64));
        alt_de_times.push(Duration::from_secs_f64((end - mid) / 1000f64));
        assert_eq!(back, bytes);

        let start = Date::now();
        for i in 0 .. BYTES {
            u16s[i] = bytes[i] as u16;
        }
        let s = utf16_to_jsstring(&u16s);
        let mid = Date::now();
        jsstring_to_utf16(&s, &mut u16s).unwrap();
        let back = u16s.iter().map(|&x| { x as u8 }).collect::<Vec<u8>>();
        let end = Date::now();
        dumb_ser_times.push(Duration::from_secs_f64((mid-start)/1000f64));
        dumb_de_times.push(Duration::from_secs_f64((end-mid)/1000f64));
        assert_eq!(back, bytes);

        unsafe {
            let mut dest = String::with_capacity(6*1024*1024);
            dest.as_mut_vec().resize(6*1024*1024, 0);
            let start = Date::now();
            let len = encode_config_slice(&bytes, STANDARD, dest.as_mut_vec().as_mut_slice());
            dest.truncate(len);
            let s = std::hint::black_box(JsString::from(dest));
            let mid = Date::now();
            let back = std::hint::black_box(decode(String::from(s)).expect("Should have been good bytes"));
            let end = Date::now();
            b64_slice_ser_times.push(Duration::from_secs_f64((mid - start)/1000f64));
            b64_de_times.push(Duration::from_secs_f64((end-mid)/1000f64));
            assert_eq!(back, bytes);
        }
    }

    ser_times.sort();
    let (ser_max, ser_min, ser_99) = (ser_times[RUNS-1], ser_times[0], ser_times[NINE_NINE_INDEX]);
    let ser_avg: Duration = ser_times.iter().sum::<Duration>() / (RUNS as u32);
    de_times.sort();
    let (de_max, de_min, de_99) = (de_times[RUNS-1], de_times[0], de_times[NINE_NINE_INDEX]);
    let de_avg: Duration = de_times.iter().sum::<Duration>() / (RUNS as u32);
    console_log!("SERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", ser_max, ser_min, ser_avg, ser_99);
    console_log!("DESERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", de_max, de_min, de_avg, de_99);

    alt_ser_times.sort();
    let (alt_ser_max, alt_ser_min, alt_ser_99) = (alt_ser_times[RUNS-1], alt_ser_times[0], alt_ser_times[NINE_NINE_INDEX]);
    let alt_ser_avg: Duration = alt_ser_times.iter().sum::<Duration>() / (RUNS as u32);
    alt_de_times.sort();
    let (alt_de_max, alt_de_min, alt_de_99) = (alt_de_times[RUNS-1], alt_de_times[0], alt_de_times[NINE_NINE_INDEX]);
    let alt_de_avg: Duration = alt_de_times.iter().sum::<Duration>() / (RUNS as u32);
    console_log!("ALT SERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", alt_ser_max, alt_ser_min, alt_ser_avg, alt_ser_99);
    console_log!("ALT DESERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", alt_de_max, alt_de_min, alt_de_avg, alt_de_99);

    b64_slice_ser_times.sort();
    let (b64_slice_ser_max, b64_slice_ser_min, b64_slice_ser_99) = (b64_slice_ser_times[RUNS-1], b64_slice_ser_times[0], b64_slice_ser_times[NINE_NINE_INDEX]);
    let b64_slice_ser_avg: Duration = b64_slice_ser_times.iter().sum::<Duration>() / (RUNS as u32);
    b64_de_times.sort();
    let (b64_de_max, b64_de_min, b64_de_99) = (b64_de_times[RUNS-1], b64_de_times[0], b64_de_times[NINE_NINE_INDEX]);
    let b64_de_avg: Duration = b64_de_times.iter().sum::<Duration>() / (RUNS as u32);
    console_log!("BASE64 SLICE SERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", b64_slice_ser_max, b64_slice_ser_min, b64_slice_ser_avg, b64_slice_ser_99);
    console_log!("BASE64 DESERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", b64_de_max, b64_de_min, b64_de_avg, b64_de_99);

    dumb_ser_times.sort();
    let (dumb_ser_max, dumb_ser_min, dumb_ser_99) = (dumb_ser_times[RUNS-1], dumb_ser_times[0], dumb_ser_times[NINE_NINE_INDEX]);
    let dumb_ser_avg: Duration = dumb_ser_times.iter().sum::<Duration>() / (RUNS as u32);
    dumb_de_times.sort();
    let (dumb_de_max, dumb_de_min, dumb_de_99) = (dumb_de_times[RUNS-1], dumb_de_times[0], dumb_de_times[NINE_NINE_INDEX]);
    let dumb_de_avg: Duration = dumb_de_times.iter().sum::<Duration>() / (RUNS as u32);
    console_log!("DUMB SERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", dumb_ser_max, dumb_ser_min, dumb_ser_avg, dumb_ser_99);
    console_log!("DUMB DESERIALIZE: Max {:?} -- Min {:?} -- Avg {:?} -- 99% {:?}", dumb_de_max, dumb_de_min, dumb_de_avg, dumb_de_99);
}
