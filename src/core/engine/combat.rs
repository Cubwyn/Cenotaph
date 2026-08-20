use glam::Vec3;

use crate::core::engine::state::EngineState;
use crate::data::world::level::PropData;
use crate::game::combat::closest_ray_sphere_hit;
use crate::systems::audio::SoundEffect;
use crate::systems::input::manager::InputManager;
use crate::systems::render::particles::ParticleBurst;

impl EngineState {
    pub(crate) fn handle_gameplay_input(&mut self, input: &InputManager) {
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

    pub(crate) fn grant_enemy_reward(&mut self, index: usize) {
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

    pub(crate) fn remove_prop_data(&mut self, index: usize) -> Option<PropData> {
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

    pub(crate) fn remove_persistent_prop_data(&mut self, index: usize) -> Option<PropData> {
        let prop = self.remove_prop_data(index)?;
        if let Some(prop_id) = prop.id.as_deref() {
            if !prop_id.starts_with(crate::data::world::level::RUNTIME_LOOT_ID_PREFIX) {
                self.removed_prop_ids.insert(prop_id.to_string());
            }
        }
        Some(prop)
    }

    pub(crate) fn remove_prop(&mut self, index: usize) {
        let Some(prop) = self.remove_persistent_prop_data(index) else {
            return;
        };
        println!(
            "[COMBAT] Destroyed prop '{}' at index {}",
            prop.asset_id, index
        );
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
