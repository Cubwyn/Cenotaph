// src/render/renderer.rs
// Renderer is a thin public facade over EngineState.
// app.rs only ever touches Renderer — it never reaches into EngineState directly.

use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::core::engine::state::EngineState;
use crate::systems::input::manager::InputManager;

pub struct Renderer {
    pub engine: EngineState,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, level_name: String) -> Self {
        Self {
            engine: EngineState::new(window, level_name).await,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.engine.render()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.engine.resize(new_size);
    }

    pub fn update_physics(&mut self, input: &InputManager, dt: f32) {
        self.engine.update_physics(input, dt);
    }

    pub fn update_visuals(&mut self, input: &mut InputManager) {
        self.engine.update_visuals(input);
    }
}
