# useHID Skill for OpenClaw

Control mouse, keyboard, gamepad, screenshots, and UI elements through natural language commands.

## Description

This skill enables OpenClaw to control your computer's input devices programmatically, capture screenshots, and read the accessibility UI tree. It provides a JSON-based API that integrates seamlessly with LLM tool-calling.

## Requirements

- Python 3.9+
- usehid Python package

### Installation

```bash
pip install usehid
```

### macOS Permissions

Grant Accessibility permissions:
1. System Settings → Privacy & Security → Accessibility
2. Add your terminal or OpenClaw application

> Screenshot capture works without Accessibility permissions.
> UI Tree features require the permission to read element attributes.

## Usage

When the user asks to control the computer (click, type, scroll, press keys), take screenshots, or inspect UI elements, use the `usehid_tool.py` script.

### Mouse Actions

```bash
# Move mouse
python usehid_tool.py '{"action": "mouse_move", "x": 100, "y": 50}'

# Click
python usehid_tool.py '{"action": "mouse_click", "button": "left"}'

# Double click
python usehid_tool.py '{"action": "mouse_double_click"}'

# Scroll
python usehid_tool.py '{"action": "mouse_scroll", "delta": -3}'

# Drag (press, move, release)
python usehid_tool.py '{"action": "mouse_down", "button": "left"}'
python usehid_tool.py '{"action": "mouse_move", "x": 200, "y": 100}'
python usehid_tool.py '{"action": "mouse_up", "button": "left"}'
```

### Keyboard Actions

```bash
# Type text
python usehid_tool.py '{"action": "type", "text": "Hello, World!"}'

# Press a key
python usehid_tool.py '{"action": "key_press", "key": "enter"}'

# Key combination (Ctrl+S, Cmd+C, etc.)
python usehid_tool.py '{"action": "key_combo", "modifiers": ["ctrl"], "key": "s"}'
python usehid_tool.py '{"action": "key_combo", "modifiers": ["cmd"], "key": "c"}'
```

### Screenshot Actions

```bash
# Full screen screenshot (returns base64 PNG in "data" field)
python usehid_tool.py '{"action": "screenshot"}'

# Region screenshot
python usehid_tool.py '{"action": "screenshot_region", "x": 100, "y": 100, "width": 400, "height": 300}'
```

### Accessibility / UI Tree Actions

```bash
# Get UI tree from focused window (depth-limited)
python usehid_tool.py '{"action": "get_ui_tree", "depth": 3}'

# Get UI tree from a specific app
python usehid_tool.py '{"action": "get_ui_tree", "app": "Safari"}'

# Find elements by role and/or title
python usehid_tool.py '{"action": "find_ui_element", "role": "AXButton"}'
python usehid_tool.py '{"action": "find_ui_element", "role": "AXButton", "title": "Submit"}'
python usehid_tool.py '{"action": "find_ui_element", "title": "Save"}'
```

**UIElement structure returned:**
```json
{
  "role": "AXButton",
  "title": "Submit",
  "value": null,
  "description": "Submit the form",
  "bounds": {"x": 200, "y": 150, "width": 80, "height": 32},
  "children": [],
  "actions": ["AXPress"]
}
```

### Common Workflows

**Open application (macOS Spotlight):**
```bash
python usehid_tool.py '{"action": "key_combo", "modifiers": ["cmd"], "key": "space"}'
sleep 0.5
python usehid_tool.py '{"action": "type", "text": "Chrome"}'
python usehid_tool.py '{"action": "key_press", "key": "enter"}'
```

**Copy and Paste:**
```bash
python usehid_tool.py '{"action": "key_combo", "modifiers": ["cmd"], "key": "a"}'
python usehid_tool.py '{"action": "key_combo", "modifiers": ["cmd"], "key": "c"}'
python usehid_tool.py '{"action": "key_combo", "modifiers": ["cmd"], "key": "v"}'
```

**Take screenshot then find a button and click it:**
```bash
# See what's on screen
python usehid_tool.py '{"action": "screenshot"}'

# Find the button
python usehid_tool.py '{"action": "find_ui_element", "role": "AXButton", "title": "OK"}'
# → returns bounds: {x: 500, y: 300, width: 60, height: 30}

# Click at center of the button
python usehid_tool.py '{"action": "mouse_move_to", "x": 530, "y": 315}'
python usehid_tool.py '{"action": "mouse_click"}'
```

## Supported Keys

**Modifiers:** `ctrl`, `shift`, `alt`, `cmd`/`meta`/`win`

**Special Keys:** `enter`, `escape`, `backspace`, `tab`, `space`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, `delete`, `insert`, `f1`-`f12`

**Mouse Buttons:** `left`, `right`, `middle`

## All Actions Reference

| Action | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `size` | — | `width`, `height` | Screen dimensions |
| `position` | — | `x`, `y` | Mouse position |
| `screenshot` | — | `data` (base64 PNG) | Full screen capture |
| `screenshot_region` | `x`, `y`, `width`, `height` | `data` (base64 PNG) | Region capture |
| `get_ui_tree` | `depth`?, `app`? | `tree` (UIElement) | Accessibility tree |
| `find_ui_element` | `role`?, `title`? | `elements` (UIElement[]) | Find UI elements |
| `mouse_move` | `x`, `y`, `duration`?, `tween`? | — | Relative move |
| `mouse_move_to` | `x`, `y`, `duration`?, `tween`? | — | Absolute move |
| `mouse_click` | `button`? | — | Click |
| `mouse_double_click` | `button`? | — | Double click |
| `mouse_down` | `button`? | — | Press and hold |
| `mouse_up` | `button`? | — | Release |
| `mouse_scroll` | `delta` | — | Scroll (+up/-down) |
| `mouse_drag` | `x`, `y`, `button`?, `duration`?, `tween`? | — | Drag relative |
| `mouse_drag_to` | `x`, `y`, `button`?, `duration`?, `tween`? | — | Drag to position |
| `type` | `text`, `interval`? | — | Type string |
| `key_press` | `key` | — | Press and release key |
| `key_down` | `key` | — | Hold key |
| `key_up` | `key` | — | Release key |
| `key_combo` | `modifiers[]`, `key` | — | Key combination |
| `failsafe_status` | — | `enabled`, `triggered` | Check failsafe |
| `failsafe_enable` | — | — | Enable failsafe |
| `failsafe_disable` | — | — | Disable failsafe |
| `failsafe_reset` | — | — | Reset failsafe |

## Safety Notes

- Always confirm destructive actions (delete, close without save)
- Use small movements for precision
- Add delays between rapid actions
- Test on non-critical applications first
- Failsafe: move mouse to any screen corner to emergency stop
