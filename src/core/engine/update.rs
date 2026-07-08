// src/engine/update.rs
// Per-frame update logic split into two passes:
//
//   update_physics(input)  — movement intent, physics step, gameplay
//   update_visuals(input)  — camera rotation, GPU buffer writes
//
// Per-frame update logic.

use glam::Vec3;

use crate::core::engine::state::{EngineState, GameMode};
use crate::game::combat::ray_hits_sphere;
use crate::game::enemy::{advance_enemy_attack, enemy_ai_intent, EnemyAiIntent, EnemyRuntimeState};
use crate::systems::audio::SoundEffect;
use crate::systems::input::manager::InputManager;

// ── Public update interface ───────────────────────────────────────────────────

impl EngineState {
    // Physics pass: movement intent → physics step → gameplay logic.
    pub fn update_physics(&mut self, input: &InputManager, dt: f32) {
        if self.game_mode == GameMode::Paused {
            return;
        }

        // Tick the shared cooldown timer
        if self.action_cooldown > 0.0 {
            self.action_cooldown = (self.action_cooldown - dt).max(0.0);
        }

        // ── Stamina & movement ────────────────────────────────────────────────
        let intent = {
            let v =
                self.camera_controller
                    .get_movement_intent(input, &self.camera, &self.config_data);
            [v.x, v.y, v.z]
        };
        let is_jumping = input.is_key_down(self.config_data.key("jump"));
        let has_movement = (intent[0] * intent[0] + intent[2] * intent[2]) > 0.001;
        let sprint_held = input.is_key_down(self.config_data.key("sprint"));
        let dash_held = input.is_key_down(self.config_data.key("dash"));

        self.player.tick_timers(dt);
        self.player.update_dash_input(
            dash_held,
            has_movement,
            &self.config_data.movement,
            &self.config_data.player,
            intent,
        );

        // ── Sprint logic ─────────────────────────────────────────────────────
        self.player.is_sprinting = !self.player.is_dashing
            && sprint_held
            && has_movement
            && self.player.stamina.current > 0.0;

        if let Some(audio) = self.audio.as_mut() {
            let movement_mag = (intent[0] * intent[0] + intent[2] * intent[2]).sqrt();
            audio.tick_footsteps(
                dt,
                movement_mag,
                self.player.is_sprinting || self.player.is_dashing,
            );
        }

        // ── Stamina consumption ──────────────────────────────────────────────
        let walk_speed = self.config_data.player.walk_speed;
        let sprint_speed = self.config_data.player.sprint_speed;
        let dash_speed = sprint_speed * self.config_data.movement.dash_speed_multiplier;
        let mut target_speed = walk_speed;
        if self.player.is_dashing {
            target_speed = dash_speed;
        } else if self.player.is_sprinting {
            target_speed = sprint_speed;
            self.player
                .stamina
                .drain(self.config_data.movement.sprint_stamina_drain_rate * dt);
            self.player
                .stamina
                .delay_regen(self.config_data.player.stamina_regen_delay);
        }
        let physics_base_speed = self.config_data.physics.player_speed.max(0.001);
        let target_speed_multiplier = target_speed / physics_base_speed;

        // ── Smooth the speed multiplier ──────────────────────────────────────
        // This prevents instant jumps from 1.0 → 1.6 when sprinting starts
        let smooth_rate = 8.0_f32; // Higher = faster response
        let lerp = (smooth_rate * dt).min(1.0);
        self.player.speed_multiplier_smoothed +=
            (target_speed_multiplier - self.player.speed_multiplier_smoothed) * lerp;
        let speed_multiplier = self.player.speed_multiplier_smoothed;

        // ── Smooth the displayed stamina (eliminates bar jump) ──────────────
        // The actual stamina changes instantly, which makes the bar jump.
        // We compute a smoothed stamina value (stamina_smoothed) that
        // interpolates toward the actual value, giving a smooth visual.
        self.player.stamina.smooth_display(dt, 8.0);

        // ── Stamina regeneration ──────────────────────────────────────────────
        if !self.player.is_sprinting && !self.player.is_dashing {
            self.player
                .stamina
                .tick_regen(dt, self.config_data.player.stamina_regen_rate);
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
            if let Some(audio) = self.audio.as_ref() {
                audio.play(crate::systems::audio::SoundEffect::LevelTransition);
            }
            self.load_level(next_level);
            self.pending_transition = None;
        }

        // Skip movement + combat if player is dead
        if !self.player.is_dead {
            self.update_progression_interactions();
            self.handle_gameplay_input(input);
            self.update_enemy_ai(dt);

            self.physics.apply_player_movement(
                if self.player.is_dashing {
                    self.player.dash_direction
                } else {
                    intent
                },
                is_jumping,
                &self.config_data.physics,
                dt,
                speed_multiplier,
            );
            self.physics.step(&self.config_data.physics, dt);
            if self.sync_enemy_positions_from_physics() {
                self.sync_instances();
            }
        } else {
            // Dead state: process respawn timer
            self.player.respawn_timer -= dt;
            if self.player.respawn_timer <= 0.0 {
                // Respawn
                println!("[RESPAWN] Player respawned");
                self.play_sound(SoundEffect::Pickup);
                self.player.restore_after_respawn(&self.config_data.player);
                // Reset physics body to spawn position
                let spawn = self
                    .progress
                    .respawn_position_or(self.level_data.player_spawn);
                self.reset_player_body_to(spawn);
            }
        }

        // ── Hurtbox proximity damage ───────────────────────────────────────────
        if !self.player.is_dead
            && !self.player.health.is_depleted()
            && self.player.hurtbox_cooldown <= 0.0
        {
            let player_pos = self.physics.get_player_pos();
            let player_v = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
            for prop in &self.level_data.props {
                if prop.is_hurtbox {
                    let prop_pos = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
                    if player_v.distance(prop_pos) < self.config_data.combat.hurtbox_radius {
                        self.player
                            .health
                            .damage(self.config_data.combat.hurtbox_damage_per_second * dt);
                        self.player.flash_hit(0.2);
                        self.player.hurtbox_cooldown =
                            self.config_data.combat.hurtbox_tick_interval;
                        if self.player.health.is_depleted() {
                            self.defeat_player();
                        }
                        break;
                    }
                }
            }
        }

        if self.config_data.debug.position_log_enabled {
            self.debug_timer += dt;
            if self.debug_timer >= self.config_data.debug.position_log_interval {
                self.debug_timer -= self.config_data.debug.position_log_interval;
                let pos = self.physics.get_player_pos();
                println!(
                    "[DEBUG] pos: ({:.1}, {:.1}, {:.1}) stamina: {:.0}/{:.0}",
                    pos[0], pos[1], pos[2], self.player.stamina.current, self.player.stamina.max
                );
            }
        } else {
            self.debug_timer = 0.0;
        }
    }

