//! Global event system for input and lifecycle events
//!
//! This module provides a centralized event system using the observer pattern.
//! It handles mouse, keyboard, and application lifecycle events.

use std::sync::LazyLock;

use parking_lot::{Mutex, MutexGuard};

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

use crate::observer::{FnSubscriber, Priority, Publisher, Subscriber, Subscription};

mod data;
pub use data::*;

/// Global event system instance
///
/// This provides access to all event publishers for the application.
/// Publishers allow subscribing to various input and lifecycle events.
pub static EVENTS: LazyLock<Events> = LazyLock::new(|| {
    let events = Events {
        start_of_frame: Mutex::new(Publisher::new()),
        update: Mutex::new(Publisher::new()),
        mouse_move: Mutex::new(Publisher::new()),
        mouse_wheel: Mutex::new(Publisher::new()),
        mouse_button: Mutex::new(Publisher::new()),
        keyboard: Mutex::new(Publisher::new()),
        end_of_frame: Mutex::new(Publisher::new()),
        last_mouse_position: Mutex::new(None),
    };
    events.init();
    events
});

// Type aliases for cleaner event publisher definitions
type EventPublisher<Data> = Publisher<Box<dyn Subscriber<Data = Data>>>;
type MutEventPublisher<Data> = Mutex<EventPublisher<Data>>;
type GuardEventPublisher<'a, Data> = MutexGuard<'a, EventPublisher<Data>>;

/// Central event system containing all event publishers
///
/// This struct provides access to publishers for various application events
/// including input events and frame lifecycle events.
pub struct Events {
    /// Published at the start of each frame
    start_of_frame: MutEventPublisher<()>,
    /// Published during the update phase of each frame
    update: MutEventPublisher<()>,
    /// Published when mouse moves
    mouse_move: MutEventPublisher<MouseMoveData>,
    /// Published when mouse wheel scrolls
    mouse_wheel: MutEventPublisher<MouseWheelData>,
    /// Published when mouse buttons are pressed/released
    mouse_button: MutEventPublisher<MouseButtonData>,
    /// Published when keyboard keys are pressed/released
    keyboard: MutEventPublisher<KeyboardData>,
    /// Published at the end of each frame
    end_of_frame: MutEventPublisher<()>,

    /// Cached last mouse position for delta calculation
    last_mouse_position: Mutex<Option<glam::Vec2>>,
}
impl Events {
    /// Initializes the event system with necessary subscriptions
    ///
    /// This sets up internal subscribers like mouse position tracking.
    fn init(&self) {
        // Subscribe to mouse move events to track the last position
        // Use high priority value to ensure this runs after most
        // subscribers
        self.mouse_move().subscribe(
            FnSubscriber::new(|data: &MouseMoveData| {
                *EVENTS.last_mouse_position.lock() = Some(data.position);
                Subscription::Keep
            })
            .with_priority(Priority::late(i32::MAX / 2))
            .boxed(),
        );
    }
    /// Returns the last known mouse position
    ///
    /// Returns Vec2::ZERO if no mouse movement has been recorded yet.
    pub(crate) fn last_mouse_position(&self) -> glam::Vec2 {
        self.last_mouse_position.lock().unwrap_or(glam::Vec2::ZERO)
    }

    /// Returns the start of frame event publisher
    pub fn start_of_frame(&self) -> GuardEventPublisher<'_, ()> {
        self.start_of_frame.lock()
    }

    /// Returns the update event publisher
    pub fn update(&self) -> GuardEventPublisher<'_, ()> {
        self.update.lock()
    }

    /// Returns the mouse move event publisher
    pub fn mouse_move(&self) -> GuardEventPublisher<'_, MouseMoveData> {
        self.mouse_move.lock()
    }

    /// Returns the mouse wheel event publisher
    pub fn mouse_wheel(&self) -> GuardEventPublisher<'_, MouseWheelData> {
        self.mouse_wheel.lock()
    }

    /// Returns the mouse button event publisher
    pub fn mouse_button(&self) -> GuardEventPublisher<'_, MouseButtonData> {
        self.mouse_button.lock()
    }

    /// Returns the keyboard event publisher
    pub fn keyboard(&self) -> GuardEventPublisher<'_, KeyboardData> {
        self.keyboard.lock()
    }

    /// Returns the end of frame event publisher
    pub fn end_of_frame(&self) -> GuardEventPublisher<'_, ()> {
        self.end_of_frame.lock()
    }
}
