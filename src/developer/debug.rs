use glam::Vec3;

use crate::core::engine::state::EngineState;
use crate::data::world::level::{LevelData, PropData};
use crate::systems::audio::SoundEffect;
use crate::systems::input::manager::InputManager;
use crate::systems::render::particles::ParticleBurst;

impl EngineState {
    pub(crate) fn handle_debug_input(&mut self, input: &InputManager) -> bool {
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

    pub(super) fn debug_print_status(&self) {
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

    pub(super) fn debug_heal_player(&mut self) {
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

    pub(super) fn debug_set_player_health(&mut self, health: f32) {
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

    pub(super) fn debug_spawn_enemy(&mut self, enemy_type: &str) {
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
        let prop = PropData::spawn_enemy(
            &enemy,
            [spawn.x, spawn.y, spawn.z],
            Self::debug_enemy_scale(&enemy.id),
        );

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

    pub(super) fn debug_respawn_loot(&mut self) {
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

    pub(super) fn debug_clear_enemies(&mut self) {
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

    fn debug_enemy_count(&self) -> usize {
        self.level_data
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some() && prop.enemy_health > 0.0)
            .count()
    }

    pub(super) fn debug_enemy_scale(enemy_id: &str) -> [f32; 3] {
        match enemy_id {
            "burdened" => [1.5, 1.5, 1.5],
            "censer" | "chainrunner" => [1.1, 1.1, 1.1],
            "ashbound" | "harpy" => [1.2, 1.2, 1.2],
            _ => [1.2, 1.2, 1.2],
        }
    }
}
