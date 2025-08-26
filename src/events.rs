use std::sync::LazyLock;

use parking_lot::{Mutex, MutexGuard};

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

use crate::observer::{Publisher, Subscriber};

pub static EVENTS: LazyLock<Events> = LazyLock::new(|| {
    Events {
        mouse_move: Mutex::new(Publisher::new()),
        mouse_wheel: Mutex::new(Publisher::new()),
        mouse_button: Mutex::new(Publisher::new()),
        keyboard: Mutex::new(Publisher::new()),
    }
});

type EventPublisher<Data> = Publisher<Data, Box<dyn Subscriber<Data>>>;
type MutEventPublisher<Data> = Mutex<EventPublisher<Data>>;
type GuardEventPublisher<'a, Data> = MutexGuard<'a, EventPublisher<Data>>;

pub struct Events {
    mouse_move: MutEventPublisher<MouseMoveData>,
    mouse_wheel: MutEventPublisher<MouseWheelData>,
    mouse_button: MutEventPublisher<MouseButtonData>,
    keyboard: MutEventPublisher<KeyboardData>,
}
impl Events {
    pub fn mouse_move(&self) -> GuardEventPublisher<'_, MouseMoveData> {
        self.mouse_move.lock()
    }
    pub fn mouse_wheel(&self) -> GuardEventPublisher<'_, MouseWheelData> {
        self.mouse_wheel.lock()
    }
    pub fn mouse_button(&self) -> GuardEventPublisher<'_, MouseButtonData> {
        self.mouse_button.lock()
    }
    pub fn keyboard(&self) -> GuardEventPublisher<'_, KeyboardData> {
        self.keyboard.lock()
    }
}

pub struct MouseMoveData {
    pub position: glam::Vec2,
}
pub struct MouseWheelData {
    pub delta: glam::Vec2,
}
pub struct MouseButtonData {
    pub button: MouseButton,
    pub is_pressed: bool,
}
pub struct KeyboardData {
    pub key_code: KeyCode,
    pub is_pressed: bool,
    pub is_repeat: bool,
}
