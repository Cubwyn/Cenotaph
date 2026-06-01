// src/engine/update.rs
// Per-frame update logic split into two passes:
//
//   update_physics(input)  — movement intent, physics step, gameplay
//   update_visuals(input)  — camera rotation, GPU buffer writes
//
// Per-frame update logic.

use glam::Vec3;

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
        // Tick the shared cooldown timer 
        if self.action_cooldown > 0.0 {
            self.action_cooldown = (self.action_cooldown - dt).max(0.0);
        }

        // ── Stamina & movement ────────────────────────────────────────────────
        let intent = {
            let v = self.camera_controller
                .get_movement_intent(input, &self.camera, &self.config_data);
            [v.x, v.y, v.z]
        };
        let is_jumping = input.is_key_down(self.config_data.key("jump"));
        let has_movement = (intent[0] * intent[0] + intent[2] * intent[2]) > 0.001;
        let sprint_held = input.is_key_down(self.config_data.key("sprint"));

        // ── Sprint logic (dash removed) ──────────────────────────────────────
        self.is_sprinting = sprint_held && has_movement && self.stamina > 0.0;

        // ── Stamina consumption (sprint only, dash removed) ───────────────────
        let mut target_speed_multiplier = 1.0_f32;
        if self.is_sprinting {
            target_speed_multiplier = self.config_data.player.sprint_speed / self.config_data.physics.player_speed;
            self.stamina -= self.config_data.movement.sprint_stamina_drain_rate * dt;
            self.stamina = self.stamina.max(0.0);
            self.stamina_regen_delay_timer = self.config_data.player.stamina_regen_delay;
        }

        // ── Smooth the speed multiplier ──────────────────────────────────────
        // This prevents instant jumps from 1.0 → 1.6 when sprinting starts
        let smooth_rate = 8.0_f32; // Higher = faster response
        let lerp = (smooth_rate * dt).min(1.0);
        self.speed_multiplier_smoothed += (target_speed_multiplier - self.speed_multiplier_smoothed) * lerp;
        let speed_multiplier = self.speed_multiplier_smoothed;

        // ── Smooth the displayed stamina (eliminates bar jump) ──────────────
        // The actual stamina changes instantly, which makes the bar jump.
        // We compute a smoothed stamina value (stamina_smoothed) that
        // interpolates toward the actual value, giving a smooth visual.
        let stamina_smooth_rate = 8.0_f32;
        let stamina_lerp = (stamina_smooth_rate * dt).min(1.0);
        self.stamina_smoothed += (self.stamina - self.stamina_smoothed) * stamina_lerp;
        self.stamina_smoothed = self.stamina_smoothed.clamp(0.0, self.config_data.player.max_stamina);

        // ── Stamina regeneration ──────────────────────────────────────────────
        if !self.is_sprinting {
            self.stamina_regen_delay_timer = (self.stamina_regen_delay_timer - dt).max(0.0);
            if self.stamina_regen_delay_timer <= 0.0 {
                self.stamina += self.config_data.player.stamina_regen_rate * dt;
                self.stamina = self.stamina.min(self.config_data.player.max_stamina);
            }
        }

        // ── Level transition check ────────────────────────────────────────────
        // Check if player is near a prop with trigger_level_id
        if self.pending_transition.is_none() {
            let player_pos = self.physics.get_player_pos();
            let player_v = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
            for prop in &self.level_data.props {
                if let Some(ref target_level) = prop.trigger_level_id {
                    let prop_pos = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
                    if player_v.distance(prop_pos) < 2.5 {
                        println!("[LEVEL] Transition to '{}' triggered", target_level);
                        self.pending_transition = Some(target_level.clone());
                        break;
                    }
                }
            }
        }

        // Process pending level transition
        if let Some(ref next_level) = self.pending_transition.clone() {
            println!("[LEVEL] Loading level: {}", next_level);
            self.load_level(next_level);
            self.pending_transition = None;
        }

        self.handle_gameplay_input(input);

        self.physics.apply_player_movement(
            intent,
            is_jumping,
            &self.config_data.physics,
            dt,
            speed_multiplier,
        );
        self.physics.step(&self.config_data.physics, dt);

        // Debug logging: print position every ~5 seconds (time-based, reduced frequency)
        self.debug_timer += dt;
        if self.debug_timer >= 5.0 {
            self.debug_timer -= 5.0;
            let pos = self.physics.get_player_pos();
            println!("[DEBUG] pos: ({:.1}, {:.1}, {:.1}) stamina: {:.0}/{:.0}",
                pos[0], pos[1], pos[2], self.stamina, self.config_data.player.max_stamina);
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
    /// Removes an enemy prop at `index` from level_data, physics, and render instances.
    fn remove_prop(&mut self, index: usize) {
        if index >= self.level_data.props.len() {
            return;
        }

        // Remove physics collider if it exists
        if index < self.physics.prop_colliders.len() {
            let handle = self.physics.prop_colliders.remove(index);
            self.physics.collider_set.remove(
                handle,
                &mut self.physics.island_manager,
                &mut self.physics.rigid_body_set,
                true,
            );
        }

        // Remove from level data
        let prop = self.level_data.props.remove(index);
        println!("[COMBAT] Destroyed prop '{}' at index {}", prop.asset_id, index);

        // Rebuild GPU instance buffers to match the new prop list
        self.sync_instances();
    }

    fn handle_gameplay_input(&mut self, input: &InputManager) {
        // Hit flash decay
        if self.hit_flash_timer > 0.0 {
            self.hit_flash_timer = (self.hit_flash_timer - 1.0 / 60.0).max(0.0);
        }

        // Primary fire from DeviceEvent (mouse button 0)
        if input.fire_primary && self.action_cooldown <= 0.0 {
            let ray_origin = self.camera.position;
            let ray_dir = self.camera.get_forward();

            let hit_idx = self
                .level_data
                .props
                .iter()
                .position(|p| {
                    p.enemy_type.is_some()
                        && p.enemy_health > 0.0
                        && ray_hits_sphere(
                            ray_origin,
                            ray_dir,
                            Vec3::new(p.position[0], p.position[1], p.position[2]),
                            2.0, // Increased hit radius for easier targeting
                        )
                });

            if let Some(idx) = hit_idx {
                self.hit_flash_timer = 0.15; // Brief visual feedback
                let prop = &mut self.level_data.props[idx];

                // Default damage per shot
                const DAMAGE_PER_SHOT: f32 = 25.0;
                prop.enemy_health -= DAMAGE_PER_SHOT;

                println!("[COMBAT] Hit! Enemy health: {:.0}", prop.enemy_health.max(0.0));

                if prop.enemy_health <= 0.0 {
                    self.remove_prop(idx);
                }
                self.action_cooldown = 0.25; // 4 shots per second
            } else {
                // Missed — short cooldown
                self.action_cooldown = 0.15;
            }
        }
    }
}
