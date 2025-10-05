use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use web_sys::{Element, KeyboardEvent};

#[wasm_bindgen]
extern "C" {
    /// Bindings to the xterm.js `Terminal` class.
    pub(crate) type Terminal;

    /// Creates a new `Terminal` instance.
    #[wasm_bindgen(constructor)]
    pub(crate) fn new() -> Terminal;

    /// Opens the terminal in a given parent DOM element.
    #[wasm_bindgen(method)]
    pub(crate) fn open(this: &Terminal, parent: Element);

    /// Writes data to the terminal.
    #[wasm_bindgen(method)]
    pub(crate) fn write(this: &Terminal, data: String);

    /// Attaches a custom key event handler to the terminal.
    #[wasm_bindgen(method, js_name = "attachCustomKeyEventHandler")]
    pub(crate) fn attach_custom_key_event_handler(
        this: &Terminal,
        handler: &Closure<dyn FnMut(KeyboardEvent) -> bool>,
    );

    /// A trait for xterm.js addons.
    #[wasm_bindgen(js_name = "ITerminalAddon")]
    pub(crate) type TerminalAddon;

    /// Activates the addon on a given terminal.
    #[wasm_bindgen(method)]
    pub(crate) fn activate(this: &TerminalAddon, terminal: Terminal);

    /// Loads an addon into the terminal.
    #[wasm_bindgen(method, js_name = loadAddon)]
    pub(crate) fn load_addon(this: &Terminal, addon: TerminalAddon);

    /// The number of columns in the terminal.
    #[wasm_bindgen(method, getter)]
    pub(crate) fn cols(this: &Terminal) -> u16;

    /// The number of rows in the terminal.
    #[wasm_bindgen(method, getter)]
    pub(crate) fn rows(this: &Terminal) -> u16;

    /// Focuses the terminal.
    #[wasm_bindgen(method)]
    pub(crate) fn focus(this: &Terminal);

    /// Bindings to the xterm.js `FitAddon` class.
    #[wasm_bindgen(js_namespace = FitAddon, js_name = FitAddon, extends = TerminalAddon)]
    pub(crate) type FitTerminalAddon;

    /// Creates a new `FitTerminalAddon` instance.
    #[wasm_bindgen(constructor, js_class = "FitAddon.FitAddon")]
    pub(crate) fn new() -> FitTerminalAddon;

    /// Fits the terminal to the size of its container.
    #[wasm_bindgen(method, js_namespace = FitAddon)]
    pub(crate) fn fit(this: &FitTerminalAddon);
}
