//! Per-frame simulation and presentation updates.
//!
//! Simulation and gameplay run in `update_physics`; camera and GPU state are
//! synchronized separately in `update_visuals`.

use std::collections::HashSet;

use glam::Vec3;

use crate::core::engine::state::{
    ActiveDialogueState, EngineState, GameMode, ManualLevelEventStatus,
};
use crate::data::world::level::{
    ColliderType, LevelData, LevelEventActionData, LevelEventActionKind, LevelEventData,
    LevelEventTriggerKind, LevelPathData, LootEntryData, LootTableData, PropData,
    RUNTIME_LOOT_ID_PREFIX,
};
use crate::game::combat::closest_ray_sphere_hit;
use crate::game::enemy::{advance_enemy_attack, enemy_ai_intent, EnemyAiIntent, EnemyRuntimeState};
use crate::game::feedback::MotionSample;
use crate::game::mountain::ActiveMountainReaction;
use crate::game::progression::{ActiveAnchorRite, AnchorRiteChoice};
use crate::game::save::{LevelSaveSnapshot, SaveData, SavedRuntimeLoot, DEFAULT_SAVE_PATH};
use crate::systems::audio::SoundEffect;
use crate::systems::input::manager::InputManager;
use crate::systems::render::mesh::try_load_model;
use crate::systems::render::particles::ParticleBurst;

