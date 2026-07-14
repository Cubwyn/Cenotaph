// src/systems/render/camera.rs
// First-person camera: position, orientation, view-projection matrix,
// mouse-look controller, and movement-intent extraction.

use crate::data::config::gameplay::GameConfig;
use crate::systems::input::manager::InputManager;
use glam::{Mat4, Vec3};

// ── Camera ────────────────────────────────────────────────────────────────────

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    /// Unit vector pointing in the direction the camera faces.
    pub fn get_forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_to_rh(self.position, self.get_forward(), Vec3::Y);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

// ── GPU uniform ───────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

// ── Controller ────────────────────────────────────────────────────────────────

pub struct CameraController {
    sensitivity: f32,
}

impl CameraController {
    pub fn new(sensitivity: f32) -> Self {
        Self { sensitivity }
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }

    /// Apply raw mouse delta to camera yaw/pitch with smoothing.
    pub fn process_mouse(&self, dx: f64, dy: f64, camera: &mut Camera) {
        camera.yaw += dx as f32 * self.sensitivity;
        camera.pitch -= dy as f32 * self.sensitivity;
        let limit = 89.0_f32.to_radians();
        camera.pitch = camera.pitch.clamp(-limit, limit);
    }

    /// Compute a normalised horizontal movement intent from current key state.
    pub fn get_movement_intent(
        &self,
        input: &InputManager,
        camera: &Camera,
        config: &GameConfig,
    ) -> Vec3 {
        let forward = Vec3::new(camera.yaw.cos(), 0.0, camera.yaw.sin()).normalize();
        let right = Vec3::new(-camera.yaw.sin(), 0.0, camera.yaw.cos()).normalize();

        let mut intent = Vec3::ZERO;
        if input.is_key_down(config.key("forward")) {
            intent += forward;
        }
        if input.is_key_down(config.key("backward")) {
            intent -= forward;
        }
        if input.is_key_down(config.key("right")) {
            intent += right;
        }
        if input.is_key_down(config.key("left")) {
            intent -= right;
        }

        if intent.length_squared() > 0.0 {
            intent = intent.normalize();
        }
        intent
    }
}
