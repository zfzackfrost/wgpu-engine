//! Re-exports of third-party dependencies
//!
//! This module provides convenient access to commonly used third-party crates
//! that are part of the engine's public API. Users can access these through
//! `wgpu_engine::third_party::*` instead of adding separate dependencies.

#[rustfmt::skip]
mod _exports {
    /// Error handling and context
    pub use anyhow;
    /// Linear algebra library for graphics
    pub use glam;
    /// Logging facade
    pub use log;
    /// Cross-platform time utilities
    pub use web_time;
    /// Graphics API abstraction
    pub use wgpu;
    /// Fast synchronization primitives
    pub use parking_lot;
    /// Derive macros for common traits
    pub use educe;

    /// Async runtime for native platforms
    #[cfg(not(target_arch = "wasm32"))]
    pub use pollster;
    /// Better panic messages for WebAssembly
    #[cfg(target_arch = "wasm32")]
    pub use console_error_panic_hook;
}

pub use _exports::*;
