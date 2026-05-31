// src/engine/update.rs
// Per-frame update logic split into two passes:
//
//   update_physics(input)  — movement intent, physics step, gameplay
//   update_visuals(input)  — camera rotation, GPU buffer writes
//
// Per-frame update logic.

use glam::Vec3;
use winit::event::MouseButton;

use crate::core::engine::state::EngineState;
use crate::systems::input::manager::InputManager;

// ── Ray helper ────────────────────────────────────────────────────────────────

// Returns true if the ray (origin + t*dir) passes within `radius` of `center`.
fn ray_hits_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> bool {
    let oc = center - origin;
    let proj = oc.dot(dir);
    if proj < 0.0 { return false; }
    let closest = origin + dir * proj;
    closest.distance_squared(center) < radius * radius
}

// ── Public update interface ───────────────────────────────────────────────────

impl EngineState {
    // Physics pass: movement intent → physics step → gameplay logic.
    pub fn update_physics(&mut self, input: &InputManager, dt: f32) {
        // Tick the shared cooldown timer (time-based, not frame-based)
        if self.action_cooldown > 0.0 {
            self.action_cooldown = (self.action_cooldown - dt).max(0.0);
        }

        // ── Gameplay movement ─────────────────────────────────────────────────
        let intent = {
            let v = self.camera_controller
                .get_movement_intent(input, &self.camera, &self.config_data);
            [v.x, v.y, v.z]
        };
        let is_jumping = input.is_key_down(self.config_data.key("jump"));

        self.handle_gameplay_input(input);

        self.physics.apply_player_movement(
            intent,
            is_jumping,
            &self.config_data.physics,
            dt,
        );
        self.physics.step(&self.config_data.physics, dt);

        // Debug logging: print position every ~1 second (time-based)
        self.debug_timer += dt;
        if self.debug_timer >= 1.0 {
            self.debug_timer -= 1.0;
            let pos = self.physics.get_player_pos();
            println!("[DEBUG] Player pos: ({:.2}, {:.2}, {:.2})", pos[0], pos[1], pos[2]);
        }
    }

    /// Visuals pass: mouse look → camera → GPU buffer write.
    pub fn update_visuals(&mut self, input: &mut InputManager) {
        let _ = input.take_scroll();

        if input.mouse_delta.0 != 0.0 || input.mouse_delta.1 != 0.0 {
            self.camera_controller
                .process_mouse(input.mouse_delta.0, input.mouse_delta.1, &mut self.camera);
        }

        // In gameplay mode the camera follows the physics body.
        let p = self.physics.get_player_pos();
        self.camera.position = Vec3::new(p[0], p[1] + 1.0, p[2]);

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.update_lighting();
        input.reset_mouse_delta();
    }
}

// ── Gameplay input (always compiled) ─────────────────────────────────────────

impl EngineState {
    fn handle_gameplay_input(&mut self, input: &InputManager) {
        // Shoot (LMB)
        if input.is_mouse_down(MouseButton::Left) && self.action_cooldown <= 0.0 {
            let ray_origin = self.camera.position;
            let ray_dir = self.camera.get_forward();

            let hit = self
                .level_data
                .props
                .iter()
                .enumerate()
                .find(|(_, p)| {
                    p.enemy_type.is_some()
                        && ray_hits_sphere(
                            ray_origin,
                            ray_dir,
                            Vec3::new(p.position[0], p.position[1], p.position[2]),
                            1.5,
                        )
                })
                .map(|(i, _)| i);

            if let Some(_i) = hit {
                println!("[COMBAT] Hit registered, but combat system is currently disabled.");
                // ~10 frames at 60fps = 0.167s cooldown
                self.action_cooldown = 0.167;
            }
        }

        // Note: Combat system has been removed as it was half-baked.
        // Once enemy AI and combat are properly implemented, re-integrate damage here.
    }
}
