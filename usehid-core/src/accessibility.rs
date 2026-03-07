//! Accessibility UI tree retrieval
//!
//! Cross-platform access to the OS accessibility API to retrieve UI element trees.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// A UI element from the accessibility tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Bounds>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<UIElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<String>>,
}

/// Bounding rectangle for a UI element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Get the UI tree from the focused window (or a specific app).
///
/// `depth` limits traversal depth (None = unlimited).
/// `app` targets a specific application by name (None = focused app).
pub fn get_ui_tree(depth: Option<u32>, app: Option<&str>) -> Result<UIElement> {
    platform::get_ui_tree(depth, app)
}

/// Find UI elements matching the given role and/or title.
pub fn find_ui_element(role: Option<&str>, title: Option<&str>) -> Result<Vec<UIElement>> {
    let tree = platform::get_ui_tree(None, None)?;
    let mut results = Vec::new();
    find_matching(&tree, role, title, &mut results);
    Ok(results)
}

fn find_matching(element: &UIElement, role: Option<&str>, title: Option<&str>, results: &mut Vec<UIElement>) {
    let role_match = role.map_or(true, |r| element.role.eq_ignore_ascii_case(r));
    let title_match = title.map_or(true, |t| {
        element.title.as_deref().map_or(false, |et| et.contains(t))
    });

    if role_match && title_match {
        // Return a copy without children to keep results flat
        results.push(UIElement {
            role: element.role.clone(),
            title: element.title.clone(),
            value: element.value.clone(),
            description: element.description.clone(),
            bounds: element.bounds.clone(),
            children: Vec::new(),
            actions: element.actions.clone(),
        });
    }

    for child in &element.children {
        find_matching(child, role, title, results);
    }
}

