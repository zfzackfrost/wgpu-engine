use super::{KeyCode, MouseButton};

/// Data for mouse movement events
#[derive(Debug, Clone)]
pub struct MouseMoveData {
    /// Current mouse position in window coordinates
    pub position: glam::Vec2,
    /// Delta movement since last mouse event
    pub delta: glam::Vec2,
}

/// Data for mouse wheel scroll events
#[derive(Debug, Clone)]
pub struct MouseWheelData {
    /// Scroll delta (positive = scroll up/right)
    pub delta: glam::Vec2,
}

/// Data for mouse button press/release events
#[derive(Debug, Clone)]
pub struct MouseButtonData {
    /// Which mouse button was affected
    pub button: MouseButton,
    /// True if pressed, false if released
    pub is_pressed: bool,
}

/// Data for keyboard press/release events
#[derive(Debug, Clone)]
pub struct KeyboardData {
    /// The key that was pressed/released
    pub key_code: KeyCode,
    /// True if pressed, false if released
    pub is_pressed: bool,
    /// True if this is a repeat event from holding the key
    pub is_repeat: bool,
}
