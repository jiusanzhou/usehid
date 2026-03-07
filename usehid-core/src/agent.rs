//! Agent-friendly HID interface
//!
//! JSON-based interface for LLM agents to control HID devices.

use crate::error::{Error, Result};
use crate::tween::Tween;
use crate::accessibility::UIElement;
use crate::{Device, Keyboard, Key, Modifiers, Mouse, MouseButton, Gamepad, GamepadButton};
use serde::{Deserialize, Serialize};

/// Action types for agent interface
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AgentAction {
    // Screen/Query actions
    Size,
    Position,
    
    // Failsafe actions
    FailsafeEnable,
    FailsafeDisable,
    FailsafeStatus,
    FailsafeReset,
    
    // Mouse actions
    MouseMove { 
        x: i32, 
        y: i32,
        #[serde(default)]
        duration: Option<u64>,  // milliseconds
        #[serde(default)]
        tween: Option<String>,
    },
    MouseMoveTo { 
        x: i32, 
        y: i32,
        #[serde(default)]
        duration: Option<u64>,
        #[serde(default)]
        tween: Option<String>,
    },
    MouseClick { button: Option<String> },
    MouseDoubleClick { button: Option<String> },
    MouseDown { button: Option<String> },
    MouseUp { button: Option<String> },
    MouseScroll { delta: i8 },
    MouseDrag { 
        x: i32, 
        y: i32, 
        button: Option<String>,
        #[serde(default)]
        duration: Option<u64>,
        #[serde(default)]
        tween: Option<String>,
    },
    MouseDragTo { 
        x: i32, 
        y: i32, 
        button: Option<String>,
        #[serde(default)]
        duration: Option<u64>,
        #[serde(default)]
        tween: Option<String>,
    },
    
    // Keyboard actions
    Type { 
        text: String,
        #[serde(default)]
        interval: Option<u64>,  // milliseconds between keystrokes
    },
    KeyPress { key: String },
    KeyDown { key: String },
    KeyUp { key: String },
    KeyCombo { modifiers: Vec<String>, key: String },
    
    // Gamepad actions
    GamepadPress { button: String },
    GamepadRelease { button: String },
    GamepadLeftStick { x: u8, y: u8 },
    GamepadRightStick { x: u8, y: u8 },
    GamepadTriggers { left: u8, right: u8 },

    // Screenshot actions
    Screenshot,
    ScreenshotRegion { x: i32, y: i32, width: u32, height: u32 },

    // Accessibility actions
    GetUiTree {
        #[serde(default)]
        depth: Option<u32>,
        #[serde(default)]
        app: Option<String>,
    },
    FindUiElement {
        #[serde(default)]
        role: Option<String>,
        #[serde(default)]
        title: Option<String>,
    },
}

/// Result of an agent action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree: Option<UIElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<UIElement>>,
}

impl AgentResult {
    fn ok() -> Self {
        Self { success: true, error: None, x: None, y: None, width: None, height: None, enabled: None, triggered: None, data: None, tree: None, elements: None }
    }

    fn err(msg: impl Into<String>) -> Self {
        Self { success: false, error: Some(msg.into()), x: None, y: None, width: None, height: None, enabled: None, triggered: None, data: None, tree: None, elements: None }
    }

    fn with_position(x: i32, y: i32) -> Self {
        Self { success: true, error: None, x: Some(x), y: Some(y), width: None, height: None, enabled: None, triggered: None, data: None, tree: None, elements: None }
    }

    fn with_size(width: u32, height: u32) -> Self {
        Self { success: true, error: None, x: None, y: None, width: Some(width), height: Some(height), enabled: None, triggered: None, data: None, tree: None, elements: None }
    }

    fn with_failsafe_status(enabled: bool, triggered: bool) -> Self {
        Self { success: true, error: None, x: None, y: None, width: None, height: None, enabled: Some(enabled), triggered: Some(triggered), data: None, tree: None, elements: None }
    }

