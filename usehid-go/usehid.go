// Package usehid provides Go bindings for the useHID virtual HID device library.
//
// This package wraps the Rust usehid-core library via CGO and provides a simple
// interface for creating virtual mice, keyboards, and gamepads.
//
// Example:
//
//	mouse := usehid.NewMouse()
//	defer mouse.Close()
//	mouse.MoveBy(100, 50)
//	mouse.Click(usehid.ButtonLeft)
package usehid

/*
#cgo LDFLAGS: -L${SRCDIR}/../target/release -lusehid_core
#cgo darwin LDFLAGS: -framework IOKit -framework CoreFoundation

#include <stdlib.h>
#include <stdint.h>

// FFI declarations - these will be provided by a C header from Rust
typedef void* HidDevice;

extern HidDevice usehid_mouse_create(const char* name);
extern void usehid_mouse_destroy(HidDevice device);
extern int usehid_mouse_move_by(HidDevice device, int32_t dx, int32_t dy);
extern int usehid_mouse_click(HidDevice device, uint8_t button);
extern int usehid_mouse_press(HidDevice device, uint8_t button);
extern int usehid_mouse_release(HidDevice device, uint8_t button);
extern int usehid_mouse_scroll(HidDevice device, int8_t delta);

extern HidDevice usehid_keyboard_create(const char* name);
extern void usehid_keyboard_destroy(HidDevice device);
extern int usehid_keyboard_type(HidDevice device, const char* text);
extern int usehid_keyboard_press(HidDevice device, uint8_t keycode);
extern int usehid_keyboard_release(HidDevice device, uint8_t keycode);
extern int usehid_keyboard_combo(HidDevice device, uint8_t modifiers, uint8_t keycode);

extern HidDevice usehid_gamepad_create(const char* name);
extern void usehid_gamepad_destroy(HidDevice device);
extern int usehid_gamepad_press_button(HidDevice device, uint16_t button);
extern int usehid_gamepad_release_button(HidDevice device, uint16_t button);
extern int usehid_gamepad_left_stick(HidDevice device, uint8_t x, uint8_t y);
extern int usehid_gamepad_right_stick(HidDevice device, uint8_t x, uint8_t y);
extern int usehid_gamepad_triggers(HidDevice device, uint8_t left, uint8_t right);

// Agent API
extern void* usehid_agent_create(void);
extern void usehid_agent_destroy(void* agent);
extern char* usehid_agent_execute(void* agent, const char* json);
extern void usehid_free_string(char* s);
*/
import "C"

import (
	"encoding/json"
	"errors"
	"unsafe"
)

// Mouse button constants
type MouseButton uint8

const (
	ButtonLeft   MouseButton = 0x01
	ButtonRight  MouseButton = 0x02
	ButtonMiddle MouseButton = 0x04
)

// Key modifier constants
type Modifier uint8

const (
	ModCtrl  Modifier = 0x01
	ModShift Modifier = 0x02
	ModAlt   Modifier = 0x04
	dCmd   Modifier = 0x08
)

// Gamepad button constants
type GamepadButton uint16

const (
	GamepadA          GamepadButton = 0x0001
	GamepadB          GamepadButton = 0x0002
	GamepadX          GamepadButton = 0x0004
	GamepadY          GamepadButton = 0x0008
	GamepadLB         GamepadButton = 0x0010
	GamepadRB         GamepadButton = 0x0020
	GamepadBack       GamepadButton = 0x0040
	GamepadStart      GamepadButton = 0x0080
	GamepadGuide      GamepadButton = 0x0100
	GamepadLeftStick  GamepadButton = 0x0200
	GamepadRightStick GamepadButton = 0x0400
	GamepadDPadUp     GamepadButton = 0x0800
	GamepadDPadDown   GamepadButton = 0x1000
	GamepadDPadLeft   GamepadButton = 0x2000
	GamepadDPadRight  GamepadButton = 0x4000
)

// ErrNotImplemented is returned when the FFI library is not available
var ErrNotImplemented = errors.New("usehid: FFI library not linked")

// Mouse represents a virtual mouse device
type Mouse struct {
	// TODO: implement with CGO when FFI is ready
	name string
}

// NewMouse creates a new virtual mouse
func NewMouse() *Mouse {
	return NewMouseWithName("useHID Virtual Mouse")
}

// NewMouseWithName creates a new virtual mouse with a custom name
func NewMouseWithName(name string) *Mouse {
	return &Mouse{name: name}
}

// Close destroys the virtual mouse
func (m *Mouse) Close() error {
	// TODO: implement
	return nil
}

// MoveBy moves the mouse by a relative offset
func (m *Mouse) MoveBy(dx, dy int) error {
	// TODO: implemeCGO
	return ErrNotImplemented
}

// Click performs a mouse click
func (m *Mouse) Click(button MouseButton) error {
	return ErrNotImplemented
}

// DoubleClick performs a double click
func (m *Mouse) DoubleClick(button MouseButton) error {
	return ErrNotImplemented
}

// Press presses a mouse button
func (m *Mouse) Press(button MouseButton) error {
	return ErrNotImplemented
}

// Release releases a mouse button
func (m *Mouse) Release(button MouseButton) error {
	return ErrNotImplemented
}