impl EngineState {
    /// Advances simulation, gameplay, and event state for one frame.
    pub fn update_physics(&mut self, input: &InputManager, dt: f32) {
        if self.game_mode == GameMode::Paused {
            return;
        }

        self.update_mountain_reaction(dt);
        let particle_center = Vec3::from_array(self.physics.get_player_pos()) + Vec3::Y;
        self.particles
            .update(&self.queue, &self.runtime_atmosphere, particle_center, dt);

        if self.action_cooldown > 0.0 {
            self.action_cooldown = (self.action_cooldown - dt).max(0.0);
        }
        self.feedback.tick(dt);
        let dialogue_finished = self
            .active_dialogue
            .as_mut()
            .is_some_and(|dialogue| dialogue.tick(dt));
        if dialogue_finished {
            self.active_dialogue = None;
        }
        if self.handle_debug_input(input) {
            return;
        }
        if self.active_anchor_rite.is_some() {
            self.update_anchor_rite(input);
            return;
        }

        // Resolve movement intent and stamina costs before stepping physics.
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
        let dash_started = self.player.update_dash_input(
            dash_held,
            has_movement,
            &self.config_data.movement,
            &self.config_data.player,
            intent,
        );
        if dash_started {
            self.feedback.on_dash();
            self.play_sound(SoundEffect::Dash);
            let origin = Vec3::from_array(self.physics.get_player_pos()) + Vec3::Y * 0.55;
            self.particles.spawn_burst(
                ParticleBurst::Dash,
                origin,
                Vec3::from_array(self.player.dash_direction),
            );
        }

        self.player.is_sprinting = !self.player.is_dashing
            && sprint_held
            && has_movement
            && self.player.stamina.current > 0.0;

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

        // Ease transitions between walking, sprinting, and dashing.
        let smooth_rate = 8.0_f32;
        let lerp = (smooth_rate * dt).min(1.0);
        self.player.speed_multiplier_smoothed +=
            (target_speed_multiplier - self.player.speed_multiplier_smoothed) * lerp;
        let speed_multiplier = self.player.speed_multiplier_smoothed;

        // Presentation smoothing does not delay the authoritative stamina value.
        self.player.stamina.smooth_display(dt, 8.0);

        if !self.player.is_sprinting && !self.player.is_dashing {
            self.player
                .stamina
                .tick_regen(dt, self.config_data.player.stamina_regen_rate);
        }

        // Queue proximity transitions before explicit world interactions.
        if self.pending_transition.is_none() {
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

        let mut world_interact_pressed = false;
        if self.pending_transition.is_none() {
            let interact_pressed = input.was_key_pressed(self.config_data.key("interact"));
            let dialogue_consumed = interact_pressed && self.active_dialogue.is_some();
            if dialogue_consumed {
                let dialogue_finished = self
                    .active_dialogue
                    .as_mut()
                    .is_some_and(ActiveDialogueState::advance);
                if dialogue_finished {
                    self.active_dialogue = None;
                }
            }
            let available_interact = interact_pressed && !dialogue_consumed;
            let event_consumed = self.update_level_events(available_interact);
            world_interact_pressed = available_interact && !event_consumed;
        }

        if let Some(ref next_level) = self.pending_transition.clone() {
            println!("[LEVEL] Loading level: {}", next_level);
            match self.load_level(next_level) {
                Ok(()) => {
                    if let Some(audio) = self.audio.as_mut() {
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

        if !self.player.is_dead {
            self.update_enemy_ai(dt);
            self.update_non_enemy_path_followers();

            let grounded_before = self.physics.is_player_grounded();
            let velocity_before = self.physics.get_player_velocity();
            let jumped = self.physics.apply_player_movement(
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
            let grounded_after = self.physics.is_player_grounded();
            let velocity_after = self.physics.get_player_velocity();
            let horizontal_speed = (velocity_after[0] * velocity_after[0]
                + velocity_after[2] * velocity_after[2])
                .sqrt();
            let landing_speed = if !grounded_before && grounded_after {
                (-velocity_before[1]).max(0.0)
            } else {
                0.0
            };
            self.feedback.update_motion(
                dt,
                MotionSample {
                    horizontal_speed,
                    walk_speed,
                    grounded: grounded_after,
                    sprinting: self.player.is_sprinting,
                    dashing: self.player.is_dashing,
                    landing_speed,
                },
            );
            if landing_speed > 2.0 {
                let position = Vec3::from_array(self.physics.get_player_pos()) - Vec3::Y * 0.85;
                self.particles
                    .spawn_burst(ParticleBurst::Land, position, Vec3::Y);
            }
            if let Some(audio) = self.audio.as_mut() {
                audio.tick_movement(
                    dt,
                    horizontal_speed / walk_speed.max(0.001),
                    self.player.is_sprinting || self.player.is_dashing,
                    grounded_after,
                    jumped,
                    landing_speed,
                );
            }
            if self.sync_dynamic_prop_positions_from_physics() {
                self.sync_dynamic_instances();
            }

            self.update_progression_interactions(world_interact_pressed);
            if self.active_anchor_rite.is_none() {
                self.handle_gameplay_input(input);
            }
        } else {
            self.feedback.update_motion(
                dt,
                MotionSample {
                    walk_speed,
                    ..MotionSample::default()
                },
            );
            self.player.respawn_timer -= dt;
            if self.player.respawn_timer <= 0.0 {
                self.play_sound(SoundEffect::Pickup);
                self.player.restore_after_respawn(&self.config_data.player);
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

        // Apply periodic proximity damage after movement resolves.
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

    /// Synchronizes camera presentation and GPU uniforms after simulation.
    pub fn update_visuals(&mut self, input: &mut InputManager) {
        input.take_scroll();

        if self.active_anchor_rite.is_none()
            && (input.mouse_delta.0 != 0.0 || input.mouse_delta.1 != 0.0)
        {
            self.camera_controller.process_mouse(
                input.mouse_delta.0,
                input.mouse_delta.1,
                &mut self.camera,
            );
        }

        let visual_dt = (self.frame_time_ms * 0.001).clamp(1.0 / 240.0, 1.0 / 20.0);
        let [visual_yaw, visual_pitch] = self.feedback.camera_rotation_offset();
        self.camera.visual_yaw_offset = visual_yaw;
        self.camera.visual_pitch_offset = visual_pitch;
        let target_fovy =
            crate::systems::render::camera::BASE_FOVY + self.feedback.camera_fov_offset();
        self.camera.fovy += (target_fovy - self.camera.fovy) * (1.0 - (-visual_dt * 10.0).exp());

        let p = self.physics.get_player_pos();
        self.camera.position =
            Vec3::new(p[0], p[1] + 1.0, p[2]) + self.feedback.camera_offset(self.camera.yaw);

        self.camera_uniform
            .update_view_proj(&self.camera, self.feedback.time);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.update_lighting();
        input.reset_mouse_delta();
    }
}

impl EngineState {
    fn play_sound(&mut self, effect: SoundEffect) {
        if let Some(audio) = self.audio.as_mut() {
            audio.play(effect);
        }
    }

    fn update_mountain_reaction(&mut self, dt: f32) {
        if self.mountain_reaction.is_none() {
            if let Some(next_reaction) = self.queued_mountain_reactions.pop_front() {
                self.start_mountain_reaction(&next_reaction);
            } else {
                self.runtime_atmosphere = self.level_data.atmosphere.clone();
                return;
            }
        }
        let Some(reaction) = self.mountain_reaction.as_mut() else {
            return;
        };

        let finished = reaction.tick(dt);
        self.runtime_atmosphere = reaction.atmosphere(&self.level_data.atmosphere);
        if !finished {
            return;
        }

        self.mountain_reaction = None;
        self.runtime_atmosphere = self.level_data.atmosphere.clone();
        if let Some(next_reaction) = self.queued_mountain_reactions.pop_front() {
            self.start_mountain_reaction(&next_reaction);
        } else if let Some(audio) = self.audio.as_mut() {
            audio.set_ambience(
                self.runtime_atmosphere.ambience_preset,
                self.runtime_atmosphere.ambience_volume,
            );
        }
        self.autosave("mountain reaction completion");
    }

    fn start_mountain_reaction(&mut self, reaction_id: &str) {
        if self
            .mountain_reaction
            .as_ref()
            .is_some_and(|reaction| reaction.id() == reaction_id)
            || self
                .queued_mountain_reactions
                .iter()
                .any(|queued| queued == reaction_id)
        {
            return;
        }
        let Some(profile) = self
            .level_data
            .mountain_reactions
            .iter()
            .find(|reaction| reaction.id == reaction_id)
            .cloned()
        else {
            eprintln!("[MOUNTAIN] Missing reaction profile '{}'", reaction_id);
            self.feedback.on_debug();
            return;
        };

        if self.mountain_reaction.is_some() {
            self.queued_mountain_reactions
                .push_back(reaction_id.to_string());
            println!("[MOUNTAIN] Reaction '{}' queued", reaction_id);
            return;
        }

        let ambience_preset = profile
            .ambience_preset
            .unwrap_or(self.level_data.atmosphere.ambience_preset);
        let ambience_volume = (self.level_data.atmosphere.ambience_volume
            * profile.ambience_volume_multiplier)
            .clamp(0.0, 1.0);
        self.mountain_reaction = Some(ActiveMountainReaction::new(profile));
        if let Some(audio) = self.audio.as_mut() {
            audio.set_ambience(ambience_preset, ambience_volume);
            audio.play(SoundEffect::MountainAnswer);
        }
        println!("[MOUNTAIN] Reaction '{}' began", reaction_id);
    }

    fn update_level_events(&mut self, interact_pressed: bool) -> bool {
        if self.level_data.events.is_empty() {
            self.queued_manual_level_events.clear();
            return false;
        }

        if self.level_event_fired.len() != self.level_data.events.len() {
            self.level_event_fired = vec![false; self.level_data.events.len()];
        }

        let player_pos = self.physics.get_player_pos();
        let player = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
        let interact_event_index = interact_pressed.then(|| {
            Self::nearest_interact_event_index(
                &self.level_data.events,
                &self.level_data.props,
                &self.level_event_fired,
                player,
                &self.level_flags,
            )
        });
        let interact_event_index = interact_event_index.flatten();
        let manual_event_ids = std::mem::take(&mut self.queued_manual_level_events);
        let mut queued_actions = Vec::new();
        let mut should_autosave = false;

        for (index, event) in self.level_data.events.iter().enumerate() {
            if event.once && self.level_event_fired.get(index).copied().unwrap_or(false) {
                continue;
            }
            if !Self::level_event_flag_ready(event, &self.level_flags) {
                continue;
            }

            let triggered = match event.trigger.kind {
                LevelEventTriggerKind::OnEnter | LevelEventTriggerKind::Proximity => {
                    Self::automatic_level_event_triggered(event, player)
                }
                LevelEventTriggerKind::Interact => interact_event_index == Some(index),
                LevelEventTriggerKind::Manual => manual_event_ids.contains(&event.id),
            };
            if !triggered {
                continue;
            }

            if let Some(fired) = self.level_event_fired.get_mut(index) {
                *fired = true;
            }
            should_autosave |= event.once;
            println!("[EVENT] Fired '{}'", event.id);
            queued_actions.extend(event.actions.iter().enumerate().map(
                |(action_index, action)| (format!("{}:{action_index}", event.id), action.clone()),
            ));
        }

        for (source_id, action) in queued_actions {
            should_autosave |= self.execute_level_event_action(action, &source_id);
            if self.pending_transition.is_some() {
                break;
            }
        }

        if should_autosave {
            self.autosave("level event");
        }
        interact_event_index.is_some()
    }

    /// Queues an authored manual event for the next gameplay event pass.
    /// Invalid IDs, automatic event kinds, unmet flags, and consumed one-shot
    /// events are rejected at the call site instead of failing silently later.
    #[allow(dead_code)] // Public integration point for upcoming scripted systems.
    pub fn queue_manual_level_event(&mut self, event_id: &str) -> Result<(), String> {
        match self.manual_level_event_status(event_id) {
            ManualLevelEventStatus::Ready => {}
            ManualLevelEventStatus::AlreadyFired => {
                return Err(format!(
                    "manual level event '{}' has already fired",
                    event_id
                ));
            }
            ManualLevelEventStatus::MissingFlag(flag_id) => {
                return Err(format!(
                    "manual level event '{}' requires flag '{}'",
                    event_id, flag_id
                ));
            }
            ManualLevelEventStatus::MissingEvent => {
                return Err(format!("manual level event '{}' does not exist", event_id));
            }
            ManualLevelEventStatus::WrongTrigger(kind) => {
                return Err(format!(
                    "level event '{}' is {:?}, not Manual",
                    event_id, kind
                ));
            }
        }

        self.queued_manual_level_events.insert(event_id.to_string());
        Ok(())
    }

    #[cfg(test)]
    fn level_event_triggered_with_flags(
        event: &LevelEventData,
        player: Vec3,
        level_flags: &HashSet<String>,
    ) -> bool {
        Self::level_event_flag_ready(event, level_flags)
            && Self::automatic_level_event_triggered(event, player)
    }

    fn level_event_flag_ready(event: &LevelEventData, level_flags: &HashSet<String>) -> bool {
        event
            .trigger
            .flag_id
            .as_deref()
            .is_none_or(|flag_id| level_flags.contains(flag_id))
    }

    fn automatic_level_event_triggered(event: &LevelEventData, player: Vec3) -> bool {
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

    pub(super) fn nearest_interact_event_index(
        events: &[LevelEventData],
        props: &[PropData],
        fired: &[bool],
        player: Vec3,
        level_flags: &HashSet<String>,
    ) -> Option<usize> {
        events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| {
                if event.trigger.kind != LevelEventTriggerKind::Interact
                    || (event.once && fired.get(index).copied().unwrap_or(false))
                    || !Self::level_event_flag_ready(event, level_flags)
                {
                    return None;
                }

                let prop_id = event.trigger.prop_id.as_deref()?;
                let prop = props
                    .iter()
                    .find(|prop| prop.id.as_deref() == Some(prop_id))?;
                let target = Vec3::from_array(prop.position);
                let distance = player.distance(target);
                (distance <= event.trigger.radius.max(0.0)).then_some((index, distance))
            })
            .min_by(
                |(left_index, left_distance), (right_index, right_distance)| {
                    left_distance
                        .total_cmp(right_distance)
                        .then_with(|| left_index.cmp(right_index))
                },
            )
            .map(|(index, _)| index)
    }

    fn execute_level_event_action(
        &mut self,
        action: LevelEventActionData,
        source_id: &str,
    ) -> bool {
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
                return self.spawn_loot_from_table(loot_table_id, spawn_position, source_id);
            }
            LevelEventActionKind::StartDialogue => {
                if let Some(dialogue_id) = action.dialogue_id.as_deref() {
                    self.start_level_dialogue(dialogue_id);
                }
            }
            LevelEventActionKind::SetFlag => {
                if let Some(flag_id) = action.flag_id {
                    println!("[EVENT] Set flag '{}'", flag_id);
                    return self.level_flags.insert(flag_id);
                }
            }
            LevelEventActionKind::ReactMountain => {
                if let Some(reaction_id) = action.reaction_id.as_deref() {
                    self.start_mountain_reaction(reaction_id);
                }
            }
        }
        false
    }

    fn spawn_loot_from_table(
        &mut self,
        loot_table_id: &str,
        position: [f32; 3],
        source_id: &str,
    ) -> bool {
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

        let seed = Self::stable_loot_seed(
            &self.level_name,
            self.cycle.number,
            loot_table_id,
            source_id,
        );
        let entries = Self::loot_entries_for_rolls(&table, seed);
        if entries.is_empty() {
            eprintln!(
                "[EVENT] Loot table '{}' had no spawnable entries",
                loot_table_id
            );
            self.feedback.on_debug();
            return false;
        }

        let mut slot = 0;
        let mut spawned = 0;
        let mut already_present = 0;
        for entry in entries {
            let count = entry.quantity.max(1);
            for _ in 0..count {
                let offset = slot as f32 * 0.55;
                let prop_position = [position[0] + offset, position[1], position[2]];
                let runtime_id = format!("{RUNTIME_LOOT_ID_PREFIX}{seed:016x}_{slot}");
                slot += 1;
                if self
                    .level_data
                    .props
                    .iter()
                    .any(|prop| prop.id.as_deref() == Some(runtime_id.as_str()))
                {
                    already_present += 1;
                    continue;
                }
                let prop =
                    Self::loot_entry_prop(&self.relic_registry, &entry, prop_position, runtime_id);
                self.add_runtime_prop(prop);
                spawned += 1;
            }
        }
        if spawned > 0 {
            self.sync_instances();
        }
        println!(
            "[EVENT] Manifested {} loot prop(s) from table '{}' ({} already present)",
            spawned, loot_table_id, already_present
        );
        spawned > 0 || already_present > 0
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

    fn stable_loot_seed(
        level_name: &str,
        cycle_number: u32,
        loot_table_id: &str,
        source_id: &str,
    ) -> u64 {
        level_name
            .bytes()
            .chain([0xff])
            .chain(cycle_number.to_le_bytes())
            .chain([0xfe])
            .chain(loot_table_id.bytes())
            .chain([0xfd])
            .chain(source_id.bytes())
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3)
            })
    }

    fn loot_entry_prop(
        relic_registry: &crate::data::relic::RelicRegistry,
        entry: &LootEntryData,
        position: [f32; 3],
        runtime_id: String,
    ) -> PropData {
        let item_id = entry.item_id.clone();
        let asset_id = item_id
            .as_deref()
            .and_then(|item_id| relic_registry.get(item_id))
            .map(|relic| relic.pickup_asset.as_str())
            .unwrap_or("pickups/resource_shard.obj")
            .to_string();
        PropData {
            id: Some(runtime_id),
            display_name: None,
            asset_id,
            position,
            rotation: [0.0, 0.0, 0.0],
            scale: [0.35, 0.35, 0.35],
            collider_type: ColliderType::None,
            surface_material: None,
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

    fn start_level_dialogue(&mut self, dialogue_id: &str) {
        let Some(dialogue) = self
            .level_data
            .dialogues
            .iter()
            .find(|dialogue| dialogue.id == dialogue_id)
            .cloned()
        else {
            eprintln!("[DIALOGUE] Missing dialogue '{}'", dialogue_id);
            self.feedback.on_debug();
            return;
        };

        for line in &dialogue.lines {
            println!("[DIALOGUE] {}: {}", dialogue.speaker, line);
        }
        self.active_dialogue = ActiveDialogueState::new(dialogue.speaker, dialogue.lines);
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
            self.debug_hud_enabled = !self.debug_hud_enabled;
            self.feedback.on_debug();
            self.debug_print_status();
            println!(
                "[DEBUG] Performance overlay {}",
                if self.debug_hud_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );
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
            "[DEBUG] Controls: I cycle relic, F1 performance/status, F2 heal, F3 damage 25, F4 set health to 1, F5 reload runtime data, F6 respawn loot, F7 Ashbound, F8 Burdened, F9 Censer, F10 Chainrunner, F11 Harpy, F12 clear enemies"
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
        self.play_sound(SoundEffect::Heal);
        self.particles.spawn_burst(
            ParticleBurst::Pickup,
            Vec3::from_array(self.physics.get_player_pos()) + Vec3::Y * 0.7,
            Vec3::Y,
        );
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
            display_name: None,
            asset_id: enemy.model_asset.clone(),
            position: [spawn.x, spawn.y, spawn.z],
            rotation: [0.0, 0.0, 0.0],
            scale: Self::debug_enemy_scale(&enemy.id),
            collider_type: enemy.collider_type,
            surface_material: None,
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

            if let Some(prop_id) = prop.id.as_deref() {
                self.removed_prop_ids.remove(prop_id);
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
        if let Some(prop_id) = prop.id.as_deref() {
            if self
                .level_data
                .props
                .iter()
                .any(|existing| existing.id.as_deref() == Some(prop_id))
            {
                eprintln!("[WORLD] Refused duplicate runtime prop id '{}'", prop_id);
                self.feedback.on_debug();
                return;
            }
        }
        let max_health = prop.enemy_type.as_ref().map_or(0.0, |_| prop.enemy_health);
        let asset_path = format!("assets/{}", prop.asset_id);
        match try_load_model(&asset_path) {
            Ok(model) => {
                self.physics
                    .add_prop(&prop, &model.physics_vertices, &model.physics_triangles);
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
        self.enemy_runtime
            .push(EnemyRuntimeState::for_max_health(max_health));
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
        self.particles.spawn_burst(
            ParticleBurst::Damage,
            Vec3::from_array(self.physics.get_player_pos()) + Vec3::Y * 0.65,
            -self.camera.get_forward(),
        );

        if self.player.health.is_depleted() {
            self.defeat_player();
            true
        } else {
            self.play_sound(SoundEffect::PlayerDamage);
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
        self.active_anchor_rite = None;
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

    pub(super) fn autosave(&self, reason: &str) {
        let save = SaveData::from_runtime_with_level_state(
            &self.level_name,
            &self.progress,
            &self.equipped_relic,
            &self.cycle,
            LevelSaveSnapshot {
                fired_level_events: self.fired_level_event_ids(),
                level_flags: self.level_flags.iter().cloned().collect(),
                removed_prop_ids: self.removed_prop_ids.iter().cloned().collect(),
                runtime_loot: self
                    .level_data
                    .props
                    .iter()
                    .filter_map(SavedRuntimeLoot::from_prop)
                    .collect(),
                pending_mountain_reactions: self
                    .mountain_reaction
                    .as_ref()
                    .map(|reaction| reaction.id().to_string())
                    .into_iter()
                    .chain(self.queued_mountain_reactions.iter().cloned())
                    .collect(),
            },
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
            .filter(|(index, event)| {
                event.once && self.level_event_fired.get(*index).copied().unwrap_or(false)
            })
            .map(|(_, event)| event.id.clone())
            .collect()
    }

    fn grant_enemy_reward(&mut self, index: usize) {
        let Some((enemy_type, source_id, authored_drop)) =
            self.level_data.props.get(index).and_then(|prop| {
                let enemy_type = prop.enemy_type.as_deref()?.to_string();
                let source_id = prop.id.clone();
                let authored_drop = prop
                    .loot_table_id
                    .as_ref()
                    .map(|table_id| (table_id.clone(), prop.position));
                Some((enemy_type, source_id, authored_drop))
            })
        else {
            return;
        };

        if let Some((loot_table_id, position)) = authored_drop {
            let Some(source_id) = source_id.as_deref() else {
                eprintln!(
                    "[LOOT] Enemy '{}' has authored loot but no stable prop id",
                    enemy_type
                );
                self.feedback.on_debug();
                return;
            };
            if self.spawn_loot_from_table(&loot_table_id, position, source_id) {
                println!(
                    "[LOOT] Enemy '{}' dropped authored table '{}'",
                    enemy_type, loot_table_id
                );
                return;
            }
            eprintln!(
                "[LOOT] Enemy '{}' could not spawn table '{}'; using cycle reward",
                enemy_type, loot_table_id
            );
        }

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

        let outcome = if !acquisition.acquired_new {
            "ALREADY BOUND"
        } else if acquisition.equipped {
            "EQUIPPED"
        } else {
            "STORED"
        };
        self.play_sound(SoundEffect::Pickup);
        self.feedback
            .on_relic_acquired(&relic.display_name, &relic.rarity, outcome);
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

        let outcome = if !acquisition.acquired_new {
            "ALREADY BOUND"
        } else if acquisition.equipped {
            "EQUIPPED"
        } else {
            "STORED"
        };
        self.play_sound(SoundEffect::Pickup);
        self.feedback
            .on_relic_acquired(&relic.display_name, &relic.rarity, outcome);
        self.autosave("relic pickup");
    }

    fn remove_prop_data(&mut self, index: usize) -> Option<PropData> {
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

    fn remove_persistent_prop_data(&mut self, index: usize) -> Option<PropData> {
        let prop = self.remove_prop_data(index)?;
        if let Some(prop_id) = prop.id.as_deref() {
            if !prop_id.starts_with(RUNTIME_LOOT_ID_PREFIX) {
                self.removed_prop_ids.insert(prop_id.to_string());
            }
        }
        Some(prop)
    }

    /// Removes one prop from level, physics, and render-aligned runtime storage.
    fn remove_prop(&mut self, index: usize) {
        let Some(prop) = self.remove_persistent_prop_data(index) else {
            return;
        };
        println!(
            "[COMBAT] Destroyed prop '{}' at index {}",
            prop.asset_id, index
        );
    }

    pub(super) fn nearest_anchor_prop_index(
        props: &[PropData],
        player: Vec3,
        radius: f32,
    ) -> Option<usize> {
        props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| {
                let anchor_id = prop.anchor_id.as_deref()?;
                if anchor_id.trim().is_empty() {
                    return None;
                }
                let distance = player.distance(Vec3::from_array(prop.position));
                (distance <= radius.max(0.0)).then_some((index, distance))
            })
            .min_by(
                |(left_index, left_distance), (right_index, right_distance)| {
                    left_distance
                        .total_cmp(right_distance)
                        .then_with(|| left_index.cmp(right_index))
                },
            )
            .map(|(index, _)| index)
    }

    fn update_anchor_rite(&mut self, input: &InputManager) {
        let select_previous = input.was_key_pressed(self.config_data.key("forward"))
            || input.was_key_pressed(self.config_data.key("left"));
        let select_next = input.was_key_pressed(self.config_data.key("backward"))
            || input.was_key_pressed(self.config_data.key("right"));
        if let Some(rite) = self.active_anchor_rite.as_mut() {
            if select_previous {
                rite.select_previous();
            } else if select_next {
                rite.select_next();
            }
        }

        if !input.was_key_pressed(self.config_data.key("interact")) {
            return;
        }
        let Some(rite) = self.active_anchor_rite.clone() else {
            return;
        };

        match rite.selected_choice() {
            AnchorRiteChoice::BindCinders => {
                let newly_activated =
                    self.progress.active_anchor_id.as_deref() != Some(rite.anchor_id.as_str());
                let ritual_event_queued = if newly_activated {
                    match rite.event_id.as_deref() {
                        Some(event_id) => match self.manual_level_event_status(event_id) {
                            ManualLevelEventStatus::Ready => {
                                self.queued_manual_level_events.insert(event_id.to_string());
                                true
                            }
                            ManualLevelEventStatus::AlreadyFired => false,
                            ManualLevelEventStatus::MissingFlag(_) => {
                                self.play_sound(SoundEffect::Blocked);
                                return;
                            }
                            ManualLevelEventStatus::MissingEvent
                            | ManualLevelEventStatus::WrongTrigger(_) => {
                                self.queue_prop_manual_event(event_id, "anchor claim");
                                self.play_sound(SoundEffect::Blocked);
                                return;
                            }
                        },
                        None => false,
                    }
                } else {
                    false
                };
                let activation = self
                    .progress
                    .activate_anchor(&rite.anchor_id, rite.position);
                self.active_anchor_rite = None;
                if activation.newly_activated || activation.banked_amount > 0 {
                    println!(
                        "[ANCHOR] '{}' claimed; bound {} Ash (total bound: {})",
                        rite.anchor_id, activation.banked_amount, self.progress.banked_resource
                    );
                    self.play_sound(if rite.event_id.is_some() {
                        SoundEffect::Pickup
                    } else {
                        SoundEffect::MountainAnswer
                    });
                    self.feedback.on_pickup();
                    self.particles.spawn_burst(
                        ParticleBurst::Pickup,
                        Vec3::from_array(rite.position) + Vec3::Y,
                        Vec3::Y,
                    );
                    if !ritual_event_queued {
                        self.autosave("Anchor rite");
                    }
                }
            }
            AnchorRiteChoice::MendVessel => {
                let cost = self.config_data.world.anchor_mend_cost;
                let vessel_wounded = self.player.health.current < self.player.health.max;
                if !vessel_wounded || !self.progress.spend_banked_resource(cost) {
                    self.play_sound(SoundEffect::Blocked);
                    return;
                }

                self.player
                    .health
                    .restore_full(self.config_data.player.max_health);
                self.player.hurtbox_cooldown = 0.0;
                self.feedback.on_heal();
                self.play_sound(SoundEffect::Heal);
                self.particles.spawn_burst(
                    ParticleBurst::Pickup,
                    Vec3::from_array(rite.position) + Vec3::Y * 0.8,
                    Vec3::Y,
                );
                self.active_anchor_rite = None;
                println!(
                    "[ANCHOR] The vessel was mended for {} Bound Ash (remaining: {})",
                    cost, self.progress.banked_resource
                );
                self.autosave("vessel mending rite");
            }
            AnchorRiteChoice::TurnAway => {
                self.active_anchor_rite = None;
                println!("[ANCHOR] The pilgrim turned away without making a claim");
            }
        }
    }

    fn queue_prop_manual_event(&mut self, event_id: &str, context: &str) -> bool {
        if self.manual_level_event_status(event_id) == ManualLevelEventStatus::AlreadyFired {
            return false;
        }
        match self.queue_manual_level_event(event_id) {
            Ok(()) => true,
            Err(error) => {
                eprintln!(
                    "[EVENT] Could not queue prop event '{}' after {}: {}",
                    event_id, context, error
                );
                self.feedback.on_debug();
                false
            }
        }
    }

    fn update_progression_interactions(&mut self, interact_pressed: bool) {
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
            if let Some(prop) = self.remove_persistent_prop_data(index) {
                self.particles.spawn_burst(
                    ParticleBurst::Pickup,
                    Vec3::from_array(prop.position) + Vec3::Y * 0.25,
                    Vec3::Y,
                );
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
            if let Some(prop) = self.remove_persistent_prop_data(index) {
                self.particles.spawn_burst(
                    ParticleBurst::Pickup,
                    Vec3::from_array(prop.position) + Vec3::Y * 0.35,
                    Vec3::Y,
                );
                if let Some(item_id) = prop.item_id.as_deref() {
                    if let Some(relic) = self.relic_registry.get(item_id).cloned() {
                        self.acquire_relic_pickup(relic);
                    } else {
                        eprintln!("[RELIC] Unknown item_id '{}'", item_id);
                    }
                }
            }
        }

        if interact_pressed {
            if let Some(index) = Self::nearest_anchor_prop_index(
                &self.level_data.props,
                player_v,
                self.config_data.world.anchor_interaction_radius,
            ) {
                let prop = &self.level_data.props[index];
                self.active_anchor_rite = Some(ActiveAnchorRite::new(
                    prop.anchor_id.clone().unwrap_or_default(),
                    prop.display_name
                        .clone()
                        .unwrap_or_else(|| prop.anchor_id.clone().unwrap_or_default()),
                    prop.position,
                    prop.event_id.clone(),
                ));
                self.play_sound(SoundEffect::Pickup);
                println!("[ANCHOR] Rite opened at prop {}", index);
            }
        }
    }

    fn ensure_enemy_runtime_matches_props(&mut self) {
        self.enemy_runtime.truncate(self.level_data.props.len());
        for prop in self.level_data.props.iter().skip(self.enemy_runtime.len()) {
            self.enemy_runtime.push(EnemyRuntimeState::for_max_health(
                prop.enemy_type.as_ref().map_or(0.0, |_| prop.enemy_health),
            ));
        }
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

        if input.fire_primary && self.action_cooldown <= 0.0 {
            self.feedback.on_fire();
            self.play_sound(SoundEffect::Fire);
            let player_pos = self.physics.get_player_pos();
            let ray_origin = Vec3::new(player_pos[0], player_pos[1] + 1.0, player_pos[2]);
            let ray_dir = self.camera.get_forward();
            self.particles
                .spawn_burst(ParticleBurst::Muzzle, ray_origin + ray_dir * 0.55, ray_dir);
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
                let hit_position = Vec3::from_array(prop.position) + Vec3::Y * 0.85;
                let death_event_id = prop.event_id.clone();

                prop.enemy_health -= damage;
                let after = prop.enemy_health.max(0.0);

                println!(
                    "[COMBAT] Hit {} for {:.1} ({:.0} -> {:.0})",
                    enemy_type, damage, before, after
                );

                if prop.enemy_health <= 0.0 {
                    self.particles
                        .spawn_burst(ParticleBurst::Kill, hit_position, -ray_dir);
                    let death_event_queued = death_event_id.as_deref().is_some_and(|event_id| {
                        self.queue_prop_manual_event(event_id, "enemy defeat")
                    });
                    self.grant_enemy_reward(idx);
                    self.feedback
                        .on_enemy_kill_amount((before - after).max(0.0));
                    self.remove_prop(idx);
                    self.play_sound(SoundEffect::Kill);
                    if !death_event_queued {
                        self.autosave("enemy defeat");
                    }
                } else if let Some(runtime) = self.enemy_runtime.get_mut(idx) {
                    self.particles
                        .spawn_burst(ParticleBurst::Hit, hit_position, -ray_dir);
                    self.feedback.on_enemy_hit_amount((before - after).max(0.0));
                    runtime.stagger(hit_stun);
                    self.play_sound(SoundEffect::Hit);
                }
                self.action_cooldown = attack_cooldown;
            } else {
                if obstruction_distance.is_some() {
                    self.feedback.on_shot_blocked();
                    self.play_sound(SoundEffect::Blocked);
                    let blocked_position =
                        ray_origin + ray_dir * obstruction_distance.unwrap_or(target_range);
                    self.particles
                        .spawn_burst(ParticleBurst::Blocked, blocked_position, -ray_dir);
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
            self.play_sound(SoundEffect::Blocked);
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

    fn interact_event(id: &str, prop_id: &str, radius: f32) -> LevelEventData {
        LevelEventData {
            id: id.to_string(),
            once: true,
            trigger: LevelEventTriggerData {
                kind: LevelEventTriggerKind::Interact,
                position: [0.0, 0.0, 0.0],
                radius,
                prop_id: Some(prop_id.to_string()),
                flag_id: None,
            },
            actions: Vec::new(),
        }
    }

    fn interactable_prop(id: &str, position: [f32; 3]) -> PropData {
        let mut prop: PropData = serde_json::from_str(r#"{ "asset_id": "Cube.obj" }"#).unwrap();
        prop.id = Some(id.to_string());
        prop.position = position;
        prop
    }

    #[test]
    fn interaction_selects_nearest_eligible_authored_prop() {
        let mut events = vec![
            interact_event("far_event", "far_prop", 3.0),
            interact_event("near_event", "near_prop", 3.0),
        ];
        events[1].trigger.flag_id = Some("near_unlocked".to_string());
        let props = vec![
            interactable_prop("far_prop", [2.0, 0.0, 0.0]),
            interactable_prop("near_prop", [1.0, 0.0, 0.0]),
        ];
        let mut flags = HashSet::new();

        assert_eq!(
            EngineState::nearest_interact_event_index(
                &events,
                &props,
                &[false, false],
                Vec3::ZERO,
                &flags,
            ),
            Some(0)
        );

        flags.insert("near_unlocked".to_string());
        assert_eq!(
            EngineState::nearest_interact_event_index(
                &events,
                &props,
                &[false, false],
                Vec3::ZERO,
                &flags,
            ),
            Some(1)
        );
        assert_eq!(
            EngineState::nearest_interact_event_index(
                &events,
                &props,
                &[false, true],
                Vec3::ZERO,
                &flags,
            ),
            Some(0)
        );
    }

    #[test]
    fn manual_event_never_fires_as_an_automatic_trigger() {
        let mut event = interact_event("manual_event", "unused", 3.0);
        event.trigger.kind = LevelEventTriggerKind::Manual;

        assert!(!EngineState::level_event_triggered_with_flags(
            &event,
            Vec3::ZERO,
            &HashSet::new(),
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

    #[test]
    fn anchor_interaction_selects_the_nearest_rite() {
        let mut far = interactable_prop("far_anchor", [2.5, 0.0, 0.0]);
        far.anchor_id = Some("far".to_string());
        let mut near = interactable_prop("near_anchor", [1.0, 0.0, 0.0]);
        near.anchor_id = Some("near".to_string());

        assert_eq!(
            EngineState::nearest_anchor_prop_index(&[far, near], Vec3::ZERO, 3.0),
            Some(1)
        );
    }

    #[test]
    fn loot_seed_uses_the_stable_authored_source() {
        let first =
            EngineState::stable_loot_seed("ashwalk_01", 1, "keeper_drop", "ashwarden_elite");
        let repeated =
            EngineState::stable_loot_seed("ashwalk_01", 1, "keeper_drop", "ashwarden_elite");
        let other_source =
            EngineState::stable_loot_seed("ashwalk_01", 1, "keeper_drop", "another_keeper");
        let other_action =
            EngineState::stable_loot_seed("ashwalk_01", 1, "keeper_drop", "keeper_event:1");
        let first_action =
            EngineState::stable_loot_seed("ashwalk_01", 1, "keeper_drop", "keeper_event:0");
        let other_cycle =
            EngineState::stable_loot_seed("ashwalk_01", 2, "keeper_drop", "ashwarden_elite");

        assert_eq!(first, repeated);
        assert_ne!(first, other_source);
        assert_ne!(first_action, other_action);
        assert_ne!(first, other_cycle);
    }
}
