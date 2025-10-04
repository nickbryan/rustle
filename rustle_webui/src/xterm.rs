use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use web_sys::{Element, KeyboardEvent};

#[wasm_bindgen]
extern "C" {
    pub(crate) type Terminal;

    #[wasm_bindgen(constructor)]
    pub(crate) fn new() -> Terminal;

    #[wasm_bindgen(method)]
    pub(crate) fn open(this: &Terminal, parent: Element);

    #[wasm_bindgen(method)]
    pub(crate) fn write(this: &Terminal, data: String);

    #[wasm_bindgen(method, js_name = "attachCustomKeyEventHandler")]
    pub(crate) fn attach_custom_key_event_handler(
        this: &Terminal,
        handler: &Closure<dyn FnMut(KeyboardEvent) -> bool>,
    );

    #[wasm_bindgen(js_name = "ITerminalAddon")]
    pub(crate) type TerminalAddon;

    #[wasm_bindgen(method)]
    pub(crate) fn activate(this: &TerminalAddon, terminal: Terminal);

    #[wasm_bindgen(method, js_name = loadAddon)]
    pub(crate) fn load_addon(this: &Terminal, addon: TerminalAddon);

    #[wasm_bindgen(method, getter)]
    pub(crate) fn cols(this: &Terminal) -> u16;

    #[wasm_bindgen(method, getter)]
    pub(crate) fn rows(this: &Terminal) -> u16;

    #[wasm_bindgen(method)]
    pub(crate) fn focus(this: &Terminal);

    #[wasm_bindgen(js_namespace = FitAddon, js_name = FitAddon, extends = TerminalAddon)]
    pub(crate) type FitTerminalAddon;

    #[wasm_bindgen(constructor, js_class = "FitAddon.FitAddon")]
    pub(crate) fn new() -> FitTerminalAddon;

    #[wasm_bindgen(method, js_namespace = FitAddon)]
    pub(crate) fn fit(this: &FitTerminalAddon);
}