    fn with_data(data: String) -> Self {
        Self { success: true, error: None, x: None, y: None, width: None, height: None, enabled: None, triggered: None, data: Some(data), tree: None, elements: None }
    }

    fn with_tree(tree: UIElement) -> Self {
        Self { success: true, error: None, x: None, y: None, width: None, height: None, enabled: None, triggered: None, data: None, tree: Some(tree), elements: None }
    }

    fn with_elements(elements: Vec<UIElement>) -> Self {
        Self { success: true, error: None, x: None, y: None, width: None, height: None, enabled: None, triggered: None, data: None, tree: None, elements: Some(elements) }
    }
}

/// Agent HID controller
pub struct AgentHID {
    mouse: Mouse,
    keyboard: Keyboard,
    gamepad: Gamepad,
    mouse_created: bool,
    keyboard_created: bool,
    gamepad_created: bool,
}

impl AgentHID {
    /// Create a new agent HID controller
    pub fn new() -> Self {
        Self {
            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
            gamepad: Gamepad::new(),
            mouse_created: false,
            keyboard_created: false,
            gamepad_created: false,
        }
    }
    
    /// Execute an action from JSON
    pub fn execute_json(&mut self, json: &str) -> AgentResult {
        match serde_json::from_str::<AgentAction>(json) {
            Ok(action) => self.execute(action),
            Err(e) => AgentResult::err(format!("Invalid JSON: {}", e)),
        }
    }
    
    /// Execute an action
    pub fn execute(&mut self, action: AgentAction) -> AgentResult {
        // Check failsafe before executing (except for failsafe control actions and read-only queries)
        if !matches!(action, AgentAction::FailsafeEnable | AgentAction::FailsafeDisable |
                            AgentAction::FailsafeStatus | AgentAction::FailsafeReset |
                            AgentAction::Size | AgentAction::Position |
                            AgentAction::Screenshot | AgentAction::ScreenshotRegion { .. } |
                            AgentAction::GetUiTree { .. } | AgentAction::FindUiElement { .. }) {
            if let Err(e) = crate::failsafe::check_failsafe_default() {
                return AgentResult::err(format!("Failsafe: {}", e));
            }
        }
        
        match action {
            // Screen/Query
            AgentAction::Size => self.get_size(),
            AgentAction::Position => self.get_position(),
            
            // Failsafe
            AgentAction::FailsafeEnable => {
                crate::failsafe::set_failsafe_enabled(true);
                AgentResult::ok()
            }
            AgentAction::FailsafeDisable => {
                crate::failsafe::set_failsafe_enabled(false);
                AgentResult::ok()
            }
            AgentAction::FailsafeStatus => {
                AgentResult::with_failsafe_status(
                    crate::failsafe::is_failsafe_enabled(),
                    crate::failsafe::is_failsafe_triggered(),
                )
            }
            AgentAction::FailsafeReset => {
                crate::failsafe::reset_failsafe();
                AgentResult::ok()
            }
            
            // Mouse
            AgentAction::MouseMove { x, y, duration, tween } => self.mouse_move_with_duration(x, y, duration, tween),
            AgentAction::MouseMoveTo { x, y, duration, tween } => self.mouse_move_to_with_duration(x, y, duration, tween),
            AgentAction::MouseClick { button } => self.mouse_click(button),
            AgentAction::MouseDoubleClick { button } => self.mouse_double_click(button),
            AgentAction::MouseDown { button } => self.mouse_down(button),
            AgentAction::MouseUp { button } => self.mouse_up(button),
            AgentAction::MouseScroll { delta } => self.mouse_scroll(delta),
            AgentAction::MouseDrag { x, y, button, duration, tween } => self.mouse_drag_with_duration(x, y, button, duration, tween),
            AgentAction::MouseDragTo { x, y, button, duration, tween } => self.mouse_drag_to_with_duration(x, y, button, duration, tween),
            
            // Keyboard
            AgentAction::Type { text, interval } => self.type_text_with_interval(&text, interval),
            AgentAction::KeyPress { key } => self.key_press(&key),
            AgentAction::KeyDown { key } => self.key_down(&key),
            AgentAction::KeyUp { key } => self.key_up(&key),
            AgentAction::KeyCombo { modifiers, key } => self.key_combo(&modifiers, &key),
            
            // Gamepad
            AgentAction::GamepadPress { button } => self.gamepad_press(&button),
            AgentAction::GamepadRelease { button } => self.gamepad_release(&button),
            AgentAction::GamepadLeftStick { x, y } => self.gamepad_left_stick(x, y),
            AgentAction::GamepadRightStick { x, y } => self.gamepad_right_stick(x, y),
            AgentAction::GamepadTriggers { left, right } => self.gamepad_triggers(left, right),

            // Screenshot
            AgentAction::Screenshot => self.take_screenshot(),
            AgentAction::ScreenshotRegion { x, y, width, height } => self.take_screenshot_region(x, y, width, height),

            // Accessibility
            AgentAction::GetUiTree { depth, app } => self.get_ui_tree(depth, app),
            AgentAction::FindUiElement { role, title } => self.find_ui_element(role, title),
        }
    }
    
