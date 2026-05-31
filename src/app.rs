// src/app.rs
// Winit ApplicationHandler — the only file that touches the OS event loop.
// It owns the EngineState and routes window/device events into the engine.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::core::engine::state::EngineState;
use crate::systems::input::manager::InputManager;

pub struct App {
    engine: Option<EngineState>,
    input: InputManager,
    last_frame: std::time::Instant,
    level_name: String,
}

impl App {
    pub fn new(level_name: String) -> Self {
        Self {
            engine: None,
            input: InputManager::new(),
            last_frame: std::time::Instant::now(),
            level_name,
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

            let engine = pollster::block_on(EngineState::new(
                window.clone(),
                self.level_name.clone(),
            ));
            self.engine = Some(engine);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };

        if !self.input.process_window_event(&event) {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),

                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(
                                winit::keyboard::KeyCode::Escape,
                            ),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => event_loop.exit(),

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
                        Err(wgpu::SurfaceError::Lost) => {
                            engine.resize(engine.size)
                        }
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
        self.input.process_device_event(&event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(engine) = &self.engine {
            engine.window.request_redraw();
        }
    }
}