// src/app.rs
// Winit ApplicationHandler — the only file that touches the OS event loop.
// It owns the Renderer and routes window/device events into the engine.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::systems::input::manager::InputManager;
use crate::systems::render::renderer::Renderer;

pub struct App {
    renderer: Option<Renderer>,
    input: InputManager,
    last_frame: std::time::Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            renderer: None,
            input: InputManager::new(),
            last_frame: std::time::Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_none() {
            let window_attrs =
                Window::default_attributes().with_title("Cenotaph: The Great Omission");
            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create window"),
            );

            let renderer = pollster::block_on(Renderer::new(
                window.clone(),
                "ashwalk_01".to_string(),
            ));
            self.renderer = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        let Some(renderer) = self.renderer.as_mut() else {
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

                WindowEvent::Resized(physical_size) => renderer.resize(physical_size),

                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = now.duration_since(self.last_frame).as_secs_f32();
                    self.last_frame = now;
                    
                    // Cap delta time to prevent physics issues on frame drops
                    let capped_dt = dt.min(1.0 / 30.0);
                    
                    renderer.update_physics(&self.input, capped_dt);
                    renderer.update_visuals(&mut self.input);

                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            renderer.resize(renderer.engine.size)
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
        if let Some(renderer) = &self.renderer {
            renderer.engine.window.request_redraw();
        }
    }
}