    fn ensure_mouse(&mut self) -> Result<()> {
        if !self.mouse_created {
            self.mouse.create()?;
            self.mouse_created = true;
        }
        Ok(())
    }
    
    fn ensure_keyboard(&mut self) -> Result<()> {
        if !self.keyboard_created {
            self.keyboard.create()?;
            self.keyboard_created = true;
        }
        Ok(())
    }
    
    fn ensure_gamepad(&mut self) -> Result<()> {
        if !self.gamepad_created {
            self.gamepad.create()?;
            self.gamepad_created = true;
        }
        Ok(())
    }
    
    fn parse_mouse_button(s: Option<String>) -> MouseButton {
        match s.as_deref() {
            Some("right") => MouseButton::RIGHT,
            Some("middle") => MouseButton::MIDDLE,
            _ => MouseButton::LEFT,
        }
    }
    
    fn parse_gamepad_button(s: &str) -> Result<GamepadButton> {
        let btn = match s.to_lowercase().as_str() {
            "a" => GamepadButton::A,
            "b" => GamepadButton::B,
            "x" => GamepadButton::X,
            "y" => GamepadButton::Y,
            "lb" | "left_bumper" => GamepadButton::LB,
            "rb" | "right_bumper" => GamepadButton::RB,
            "back" | "select" => GamepadButton::BACK,
            "start" => GamepadButton::START,
            "guide" | "home" => GamepadButton::GUIDE,
            "left_stick" | "ls" => GamepadButton::LEFT_STICK,
            "right_stick" | "rs" => GamepadButton::RIGHT_STICK,
            "dpad_up" | "up" => GamepadButton::DPAD_UP,
            "dpad_down" | "down" => GamepadButton::DPAD_DOWN,
            "dpad_left" | "left" => GamepadButton::DPAD_LEFT,
            "dpad_right" | "right" => GamepadButton::DPAD_RIGHT,
            _ => return Err(Error::InvalidAction(format!("Unknown gamepad button: {}", s))),
        };
        Ok(btn)
    }
    
    fn parse_modifiers(mods: &[String]) -> Modifiers {
        let mut result = Modifiers::empty();
        for m in mods {
            match m.to_lowercase().as_str() {
                "ctrl" | "control" => result |= Modifiers::CTRL,
                "shift" => result |= Modifiers::SHIFT,
                "alt" | "option" => result |= Modifiers::ALT,
                "cmd" | "command" | "meta" | "gui" | "win" => result |= Modifiers::CMD,
                _ => {}
            }
        }
        result
    }
    
