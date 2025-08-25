use std::sync::LazyLock;

use parking_lot::Mutex;
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

pub type EventPublisher<Data> = Publisher<Data, Box<dyn Subscriber<Data>>>;
type MutEventPublisher<Data> = Mutex<EventPublisher<Data>>;

#[non_exhaustive]
pub struct Events {
    pub mouse_move: MutEventPublisher<MouseMoveData>,
    pub mouse_wheel: MutEventPublisher<MouseWheelData>,
    pub mouse_button: MutEventPublisher<MouseButtonData>,
    pub keyboard: MutEventPublisher<KeyboardData>,
}

pub struct MouseMoveData {
    pub position: glam::Vec2,
}
pub struct MouseWheelData {
    pub delta: glam::Vec2,
}
pub struct MouseButtonData {
    pub is_pressed: bool,
    pub button: MouseButton,
}
pub struct KeyboardData {
    pub key_code: KeyCode,
    pub is_pressed: bool,
    pub is_repeat: bool,
}