// Scroll scrolls the mouse wheel
func (m *Mouse) Scroll(delta int8) error {
	return ErrNotImplemented
}

// Keyboard represents a virtual keyboard device
type Keyboard struct {
	name string
}

// NewKeyboard creates a new virtual keyboard
func NewKeyboard() *Keyboard {
	return NewKeyboardWithName("useHID Virtual Keyboard")
}

// NewKeyboardWithName creates a new virtual keyboard with a custom name
func NewKeyboardWithName(name string) *Keyboard {
	return &Keyboard{name: name}
}

// Close destroys the virtual keyboard
func (k *Keyboard) Close() error {
	return nil
}

// TypeText types a string
func (k *Keyboard) TypeText(text string) error {
	return ErrNotImplemented
}

// Press presses and releases a key
func (k *Keyboard) Press(key string) error {
	return ErrNotImplemented
}

// Combo presses a key combination
func (k *Keyboard) Combo(modifiers []Modifier, key string) error {
	return ErrNotImplemented
}

// Gamepad represents a virtual gamepad device
type Gamepad struct {
	name string
}

// NewGamepad creates a new virtual gamepad
func NewGamepad() *Gamepad {
	return NewGamepadWithName("useHID Virtual Gamepad")
}

// NewGamepadWithName creates a new virtual gamepad with a custom name
func NewGamepadWithName(name string) *Gamepad {
	return &Gamepad{name: name}
}

// Close destroys the virtual gamepad
func (g *Gamepad) Close() error {
	return nil
}

// Press presses a gamepad button
func (g *Gamepad) Press(button GamepadButton) error {
	return ErrNotImplemented
}

// Release releases a gamepad button
func (g *Gamepad) Release(button GamepadButton) error {
	return ErrNotImplemented
}

// Tap presses and releases a gamepad button
func (g *Gamepad) Tap(button GamepadButton) error {
	return ErrNotImplemented
}

// LeftStick sets the left stick position (0-255, 128=center)
func (g *Gamepad) LeftStick(x, y uint8) error {
	return ErrNotImplemented
}

// RightStick sets the right stick position (0-255, 128=center)
func (g *Gamepad) RightStick(x, y uint8) error {
	return ErrNotImplemented
}

// Triggers sets the trigger values (0-255)
func (g *Gamepad) Triggers(left, right uint8) error {
	return ErrNotImplemented
}

// AgentHID provides a JSON-based interface for LLM agents
type AgentHID struct {
	// TODO: implement with CGO
}

// AgentResult is the result of an agent action
type AgentResult struct {
	Success  bool        `json:"success"`
	Error    string      `json:"error,omitempty"`
	Data     string      `json:"data,omitempty"`
	Tree     *UIElement  `json:"tree,omitempty"`
	Elements []UIElement `json:"elements,omitempty"`
}

// UIElement represents an accessibility UI element
type UIElement struct {
	Role        string      `json:"role"`
	Title       string      `json:"title,omitempty"`
	Value       string      `json:"value,omitempty"`
	Description string      `json:"description,omitempty"`
	Bounds      *Bounds     `json:"bounds,omitempty"`
	Children    []UIElement `json:"children,omitempty"`
	Actions     []string    `json:"actions,omitempty"`
}

// Bounds represents the bounding rectangle of a UI element
type Bounds struct {
	X      float64 `json:"x"`
	Y      float64 `json:"y"`
	Width  float64 `json:"width"`
	Height float64 `json:"height"`
}

// NewAgentHID creates a new agent HID controller
func NewAgentHID() *AgentHID {
	return &AgentHID{}
}

// Close destroys the agent HID controller
func (a *AgentHID) Close() error {
	return nil
}

// Execute executes an action from a map
func (a *AgentHID) Execute(action map[string]interface{}) AgentResult {
	jsonBytes, err := json.Marshal(action)
	if err != nil {
		return AgentResult{Success: false, Error: err.Error()}
	}
	return a.ExecuteJSON(string(jsonBytes))
}

// ExecuteJSON executes an action from a JSON string
func (a *AgentHID) ExecuteJSON(jsonStr string) AgentResult {
	// TODO: implement with CGO
	return AgentResult{Success: false, Error: "not implemented"}
}

// Helper to combine modifiers
func CombineModifiers(mods ...Modifier) Modifier {
	var result Modifier
	for _, m := range mods {
		result |= m
	}
	return result
}

// Screenshot captures a full screenshot and returns PNG bytes
func Screenshot() ([]byte, error) {
	return nil, ErrNotImplemented
}

// ScreenshotRegion captures a region of the screen and returns PNG bytes
func ScreenshotRegion(x, y int, width, height uint) ([]byte, error) {
	return nil, ErrNotImplemented
}

// GetUITree retrieves the accessibility UI tree from the focused window
// depth limits traversal depth (0 = unlimited), app targets a specific app (empty = focused)
func GetUITree(depth int, app string) (*UIElement, error) {
	return nil, ErrNotImplemented
}

// FindUIElement finds UI elements matching the given role and/or title
func FindUIElement(role, title string) ([]UIElement, error) {
	return nil, ErrNotImplemented
}