    /// Visuals pass: mouse look → camera → GPU buffer write.
    pub fn update_visuals(&mut self, input: &mut InputManager) {
        let _ = input.take_scroll();

        if input.mouse_delta.0 != 0.0 || input.mouse_delta.1 != 0.0 {
            self.camera_controller.process_mouse(
                input.mouse_delta.0,
                input.mouse_delta.1,
                &mut self.camera,
            );
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
    fn play_sound(&self, effect: SoundEffect) {
        if let Some(audio) = self.audio.as_ref() {
            audio.play(effect);
        }
    }

    fn reset_player_body_to(&mut self, position: [f32; 3]) {
        let Some(body) = self
            .physics
            .rigid_body_set
            .get_mut(self.physics.player_body_handle)
        else {
            return;
        };

        use rapier3d::na::Translation3;
        let id = rapier3d::na::Isometry3::from_parts(
            Translation3::new(position[0], position[1], position[2]),
            rapier3d::na::UnitQuaternion::identity(),
        );
        body.set_position(id.into(), true);
        body.set_linvel(rapier3d::math::Vec3::splat(0.0), true);
    }

    fn defeat_player(&mut self) {
        if self.player.is_dead {
            return;
        }

        let lost = self.progress.lose_unsecured_on_death();
        self.player
            .begin_death(self.config_data.combat.respawn_delay);
        if lost > 0 {
            println!("[DEATH] Player defeated; lost {} unsecured resource", lost);
        } else {
            println!("[DEATH] Player defeated");
        }
        self.play_sound(SoundEffect::DeathSting);
    }

    fn remove_prop_data(&mut self, index: usize) -> Option<crate::data::world::level::PropData> {
        if index >= self.level_data.props.len() {
            return None;
        }

        self.physics.remove_prop(index);
        if index < self.enemy_runtime.len() {
            self.enemy_runtime.remove(index);
        }

        let prop = self.level_data.props.remove(index);
        self.sync_instances();
        Some(prop)
    }

    /// Removes an enemy prop at `index` from level_data, physics, and render instances.
    fn remove_prop(&mut self, index: usize) {
        let Some(prop) = self.remove_prop_data(index) else {
            return;
        };
        println!(
            "[COMBAT] Destroyed prop '{}' at index {}",
            prop.asset_id, index
        );
    }

    fn update_progression_interactions(&mut self) {
        let player_pos = self.physics.get_player_pos();
        let player_v = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);

        let resource_index = self
            .level_data
            .props
            .iter()
            .enumerate()
            .find(|(_, prop)| {
                prop.resource_value > 0
                    && player_v.distance(Vec3::new(
                        prop.position[0],
                        prop.position[1],
                        prop.position[2],
                    )) < 2.0
            })
            .map(|(index, _)| index);

        if let Some(index) = resource_index {
            if let Some(prop) = self.remove_prop_data(index) {
                if self.progress.collect_resource(prop.resource_value) {
                    println!(
                        "[RESOURCE] Collected {} unsecured resource ({}/{} banked)",
                        prop.resource_value,
                        self.progress.unsecured_resource,
                        self.progress.banked_resource
                    );
                    self.play_sound(SoundEffect::Pickup);
                }
            }
        }

        let anchor = self
            .level_data
            .props
            .iter()
            .filter_map(|prop| {
                let anchor_id = prop.anchor_id.as_ref()?;
                let position = prop.position;
                let distance = player_v.distance(Vec3::new(position[0], position[1], position[2]));
                (distance < 2.5).then(|| (anchor_id.clone(), position))
            })
            .next();

        if let Some((anchor_id, position)) = anchor {
            let activation = self.progress.activate_anchor(&anchor_id, position);
            if activation.newly_activated || activation.banked_amount > 0 {
                println!(
                    "[ANCHOR] '{}' active; banked {} resource (total banked: {})",
                    anchor_id, activation.banked_amount, self.progress.banked_resource
                );
                self.play_sound(SoundEffect::Pickup);
            }
        }
    }

    fn ensure_enemy_runtime_matches_props(&mut self) {
        self.enemy_runtime
            .resize(self.level_data.props.len(), EnemyRuntimeState::default());
    }

    fn update_enemy_ai(&mut self, dt: f32) {
        self.ensure_enemy_runtime_matches_props();

        for runtime in &mut self.enemy_runtime {
            runtime.attack_cooldown_remaining = (runtime.attack_cooldown_remaining - dt).max(0.0);
        }

        let player_pos = self.physics.get_player_pos();
        let player_v = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
        let enemies: Vec<_> = self
            .level_data
            .props
            .iter()
            .enumerate()
            .filter(|(_, prop)| prop.enemy_type.is_some() && prop.enemy_health > 0.0)
            .filter_map(|(index, prop)| {
                let enemy_type = prop.enemy_type.as_deref()?;
                let enemy = self.enemy_registry.get(enemy_type)?;
                Some((index, enemy.clone()))
            })
            .collect();

        for (index, enemy) in enemies {
            if self.player.is_dead || self.player.health.is_depleted() {
                self.physics.set_prop_horizontal_velocity(index, 0.0, 0.0);
                continue;
            }

            let Some(enemy_pos) = self
                .physics
                .get_prop_pos(index)
                .or_else(|| self.level_data.props.get(index).map(|prop| prop.position))
            else {
                continue;
            };
            let enemy_v = Vec3::new(enemy_pos[0], enemy_pos[1], enemy_pos[2]);

            match enemy_ai_intent(&enemy, enemy_v, player_v) {
                EnemyAiIntent::Idle => {
                    self.physics.set_prop_horizontal_velocity(index, 0.0, 0.0);
                    if let Some(runtime) = self.enemy_runtime.get_mut(index) {
                        runtime.attack_windup_remaining = 0.0;
                    }
                }
                EnemyAiIntent::Chase {
                    velocity_x,
                    velocity_z,
                } => {
                    self.physics
                        .set_prop_horizontal_velocity(index, velocity_x, velocity_z);
                    if let Some(runtime) = self.enemy_runtime.get_mut(index) {
                        runtime.attack_windup_remaining = 0.0;
                    }
                }
                EnemyAiIntent::Attack => {
                    self.physics.set_prop_horizontal_velocity(index, 0.0, 0.0);
                    let should_damage = self
                        .enemy_runtime
                        .get_mut(index)
                        .is_some_and(|runtime| advance_enemy_attack(runtime, &enemy, dt));
                    if !should_damage {
                        continue;
                    }

                    self.player.health.damage(enemy.damage);
                    self.player.flash_hit(0.2);

                    if self.player.health.is_depleted() {
                        self.defeat_player();
                        break;
                    }
                }
            }
        }
    }

    fn sync_enemy_positions_from_physics(&mut self) -> bool {
        let mut changed = false;

        for (index, prop) in self.level_data.props.iter_mut().enumerate() {
            if prop.enemy_type.is_none() {
                continue;
            }
            let Some(position) = self.physics.get_prop_pos(index) else {
                continue;
            };

            let current = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
            let next = Vec3::new(position[0], position[1], position[2]);
            if current.distance_squared(next) > 0.000001 {
                prop.position = position;
                changed = true;
            }
        }

        changed
    }

    fn handle_gameplay_input(&mut self, input: &InputManager) {
        // Primary fire from DeviceEvent (mouse button 0)
        if input.fire_primary && self.action_cooldown <= 0.0 {
            let ray_origin = self.camera.position;
            let ray_dir = self.camera.get_forward();

            let hit_idx = self.level_data.props.iter().position(|p| {
                p.enemy_type.is_some()
                    && p.enemy_health > 0.0
                    && ray_hits_sphere(
                        ray_origin,
                        ray_dir,
                        Vec3::new(p.position[0], p.position[1], p.position[2]),
                        self.config_data.combat.enemy_hit_radius,
                    )
            });

            if let Some(idx) = hit_idx {
                self.player.flash_hit(0.15);
                let prop = &mut self.level_data.props[idx];

                prop.enemy_health -= self.config_data.combat.base_damage;

                println!(
                    "[COMBAT] Hit! Enemy health: {:.0}",
                    prop.enemy_health.max(0.0)
                );

                if prop.enemy_health <= 0.0 {
                    self.remove_prop(idx);
                    self.play_sound(SoundEffect::Hit);
                }
                self.action_cooldown = self.config_data.combat.attack_cooldown;
            } else {
                // Missed — short cooldown
                self.action_cooldown = self.config_data.combat.miss_cooldown;
            }
        }
    }
}
