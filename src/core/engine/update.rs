// src/engine/update.rs
// Per-frame update logic split into two passes:
//
//   update_physics(input)  — movement intent, physics step, gameplay
//   update_visuals(input)  — camera rotation, GPU buffer writes
//
// Per-frame update logic.

use glam::Vec3;

use crate::core::engine::state::{EngineState, GameMode};
use crate::data::world::level::{LevelData, PropData};
use crate::game::combat::closest_ray_sphere_hit;
use crate::game::enemy::{advance_enemy_attack, enemy_ai_intent, EnemyAiIntent, EnemyRuntimeState};
use crate::game::save::{SaveData, DEFAULT_SAVE_PATH};
use crate::systems::audio::SoundEffect;
use crate::systems::input::manager::InputManager;
use crate::systems::render::mesh::try_load_model;

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
        self.feedback.tick(dt);

        if self.handle_debug_input(input) {
            return;
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
            self.feedback.on_transition();
            self.load_level(next_level);
            self.autosave("level transition");
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
                self.play_sound(SoundEffect::Pickup);
                self.player.restore_after_respawn(&self.config_data.player);
                // Reset physics body to spawn position
                let spawn = self
                    .progress
                    .respawn_position_or(self.level_data.player_spawn);
                self.reset_player_body_to(spawn);
                println!(
                    "[RESPAWN] Player respawned at ({:.1}, {:.1}, {:.1}) with {:.0}/{:.0} health",
                    spawn[0],
                    spawn[1],
                    spawn[2],
                    self.player.health.current,
                    self.player.health.max
                );
            }
        }

        // ── Hurtbox proximity damage ───────────────────────────────────────────
        if !self.player.is_dead
            && !self.player.health.is_depleted()
            && self.player.hurtbox_cooldown <= 0.0
        {
            let player_pos = self.physics.get_player_pos();
            let player_v = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);

            let hurtbox_hit = self
                .level_data
                .props
                .iter()
                .enumerate()
                .filter(|(_, prop)| prop.is_hurtbox)
                .find_map(|(index, prop)| {
                    let prop_pos = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
                    let distance = player_v.distance(prop_pos);
                    (distance < self.config_data.combat.hurtbox_radius).then(|| {
                        (
                            index,
                            prop.asset_id.clone(),
                            distance,
                            self.config_data.combat.hurtbox_damage_per_second * dt,
                        )
                    })
                });

            if let Some((index, asset_id, distance, damage)) = hurtbox_hit {
                let source = format!("hurtbox {} '{}' at {:.2}m", index, asset_id, distance);
                self.player.hurtbox_cooldown = self.config_data.combat.hurtbox_tick_interval;
                self.apply_player_damage(&source, damage);
            }
        }

        if self.config_data.debug.position_log_enabled {
            self.debug_timer += dt;
            if self.debug_timer >= self.config_data.debug.position_log_interval {
                self.debug_timer -= self.config_data.debug.position_log_interval;
                let pos = self.physics.get_player_pos();
                println!(
                    "[DEBUG] pos ({:.1}, {:.1}, {:.1}) | hp {:.0}/{:.0} | stamina {:.0}/{:.0} | enemies {} | loot {} | res {}/{} | cycle {}",
                    pos[0],
                    pos[1],
                    pos[2],
                    self.player.health.current,
                    self.player.health.max,
                    self.player.stamina.current,
                    self.player.stamina.max,
                    self.debug_enemy_count(),
                    self.debug_loot_count(),
                    self.progress.unsecured_resource,
                    self.progress.banked_resource,
                    self.cycle.number
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
        self.camera.position =
            Vec3::new(p[0], p[1] + 1.0, p[2]) + self.feedback.camera_offset(self.camera.yaw);

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

    fn handle_debug_input(&mut self, input: &InputManager) -> bool {
        if input.was_key_pressed(self.config_data.key("debug_reload_level")) {
            let level_name = self.level_name.clone();
            println!("[DEBUG] Reloading level '{}'", level_name);
            self.load_level(&level_name);
            self.player.restore_after_respawn(&self.config_data.player);
            self.feedback.on_debug_reload();
            println!(
                "[DEBUG] Level '{}' reloaded; player restored to {:.0}/{:.0} health",
                self.level_name, self.player.health.current, self.player.health.max
            );
            return true;
        }

        if input.was_key_pressed(self.config_data.key("debug_help")) {
            self.feedback.on_debug();
            self.debug_print_status();
        }
        if input.was_key_pressed(self.config_data.key("debug_heal_player")) {
            self.debug_heal_player();
        }
        if input.was_key_pressed(self.config_data.key("debug_damage_player")) {
            self.apply_player_damage("debug damage", 25.0);
        }
        if input.was_key_pressed(self.config_data.key("debug_set_player_low_health")) {
            self.debug_set_player_health(1.0);
        }
        if input.was_key_pressed(self.config_data.key("debug_respawn_loot")) {
            self.debug_respawn_loot();
        }
        if input.was_key_pressed(self.config_data.key("debug_spawn_ashbound")) {
            self.debug_spawn_enemy("ashbound");
        }
        if input.was_key_pressed(self.config_data.key("debug_spawn_burdened")) {
            self.debug_spawn_enemy("burdened");
        }
        if input.was_key_pressed(self.config_data.key("debug_spawn_censer")) {
            self.debug_spawn_enemy("censer");
        }
        if input.was_key_pressed(self.config_data.key("debug_spawn_chainrunner")) {
            self.debug_spawn_enemy("chainrunner");
        }
        if input.was_key_pressed(self.config_data.key("debug_spawn_harpy")) {
            self.debug_spawn_enemy("harpy");
        }
        if input.was_key_pressed(self.config_data.key("debug_clear_enemies")) {
            self.debug_clear_enemies();
        }

        false
    }

    fn debug_print_status(&self) {
        let pos = self.physics.get_player_pos();
        let enemy_count = self
            .level_data
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some() && prop.enemy_health > 0.0)
            .count();
        let loot_count = self
            .level_data
            .props
            .iter()
            .filter(|prop| Self::is_loot_prop(prop))
            .count();

        println!(
            "[DEBUG] Controls: I cycle relic, F1 status, F2 heal, F3 damage 25, F4 set health to 1, F5 reload level, F6 respawn loot, F7 Ashbound, F8 Burdened, F9 Censer, F10 Chainrunner, F11 Harpy, F12 clear enemies"
        );
        println!(
            "[DEBUG] Level '{}' | pos ({:.1}, {:.1}, {:.1}) | health {:.0}/{:.0} | stamina {:.0}/{:.0} | props {} | enemies {} | loot {} | res {}/{} | cycle {}",
            self.level_name,
            pos[0],
            pos[1],
            pos[2],
            self.player.health.current,
            self.player.health.max,
            self.player.stamina.current,
            self.player.stamina.max,
            self.level_data.props.len(),
            enemy_count,
            loot_count,
            self.progress.unsecured_resource,
            self.progress.banked_resource,
            self.cycle.number
        );
    }

    fn debug_heal_player(&mut self) {
        let before = self.player.health.current;
        if self.player.is_dead {
            self.player.restore_after_respawn(&self.config_data.player);
            let spawn = self
                .progress
                .respawn_position_or(self.level_data.player_spawn);
            self.reset_player_body_to(spawn);
        } else {
            self.player
                .health
                .restore_full(self.config_data.player.max_health);
            self.player.hurtbox_cooldown = 0.0;
            self.player.respawn_timer = 0.0;
        }
        self.feedback.on_heal();
        println!(
            "[DEBUG] Player healed ({:.0} -> {:.0}/{:.0})",
            before, self.player.health.current, self.player.health.max
        );
    }

    fn debug_set_player_health(&mut self, health: f32) {
        let before = self.player.health.current;
        if self.player.is_dead && health > 0.0 {
            self.player.restore_after_respawn(&self.config_data.player);
            let spawn = self
                .progress
                .respawn_position_or(self.level_data.player_spawn);
            self.reset_player_body_to(spawn);
        }

        let target = health.clamp(0.0, self.player.health.max);
        self.player.health.current = target;
        self.player.is_dead = target <= 0.0;
        if !self.player.is_dead {
            self.player.respawn_timer = 0.0;
        }
        self.player.flash_hit(0.15);
        self.feedback
            .on_player_damage_amount((before - target).max(0.0));
        println!(
            "[DEBUG] Player health set ({:.0} -> {:.0}/{:.0})",
            before, self.player.health.current, self.player.health.max
        );
    }

    fn debug_spawn_enemy(&mut self, enemy_type: &str) {
        let Some(enemy) = self.enemy_registry.get(enemy_type).cloned() else {
            eprintln!("[DEBUG] Cannot spawn unknown enemy '{}'", enemy_type);
            return;
        };

        let player_pos = self.physics.get_player_pos();
        let forward = self.camera.get_forward();
        let horizontal = Vec3::new(forward.x, 0.0, forward.z);
        let direction = if horizontal.length_squared() > 0.001 {
            horizontal.normalize()
        } else {
            Vec3::Z
        };
        let spawn = Vec3::new(player_pos[0], player_pos[1], player_pos[2]) + direction * 8.0;
        let prop = PropData {
            asset_id: enemy.model_asset.clone(),
            position: [spawn.x, spawn.y, spawn.z],
            rotation: [0.0, 0.0, 0.0],
            scale: Self::debug_enemy_scale(&enemy.id),
            collider_type: enemy.collider_type.clone(),
            is_climbable: false,
            is_hurtbox: false,
            item_id: None,
            resource_value: 0,
            anchor_id: None,
            enemy_type: Some(enemy.id.clone()),
            enemy_health: enemy.health,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
        };

        self.add_runtime_prop(prop);
        self.sync_instances();
        self.feedback.on_debug_spawn_count(1);
        println!(
            "[DEBUG] Spawned {} at ({:.1}, {:.1}, {:.1}); enemies now {}",
            enemy.display_name,
            spawn.x,
            spawn.y,
            spawn.z,
            self.debug_enemy_count()
        );
    }

    fn debug_respawn_loot(&mut self) {
        let level_path = format!("levels/{}.json", self.level_name);
        let level_data = LevelData::load(&level_path);
        let mut restored = 0;

        for prop in level_data.props.into_iter().filter(Self::is_loot_prop) {
            if self
                .level_data
                .props
                .iter()
                .any(|existing| Self::same_loot_prop(existing, &prop))
            {
                continue;
            }

            self.add_runtime_prop(prop);
            restored += 1;
        }

        if restored > 0 {
            self.sync_instances();
            self.feedback.on_debug_loot_count(restored);
        } else {
            self.feedback.on_debug();
        }
        println!("[DEBUG] Respawned {} loot pickup(s)", restored);
    }

    fn debug_clear_enemies(&mut self) {
        let indexes: Vec<usize> = self
            .level_data
            .props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| prop.enemy_type.is_some().then_some(index))
            .collect();
        let count = indexes.len();

        for index in indexes.into_iter().rev() {
            self.remove_prop_data(index);
        }

        self.feedback.on_debug();
        println!("[DEBUG] Cleared {} enemy prop(s)", count);
    }

    fn add_runtime_prop(&mut self, prop: PropData) {
        let asset_path = format!("assets/{}", prop.asset_id);
        match try_load_model(&asset_path) {
            Ok((_vertices, _parts, points, indices)) => {
                self.physics.add_prop(&prop, &points, &indices);
            }
            Err(error) => {
                eprintln!(
                    "[DEBUG] Failed to load runtime prop model '{}': {}",
                    asset_path, error
                );
                self.physics.add_prop(&prop, &[], &[]);
            }
        }

        self.level_data.props.push(prop);
        self.enemy_runtime.push(EnemyRuntimeState::default());
    }

    fn debug_enemy_count(&self) -> usize {
        self.level_data
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some() && prop.enemy_health > 0.0)
            .count()
    }

    fn debug_loot_count(&self) -> usize {
        self.level_data
            .props
            .iter()
            .filter(|prop| Self::is_loot_prop(prop))
            .count()
    }

    fn debug_enemy_scale(enemy_id: &str) -> [f32; 3] {
        match enemy_id {
            "burdened" => [1.5, 1.5, 1.5],
            "censer" | "chainrunner" => [1.1, 1.1, 1.1],
            "ashbound" | "harpy" => [1.2, 1.2, 1.2],
            _ => [1.2, 1.2, 1.2],
        }
    }

    fn is_loot_prop(prop: &PropData) -> bool {
        prop.resource_value > 0
            || prop
                .item_id
                .as_ref()
                .is_some_and(|item_id| !item_id.trim().is_empty())
    }

    fn same_loot_prop(left: &PropData, right: &PropData) -> bool {
        left.asset_id == right.asset_id
            && left.item_id == right.item_id
            && left.resource_value == right.resource_value
            && Self::positions_close(left.position, right.position)
    }

    fn positions_close(left: [f32; 3], right: [f32; 3]) -> bool {
        left.iter()
            .zip(right.iter())
            .all(|(left, right)| (*left - *right).abs() <= 0.001)
    }

    fn apply_player_damage(&mut self, source: &str, amount: f32) -> bool {
        if self.player.is_dead || self.player.health.is_depleted() || amount <= 0.0 {
            return false;
        }

        let before = self.player.health.current;
        self.player.health.damage(amount);
        let after = self.player.health.current;
        println!(
            "[DAMAGE] Player took {:.1} from {} ({:.0} -> {:.0}/{:.0})",
            (before - after).max(0.0),
            source,
            before,
            after,
            self.player.health.max
        );

        self.player.flash_hit(0.2);
        self.feedback
            .on_player_damage_amount((before - after).max(0.0));

        if self.player.health.is_depleted() {
            self.defeat_player();
            true
        } else {
            false
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
        self.cycle.advance();
        self.player
            .begin_death(self.config_data.combat.respawn_delay);
        if lost > 0 {
            println!("[DEATH] Player defeated; lost {} unsecured resource", lost);
        } else {
            println!("[DEATH] Player defeated");
        }
        println!(
            "[CYCLE] Cycle {} active: {:?}",
            self.cycle.number, self.cycle.modifier
        );
        self.play_sound(SoundEffect::DeathSting);
        self.feedback.on_death();
        self.autosave("death");
    }

    fn autosave(&self, reason: &str) {
        let save = SaveData::from_runtime(
            &self.level_name,
            &self.progress,
            &self.equipped_relic,
            &self.cycle,
        );
        match save.save_to_path(DEFAULT_SAVE_PATH) {
            Ok(()) => println!("[SAVE] Autosaved after {}", reason),
            Err(error) => eprintln!("[SAVE] {}", error),
        }
    }

    fn grant_enemy_reward(&mut self, index: usize) {
        let Some(enemy_type) = self
            .level_data
            .props
            .get(index)
            .and_then(|prop| prop.enemy_type.as_deref())
            .map(ToOwned::to_owned)
        else {
            return;
        };
        let relic_id = self.cycle.reward_relic_id(&enemy_type);
        let Some(relic) = self.relic_registry.get(relic_id).cloned() else {
            eprintln!(
                "[LOOT] Enemy '{}' wanted missing relic '{}'",
                enemy_type, relic_id
            );
            return;
        };

        let acquisition = self.equipped_relic.acquire(relic.clone());
        if acquisition.acquired_new && acquisition.equipped {
            println!(
                "[LOOT] Enemy '{}' dropped '{}'; equipped",
                enemy_type, relic.display_name
            );
        } else if acquisition.acquired_new {
            println!(
                "[LOOT] Enemy '{}' dropped '{}'; stored in relic slot {}/{}",
                enemy_type, relic.display_name, acquisition.slot, acquisition.total
            );
        } else {
            println!(
                "[LOOT] Enemy '{}' reinforced '{}'; already owned in relic slot {}/{}",
                enemy_type, relic.display_name, acquisition.slot, acquisition.total
            );
        }

        self.play_sound(SoundEffect::Pickup);
        self.feedback.on_relic_changed();
        self.autosave("enemy reward");
    }

    fn acquire_relic_pickup(&mut self, relic: crate::data::relic::RelicDefinition) {
        let acquisition = self.equipped_relic.acquire(relic.clone());
        if acquisition.acquired_new && acquisition.equipped {
            println!("[RELIC] Acquired and equipped '{}'", relic.display_name);
        } else if acquisition.acquired_new {
            println!(
                "[RELIC] Acquired '{}' into slot {}/{}",
                relic.display_name, acquisition.slot, acquisition.total
            );
        } else {
            println!(
                "[RELIC] '{}' already owned in slot {}/{}",
                relic.display_name, acquisition.slot, acquisition.total
            );
        }

        self.play_sound(SoundEffect::Pickup);
        self.feedback.on_relic_changed();
        self.autosave("relic pickup");
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
                let reward = self.cycle.resource_reward(prop.resource_value);
                if self.progress.collect_resource(reward) {
                    println!(
                        "[RESOURCE] Collected {} unsecured resource ({}/{} banked)",
                        reward, self.progress.unsecured_resource, self.progress.banked_resource
                    );
                    self.play_sound(SoundEffect::Pickup);
                    self.feedback.on_resource_pickup(reward);
                    self.autosave("resource pickup");
                }
            }
        }

        let item_index = self
            .level_data
            .props
            .iter()
            .enumerate()
            .find(|(_, prop)| {
                prop.item_id
                    .as_ref()
                    .is_some_and(|item_id| !item_id.trim().is_empty())
                    && player_v.distance(Vec3::new(
                        prop.position[0],
                        prop.position[1],
                        prop.position[2],
                    )) < 2.0
            })
            .map(|(index, _)| index);

        if let Some(index) = item_index {
            if let Some(prop) = self.remove_prop_data(index) {
                if let Some(item_id) = prop.item_id.as_deref() {
                    if let Some(relic) = self.relic_registry.get(item_id).cloned() {
                        self.acquire_relic_pickup(relic);
                    } else {
                        eprintln!("[RELIC] Unknown item_id '{}'", item_id);
                    }
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
                self.feedback.on_pickup();
                self.autosave("anchor activation");
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
            runtime.tick(dt);
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
                EnemyAiIntent::Move {
                    velocity_x,
                    velocity_z,
                } => {
                    self.physics
                        .set_prop_horizontal_velocity(index, velocity_x, velocity_z);
                    if let Some(runtime) = self.enemy_runtime.get_mut(index) {
                        runtime.clear_windup();
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

                    let damage = self.cycle.enemy_damage(enemy.damage);
                    let source = format!("{} attack", enemy.display_name);
                    if self.apply_player_damage(&source, damage) {
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
        if input.was_key_pressed(self.config_data.key("inventory")) {
            self.cycle_equipped_relic();
        }

        // Primary fire from DeviceEvent (mouse button 0)
        if input.fire_primary && self.action_cooldown <= 0.0 {
            self.feedback.on_fire();
            let player_pos = self.physics.get_player_pos();
            let ray_origin = Vec3::new(player_pos[0], player_pos[1] + 1.0, player_pos[2]);
            let ray_dir = self.camera.get_forward();
            let damage = self
                .equipped_relic
                .damage(self.cycle.relic_damage(self.config_data.combat.base_damage));
            let range = self
                .equipped_relic
                .range(self.config_data.combat.primary_fire_range);
            let attack_cooldown = self
                .equipped_relic
                .cooldown(self.config_data.combat.attack_cooldown);
            let hit_stun = self
                .equipped_relic
                .hit_stun(self.config_data.combat.enemy_hit_stun);

            let targets = self
                .level_data
                .props
                .iter()
                .enumerate()
                .filter(|(_, prop)| prop.enemy_type.is_some() && prop.enemy_health > 0.0)
                .map(|(index, prop)| {
                    (
                        index,
                        Vec3::new(prop.position[0], prop.position[1], prop.position[2]),
                    )
                });

            let hit_idx = closest_ray_sphere_hit(
                ray_origin,
                ray_dir,
                targets,
                self.config_data.combat.enemy_hit_radius,
                range,
            );

            if let Some(idx) = hit_idx {
                self.player.flash_hit(0.15);
                let prop = &mut self.level_data.props[idx];
                let enemy_type = prop.enemy_type.as_deref().unwrap_or("enemy");
                let before = prop.enemy_health;

                prop.enemy_health -= damage;
                let after = prop.enemy_health.max(0.0);

                println!(
                    "[COMBAT] Hit {} for {:.1} ({:.0} -> {:.0})",
                    enemy_type, damage, before, after
                );

                if prop.enemy_health <= 0.0 {
                    self.grant_enemy_reward(idx);
                    self.feedback
                        .on_enemy_kill_amount((before - after).max(0.0));
                    self.remove_prop(idx);
                } else if let Some(runtime) = self.enemy_runtime.get_mut(idx) {
                    self.feedback.on_enemy_hit_amount((before - after).max(0.0));
                    runtime.stagger(hit_stun);
                }
                self.play_sound(SoundEffect::Hit);
                self.action_cooldown = attack_cooldown;
            } else {
                // Missed — short cooldown
                self.action_cooldown = self.config_data.combat.miss_cooldown;
            }
        }
    }

    fn cycle_equipped_relic(&mut self) {
        let owned_count = self.equipped_relic.owned_count();
        let Some(selection) = self.equipped_relic.cycle_next() else {
            println!(
                "[RELIC] Need at least two owned relics to cycle (owned: {})",
                owned_count
            );
            self.feedback.on_debug();
            return;
        };

        println!(
            "[RELIC] Equipped '{}' ({}/{})",
            selection.relic.display_name, selection.slot, selection.total
        );
        self.play_sound(SoundEffect::Pickup);
        self.feedback.on_relic_changed();
        self.autosave("relic swap");
    }
}
