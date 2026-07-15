// src/app.rs
// Winit ApplicationHandler — the only file that touches the OS event loop.
// It owns the EngineState and routes window/device events into the engine.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::core::engine::state::{EngineState, GameMode};
use crate::systems::input::manager::InputManager;

pub struct App {
    engine: Option<EngineState>,
    input: InputManager,
    last_frame: std::time::Instant,
    level_name: String,
    cursor_grabbed: bool,
    fatal_error: Option<String>,
}

impl App {
    pub fn new(level_name: String) -> Self {
        Self {
            engine: None,
            input: InputManager::new(),
            last_frame: std::time::Instant::now(),
            level_name,
            cursor_grabbed: false,
            fatal_error: None,
        }
    }

    pub fn take_fatal_error(&mut self) -> Option<String> {
        self.fatal_error.take()
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

        self.input.reset_all();

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
            let window = match event_loop.create_window(window_attrs) {
                Ok(window) => Arc::new(window),
                Err(error) => {
                    self.fatal_error = Some(format!("failed to create game window: {}", error));
                    event_loop.exit();
                    return;
                }
            };

            match pollster::block_on(EngineState::new(window.clone(), self.level_name.clone())) {
                Ok(engine) => {
                    Self::grab_cursor(&window);
                    self.cursor_grabbed = true;
                    self.engine = Some(engine);
                }
                Err(error) => {
                    Self::release_cursor(&window);
                    self.fatal_error = Some(format!(
                        "{} Run `cargo run -- doctor` for a complete project diagnosis.",
                        error
                    ));
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };

        let pause_pressed = matches!(
            &event,
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } if Some(*keycode) == engine.config_data.key("pause")
        );
        if pause_pressed {
            self.input.process_window_event(&event);
            self.toggle_pause_state();
            return;
        }

        if !self.input.process_window_event(&event) {
            match event {
                WindowEvent::CloseRequested => {
                    Self::release_cursor(&engine.window);
                    event_loop.exit();
                }

                WindowEvent::Focused(false) => {
                    self.input.reset_all();
                    if engine.game_mode == GameMode::Playing {
                        engine.game_mode = GameMode::Paused;
                        if let Some(audio) = engine.audio.as_ref() {
                            audio.pause_ambient();
                        }
                    }
                    Self::release_cursor(&engine.window);
                    self.cursor_grabbed = false;
                }

                WindowEvent::Resized(physical_size) => engine.resize(physical_size),

                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = now.duration_since(self.last_frame).as_secs_f32();
                    self.last_frame = now;
                    engine.record_frame_time(dt);

                    // Cap delta time to prevent physics issues on frame drops
                    let capped_dt = dt.min(1.0 / 30.0);

                    engine.update_physics(&self.input, capped_dt);
                    engine.update_visuals(&mut self.input);

                    match engine.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => engine.resize(engine.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            self.fatal_error = Some(
                                "graphics device ran out of memory while rendering".to_string(),
                            );
                            event_loop.exit();
                        }
                        Err(e) => eprintln!("[RENDER ERROR] {:?}", e),
                    }
                    self.input.end_frame();
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
