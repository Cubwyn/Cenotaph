// src/app.rs
// Winit ApplicationHandler — the only file that touches the OS event loop.
// It owns the EngineState and routes window/device events into the engine.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::core::engine::state::{EngineState, GameMode};
use crate::systems::input::manager::InputManager;

pub struct App {
    engine: Option<EngineState>,
    input: InputManager,
    last_frame: std::time::Instant,
    level_name: String,
    cursor_grabbed: bool,
}

impl App {
    pub fn new(level_name: String) -> Self {
        Self {
            engine: None,
            input: InputManager::new(),
            last_frame: std::time::Instant::now(),
            level_name,
            cursor_grabbed: false,
        }
    }

    /// Attempt to grab the cursor (confine to window + hide).
    fn grab_cursor(window: &Window) {
        let _ = window.set_cursor_grab(CursorGrabMode::Confined);
        window.set_cursor_visible(false);
    }

    /// Release the cursor (un-confine + show).
    fn release_cursor(window: &Window) {
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
    }

    /// Toggle pause state. Escape now pauses/unpauses the game,
    /// and cursor visibility follows the game state.
    fn toggle_pause_state(&mut self) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };

        self.input.reset_combat_inputs();
        self.input.reset_mouse_delta();

        match engine.game_mode {
            GameMode::Playing => {
                engine.game_mode = GameMode::Paused;
                Self::release_cursor(&engine.window);
                self.cursor_grabbed = false;
                if let Some(audio) = engine.audio.as_ref() {
                    audio.pause_ambient();
                }
            }
            GameMode::Paused => {
                engine.game_mode = GameMode::Playing;
                Self::grab_cursor(&engine.window);
                self.cursor_grabbed = true;
                if let Some(audio) = engine.audio.as_ref() {
                    audio.resume_ambient();
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_none() {
            let window_attrs =
                Window::default_attributes().with_title("Cenotaph: The Great Omission");
            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create window"),
            );

            // Grab cursor immediately on window creation
            Self::grab_cursor(&window);
            self.cursor_grabbed = true;

            let engine =
                pollster::block_on(EngineState::new(window.clone(), self.level_name.clone()));
            self.engine = Some(engine);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };

        if !self.input.process_window_event(&event) {
            match event {
                WindowEvent::CloseRequested => {
                    Self::release_cursor(&engine.window);
                    event_loop.exit();
                }

                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    // Toggle pause on Escape
                    self.toggle_pause_state();
                }

                WindowEvent::Resized(physical_size) => engine.resize(physical_size),

                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = now.duration_since(self.last_frame).as_secs_f32();
                    self.last_frame = now;

                    // Cap delta time to prevent physics issues on frame drops
                    let capped_dt = dt.min(1.0 / 30.0);

                    engine.update_physics(&self.input, capped_dt);
                    engine.update_visuals(&mut self.input);

                    match engine.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => engine.resize(engine.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("[RENDER ERROR] {:?}", e),
                    }
                }

                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // Only process mouse motion when cursor is grabbed
        if self.cursor_grabbed {
            self.input.process_device_event(&event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(engine) = &self.engine {
            engine.window.request_redraw();
        }
    }
}
