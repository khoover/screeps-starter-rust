use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "export function test_log(f) { console.log(f('world')); }")]
extern "C" {
    fn test_log(f: &mut dyn FnMut(String) -> String);
}

#[wasm_bindgen]
pub fn make_call() {
    test_log(&mut |x| { format!("Hello {}", x) });
}