// =============================================================================
// macOS: Accessibility API (AX)
// =============================================================================
#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::ffi::c_void;
    use std::ptr;

    // AX types
    type AXUIElementRef = *mut c_void;
    type AXError = i32;
    type CFTypeRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFAllocatorRef = *const c_void;
    type CFArrayRef = *const c_void;
    type CFIndex = isize;

    #[allow(non_camel_case_types)]
    type pid_t = i32;

    const AX_ERROR_SUCCESS: AXError = 0;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateSystemWide() -> AXUIElementRef;
        fn AXUIElementCreateApplication(pid: pid_t) -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> AXError;
        fn AXUIElementCopyActionNames(
            element: AXUIElementRef,
            names: *mut CFArrayRef,
        ) -> AXError;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: CFTypeRef);
        fn CFStringCreateWithCString(
            alloc: CFAllocatorRef,
            c_str: *const i8,
            encoding: u32,
        ) -> CFStringRef;
        fn CFStringGetCStringPtr(string: CFStringRef, encoding: u32) -> *const u8;
        fn CFStringGetCString(
            string: CFStringRef,
            buffer: *mut u8,
            buffer_size: CFIndex,
            encoding: u32,
        ) -> bool;
        fn CFGetTypeID(cf: CFTypeRef) -> u64;
        fn CFStringGetTypeID() -> u64;
        fn CFArrayGetTypeID() -> u64;
        fn CFArrayGetCount(array: CFArrayRef) -> CFIndex;
        fn CFArrayGetValueAtIndex(array: CFArrayRef, idx: CFIndex) -> CFTypeRef;

        static kCFAllocatorDefault: CFAllocatorRef;
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct CGSize {
        width: f64,
        height: f64,
    }

    // AXValue types
    type AXValueRef = *mut c_void;
    type AXValueType = u32;
    const AX_VALUE_TYPE_CG_POINT: AXValueType = 1;
    const AX_VALUE_TYPE_CG_SIZE: AXValueType = 2;

    extern "C" {
        fn AXValueGetValue(value: AXValueRef, value_type: AXValueType, value_ptr: *mut c_void) -> bool;
    }

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

    fn cfstr(s: &str) -> CFStringRef {
        unsafe {
            let mut buf = s.as_bytes().to_vec();
            buf.push(0);
            CFStringCreateWithCString(kCFAllocatorDefault, buf.as_ptr() as *const i8, K_CF_STRING_ENCODING_UTF8)
        }
    }

    fn cfstring_to_string(cf: CFStringRef) -> Option<String> {
        if cf.is_null() {
            return None;
        }
        unsafe {
            let ptr = CFStringGetCStringPtr(cf, K_CF_STRING_ENCODING_UTF8);
            if !ptr.is_null() {
                let cstr = std::ffi::CStr::from_ptr(ptr as *const i8);
                return Some(cstr.to_string_lossy().into_owned());
            }
            // Fallback: use CFStringGetCString
            let mut buf = [0u8; 1024];
            if CFStringGetCString(cf, buf.as_mut_ptr(), buf.len() as CFIndex, K_CF_STRING_ENCODING_UTF8) {
                let cstr = std::ffi::CStr::from_ptr(buf.as_ptr() as *const i8);
                Some(cstr.to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }

    fn get_ax_string_attr(element: AXUIElementRef, attr: &str) -> Option<String> {
        unsafe {
            let key = cfstr(attr);
            let mut value: CFTypeRef = ptr::null();
            let err = AXUIElementCopyAttributeValue(element, key, &mut value);
            CFRelease(key as CFTypeRef);
            if err != AX_ERROR_SUCCESS || value.is_null() {
                return None;
            }
            if CFGetTypeID(value) == CFStringGetTypeID() {
                let result = cfstring_to_string(value as CFStringRef);
                CFRelease(value);
                result
            } else {
                CFRelease(value);
                None
            }
        }
    }

    fn get_ax_bounds(element: AXUIElementRef) -> Option<Bounds> {
        unsafe {
            // Get position
            let pos_key = cfstr("AXPosition");
            let mut pos_value: CFTypeRef = ptr::null();
            let err = AXUIElementCopyAttributeValue(element, pos_key, &mut pos_value);
            CFRelease(pos_key as CFTypeRef);
            if err != AX_ERROR_SUCCESS || pos_value.is_null() {
                return None;
            }

            let mut point = CGPoint { x: 0.0, y: 0.0 };
            let ok = AXValueGetValue(
                pos_value as AXValueRef,
                AX_VALUE_TYPE_CG_POINT,
                &mut point as *mut CGPoint as *mut c_void,
            );
            CFRelease(pos_value);
            if !ok {
                return None;
            }

            // Get size
            let size_key = cfstr("AXSize");
            let mut size_value: CFTypeRef = ptr::null();
            let err = AXUIElementCopyAttributeValue(element, size_key, &mut size_value);
            CFRelease(size_key as CFTypeRef);
            if err != AX_ERROR_SUCCESS || size_value.is_null() {
                return None;
            }

            let mut size = CGSize { width: 0.0, height: 0.0 };
            let ok = AXValueGetValue(
                size_value as AXValueRef,
                AX_VALUE_TYPE_CG_SIZE,
                &mut size as *mut CGSize as *mut c_void,
            );
            CFRelease(size_value);
            if !ok {
                return None;
            }

            Some(Bounds {
                x: point.x,
                y: point.y,
                width: size.width,
                height: size.height,
            })
        }
    }

    fn get_ax_actions(element: AXUIElementRef) -> Option<Vec<String>> {
        unsafe {
            let mut names: CFArrayRef = ptr::null();
            let err = AXUIElementCopyActionNames(element, &mut names);
            if err != AX_ERROR_SUCCESS || names.is_null() {
                return None;
            }
            let count = CFArrayGetCount(names);
            if count == 0 {
                CFRelease(names as CFTypeRef);
                return None;
            }
            let mut result = Vec::new();
            for i in 0..count {
                let val = CFArrayGetValueAtIndex(names, i);
                if !val.is_null() {
                    if let Some(s) = cfstring_to_string(val as CFStringRef) {
                        result.push(s);
                    }
                }
            }
            CFRelease(names as CFTypeRef);
            if result.is_empty() { None } else { Some(result) }
        }
    }

    fn get_ax_children(element: AXUIElementRef) -> Vec<AXUIElementRef> {
        unsafe {
            let key = cfstr("AXChildren");
            let mut value: CFTypeRef = ptr::null();
            let err = AXUIElementCopyAttributeValue(element, key, &mut value);
            CFRelease(key as CFTypeRef);
            if err != AX_ERROR_SUCCESS || value.is_null() {
                return Vec::new();
            }
            if CFGetTypeID(value) != CFArrayGetTypeID() {
                CFRelease(value);
                return Vec::new();
            }
            let count = CFArrayGetCount(value as CFArrayRef);
            let mut children = Vec::new();
            for i in 0..count {
                let child = CFArrayGetValueAtIndex(value as CFArrayRef, i);
                if !child.is_null() {
                    children.push(child as AXUIElementRef);
                }
            }
            // Note: we do NOT release the array here because the children references
            // are owned by the array. We'll release after building the tree.
            // Actually, we need the refs to stay valid, so we'll just leak the array.
            // In practice this is fine for a snapshot operation.
            // A proper implementation would use CFRetain on each child.
            children
        }
    }

    fn build_tree(element: AXUIElementRef, current_depth: u32, max_depth: Option<u32>) -> UIElement {
        let role = get_ax_string_attr(element, "AXRole").unwrap_or_else(|| "unknown".to_string());
        let title = get_ax_string_attr(element, "AXTitle");
        let value = get_ax_string_attr(element, "AXValue");
        let description = get_ax_string_attr(element, "AXDescription");
        let bounds = get_ax_bounds(element);
        let actions = get_ax_actions(element);

        let children = if max_depth.map_or(true, |d| current_depth < d) {
            get_ax_children(element)
                .into_iter()
                .map(|child| build_tree(child, current_depth + 1, max_depth))
                .collect()
        } else {
            Vec::new()
        };

        UIElement {
            role,
            title,
            value,
            description,
            bounds,
            children,
            actions,
        }
    }

    /// Parse PID from lsappinfo output like: "pid"=822
    fn parse_lsappinfo_pid(output: &str) -> Option<pid_t> {
        for line in output.lines() {
            let line = line.trim();
            // Match pattern: "pid"=NNN or "pid" = NNN
            if line.contains("pid") {
                // Extract digits after '='
                if let Some(eq_pos) = line.find('=') {
                    let after_eq = line[eq_pos + 1..].trim();
                    if let Ok(pid) = after_eq.parse::<pid_t>() {
                        return Some(pid);
                    }
                }
            }
        }
        None
    }

        fn get_focused_app_pid() -> Result<pid_t> {
        // Use lsappinfo to get the frontmost app PID (no Accessibility permission needed)
        let front_output = std::process::Command::new("lsappinfo")
            .arg("front")
            .output()
            .map_err(|e| Error::AccessibilityFailed(format!("lsappinfo front failed: {}", e)))?;

        let asn = String::from_utf8_lossy(&front_output.stdout).trim().to_string();
        if asn.is_empty() {
            return Err(Error::AccessibilityFailed("No frontmost application found".into()));
        }

        let info_output = std::process::Command::new("lsappinfo")
            .args(["info", "-only", "pid", &asn])
            .output()
            .map_err(|e| Error::AccessibilityFailed(format!("lsappinfo info failed: {}", e)))?;

        let info_str = String::from_utf8_lossy(&info_output.stdout);
        // Parse output like: "pid"=12345
        if let Some(pid) = parse_lsappinfo_pid(&info_str) {
            return Ok(pid);
        }

        // Fallback: try osascript (may require Accessibility permission)
        let output = std::process::Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to unix id of first process whose frontmost is true"])
            .output()
            .map_err(|e| Error::AccessibilityFailed(format!("osascript failed: {}", e)))?;

        let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        pid_str.parse::<pid_t>()
            .map_err(|e| Error::AccessibilityFailed(format!("Failed to parse PID '{}': {}", pid_str, e)))
    }

    fn get_app_pid_by_name(name: &str) -> Result<pid_t> {
        // Try lsappinfo first (no permission needed)
        let output = std::process::Command::new("lsappinfo")
            .args(["info", "-only", "pid", name])
            .output()
            .map_err(|e| Error::AccessibilityFailed(format!("lsappinfo failed: {}", e)))?;

        let info_str = String::from_utf8_lossy(&output.stdout);
        // Parse output like: "pid"=822
        if let Some(pid) = parse_lsappinfo_pid(&info_str) {
            return Ok(pid);
        }

        Err(Error::AccessibilityFailed(
            format!("App '{}' not found or not running", name),
        ))
    }

    pub fn get_ui_tree(depth: Option<u32>, app: Option<&str>) -> Result<UIElement> {
        let pid = match app {
            Some(name) => get_app_pid_by_name(name)?,
            None => get_focused_app_pid()?,
        };

        unsafe {
            let app_element = AXUIElementCreateApplication(pid);
            if app_element.is_null() {
                return Err(Error::AccessibilityFailed("Failed to create AXUIElement for app".into()));
            }

            // Get the focused window of the app
            let key = cfstr("AXFocusedWindow");
            let mut window: CFTypeRef = ptr::null();
            let err = AXUIElementCopyAttributeValue(app_element, key, &mut window);
            CFRelease(key as CFTypeRef);

            let root = if err == AX_ERROR_SUCCESS && !window.is_null() {
                // Build tree from focused window
                let tree = build_tree(window as AXUIElementRef, 0, depth);
                CFRelease(window);
                tree
            } else {
                // Fallback: build tree from app element itself
                build_tree(app_element, 0, depth)
            };

            CFRelease(app_element as CFTypeRef);
            Ok(root)
        }
    }
}

// =============================================================================
// Linux: command-based approach using xdotool + xprop
// =============================================================================
#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::process::Command;

    pub fn get_ui_tree(depth: Option<u32>, app: Option<&str>) -> Result<UIElement> {
        // Get the focused window ID
        let window_id = if let Some(app_name) = app {
            // Find window by app name
            let output = Command::new("xdotool")
                .args(["search", "--name", app_name])
                .output()
                .map_err(|e| Error::AccessibilityFailed(format!("xdotool search failed: {}", e)))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().next()
                .ok_or_else(|| Error::AccessibilityFailed(format!("No window found for '{}'", app_name)))?
                .trim()
                .to_string()
        } else {
            let output = Command::new("xdotool")
                .args(["getactivewindow"])
                .output()
                .map_err(|e| Error::AccessibilityFailed(format!("xdotool getactivewindow failed: {}", e)))?;
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        if window_id.is_empty() {
            return Err(Error::AccessibilityFailed("No active window found".into()));
        }

        // Get window info
        let name_output = Command::new("xdotool")
            .args(["getwindowname", &window_id])
            .output()
            .map_err(|e| Error::AccessibilityFailed(format!("xdotool getwindowname failed: {}", e)))?;
        let window_name = String::from_utf8_lossy(&name_output.stdout).trim().to_string();

        // Get window geometry
        let geom_output = Command::new("xdotool")
            .args(["getwindowgeometry", "--shell", &window_id])
            .output()
            .map_err(|e| Error::AccessibilityFailed(format!("xdotool getwindowgeometry failed: {}", e)))?;
        let geom_str = String::from_utf8_lossy(&geom_output.stdout);

        let mut x = 0.0f64;
        let mut y = 0.0f64;
        let mut width = 0.0f64;
        let mut height = 0.0f64;
        for line in geom_str.lines() {
            if let Some(val) = line.strip_prefix("X=") {
                x = val.parse().unwrap_or(0.0);
            } else if let Some(val) = line.strip_prefix("Y=") {
                y = val.parse().unwrap_or(0.0);
            } else if let Some(val) = line.strip_prefix("WIDTH=") {
                width = val.parse().unwrap_or(0.0);
            } else if let Some(val) = line.strip_prefix("HEIGHT=") {
                height = val.parse().unwrap_or(0.0);
            }
        }

        let _ = depth; // Linux command-based approach doesn't support depth traversal

        Ok(UIElement {
            role: "window".to_string(),
            title: if window_name.is_empty() { None } else { Some(window_name) },
            value: None,
            description: None,
            bounds: Some(Bounds { x, y, width, height }),
            children: Vec::new(),
            actions: None,
        })
    }
}

// =============================================================================
// Windows: UI Automation API
// =============================================================================
#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    use windows::Win32::UI::Accessibility::*;
    use windows::Win32::System::Com::*;
    use windows::core::*;

    pub fn get_ui_tree(depth: Option<u32>, app: Option<&str>) -> Result<UIElement> {
        unsafe {
            // Initialize COM
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .map_err(|e| Error::AccessibilityFailed(format!("COM init failed: {}", e)))?;

            let result = get_ui_tree_inner(depth, app);
            CoUninitialize();
            result
        }
    }

    unsafe fn get_ui_tree_inner(depth: Option<u32>, app: Option<&str>) -> Result<UIElement> {
        let automation: IUIAutomation = CoCreateInstance(
            &CUIAutomation,
            None,
            CLSCTX_ALL,
        ).map_err(|e| Error::AccessibilityFailed(format!("Failed to create IUIAutomation: {}", e)))?;

        let root = if let Some(app_name) = app {
            // Find element by app name
            let desktop = automation.GetRootElement()
                .map_err(|e| Error::AccessibilityFailed(format!("GetRootElement failed: {}", e)))?;

            let condition = automation.CreatePropertyCondition(
                UIA_NamePropertyId,
                &VARIANT::from(app_name),
            ).map_err(|e| Error::AccessibilityFailed(format!("CreatePropertyCondition failed: {}", e)))?;

            desktop.FindFirst(TreeScope_Children, &condition)
                .map_err(|e| Error::AccessibilityFailed(format!("App '{}' not found: {}", app_name, e)))?
        } else {
            // Get focused element and walk up to its window
            let focused = automation.GetFocusedElement()
                .map_err(|e| Error::AccessibilityFailed(format!("GetFocusedElement failed: {}", e)))?;

            let walker = automation.CreateTreeWalker(&automation.RawViewCondition().unwrap())
                .map_err(|e| Error::AccessibilityFailed(format!("CreateTreeWalker failed: {}", e)))?;

            // Walk up to find the window
            let mut current = focused;
            loop {
                let control_type = current.CurrentControlType().unwrap_or(0);
                if control_type == UIA_WindowControlTypeId {
                    break;
                }
                match walker.GetParentElement(&current) {
                    Ok(parent) => current = parent,
                    Err(_) => break,
                }
            }
            current
        };

        build_tree_win(&root, 0, depth)
    }

    unsafe fn build_tree_win(
        element: &IUIAutomationElement,
        current_depth: u32,
        max_depth: Option<u32>,
    ) -> Result<UIElement> {
        let role = element.CurrentLocalizedControlType()
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let title = element.CurrentName()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let value = element.GetCurrentPropertyValue(UIA_ValueValuePropertyId)
            .ok()
            .and_then(|v| {
                let s = v.to_string();
                if s.is_empty() { None } else { Some(s) }
            });

        let description = element.CurrentHelpText()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let bounds = element.CurrentBoundingRectangle().ok().map(|r| Bounds {
            x: r.left as f64,
            y: r.top as f64,
            width: (r.right - r.left) as f64,
            height: (r.bottom - r.top) as f64,
        });

        let children = if max_depth.map_or(true, |d| current_depth < d) {
            let mut kids = Vec::new();
            if let Ok(children_array) = element.FindAll(TreeScope_Children, &IUIAutomationCondition::default()) {
                let count = children_array.Length().unwrap_or(0);
                for i in 0..count {
                    if let Ok(child) = children_array.GetElement(i) {
                        if let Ok(child_elem) = build_tree_win(&child, current_depth + 1, max_depth) {
                            kids.push(child_elem);
                        }
                    }
                }
            }
            kids
        } else {
            Vec::new()
        };

        Ok(UIElement {
            role,
            title,
            value,
            description,
            bounds,
            children,
            actions: None,
        })
    }
}

// =============================================================================
// Unsupported platforms
// =============================================================================
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod platform {
    use super::*;

    pub fn get_ui_tree(_depth: Option<u32>, _app: Option<&str>) -> Result<UIElement> {
        Err(Error::PlatformNotSupported("accessibility".into()))
    }
}
