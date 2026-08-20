//! Shared engine services — sound, mountain reactions, damage, persistence.

use glam::Vec3;

use crate::core::engine::state::EngineState;
use crate::game::mountain::ActiveMountainReaction;
use crate::game::save::{LevelSaveSnapshot, SaveData, SavedRuntimeLoot, DEFAULT_SAVE_PATH};
use crate::systems::audio::SoundEffect;
use crate::systems::render::particles::ParticleBurst;

impl EngineState {
    pub(crate) fn play_sound(&mut self, effect: SoundEffect) {
        if let Some(audio) = self.audio.as_mut() {
            audio.play(effect);
        }
    }

    pub(super) fn update_mountain_reaction(&mut self, dt: f32) {
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

    pub(crate) fn start_mountain_reaction(&mut self, reaction_id: &str) {
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

    pub(crate) fn apply_player_damage(&mut self, source: &str, amount: f32) -> bool {
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

    pub(crate) fn reset_player_body_to(&mut self, position: [f32; 3]) {
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

    pub(super) fn defeat_player(&mut self) {
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

    pub(super) fn fired_level_event_ids(&self) -> Vec<String> {
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
}
