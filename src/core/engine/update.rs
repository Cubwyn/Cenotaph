// src/engine/update.rs
// Per-frame update logic split into two passes:
//
//   update_physics(input)  — movement intent, physics step, gameplay
//   update_visuals(input)  — camera rotation, GPU buffer writes
//
// Per-frame update logic.

use std::collections::HashSet;

use glam::Vec3;

use crate::core::engine::state::{EngineState, GameMode};
use crate::data::relic::normalize_relic_id;
use crate::data::world::level::{
    ColliderType, LevelData, LevelEventActionData, LevelEventActionKind, LevelEventData,
    LevelEventTriggerKind, LevelPathData, LootEntryData, LootTableData, PropData,
};
use crate::game::combat::closest_ray_sphere_hit;
use crate::game::editor::{cursor_can_pick_prop, snap_position};
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
        self.editor.tick(dt);

        if input.was_key_pressed(self.config_data.key("editor_toggle")) {
            self.toggle_editor_mode();
        }
        if self.editor.enabled {
            self.update_editor_cursor();
            self.handle_editor_hot_reload(dt);
        }

        if !self.editor.enabled && self.handle_debug_input(input) {
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
        if !self.editor.enabled && self.pending_transition.is_none() {
            let player_pos = self.physics.get_player_pos();
            let player_v = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
            for prop in &self.level_data.props {
                if let Some(ref target_level) = prop.trigger_level_id {
                    if self.failed_transition.as_deref() == Some(target_level) {
                        continue;
                    }
                    let prop_pos = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
                    if player_v.distance(prop_pos) < 2.5 {
                        println!("[LEVEL] Transition to '{}' triggered", target_level);
                        self.pending_transition = Some(target_level.clone());
                        break;
                    }
                }
            }
        }

        if !self.editor.enabled && self.pending_transition.is_none() {
            self.update_level_events();
        }

        // Process pending level transition
        if !self.editor.enabled {
            if let Some(ref next_level) = self.pending_transition.clone() {
                println!("[LEVEL] Loading level: {}", next_level);
                match self.load_level(next_level) {
                    Ok(()) => {
                        if let Some(audio) = self.audio.as_ref() {
                            audio.play(crate::systems::audio::SoundEffect::LevelTransition);
                        }
                        self.feedback.on_transition();
                        self.autosave("level transition");
                    }
                    Err(error) => {
                        self.failed_transition = Some(next_level.clone());
                        self.feedback.on_debug();
                        eprintln!(
                            "[LEVEL] Transition to '{}' was rejected; current level remains active: {}",
                            next_level, error
                        );
                    }
                }
                self.pending_transition = None;
            }
        }

        // Skip movement + combat if player is dead
        if !self.player.is_dead {
            if !self.editor.enabled {
                self.update_enemy_ai(dt);
                self.update_non_enemy_path_followers();
            }

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
            if self.sync_dynamic_prop_positions_from_physics() {
                self.sync_instances();
            }

            if self.editor.enabled {
                self.update_editor_cursor();
                self.handle_editor_input(input);
            } else {
                self.update_progression_interactions();
                self.handle_gameplay_input(input);
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
            && !self.editor.enabled
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
        let scroll = input.take_scroll();
        if self.editor.enabled && scroll.abs() > f32::EPSILON {
            self.editor.adjust_distance(scroll.signum());
            self.update_editor_cursor();
        }

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

    fn level_file_path(&self) -> String {
        format!("levels/{}.json", self.level_name)
    }

    fn current_level_modified(&self) -> Option<std::time::SystemTime> {
        std::fs::metadata(self.level_file_path())
            .and_then(|metadata| metadata.modified())
            .ok()
    }

    fn toggle_editor_mode(&mut self) {
        self.editor.toggle();
        self.editor.clamp_selection(self.level_data.props.len());
        self.editor
            .set_known_file_modified(self.current_level_modified());
        self.update_editor_cursor();
        self.feedback.on_debug();
        println!(
            "[EDITOR] {} for level '{}' ({} prop(s))",
            if self.editor.enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            self.level_name,
            self.level_data.props.len()
        );
        if self.editor.enabled {
            self.editor_validate_level();
        }
    }

    fn update_editor_cursor(&mut self) {
        if !self.editor.enabled {
            return;
        }

        let player_pos = self.physics.get_player_pos();
        let origin = Vec3::new(player_pos[0], player_pos[1] + 1.0, player_pos[2]);
        let forward = self.camera.get_forward();
        let placement_distance = self
            .physics
            .weapon_obstruction_distance(origin, forward, self.editor.placement_distance)
            .unwrap_or(self.editor.placement_distance);
        let target = origin + forward * placement_distance;
        let snapped = snap_position([target.x, target.y, target.z], self.editor.grid_size);
        self.editor.set_cursor_position(snapped);
    }

    fn handle_editor_hot_reload(&mut self, dt: f32) {
        if !self.editor.should_check_hot_reload(dt) {
            return;
        }

        let modified = self.current_level_modified();
        if !self.editor.disk_changed(modified) {
            return;
        }

        if self.editor.dirty {
            self.editor.set_message("DISK CHANGED SAVE FIRST");
            println!(
                "[EDITOR] '{}' changed on disk, but editor has unsaved changes.",
                self.level_file_path()
            );
            return;
        }

        let level_name = self.level_name.clone();
        println!("[EDITOR] Hot reloading '{}'", self.level_file_path());
        match self.load_level(&level_name) {
            Ok(()) => {
                self.editor.enabled = true;
                self.editor.mark_reloaded(self.current_level_modified());
                self.update_editor_cursor();
                self.editor_validate_level();
            }
            Err(error) => {
                self.editor.enabled = true;
                self.editor.set_known_file_modified(modified);
                self.editor.set_message("HOT RELOAD FAILED CHECK CONSOLE");
                self.feedback.on_debug();
                eprintln!(
                    "[EDITOR] Hot reload rejected; current level remains active: {}",
                    error
                );
            }
        }
    }

    fn handle_editor_input(&mut self, input: &InputManager) {
        if input.was_key_pressed(self.config_data.key("editor_next_mode")) {
            self.editor.next_mode();
            self.feedback.on_debug();
        }
        if input.was_key_pressed(self.config_data.key("editor_next_template")) {
            self.editor.next_template();
            self.feedback.on_debug();
        }
        if input.was_key_pressed(self.config_data.key("editor_previous_template")) {
            self.editor.previous_template();
            self.feedback.on_debug();
        }
        if input.was_key_pressed(self.config_data.key("editor_select_next")) {
            self.editor.select_next(self.level_data.props.len());
            self.feedback.on_debug();
        }
        if input.was_key_pressed(self.config_data.key("editor_select_previous")) {
            self.editor.select_previous(self.level_data.props.len());
            self.feedback.on_debug();
        }
        if input.was_key_pressed(self.config_data.key("editor_place")) {
            self.editor_place_current();
        }
        if input.was_key_pressed(self.config_data.key("editor_delete")) {
            self.editor_delete_selected();
        }
        if input.was_key_pressed(self.config_data.key("editor_save")) {
            self.editor_save_level();
        }
        if input.was_key_pressed(self.config_data.key("editor_reload")) {
            self.editor_reload_level();
        }
        if input.was_key_pressed(self.config_data.key("editor_validate")) {
            self.editor_validate_level();
        }
    }

    fn editor_place_current(&mut self) {
        let template = *self.editor.current_template();
        let mut prop = template.prop_at(self.editor.cursor_position);
        self.materialize_editor_prop(&mut prop);
        let position = prop.position;

        self.add_runtime_prop(prop);
        self.sync_instances();
        let new_index = self.level_data.props.len().saturating_sub(1);
        self.editor.selected_prop = Some(new_index);
        self.editor.mark_dirty(format!("PLACED {}", template.label));
        self.feedback.on_debug_spawn_count(1);
        println!(
            "[EDITOR] Placed {} at ({:.1}, {:.1}, {:.1})",
            template.label, position[0], position[1], position[2]
        );
    }

    fn materialize_editor_prop(&self, prop: &mut PropData) {
        if let Some(enemy_type) = prop.enemy_type.as_deref() {
            if let Some(enemy) = self.enemy_registry.get(enemy_type) {
                prop.asset_id = enemy.model_asset.clone();
                prop.collider_type = enemy.collider_type;
                prop.enemy_health = enemy.health;
            }
        }

        if prop.anchor_id.as_deref() == Some("editor_anchor") {
            prop.anchor_id = Some(format!("editor_anchor_{}", self.level_data.props.len() + 1));
        }
    }

    fn editor_delete_selected(&mut self) {
        let index = self
            .editor
            .selected_prop
            .filter(|index| *index < self.level_data.props.len())
            .or_else(|| self.nearest_prop_to_editor_cursor());

        let Some(index) = index else {
            self.editor.set_message("NO PROP NEAR");
            self.feedback.on_debug();
            return;
        };

        let Some(prop) = self.remove_prop_data(index) else {
            self.editor.set_message("NO PROP NEAR");
            self.feedback.on_debug();
            return;
        };

        self.editor.clamp_selection(self.level_data.props.len());
        self.editor
            .mark_dirty(format!("REMOVED PROP {}", index + 1));
        self.feedback.on_debug();
        println!("[EDITOR] Removed prop {} '{}'", index, prop.asset_id);
    }

    fn nearest_prop_to_editor_cursor(&self) -> Option<usize> {
        let cursor = self.editor.cursor_position;
        let cursor_v = Vec3::new(cursor[0], cursor[1], cursor[2]);
        self.level_data
            .props
            .iter()
            .enumerate()
            .filter(|(_, prop)| cursor_can_pick_prop(cursor, prop))
            .map(|(index, prop)| {
                let pos = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
                (index, pos.distance_squared(cursor_v))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
    }

    fn editor_save_level(&mut self) {
        if let Err(errors) = self.level_data.validate() {
            self.editor_apply_validation_errors(&errors, "SAVE BLOCKED");
            return;
        }

        let path = self.level_file_path();
        match self.level_data.save_to_path(&path) {
            Ok(()) => {
                let modified = self.current_level_modified();
                self.editor.mark_saved(modified);
                self.feedback.on_debug_reload();
                println!("[EDITOR] Saved {}", path);
            }
            Err(error) => {
                self.editor.set_message("SAVE FAILED CHECK CONSOLE");
                self.feedback.on_debug();
                eprintln!("[EDITOR] Save failed for '{}': {}", path, error);
            }
        }
    }

    fn editor_reload_level(&mut self) {
        if !self.editor.request_reload() {
            self.feedback.on_debug();
            return;
        }

        let level_name = self.level_name.clone();
        match self.load_level(&level_name) {
            Ok(()) => {
                self.editor.enabled = true;
                self.editor.mark_reloaded(self.current_level_modified());
                self.update_editor_cursor();
                self.editor_validate_level();
                println!("[EDITOR] Reloaded {}", self.level_file_path());
            }
            Err(error) => {
                self.editor.enabled = true;
                self.editor.set_message("RELOAD FAILED CHECK CONSOLE");
                self.feedback.on_debug();
                eprintln!(
                    "[EDITOR] Reload rejected; current level remains active: {}",
                    error
                );
            }
        }
    }

    fn editor_validate_level(&mut self) -> bool {
        match self.level_data.validate() {
            Ok(()) => {
                self.editor.mark_validation_passed();
                self.feedback.on_debug_reload();
                println!(
                    "[EDITOR] Validation passed for '{}' ({} prop(s))",
                    self.level_name,
                    self.level_data.props.len()
                );
                true
            }
            Err(errors) => {
                self.editor_apply_validation_errors(&errors, "VALIDATION");
                false
            }
        }
    }

    fn editor_apply_validation_errors(&mut self, errors: &[String], message_prefix: &str) {
        let issue_count = errors.len();
        self.editor.mark_validation_failed(issue_count);
        self.editor.set_message(format!(
            "{} {} {}",
            message_prefix,
            issue_count,
            Self::editor_issue_word(issue_count)
        ));
        self.feedback.on_debug();
        eprintln!(
            "[EDITOR] {} failed for '{}' with {} {}:",
            message_prefix,
            self.level_name,
            issue_count,
            Self::editor_issue_word(issue_count).to_ascii_lowercase()
        );
        for error in errors {
            eprintln!("  - {}", error);
        }
    }

    fn editor_issue_word(issue_count: usize) -> &'static str {
        if issue_count == 1 {
            "ISSUE"
        } else {
            "ISSUES"
        }
    }

    fn update_level_events(&mut self) {
        if self.level_data.events.is_empty() {
            return;
        }

        if self.level_event_fired.len() != self.level_data.events.len() {
            self.level_event_fired = vec![false; self.level_data.events.len()];
        }

        let player_pos = self.physics.get_player_pos();
        let player = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
        let mut queued_actions = Vec::new();
        let mut should_autosave = false;

        for (index, event) in self.level_data.events.iter().enumerate() {
            if event.once && self.level_event_fired.get(index).copied().unwrap_or(false) {
                continue;
            }
            if !self.level_event_triggered(event, player) {
                continue;
            }

            if let Some(fired) = self.level_event_fired.get_mut(index) {
                *fired = true;
            }
            should_autosave |= event.once;
            println!("[EVENT] Fired '{}'", event.id);
            queued_actions.extend(event.actions.iter().cloned());
        }

        for action in queued_actions {
            should_autosave |= self.execute_level_event_action(action);
            if self.pending_transition.is_some() {
                break;
            }
        }

        if should_autosave {
            self.autosave("level event");
        }
    }

    fn level_event_triggered(&self, event: &LevelEventData, player: Vec3) -> bool {
        Self::level_event_triggered_with_flags(event, player, &self.level_flags)
    }

    fn level_event_triggered_with_flags(
        event: &LevelEventData,
        player: Vec3,
        level_flags: &HashSet<String>,
    ) -> bool {
        if let Some(flag_id) = event.trigger.flag_id.as_deref() {
            if !level_flags.contains(flag_id) {
                return false;
            }
        }

        match event.trigger.kind {
            LevelEventTriggerKind::OnEnter => true,
            LevelEventTriggerKind::Proximity => {
                let target = Vec3::new(
                    event.trigger.position[0],
                    event.trigger.position[1],
                    event.trigger.position[2],
                );
                player.distance(target) <= event.trigger.radius.max(0.0)
            }
            LevelEventTriggerKind::Interact | LevelEventTriggerKind::Manual => false,
        }
    }

    fn execute_level_event_action(&mut self, action: LevelEventActionData) -> bool {
        match action.kind {
            LevelEventActionKind::LoadLevel => {
                if let Some(target_level_id) = action.target_level_id {
                    println!("[EVENT] Queue level transition '{}'", target_level_id);
                    self.pending_transition = Some(target_level_id);
                    return true;
                }
            }
            LevelEventActionKind::GrantResource => {
                if action.resource_value == 0 {
                    return false;
                }
                let reward = self.cycle.resource_reward(action.resource_value);
                if self.progress.collect_resource(reward) {
                    self.feedback.on_resource_pickup(reward);
                    println!("[EVENT] Granted {} unsecured resource", reward);
                    return true;
                }
            }
            LevelEventActionKind::SpawnLoot => {
                let Some(loot_table_id) = action.loot_table_id.as_deref() else {
                    return false;
                };
                let fallback = self.physics.get_player_pos();
                let spawn_position = action.spawn_position.unwrap_or(fallback);
                return self.spawn_loot_from_table(loot_table_id, spawn_position);
            }
            LevelEventActionKind::StartDialogue => {
                if let Some(dialogue_id) = action.dialogue_id.as_deref() {
                    self.start_level_dialogue(dialogue_id);
                }
            }
            LevelEventActionKind::SetFlag => {
                if let Some(flag_id) = action.flag_id {
                    println!("[EVENT] Set flag '{}'", flag_id);
                    let inserted = self.level_flags.insert(flag_id);
                    self.feedback.on_debug();
                    return inserted;
                }
            }
        }
        false
    }

    fn spawn_loot_from_table(&mut self, loot_table_id: &str, position: [f32; 3]) -> bool {
        let Some(table) = self
            .level_data
            .loot_tables
            .iter()
            .find(|table| table.id == loot_table_id)
            .cloned()
        else {
            eprintln!("[EVENT] Missing or empty loot table '{}'", loot_table_id);
            self.feedback.on_debug();
            return false;
        };

        let seed = Self::stable_loot_seed(loot_table_id, self.level_data.props.len() as u64);
        let entries = Self::loot_entries_for_rolls(&table, seed);
        if entries.is_empty() {
            eprintln!(
                "[EVENT] Loot table '{}' had no spawnable entries",
                loot_table_id
            );
            self.feedback.on_debug();
            return false;
        }

        let mut spawned = 0;
        for entry in entries {
            let count = entry.quantity.max(1);
            for _ in 0..count {
                let offset = spawned as f32 * 0.55;
                let prop_position = [position[0] + offset, position[1], position[2]];
                let prop = Self::loot_entry_prop(&entry, prop_position);
                self.add_runtime_prop(prop);
                spawned += 1;
            }
        }
        self.sync_instances();
        self.feedback.on_debug_loot_count(spawned);
        println!(
            "[EVENT] Spawned {} loot prop(s) from table '{}'",
            spawned, loot_table_id
        );
        true
    }

    fn loot_entries_for_rolls(table: &LootTableData, seed: u64) -> Vec<LootEntryData> {
        let total_weight: u32 = table.entries.iter().map(|entry| entry.weight).sum();
        if table.rolls == 0 || total_weight == 0 {
            return Vec::new();
        }

        (0..table.rolls)
            .filter_map(|roll| Self::weighted_loot_entry(table, seed.wrapping_add(roll as u64)))
            .cloned()
            .collect()
    }

    fn weighted_loot_entry(table: &LootTableData, seed: u64) -> Option<&LootEntryData> {
        let total_weight: u32 = table.entries.iter().map(|entry| entry.weight).sum();
        if total_weight == 0 {
            return None;
        }

        let mut pick = (seed % total_weight as u64) as u32;
        for entry in &table.entries {
            if entry.weight == 0 {
                continue;
            }
            if pick < entry.weight {
                return Some(entry);
            }
            pick -= entry.weight;
        }

        table.entries.iter().find(|entry| entry.weight > 0)
    }

    fn stable_loot_seed(loot_table_id: &str, salt: u64) -> u64 {
        loot_table_id
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325_u64 ^ salt, |hash, byte| {
                hash.wrapping_mul(0x0000_0100_0000_01b3) ^ byte as u64
            })
    }

    fn loot_entry_prop(entry: &LootEntryData, position: [f32; 3]) -> PropData {
        let item_id = entry.item_id.clone();
        let asset_id = item_id
            .as_deref()
            .map(Self::pickup_asset_for_item)
            .unwrap_or("pickups/resource_shard.obj")
            .to_string();
        PropData {
            id: None,
            asset_id,
            position,
            rotation: [0.0, 0.0, 0.0],
            scale: [0.35, 0.35, 0.35],
            collider_type: ColliderType::None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id,
            resource_value: entry.resource_value,
            anchor_id: None,
            enemy_type: None,
            enemy_health: 0.0,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            loot_table_id: None,
            path_id: None,
            dialogue_id: None,
            event_id: None,
        }
    }

    fn pickup_asset_for_item(item_id: &str) -> &'static str {
        match normalize_relic_id(item_id).as_str() {
            "ash_splinter" => "pickups/relic_ash_splinter.obj",
            "veil_cinder" => "pickups/relic_veil_cinder.obj",
            "chain_sigil" => "pickups/relic_chain_sigil.obj",
            _ => "pickups/relic_ash_splinter.obj",
        }
    }

    fn start_level_dialogue(&mut self, dialogue_id: &str) {
        let Some(dialogue) = self
            .level_data
            .dialogues
            .iter()
            .find(|dialogue| dialogue.id == dialogue_id)
        else {
            eprintln!("[DIALOGUE] Missing dialogue '{}'", dialogue_id);
            self.feedback.on_debug();
            return;
        };

        for line in &dialogue.lines {
            println!("[DIALOGUE] {}: {}", dialogue.speaker, line);
        }
        self.feedback.on_debug();
    }

    fn handle_debug_input(&mut self, input: &InputManager) -> bool {
        if input.was_key_pressed(self.config_data.key("debug_reload_level")) {
            let level_name = self.level_name.clone();
            println!("[RELOAD] Preparing runtime data for '{}'", level_name);
            match self.reload_runtime_content() {
                Ok(()) => {
                    self.player.restore_after_respawn(&self.config_data.player);
                    self.feedback.on_debug_reload();
                    println!(
                        "[RELOAD] Applied config, bindings, registries, and level; player restored to {:.0}/{:.0} health",
                        self.player.health.current, self.player.health.max
                    );
                }
                Err(error) => {
                    self.feedback.on_debug();
                    eprintln!(
                        "[RELOAD] Rejected; the current runtime state remains active: {}",
                        error
                    );
                }
            }
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
            "[DEBUG] Controls: I cycle relic, F1 status, F2 heal, F3 damage 25, F4 set health to 1, F5 reload runtime data, F6 respawn loot, F7 Ashbound, F8 Burdened, F9 Censer, F10 Chainrunner, F11 Harpy, F12 clear enemies"
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
            id: None,
            asset_id: enemy.model_asset.clone(),
            position: [spawn.x, spawn.y, spawn.z],
            rotation: [0.0, 0.0, 0.0],
            scale: Self::debug_enemy_scale(&enemy.id),
            collider_type: enemy.collider_type,
            brush_geometry: None,
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
            loot_table_id: None,
            path_id: None,
            dialogue_id: None,
            event_id: None,
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
        let level_data = match LevelData::try_load(&level_path) {
            Ok(level) => level,
            Err(error) => {
                self.feedback.on_debug();
                eprintln!(
                    "[DEBUG] Loot respawn rejected; current runtime state remains active: {}",
                    error
                );
                return;
            }
        };
        if let Err(errors) = level_data.validate() {
            self.feedback.on_debug();
            eprintln!(
                "[DEBUG] Loot respawn rejected; '{}' failed validation: {}",
                level_path,
                errors.join("; ")
            );
            return;
        }
        if let Some(unknown_item) = level_data
            .props
            .iter()
            .filter_map(|prop| prop.item_id.as_deref())
            .find(|item_id| self.relic_registry.get(item_id).is_none())
        {
            self.feedback.on_debug();
            eprintln!(
                "[DEBUG] Loot respawn rejected; unknown item_id '{}'",
                unknown_item
            );
            return;
        }
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
        let save = SaveData::from_runtime_with_level_state(
            &self.level_name,
            &self.progress,
            &self.equipped_relic,
            &self.cycle,
            self.fired_level_event_ids(),
            self.level_flags.iter().cloned().collect(),
        );
        match save.save_to_path(DEFAULT_SAVE_PATH) {
            Ok(()) => println!("[SAVE] Autosaved after {}", reason),
            Err(error) => eprintln!("[SAVE] {}", error),
        }
    }

    fn fired_level_event_ids(&self) -> Vec<String> {
        self.level_data
            .events
            .iter()
            .enumerate()
            .filter(|(index, _)| self.level_event_fired.get(*index).copied().unwrap_or(false))
            .map(|(_, event)| event.id.clone())
            .collect()
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
                    let velocity = self
                        .path_follow_velocity(index, enemy_pos, enemy.move_speed)
                        .unwrap_or((0.0, 0.0));
                    self.physics
                        .set_prop_horizontal_velocity(index, velocity.0, velocity.1);
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

    fn update_non_enemy_path_followers(&mut self) {
        self.ensure_enemy_runtime_matches_props();
        let followers: Vec<_> = self
            .level_data
            .props
            .iter()
            .enumerate()
            .filter(|(_, prop)| prop.enemy_type.is_none() && prop.path_id.is_some())
            .filter_map(|(index, prop)| {
                self.physics
                    .get_prop_pos(index)
                    .or(Some(prop.position))
                    .map(|position| (index, position))
            })
            .collect();

        for (index, position) in followers {
            let velocity = self
                .path_follow_velocity(index, position, 1.0)
                .unwrap_or((0.0, 0.0));
            self.physics
                .set_prop_horizontal_velocity(index, velocity.0, velocity.1);
        }
    }

    fn path_follow_velocity(
        &mut self,
        prop_index: usize,
        prop_position: [f32; 3],
        base_speed: f32,
    ) -> Option<(f32, f32)> {
        let path_id = self
            .level_data
            .props
            .get(prop_index)?
            .path_id
            .as_deref()?
            .to_string();
        let path = self
            .level_data
            .paths
            .iter()
            .find(|path| path.id == path_id)?
            .clone();
        let runtime = self.enemy_runtime.get_mut(prop_index)?;
        Self::path_velocity_for_runtime(runtime, &path, prop_position, base_speed)
    }

    fn path_velocity_for_runtime(
        runtime: &mut EnemyRuntimeState,
        path: &LevelPathData,
        prop_position: [f32; 3],
        base_speed: f32,
    ) -> Option<(f32, f32)> {
        if path.waypoints.len() < 2 || base_speed <= 0.0 {
            return None;
        }

        runtime.path_waypoint = runtime.path_waypoint.min(path.waypoints.len() - 1);
        let mut target = Vec3::new(
            path.waypoints[runtime.path_waypoint][0],
            path.waypoints[runtime.path_waypoint][1],
            path.waypoints[runtime.path_waypoint][2],
        );
        let position = Vec3::new(prop_position[0], prop_position[1], prop_position[2]);
        let mut delta = Vec3::new(target.x - position.x, 0.0, target.z - position.z);

        if delta.length() <= 0.35 {
            if runtime.path_waypoint + 1 < path.waypoints.len() {
                runtime.path_waypoint += 1;
            } else if path.looped {
                runtime.path_waypoint = 0;
            } else {
                return Some((0.0, 0.0));
            }

            target = Vec3::new(
                path.waypoints[runtime.path_waypoint][0],
                path.waypoints[runtime.path_waypoint][1],
                path.waypoints[runtime.path_waypoint][2],
            );
            delta = Vec3::new(target.x - position.x, 0.0, target.z - position.z);
        }

        let distance = delta.length();
        if distance <= 0.001 {
            return Some((0.0, 0.0));
        }

        let speed = base_speed * path.speed_multiplier.max(0.0);
        let direction = delta / distance;
        Some((direction.x * speed, direction.z * speed))
    }

    fn sync_dynamic_prop_positions_from_physics(&mut self) -> bool {
        let mut changed = false;

        for (index, prop) in self.level_data.props.iter_mut().enumerate() {
            if prop.enemy_type.is_none() && prop.path_id.is_none() {
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
            let obstruction_distance = self
                .physics
                .weapon_obstruction_distance(ray_origin, ray_dir, range);
            let target_range = obstruction_distance
                .map(|distance| (distance - 0.05).max(0.0))
                .unwrap_or(range)
                .min(range);

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
                target_range,
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
                if obstruction_distance.is_some() {
                    self.feedback.on_shot_blocked();
                    println!("[COMBAT] Shot blocked by solid world geometry");
                } else {
                    self.feedback.on_shot_missed();
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::world::level::{LevelEventTriggerData, LevelPathKind};

    fn test_path(looped: bool) -> LevelPathData {
        LevelPathData {
            id: "test_path".to_string(),
            kind: LevelPathKind::Enemy,
            looped,
            speed_multiplier: 0.5,
            waypoints: vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
        }
    }

    #[test]
    fn path_velocity_advances_from_reached_waypoint() {
        let path = test_path(true);
        let mut runtime = EnemyRuntimeState::default();

        let velocity =
            EngineState::path_velocity_for_runtime(&mut runtime, &path, [0.0, 0.0, 0.0], 2.0)
                .unwrap();

        assert_eq!(runtime.path_waypoint, 1);
        assert!((velocity.0 - 1.0).abs() < 0.001);
        assert!(velocity.1.abs() < 0.001);
    }

    #[test]
    fn path_velocity_stops_at_end_of_non_looping_path() {
        let path = test_path(false);
        let mut runtime = EnemyRuntimeState {
            path_waypoint: 1,
            ..EnemyRuntimeState::default()
        };

        let velocity =
            EngineState::path_velocity_for_runtime(&mut runtime, &path, [4.0, 0.0, 0.0], 2.0)
                .unwrap();

        assert_eq!(velocity, (0.0, 0.0));
        assert_eq!(runtime.path_waypoint, 1);
    }

    #[test]
    fn flagged_level_event_waits_for_matching_flag() {
        let event = LevelEventData {
            id: "flagged_event".to_string(),
            once: true,
            trigger: LevelEventTriggerData {
                kind: LevelEventTriggerKind::OnEnter,
                position: [0.0, 0.0, 0.0],
                radius: 2.5,
                prop_id: None,
                flag_id: Some("gate_open".to_string()),
            },
            actions: Vec::new(),
        };
        let mut flags = HashSet::new();

        assert!(!EngineState::level_event_triggered_with_flags(
            &event,
            Vec3::ZERO,
            &flags
        ));

        flags.insert("gate_open".to_string());
        assert!(EngineState::level_event_triggered_with_flags(
            &event,
            Vec3::ZERO,
            &flags
        ));
    }

    #[test]
    fn loot_rolls_respect_weights_and_roll_count() {
        let table = LootTableData {
            id: "weighted".to_string(),
            rolls: 3,
            entries: vec![
                LootEntryData {
                    weight: 1,
                    item_id: None,
                    resource_value: 10,
                    quantity: 1,
                },
                LootEntryData {
                    weight: 2,
                    item_id: None,
                    resource_value: 20,
                    quantity: 1,
                },
            ],
        };

        let entries = EngineState::loot_entries_for_rolls(&table, 0);

        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.resource_value)
                .collect::<Vec<_>>(),
            vec![10, 20, 20]
        );
    }
}
