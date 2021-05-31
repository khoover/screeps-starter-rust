'use strict';

Object.defineProperty(exports, '__esModule', { value: true });

var fastestsmallesttextencoderdecoder = require('fastestsmallesttextencoderdecoder');

function test_log(f) { console.log(f('world')); }

let wasm$1;

let cachedTextDecoder = new fastestsmallesttextencoderdecoder.TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm$1.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm$1.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error('expected a number argument');
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new fastestsmallesttextencoderdecoder.TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (typeof(arg) !== 'string') throw new Error('expected a string argument');

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);
        if (ret.read !== arg.length) throw new Error('failed to pass whole string');
        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm$1.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm$1.memory.buffer);
    }
    return cachegetInt32Memory0;
}
function __wbg_adapter_2(arg0, arg1, arg2) {
    try {
        const retptr = wasm$1.__wbindgen_add_to_stack_pointer(-16);
        _assertNum(arg0);
        _assertNum(arg1);
        var ptr0 = passStringToWasm0(arg2, wasm$1.__wbindgen_malloc, wasm$1.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm$1.wasm_bindgen__convert__closures__invoke1_mut__h6907cf43884a2641(retptr, arg0, arg1, ptr0, len0);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm$1.__wbindgen_add_to_stack_pointer(16);
        wasm$1.__wbindgen_free(r0, r1);
    }
}

function logError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        let error = (function () {
            try {
                return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
            } catch(_) {
                return "<failed to stringify thrown value>";
            }
        }());
        console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
        throw e;
    }
}

function init(module) {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_testlog_be09fbd0c2423dd8 = function() { return logError(function (arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = (arg0) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_2(a, state0.b, arg0);
                } finally {
                    state0.a = a;
                }
            };
            test_log(cb0);
        } finally {
            state0.a = state0.b = 0;
        }
    }, arguments) };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };



    const instance = new WebAssembly.Instance(module, imports);

    wasm$1 = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm$1;
}

let wasm_module = new WebAssembly.Module(require('fs').readFileSync('../rust-out/screeps_starter_rust_bg.wasm'));
let wasm = init(wasm_module);

function console_error(...args) {
    console.log(...args);
    Game.notify(args.join(' '));
}

const loop = function() {
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
};

loop();

exports.loop = loop;
