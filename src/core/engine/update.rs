//! Per-frame simulation and presentation updates.
//!
//! Simulation and gameplay run in `update_physics`; camera and GPU state are
//! synchronized separately in `update_visuals`.

use glam::Vec3;

use crate::core::engine::state::{ActiveDialogueState, EngineState, GameMode};
use crate::game::feedback::MotionSample;
use crate::systems::audio::SoundEffect;
use crate::systems::input::manager::InputManager;
use crate::systems::render::particles::ParticleBurst;

impl EngineState {
    /// Advances simulation, gameplay, and event state for one frame.
    pub fn update_physics(&mut self, input: &InputManager, dt: f32) {
        if self.game_mode == GameMode::Paused {
            return;
        }

        self.tick_atmosphere(dt);
        self.tick_timers(dt);

        if self.handle_debug_input(input) {
            return;
        }
        if self.active_anchor_rite.is_some() {
            self.update_anchor_rite(input);
            return;
        }

        self.tick_movement_input(input, dt);
        self.tick_proximity_transitions();
        let world_interact_pressed = self.tick_world_interactions(input);
        self.execute_pending_transition();

        if !self.player.is_dead {
            self.tick_alive_gameplay(input, dt, world_interact_pressed);
        } else {
            self.tick_dead_respawn(dt);
        }

        self.tick_hurtboxes(dt);
        self.tick_debug_log(dt);
    }

    fn tick_atmosphere(&mut self, dt: f32) {
        self.update_mountain_reaction(dt);
        let particle_center = Vec3::from_array(self.physics.get_player_pos()) + Vec3::Y;
        self.particles
            .update(&self.queue, &self.runtime_atmosphere, particle_center, dt);
    }

    fn tick_timers(&mut self, dt: f32) {
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
    }

    fn tick_movement_input(&mut self, input: &InputManager, dt: f32) {
        let intent = {
            let v =
                self.camera_controller
                    .get_movement_intent(input, &self.camera, &self.config_data);
            [v.x, v.y, v.z]
        };
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

        let smooth_rate = 8.0_f32;
        let lerp = (smooth_rate * dt).min(1.0);
        self.player.speed_multiplier_smoothed +=
            (target_speed_multiplier - self.player.speed_multiplier_smoothed) * lerp;

        self.player.stamina.smooth_display(dt, 8.0);

        if !self.player.is_sprinting && !self.player.is_dashing {
            self.player
                .stamina
                .tick_regen(dt, self.config_data.player.stamina_regen_rate);
        }
    }

    fn tick_proximity_transitions(&mut self) {
        if self.pending_transition.is_some() {
            return;
        }
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

    fn tick_world_interactions(&mut self, input: &InputManager) -> bool {
        if self.pending_transition.is_some() {
            return false;
        }
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
        available_interact && !event_consumed
    }

    fn execute_pending_transition(&mut self) {
        let Some(ref next_level) = self.pending_transition.clone() else {
            return;
        };
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

    fn tick_alive_gameplay(
        &mut self,
        input: &InputManager,
        dt: f32,
        world_interact_pressed: bool,
    ) {
        self.update_enemy_ai(dt);
        self.update_non_enemy_path_followers();

        let walk_speed = self.config_data.player.walk_speed;
        let grounded_before = self.physics.is_player_grounded();
        let velocity_before = self.physics.get_player_velocity();
        let is_jumping = input.is_key_down(self.config_data.key("jump"));
        let jumped = self.physics.apply_player_movement(
            if self.player.is_dashing {
                self.player.dash_direction
            } else {
                let v = self
                    .camera_controller
                    .get_movement_intent(input, &self.camera, &self.config_data);
                [v.x, v.y, v.z]
            },
            is_jumping,
            &self.config_data.physics,
            dt,
            self.player.speed_multiplier_smoothed,
        );
        self.physics.step(&self.config_data.physics, dt);
        let grounded_after = self.physics.is_player_grounded();
        let velocity_after = self.physics.get_player_velocity();
        let horizontal_speed =
            (velocity_after[0] * velocity_after[0] + velocity_after[2] * velocity_after[2]).sqrt();
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
    }

    fn tick_dead_respawn(&mut self, dt: f32) {
        let walk_speed = self.config_data.player.walk_speed;
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

    fn tick_hurtboxes(&mut self, dt: f32) {
        if self.player.is_dead
            || self.player.health.is_depleted()
            || self.player.hurtbox_cooldown > 0.0
        {
            return;
        }

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

    fn tick_debug_log(&mut self, dt: f32) {
        if !self.config_data.debug.position_log_enabled {
            self.debug_timer = 0.0;
            return;
        }
        self.debug_timer += dt;
        if self.debug_timer < self.config_data.debug.position_log_interval {
            return;
        }
        self.debug_timer -= self.config_data.debug.position_log_interval;
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
            "[DEBUG] pos ({:.1}, {:.1}, {:.1}) | hp {:.0}/{:.0} | stamina {:.0}/{:.0} | enemies {} | loot {} | res {}/{} | cycle {}",
            pos[0],
            pos[1],
            pos[2],
            self.player.health.current,
            self.player.health.max,
            self.player.stamina.current,
            self.player.stamina.max,
            enemy_count,
            loot_count,
            self.progress.unsecured_resource,
            self.progress.banked_resource,
            self.cycle.number,
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::engine::level_events::{loot_entries_for_rolls, stable_loot_seed};
    use crate::data::world::level::{
        LevelEventData, LevelEventTriggerData, LevelEventTriggerKind, LevelPathData,
        LevelPathKind, LootEntryData, LootTableData, PropData,
    };
    use crate::game::enemy::EnemyRuntimeState;
    use crate::game::enemy_ai::path_velocity_for_runtime;
    use std::collections::HashSet;

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
            path_velocity_for_runtime(&mut runtime, &path, [0.0, 0.0, 0.0], 2.0)
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
            path_velocity_for_runtime(&mut runtime, &path, [4.0, 0.0, 0.0], 2.0)
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

        let entries = loot_entries_for_rolls(&table, 0);

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
            stable_loot_seed("ashwalk_01", 1, "keeper_drop", "ashwarden_elite");
        let repeated =
            stable_loot_seed("ashwalk_01", 1, "keeper_drop", "ashwarden_elite");
        let other_source =
            stable_loot_seed("ashwalk_01", 1, "keeper_drop", "another_keeper");
        let other_action =
            stable_loot_seed("ashwalk_01", 1, "keeper_drop", "keeper_event:1");
        let first_action =
            stable_loot_seed("ashwalk_01", 1, "keeper_drop", "keeper_event:0");
        let other_cycle =
            stable_loot_seed("ashwalk_01", 2, "keeper_drop", "ashwarden_elite");

        assert_eq!(first, repeated);
        assert_ne!(first, other_source);
        assert_ne!(first_action, other_action);
        assert_ne!(first, other_cycle);
    }
}
