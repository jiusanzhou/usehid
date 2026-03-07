//! useHID - Cross-platform virtual HID device library
//!
//! This library provides a unified API for creating virtual HID devices
//! (mouse, keyboard, gamepad) across macOS, Linux, and Windows.

pub mod error;
pub mod hid;
pub mod keyboard;
pub mod mouse;
pub mod gamepad;
pub mod platform;
pub mod agent;
pub mod screen;
pub mod screenshot;
pub mod accessibility;
pub mod tween;
pub mod failsafe;

pub use error::{Error, Result};
pub use keyboard::{Keyboard, Key, Modifiers};
pub use mouse::{Mouse, MouseButton};
pub use gamepad::{Gamepad, GamepadButton};
pub use agent::AgentHID;
pub use screen::{size, position, move_to, ScreenSize, Position};
pub use screenshot::{screenshot, screenshot_region};
pub use accessibility::{UIElement, Bounds, get_ui_tree, find_ui_element};
pub use tween::{Tween, TweenAnimation};
pub use failsafe::{
    FailsafeConfig, FailsafeCorner, FailsafeError, FailsafeGuard,
    set_failsafe_enabled, is_failsafe_enabled, is_failsafe_triggered,
    reset_failsafe, check_failsafe, check_failsafe_default,
};

/// Device trait for all virtual HID devices
pub trait Device {
    /// Create and register the virtual device
    fn create(&mut self) -> Result<()>;
    
    /// Destroy and unregister the virtual device
    fn destroy(&mut self) -> Result<()>;
    
    /// Check if device is created
    fn is_created(&self) -> bool;
}
