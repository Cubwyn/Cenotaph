// src/input/manager.rs
// Captures raw keyboard, mouse button, mouse motion, and scroll events from
// winit and exposes a clean query interface to the rest of the engine.
#![allow(dead_code)]

use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

pub struct InputManager {
    keys_pressed: HashSet<KeyCode>,
    mouse_pressed: HashSet<MouseButton>,
    modifiers: ModifiersState,
    pub mouse_delta: (f64, f64),
    /// Accumulated scroll lines this frame (+ve = scroll up / forward).
    pub scroll_delta: f32,
    // Raw input states (pure input, no game logic)
    pub fire_primary: bool,
    pub fire_secondary: bool,
    pub reload: bool,
    pub dash: bool,
    pub aim: bool,
    pub weapon_swap: bool,
    pub selected_art: Option<u8>, // 1-4
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_pressed: HashSet::new(),
            modifiers: ModifiersState::default(),
            mouse_delta: (0.0, 0.0),
            scroll_delta: 0.0,
            fire_primary: false,
            fire_secondary: false,
            reload: false,
            dash: false,
            aim: false,
            weapon_swap: false,
            selected_art: None,
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
                        self.keys_pressed.insert(*keycode);
                        self.process_combat_keydown(*keycode);
                    }
                    ElementState::Released => { 
                        self.keys_pressed.remove(keycode);
                        self.process_combat_keyup(*keycode);
                    }
                }
                true
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
                true
            }

            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed  => { self.mouse_pressed.insert(*button); }
                    ElementState::Released => { self.mouse_pressed.remove(button); }
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
            DeviceEvent::Button { button: 0, state: ElementState::Pressed } => {
                self.fire_primary = true;
            }
            DeviceEvent::Button { button: 0, state: ElementState::Released } => {
                self.fire_primary = false;
            }
            DeviceEvent::Button { button: 1, state: ElementState::Pressed } => {
                self.fire_secondary = true;
            }
            DeviceEvent::Button { button: 1, state: ElementState::Released } => {
                self.fire_secondary = false;
            }
            _ => {}
        }
    }

    // ── Query interface ───────────────────────────────────────────────────────

    pub fn is_key_down(&self, keycode: Option<KeyCode>) -> bool {
        keycode.map_or(false, |k| self.keys_pressed.contains(&k))
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.modifiers.control_key()
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

    fn process_combat_keydown(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::KeyR => self.reload = true,
            KeyCode::ShiftLeft => self.dash = true,
            KeyCode::ControlLeft => self.aim = true,
            KeyCode::Tab => self.weapon_swap = true,
            KeyCode::Digit1 => self.selected_art = Some(1),
            KeyCode::Digit2 => self.selected_art = Some(2),
            KeyCode::Digit3 => self.selected_art = Some(3),
            KeyCode::Digit4 => self.selected_art = Some(4),
            _ => {}
        }
    }

    fn process_combat_keyup(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::KeyR => self.reload = false,
            KeyCode::ShiftLeft => self.dash = false,
            KeyCode::ControlLeft => self.aim = false,
            KeyCode::Tab => self.weapon_swap = false,
            _ => {}
        }
    }

    pub fn reset_combat_inputs(&mut self) {
        self.fire_primary = false;
        self.fire_secondary = false;
        self.reload = false;
        self.dash = false;
        self.aim = false;
        self.weapon_swap = false;
        self.selected_art = None;
    }
}
