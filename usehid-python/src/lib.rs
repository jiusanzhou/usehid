//! Python bindings for useHID

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use usehid as usehid_crate;
use usehid_crate::{
    AgentHID as CoreAgentHID,
    Device,
    Keyboard as CoreKeyboard,
    Key,
    Modifiers,
    Mouse as CoreMouse,
    MouseButton,
    Gamepad as CoreGamepad,
    GamepadButton,
};

/// Get the screen size (width, height)
#[pyfunction]
fn size() -> PyResult<(u32, u32)> {
    let s = usehid_crate::size()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok((s.width, s.height))
}

/// Get the current mouse position (x, y)
#[pyfunction]
fn position() -> PyResult<(i32, i32)> {
    let p = usehid_crate::position()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok((p.x, p.y))
}

/// Move mouse to absolute coordinates
#[pyfunction]
fn move_to(x: i32, y: i32) -> PyResult<()> {
    usehid_crate::move_to(x, y)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Take a full screenshot, returns PNG bytes
#[pyfunction]
fn screenshot(py: Python<'_>) -> PyResult<Py<pyo3::types::PyBytes>> {
    let bytes = usehid_crate::screenshot()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(pyo3::types::PyBytes::new(py, &bytes).unbind())
}

/// Take a screenshot of a region, returns PNG bytes
#[pyfunction]
fn screenshot_region(py: Python<'_>, x: i32, y: i32, width: u32, height: u32) -> PyResult<Py<pyo3::types::PyBytes>> {
    let bytes = usehid_crate::screenshot_region(x, y, width, height)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(pyo3::types::PyBytes::new(py, &bytes).unbind())
}

/// Get the accessibility UI tree
#[pyfunction]
#[pyo3(signature = (depth=None, app=None))]
fn get_ui_tree(py: Python<'_>, depth: Option<u32>, app: Option<&str>) -> PyResult<Py<pyo3::types::PyDict>> {
    let tree = usehid_crate::get_ui_tree(depth, app)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let json_str = serde_json::to_string(&tree)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let json_mod = pyo3::types::PyModule::import(py, "json")?;
    let dict = json_mod.call_method1("loads", (json_str,))?;
    Ok(dict.downcast::<pyo3::types::PyDict>()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        .clone()
        .unbind())
}

/// Find UI elements matching role and/or title
#[pyfunction]
#[pyo3(signature = (role=None, title=None))]
fn find_ui_element(py: Python<'_>, role: Option<&str>, title: Option<&str>) -> PyResult<PyObject> {
    let elements = usehid_crate::find_ui_element(role, title)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let json_str = serde_json::to_string(&elements)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let json_mod = pyo3::types::PyModule::import(py, "json")?;
    let list = json_mod.call_method1("loads", (json_str,))?;
    Ok(list.unbind())
}

/// Virtual Mouse
#[pyclass]
struct Mouse {
    inner: CoreMouse,
}

#[pymethods]
impl Mouse {
    #[new]
    #[pyo3(signature = (name=None))]
    fn new(name: Option<&str>) -> PyResult<Self> {
        let mut mouse = match name {
            Some(n) => CoreMouse::with_name(n),
            None => CoreMouse::new(),
        };
        mouse.create().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner: mouse })
    }
    
    /// Move mouse by relative offset
    fn move_by(&mut self, dx: i32, dy: i32) -> PyResult<()> {
        self.inner.move_by(dx, dy)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Drag mouse by relative offset (press, move, release)
    #[pyo3(signature = (dx, dy, button="left"))]
    fn drag(&mut self, dx: i32, dy: i32, button: &str) -> PyResult<()> {
        let btn = parse_mouse_button(button);
        self.inner.press(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.inner.move_by(dx, dy)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.inner.release(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Click mouse button
    #[pyo3(signature = (button="left"))]
    fn click(&mut self, button: &str) -> PyResult<()> {
        let btn = parse_mouse_button(button);
        self.inner.click(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Double click mouse button
    #[pyo3(signature = (button="left"))]
    fn double_click(&mut self, button: &str) -> PyResult<()> {
        let btn = parse_mouse_button(button);
        self.inner.double_click(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Press mouse button
    #[pyo3(signature = (button="left"))]
    fn press(&mut self, button: &str) -> PyResult<()> {
        let btn = parse_mouse_button(button);
        self.inner.press(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Release mouse button
    #[pyo3(signature = (button="left"))]
    fn release(&mut self, button: &str) -> PyResult<()> {
        let btn = parse_mouse_button(button);
        self.inner.release(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Scroll wheel
    fn scroll(&mut self, delta: i8) -> PyResult<()> {
        self.inner.scroll(delta)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

fn parse_mouse_button(s: &str) -> MouseButton {
    match s.to_lowercase().as_str() {
        "right" => MouseButton::RIGHT,
        "middle" => MouseButton::MIDDLE,
        _ => MouseButton::LEFT,
    }
}

/// Virtual Keyboard
#[pyclass]
struct Keyboard {
    inner: CoreKeyboard,
}

#[pymethods]
impl Keyboard {
    #[new]
    #[pyo3(signature = (name=None))]
    fn new(name: Option<&str>) -> PyResult<Self> {
        let mut keyboard = match name {
            Some(n) => CoreKeyboard::with_name(n),
            None => CoreKeyboard::new(),
        };
        keyboard.create().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner: keyboard })
    }
    
    /// Type a string
    fn type_text(&mut self, text: &str) -> PyResult<()> {
        self.inner.type_text(text)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Press a key
    fn press(&mut self, key: &str) -> PyResult<()> {
        let k = Key::from_str(key)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        self.inner.tap(k)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Press key combination (e.g., combo(["ctrl"], "c"))
    fn combo(&mut self, modifiers: Vec<String>, key: &str) -> PyResult<()> {
        let mut mods = Modifiers::empty();
        for m in &modifiers {
            match m.to_lowercase().as_str() {
                "ctrl" | "control" => mods |= Modifiers::CTRL,
                "shift" => mods |= Modifiers::SHIFT,
                "alt" | "option" => mods |= Modifiers::ALT,
                "cmd" | "command" | "meta" | "gui" | "win" => mods |= Modifiers::CMD,
                _ => {}
            }
        }
        let k = Key::from_str(key)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        self.inner.press_combo(mods, k)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Release all keys
    fn release_all(&mut self) -> PyResult<()> {
        self.inner.release_all()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

/// Virtual Gamepad
#[pyclass]
struct Gamepad {
    inner: CoreGamepad,
}

#[pymethods]
impl Gamepad {
    #[new]
    #[pyo3(signature = (name=None))]
    fn new(name: Option<&str>) -> PyResult<Self> {
        let mut gamepad = match name {
            Some(n) => CoreGamepad::with_name(n),
            None => CoreGamepad::new(),
        };
        gamepad.create().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner: gamepad })
    }
    
    /// Press button
    fn press(&mut self, button: &str) -> PyResult<()> {
        let btn = parse_gamepad_button(button)?;
        self.inner.press(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Release button
    fn release(&mut self, button: &str) -> PyResult<()> {
        let btn = parse_gamepad_button(button)?;
        self.inner.release(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Tap button (press and release)
    fn tap(&mut self, button: &str) -> PyResult<()> {
        let btn = parse_gamepad_button(button)?;
        self.inner.tap(btn)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Set left stick position (0-255, 128 = center)
    fn left_stick(&mut self, x: u8, y: u8) -> PyResult<()> {
        self.inner.set_left_stick(x, y)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Set right stick position (0-255, 128 = center)
    fn right_stick(&mut self, x: u8, y: u8) -> PyResult<()> {
        self.inner.set_right_stick(x, y)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Set triggers (0-255)
    fn triggers(&mut self, left: u8, right: u8) -> PyResult<()> {
        self.inner.set_left_trigger(left)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        self.inner.set_right_trigger(right)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
    
    /// Reset all to default
    fn reset(&mut self) -> PyResult<()> {
        self.inner.reset()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

fn parse_gamepad_button(s: &str) -> PyResult<GamepadButton> {
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
        "dpad_left" => GamepadButton::DPAD_LEFT,
        "dpad_right" => GamepadButton::DPAD_RIGHT,
        _ => return Err(PyRuntimeError::new_err(format!("Unknown button: {}", s))),
    };
    Ok(btn)
}

/// Agent HID controller for LLM agents
#[pyclass]
struct AgentHID {
    inner: CoreAgentHID,
}

#[pymethods]
impl AgentHID {
    #[new]
    fn new() -> Self {
        Self {
            inner: CoreAgentHID::new(),
        }
    }
    
    /// Execute an action from dict/JSON
    fn execute(&mut self, py: Python<'_>, action: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Py<pyo3::types::PyDict>> {
        let json = pyo3::types::PyModule::import(py, "json")?
            .call_method1("dumps", (action,))?
            .extract::<String>()?;

        let result = self.inner.execute_json(&json);

        // Serialize via JSON to capture all fields (data, tree, elements, etc.)
        let result_json = serde_json::to_string(&result)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let json_mod = pyo3::types::PyModule::import(py, "json")?;
        let dict = json_mod.call_method1("loads", (result_json,))?;
        Ok(dict.downcast::<pyo3::types::PyDict>()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
            .clone()
            .unbind())
    }

    /// Execute action from JSON string
    fn execute_json(&mut self, json: &str) -> PyResult<String> {
        let result = self.inner.execute_json(json);
        serde_json::to_string(&result)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

/// Python module
#[pymodule]
fn usehid_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mouse>()?;
    m.add_class::<Keyboard>()?;
    m.add_class::<Gamepad>()?;
    m.add_class::<AgentHID>()?;
    m.add_function(wrap_pyfunction!(size, m)?)?;
    m.add_function(wrap_pyfunction!(position, m)?)?;
    m.add_function(wrap_pyfunction!(move_to, m)?)?;
    m.add_function(wrap_pyfunction!(screenshot, m)?)?;
    m.add_function(wrap_pyfunction!(screenshot_region, m)?)?;
    m.add_function(wrap_pyfunction!(get_ui_tree, m)?)?;
    m.add_function(wrap_pyfunction!(find_ui_element, m)?)?;
    Ok(())
}