    // Mouse actions
    fn get_size(&self) -> AgentResult {
        match crate::screen::size() {
            Ok(s) => AgentResult::with_size(s.width, s.height),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn get_position(&self) -> AgentResult {
        match crate::screen::position() {
            Ok(p) => AgentResult::with_position(p.x, p.y),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_move(&mut self, x: i32, y: i32) -> AgentResult {
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        match self.mouse.move_by(x, y) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_move_with_duration(&mut self, x: i32, y: i32, duration: Option<u64>, tween: Option<String>) -> AgentResult {
        let duration_ms = duration.unwrap_or(0);
        if duration_ms == 0 {
            return self.mouse_move(x, y);
        }
        
        // Get current position for relative move calculation
        let current = match crate::screen::position() {
            Ok(p) => p,
            Err(e) => return AgentResult::err(e.to_string()),
        };
        
        let target_x = current.x + x;
        let target_y = current.y + y;
        
        self.animate_move(current.x, current.y, target_x, target_y, duration_ms, tween)
    }
    
    fn mouse_move_to(&mut self, x: i32, y: i32) -> AgentResult {
        match crate::screen::move_to(x, y) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_move_to_with_duration(&mut self, x: i32, y: i32, duration: Option<u64>, tween: Option<String>) -> AgentResult {
        let duration_ms = duration.unwrap_or(0);
        if duration_ms == 0 {
            return self.mouse_move_to(x, y);
        }
        
        let current = match crate::screen::position() {
            Ok(p) => p,
            Err(e) => return AgentResult::err(e.to_string()),
        };
        
        self.animate_move(current.x, current.y, x, y, duration_ms, tween)
    }
    
    fn animate_move(&mut self, start_x: i32, start_y: i32, end_x: i32, end_y: i32, duration_ms: u64, tween: Option<String>) -> AgentResult {
        let tween_fn = tween
            .as_ref()
            .and_then(|s| Tween::from_str(s))
            .unwrap_or(Tween::EaseInOutQuad);
        
        let anim = crate::tween::TweenAnimation::new(
            start_x as f64, start_y as f64,
            end_x as f64, end_y as f64,
            duration_ms,
            tween_fn,
        );
        
        // Use 60 FPS for smooth animation
        let positions = anim.generate_positions(60);
        let frame_delay = std::time::Duration::from_millis(duration_ms / positions.len().max(1) as u64);
        
        for (x, y) in positions {
            if let Err(e) = crate::screen::move_to(x, y) {
                return AgentResult::err(e.to_string());
            }
            std::thread::sleep(frame_delay);
        }
        
        AgentResult::ok()
    }
    
    fn mouse_click(&mut self, button: Option<String>) -> AgentResult {
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        let btn = Self::parse_mouse_button(button);
        match self.mouse.click(btn) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_double_click(&mut self, button: Option<String>) -> AgentResult {
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        let btn = Self::parse_mouse_button(button);
        match self.mouse.double_click(btn) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_down(&mut self, button: Option<String>) -> AgentResult {
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        let btn = Self::parse_mouse_button(button);
        match self.mouse.press(btn) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_up(&mut self, button: Option<String>) -> AgentResult {
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        let btn = Self::parse_mouse_button(button);
        match self.mouse.release(btn) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_scroll(&mut self, delta: i8) -> AgentResult {
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        match self.mouse.scroll(delta) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    // Keyboard actions
    fn type_text(&mut self, text: &str) -> AgentResult {
        if let Err(e) = self.ensure_keyboard() {
            return AgentResult::err(e.to_string());
        }
        match self.keyboard.type_text(text) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn type_text_with_interval(&mut self, text: &str, interval: Option<u64>) -> AgentResult {
        let interval_ms = interval.unwrap_or(0);
        if interval_ms == 0 {
            return self.type_text(text);
        }
        
        if let Err(e) = self.ensure_keyboard() {
            return AgentResult::err(e.to_string());
        }
        
        let delay = std::time::Duration::from_millis(interval_ms);
        for c in text.chars() {
            if let Err(e) = self.keyboard.type_char(c) {
                return AgentResult::err(e.to_string());
            }
            std::thread::sleep(delay);
        }
        AgentResult::ok()
    }
    
    fn key_press(&mut self, key: &str) -> AgentResult {
        if let Err(e) = self.ensure_keyboard() {
            return AgentResult::err(e.to_string());
        }
        match Key::from_str(key) {
            Ok(k) => match self.keyboard.tap(k) {
                Ok(_) => AgentResult::ok(),
                Err(e) => AgentResult::err(e.to_string()),
            },
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn key_down(&mut self, key: &str) -> AgentResult {
        if let Err(e) = self.ensure_keyboard() {
            return AgentResult::err(e.to_string());
        }
        match Key::from_str(key) {
            Ok(k) => match self.keyboard.press_key(k) {
                Ok(_) => AgentResult::ok(),
                Err(e) => AgentResult::err(e.to_string()),
            },
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn key_up(&mut self, key: &str) -> AgentResult {
        if let Err(e) = self.ensure_keyboard() {
            return AgentResult::err(e.to_string());
        }
        match Key::from_str(key) {
            Ok(k) => match self.keyboard.release_key(k) {
                Ok(_) => AgentResult::ok(),
                Err(e) => AgentResult::err(e.to_string()),
            },
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn key_combo(&mut self, modifiers: &[String], key: &str) -> AgentResult {
        if let Err(e) = self.ensure_keyboard() {
            return AgentResult::err(e.to_string());
        }
        let mods = Self::parse_modifiers(modifiers);
        match Key::from_str(key) {
            Ok(k) => match self.keyboard.press_combo(mods, k) {
                Ok(_) => AgentResult::ok(),
                Err(e) => AgentResult::err(e.to_string()),
            },
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    // Gamepad actions
    fn gamepad_press(&mut self, button: &str) -> AgentResult {
        if let Err(e) = self.ensure_gamepad() {
            return AgentResult::err(e.to_string());
        }
        match Self::parse_gamepad_button(button) {
            Ok(btn) => match self.gamepad.press(btn) {
                Ok(_) => AgentResult::ok(),
                Err(e) => AgentResult::err(e.to_string()),
            },
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn gamepad_release(&mut self, button: &str) -> AgentResult {
        if let Err(e) = self.ensure_gamepad() {
            return AgentResult::err(e.to_string());
        }
        match Self::parse_gamepad_button(button) {
            Ok(btn) => match self.gamepad.release(btn) {
                Ok(_) => AgentResult::ok(),
                Err(e) => AgentResult::err(e.to_string()),
            },
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn gamepad_left_stick(&mut self, x: u8, y: u8) -> AgentResult {
        if let Err(e) = self.ensure_gamepad() {
            return AgentResult::err(e.to_string());
        }
        match self.gamepad.set_left_stick(x, y) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn gamepad_right_stick(&mut self, x: u8, y: u8) -> AgentResult {
        if let Err(e) = self.ensure_gamepad() {
            return AgentResult::err(e.to_string());
        }
        match self.gamepad.set_right_stick(x, y) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn gamepad_triggers(&mut self, left: u8, right: u8) -> AgentResult {
        if let Err(e) = self.ensure_gamepad() {
            return AgentResult::err(e.to_string());
        }
        if let Err(e) = self.gamepad.set_left_trigger(left) {
            return AgentResult::err(e.to_string());
        }
        match self.gamepad.set_right_trigger(right) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    // Drag operations
    fn mouse_drag(&mut self, x: i32, y: i32, button: Option<String>) -> AgentResult {
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        let btn = Self::parse_mouse_button(button);
        
        // Press, move, release
        if let Err(e) = self.mouse.press(btn) {
            return AgentResult::err(e.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        if let Err(e) = self.mouse.move_by(x, y) {
            let _ = self.mouse.release(btn);
            return AgentResult::err(e.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        match self.mouse.release(btn) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_drag_with_duration(&mut self, x: i32, y: i32, button: Option<String>, duration: Option<u64>, tween: Option<String>) -> AgentResult {
        let duration_ms = duration.unwrap_or(0);
        if duration_ms == 0 {
            return self.mouse_drag(x, y, button);
        }
        
        let btn = Self::parse_mouse_button(button);
        let current = match crate::screen::position() {
            Ok(p) => p,
            Err(e) => return AgentResult::err(e.to_string()),
        };
        
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        if let Err(e) = self.mouse.press(btn) {
            return AgentResult::err(e.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let target_x = current.x + x;
        let target_y = current.y + y;
        let result = self.animate_move(current.x, current.y, target_x, target_y, duration_ms, tween);
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _ = self.mouse.release(btn);
        
        result
    }
    
    fn mouse_drag_to(&mut self, x: i32, y: i32, button: Option<String>) -> AgentResult {
        let btn = Self::parse_mouse_button(button);
        
        // Get current position
        let current = match crate::screen::position() {
            Ok(p) => p,
            Err(e) => return AgentResult::err(e.to_string()),
        };
        
        // Press at current position
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        if let Err(e) = self.mouse.press(btn) {
            return AgentResult::err(e.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Move to target
        let dx = x - current.x;
        let dy = y - current.y;
        if let Err(e) = self.mouse.move_by(dx, dy) {
            let _ = self.mouse.release(btn);
            return AgentResult::err(e.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Release
        match self.mouse.release(btn) {
            Ok(_) => AgentResult::ok(),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
    
    fn mouse_drag_to_with_duration(&mut self, x: i32, y: i32, button: Option<String>, duration: Option<u64>, tween: Option<String>) -> AgentResult {
        let duration_ms = duration.unwrap_or(0);
        if duration_ms == 0 {
            return self.mouse_drag_to(x, y, button);
        }
        
        let btn = Self::parse_mouse_button(button);
        let current = match crate::screen::position() {
            Ok(p) => p,
            Err(e) => return AgentResult::err(e.to_string()),
        };
        
        if let Err(e) = self.ensure_mouse() {
            return AgentResult::err(e.to_string());
        }
        if let Err(e) = self.mouse.press(btn) {
            return AgentResult::err(e.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let result = self.animate_move(current.x, current.y, x, y, duration_ms, tween);
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _ = self.mouse.release(btn);
        
        result
    }

    // Screenshot actions
    fn take_screenshot(&self) -> AgentResult {
        match crate::screenshot::screenshot() {
            Ok(png_bytes) => {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                AgentResult::with_data(b64)
            }
            Err(e) => AgentResult::err(e.to_string()),
        }
    }

    fn take_screenshot_region(&self, x: i32, y: i32, width: u32, height: u32) -> AgentResult {
        match crate::screenshot::screenshot_region(x, y, width, height) {
            Ok(png_bytes) => {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                AgentResult::with_data(b64)
            }
            Err(e) => AgentResult::err(e.to_string()),
        }
    }

    // Accessibility actions
    fn get_ui_tree(&self, depth: Option<u32>, app: Option<String>) -> AgentResult {
        match crate::accessibility::get_ui_tree(depth, app.as_deref()) {
            Ok(tree) => AgentResult::with_tree(tree),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }

    fn find_ui_element(&self, role: Option<String>, title: Option<String>) -> AgentResult {
        match crate::accessibility::find_ui_element(role.as_deref(), title.as_deref()) {
            Ok(elements) => AgentResult::with_elements(elements),
            Err(e) => AgentResult::err(e.to_string()),
        }
    }
}

impl Default for AgentHID {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AgentHID {
    fn drop(&mut self) {
        if self.mouse_created {
            let _ = self.mouse.destroy();
        }
        if self.keyboard_created {
            let _ = self.keyboard.destroy();
        }
        if self.gamepad_created {
            let _ = self.gamepad.destroy();
        }
    }
}
