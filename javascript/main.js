"use strict";
/*import { Codec } from "./utf15";
import { randomBytes } from 'crypto';*/
import { readFileSync } from 'fs';
import init from '../rust-out/screeps_starter_rust.js';
let start = Date.now();
let wasm_module = new WebAssembly.Module(readFileSync('dist/screeps_starter_rust_bg.wasm'));
let mid = Date.now();
let wasm = init(wasm_module);
let end = Date.now();
console.log("Compile: ", (mid - start));
console.log("Instantiate: ", (end - mid));
wasm.make_call();

/*function arraysEqual(a, b) {
      if (a === b) return true;
      if (a == null || b == null) return false;
      if (a.length !== b.length) return false;

      // If you don't care about the order of the elements inside
      // the array, you should sort both arrays here.
      // Please note that calling sort on an array will modify that array.
      // you might want to clone your array first.

      for (let i = 0; i < a.length; ++i) {
          if (a[i] !== b[i]) return false;
      }
      return true;
}

const start = Date.now();
const wasm = require("../rust-out/screeps_starter_rust.js");
const end = Date.now();
console.log("Wasm compile + instantiate: ", end - start);
wasm.make_call();*/
/*const codec = new Codec({ array: true, depth: 8 });
const RUNS = 10;
const NINE_NINE = 8;
const ser_times = [];
const de_times = [];
for (let i = 0; i < RUNS; ++i) {
    const bytes = randomBytes(32767);
    const bytes_array = [];
    for (const val of bytes) {
        bytes_array.push(val);
    }
    const start = Date.now();
    const s = codec.encode(bytes_array);
    const mid = Date.now();
    const arr = codec.decode(s);
    const end = Date.now();
    if (!arraysEqual(bytes_array, arr)) {
        console.error("utf15 is busted!");
        break;
    }
    ser_times.push(mid - start);
    de_times.push(end - mid);
}
ser_times.sort((a, b) => { return a - b; });
const ser_min = ser_times[0];
const ser_max = ser_times[RUNS-1];
const ser_99 = ser_times[NINE_NINE];
const ser_avg = ser_times.reduce((a, b) => { return a + b; }) / RUNS;
de_times.sort((a, b) => { return a - b; });
const de_min = de_times[0];
const de_max = de_times[RUNS-1];
const de_99 = de_times[NINE_NINE];
const de_avg = de_times.reduce((a, b) => { return a + b; }) / RUNS;
console.log("JS SERIALIZATION: Max ", ser_max, " -- Min ", ser_min, " -- Avg ", ser_avg, " -- 99% ", ser_99);
console.log("JS DESERIALIZATION: Max ", de_max, " -- Min ", de_min, " -- Avg ", de_avg, " -- 99% ", de_99);*/
