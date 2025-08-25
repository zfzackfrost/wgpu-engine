#[macro_export]
macro_rules! define_entry_point {
    ($make_client:expr) => {
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen::prelude::*;

        pub fn run() -> $crate::third_party::anyhow::Result<()> {
            $crate::run($make_client)
        }

        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen(start)]
        pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
            $crate::third_party::console_error_panic_hook::set_once();
            run().unwrap_throw();
            Ok(())
        }
    };
}
