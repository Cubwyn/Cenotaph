use std::collections::HashSet;

use glam::Vec3;

use crate::core::engine::state::{
    ActiveDialogueState, EngineState, ManualLevelEventStatus,
};
use crate::data::world::level::{
    LevelEventActionData, LevelEventActionKind, LevelEventData,
    LevelEventTriggerKind, LootEntryData, LootTableData, PropData,
    RUNTIME_LOOT_ID_PREFIX,
};
use crate::game::progression::{ActiveAnchorRite, AnchorRiteChoice};
use crate::systems::audio::SoundEffect;
use crate::systems::input::manager::InputManager;
use crate::systems::render::mesh::try_load_model;
use crate::systems::render::particles::ParticleBurst;

impl EngineState {
    pub(crate) fn update_level_events(&mut self, interact_pressed: bool) -> bool {
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

    #[allow(dead_code)]
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
    pub(crate) fn level_event_triggered_with_flags(
        event: &LevelEventData,
        player: Vec3,
        level_flags: &HashSet<String>,
    ) -> bool {
        Self::level_event_flag_ready(event, level_flags)
            && Self::automatic_level_event_triggered(event, player)
    }

    pub(crate) fn level_event_flag_ready(
        event: &LevelEventData,
        level_flags: &HashSet<String>,
    ) -> bool {
        event
            .trigger
            .flag_id
            .as_deref()
            .is_none_or(|flag_id| level_flags.contains(flag_id))
    }

    pub(crate) fn automatic_level_event_triggered(event: &LevelEventData, player: Vec3) -> bool {
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

    pub(crate) fn execute_level_event_action(
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

    pub(crate) fn spawn_loot_from_table(
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

        let seed = stable_loot_seed(
            &self.level_name,
            self.cycle.number,
            loot_table_id,
            source_id,
        );
        let entries = loot_entries_for_rolls(&table, seed);
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
                    loot_entry_prop(&self.relic_registry, &entry, prop_position, runtime_id);
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

    pub(crate) fn add_runtime_prop(&mut self, prop: PropData) {
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
            .push(crate::game::enemy::EnemyRuntimeState::for_max_health(max_health));
    }

    pub(crate) fn is_loot_prop(prop: &PropData) -> bool {
        prop.resource_value > 0
            || prop
                .item_id
                .as_ref()
                .is_some_and(|item_id| !item_id.trim().is_empty())
    }

    pub(crate) fn same_loot_prop(left: &PropData, right: &PropData) -> bool {
        left.asset_id == right.asset_id
            && left.item_id == right.item_id
            && left.resource_value == right.resource_value
            && left
                .position
                .iter()
                .zip(right.position.iter())
                .all(|(l, r)| (*l - *r).abs() <= 0.001)
    }

    pub(crate) fn acquire_relic_pickup(&mut self, relic: crate::data::relic::RelicDefinition) {
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

    pub(crate) fn update_anchor_rite(&mut self, input: &InputManager) {
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

    pub(crate) fn queue_prop_manual_event(&mut self, event_id: &str, context: &str) -> bool {
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

    pub(crate) fn update_progression_interactions(&mut self, interact_pressed: bool) {
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
}

pub(crate) fn loot_entries_for_rolls(table: &LootTableData, seed: u64) -> Vec<LootEntryData> {
    let total_weight: u32 = table.entries.iter().map(|entry| entry.weight).sum();
    if table.rolls == 0 || total_weight == 0 {
        return Vec::new();
    }

    (0..table.rolls)
        .filter_map(|roll| weighted_loot_entry(table, seed.wrapping_add(roll as u64)))
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

pub(crate) fn stable_loot_seed(
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
    PropData::loot(item_id, entry.resource_value, asset_id, position, runtime_id)
}
