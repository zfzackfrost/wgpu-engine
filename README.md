# WGPU Engine - A lightweight graphics engine built on WGPU

This crate provides a simple framework for creating graphics applications
with cross-platform support (native and WebAssembly). It includes:

- Application lifecycle management with client pattern
- Event handling system with observer pattern
- WGPU-based rendering state management
- Cross-platform window and input handling

## Usage

Create an application by implementing the `AppClient` trait and using
the `define_entry_point!` macro:

```rust
use wgpu_engine::*;

#[derive(Debug)]
struct MyApp;

impl AppClient for MyApp {
    fn render(&self, rpass: &mut wgpu::RenderPass<'_>) {
        // Your rendering code here
    }
}

define_entry_point!(std::sync::Arc::new(MyApp) as SharedAppClient);
```
