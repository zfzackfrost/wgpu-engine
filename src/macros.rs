//! Macros for defining application entry points

/// Defines the main entry point for a WGPU engine application
///
/// This macro generates both native and WebAssembly entry points for your application.
/// It creates a `run()` function for native platforms and a `run_web()` function
/// for WebAssembly builds.
///
/// # Arguments
///
/// * `$make_client` - An expression that returns a `SharedAppClient` (Arc<dyn AppClient>)
///
/// # Example
///
/// ```ignore
/// use wgpu_engine::*;
///
/// #[derive(Debug)]
/// struct MyApp;
///
/// impl AppClient for MyApp {
///     fn render(&self, rpass: &mut wgpu::RenderPass<'_>) {
///         // Your rendering code
///     }
/// }
///
/// define_entry_point!(|| std::sync::Arc::new(MyApp) as SharedAppClient);
///
/// fn main() {
///     run().unwrap();
/// }
/// ```
#[macro_export]
macro_rules! define_entry_point {
    ($make_client:expr) => {
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen::prelude::*;

        /// Main entry point for native platforms
        pub fn run() -> $crate::third_party::anyhow::Result<()> {
            $crate::run($make_client)
        }

        /// WebAssembly entry point
        /// 
        /// This function is automatically called when the WASM module is loaded.
        /// It sets up panic hooks and starts the application.
        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen(start)]
        pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
            // Set up better panic messages for debugging
            $crate::third_party::console_error_panic_hook::set_once();
            run().unwrap_throw();
            Ok(())
        }
    };
}
