"use strict";
/*import { readFileSync } from 'fs';
import init from '../rust-out/screeps_starter_rust.js';
let start = Date.now();
let wasm_module = new WebAssembly.Module(readFileSync('dist/screeps_starter_rust_bg.wasm'));
let mid = Date.now();
let wasm = init(wasm_module);
let end = Date.now();
console.log("Compile: ", (mid - start));
console.log("Instantiate: ", (end - mid));
wasm.then((m) => {m.make_call()});*/
const wasm = require("../rust-out/screeps_starter_rust.js");
wasm.make_call();
