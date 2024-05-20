use trax_lsp_server::TraxLspServer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

fn log(s: String) {
    #[allow(unused_unsafe)]
    unsafe {
        console_log(&format!("[trax-lsp-server] {s}"))
    }
}

#[wasm_bindgen]
pub struct TraxLspServerWasm {
    ls: TraxLspServer<fn(String)>,
}

#[wasm_bindgen]
impl TraxLspServerWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            ls: TraxLspServer::new(log),
        }
    }

    #[wasm_bindgen(js_name = onNotification)]
    pub fn on_notification(&mut self, method: &str, params: JsValue) -> JsValue {
        if let Some(v) = self
            .ls
            .on_notification(method, serde_wasm_bindgen::from_value(params).unwrap())
        {
            serde_wasm_bindgen::to_value(&v).unwrap()
        } else {
            JsValue::null()
        }
    }
}

impl Default for TraxLspServerWasm {
    fn default() -> Self {
        Self::new()
    }
}
