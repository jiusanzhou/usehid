// Stub implementation - actual implementation will come from napi-rs

class Mouse {
  constructor(name) {
    this.name = name || 'useHID Virtual Mouse';
    console.warn('useHID: Native module not loaded. Install native bindings.');
  }
  moveBy(dx, dy) { throw new Error('Native module not loaded'); }
  click(button) { throw new Error('Native module not loaded'); }
  doubleClick(button) { throw new Error('Native module not loaded'); }
  press(button) { throw new Error('Native module not loaded'); }
  release(button) { throw new Error('Native module not loaded'); }
  scroll(delta) { throw new Error('Native module not loaded'); }
  close() {}
}

class Keyboard {
  constructor(name) {
    this.name = name || 'useHID Virtual Keyboard';
    console.warn('useHID: Native module not loaded. Install native bindings.');
  }
  typeText(text) { throw new Error('Native module not loaded'); }
  press(key) { throw new Error('Native module not loaded'); }
  combo(modifiers, key) { throw new Error('Native module not loaded'); }
  releaseAll() { throw new Error('Native module not loaded'); }
  close() {}
}

class Gamepad {
  constructor(name) {
    this.name = name || 'useHID Virtual Gamepad';
    console.warn('useHID: Native module not loaded. Install native bindings.');
  }
  press(button) { throw new Error('Native module not loaded'); }
  release(button) { throw new Error('Native module not loaded'); }
  tap(button) { throw new Error('Native module not loaded'); }
  leftStick(x, y) { throw new Error('Native module not loaded'); }
  rightStick(x, y) { throw new Error('Native module not loaded'); }
  triggers(left, right) { throw new Error('Native module not loaded'); }
  reset() { throw new Error('Native module not loaded'); }
  close() {}
}

class AgentHID {
  constructor() {
    console.warn('useHID: Native module not loaded. Install native bindings.');
  }
  execute(action) { return { success: false, error: 'Native module not loaded' }; }
  executeJson(json) { return JSON.stringify({ success: false, error: 'Native module not loaded' }); }
  close() {}
}

function screenshot() { throw new Error('Native module not loaded'); }
function screenshotRegion(x, y, width, height) { throw new Error('Native module not loaded'); }
function getUiTree(depth, app) { throw new Error('Native module not loaded'); }
function findUiElement(role, title) { throw new Error('Native module not loaded'); }

// Try to load native module
let native;
try {
  native = require('./usehid.node');
} catch (e) {
  // Native module not available, use stubs
}

module.exports = native || { Mouse, Keyboard, Gamepad, AgentHID, screenshot, screenshotRegion, getUiTree, findUiElement };
