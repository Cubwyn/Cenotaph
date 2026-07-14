// src/systems/input/manager.rs
// Captures raw keyboard, mouse button, mouse motion, and scroll events from
// winit and exposes a clean query interface to the rest of the engine.

use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputManager {
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    pub mouse_delta: (f64, f64),
    /// Accumulated scroll lines this frame (+ve = scroll up / forward).
    pub scroll_delta: f32,
    // Raw input states (pure input, no game logic)
    pub fire_primary: bool,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            scroll_delta: 0.0,
            fire_primary: false,
        }
    }

    // ── Event ingestion ───────────────────────────────────────────────────────

    /// Returns `true` if the event was consumed (keyboard / mouse input).
    pub fn process_window_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        if self.keys_pressed.insert(*keycode) {
                            self.keys_just_pressed.insert(*keycode);
                        }
                    }
                    ElementState::Released => {
                        self.keys_pressed.remove(keycode);
                    }
                }
                true
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 20.0,
                };
                self.scroll_delta += lines;
                true
            }

            _ => false,
        }
    }

    pub fn process_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta.0 += delta.0;
                self.mouse_delta.1 += delta.1;
            }
            // Edge-triggered: only fire on the Pressed transition, not held
            DeviceEvent::Button {
                button: 0,
                state: ElementState::Pressed,
            } => {
                self.fire_primary = true;
            }
            DeviceEvent::Button {
                button: 0,
                state: ElementState::Released,
            } => {
                self.fire_primary = false;
            }
            _ => {}
        }
    }

    // ── Query interface ───────────────────────────────────────────────────────

    pub fn is_key_down(&self, keycode: Option<KeyCode>) -> bool {
        keycode.is_some_and(|k| self.keys_pressed.contains(&k))
    }

    pub fn was_key_pressed(&self, keycode: Option<KeyCode>) -> bool {
        keycode.is_some_and(|k| self.keys_just_pressed.contains(&k))
    }

    /// Consume and return the accumulated scroll delta for this frame.
    /// Returns a non-zero value only once per scroll event.
    pub fn take_scroll(&mut self) -> f32 {
        let v = self.scroll_delta;
        self.scroll_delta = 0.0;
        v
    }

    pub fn reset_mouse_delta(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn reset_all(&mut self) {
        self.keys_pressed.clear();
        self.keys_just_pressed.clear();
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
        self.fire_primary = false;
    }

    pub fn end_frame(&mut self) {
        self.keys_just_pressed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_all_clears_sticky_focus_loss_state() {
        let mut input = InputManager::new();
        input.keys_pressed.insert(KeyCode::KeyW);
        input.keys_just_pressed.insert(KeyCode::KeyW);
        input.mouse_delta = (12.0, -8.0);
        input.scroll_delta = 3.0;
        input.fire_primary = true;

        input.reset_all();

        assert!(input.keys_pressed.is_empty());
        assert!(input.keys_just_pressed.is_empty());
        assert_eq!(input.mouse_delta, (0.0, 0.0));
        assert_eq!(input.scroll_delta, 0.0);
        assert!(!input.fire_primary);
    }
}
