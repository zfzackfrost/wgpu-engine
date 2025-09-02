//! Observer pattern implementation
//!
//! This module provides a type-safe observer pattern implementation with 
//! priority-based event handling. Publishers can notify multiple subscribers
//! of events, with subscribers being called in priority order.

mod publisher;
mod subscriber;

pub use publisher::*;
pub use subscriber::*;
