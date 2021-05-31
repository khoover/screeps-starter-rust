"use strict";
let wasm_module = Game.cpu.bucket >= 500 ? new WebAssembly.Module(require('screeps_starter_rust_bg')) : null;
import init from '../rust-out/screeps_starter_rust.js';
let wasm = Game.cpu.bucket >= 500 ? init(wasm_module) : null;

function console_error(...args) {
    console.log(...args);
    Game.notify(args.join(' '));
}

export const loop = function() {
    try {
        if (!wasm) {
            wasm = init(wasm_module);
        }
        wasm.make_call();
        wasm = null;
    } catch(error) {
        console_error("caught exception:", error);
        if (error.stack) {
            console_error("stack trace:", error.stack);
        }
        console_error("resetting VM next tick.");
        wasm = null;
    }
}

loop();
