use crate::data::relic::{normalize_relic_id, RelicDefinition, RelicRegistry};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EquippedRelic {
    owned: Vec<RelicDefinition>,
    equipped_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelicAcquisition {
    pub relic: RelicDefinition,
    pub acquired_new: bool,
    pub equipped: bool,
    pub slot: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelicSelection {
    pub relic: RelicDefinition,
    pub slot: usize,
    pub total: usize,
}

impl EquippedRelic {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&mut self, relic: RelicDefinition) -> RelicAcquisition {
        let normalized_id = normalize_relic_id(&relic.id);
        let existing_index = self.find_owned_index(&normalized_id);
        let acquired_new = existing_index.is_none();
        let slot_index = match existing_index {
            Some(index) => {
                self.owned[index] = relic.clone();
                index
            }
            None => {
                self.owned.push(relic.clone());
                self.owned.len() - 1
            }
        };

        let should_equip = self.equipped_id.is_none();
        if should_equip {
            self.equipped_id = Some(normalized_id);
        }

        RelicAcquisition {
            relic,
            acquired_new,
            equipped: should_equip || self.equipped_index() == Some(slot_index),
            slot: slot_index + 1,
            total: self.owned.len(),
        }
    }

    pub fn add(&mut self, relic: RelicDefinition) -> bool {
        let normalized_id = normalize_relic_id(&relic.id);
        if self.find_owned_index(&normalized_id).is_some() {
            return false;
        }

        self.owned.push(relic);
        if self.equipped_id.is_none() {
            self.equipped_id = Some(normalized_id);
        }
        true
    }

    pub fn equip_id(&mut self, relic_id: &str) -> Result<(), String> {
        let normalized_id = normalize_relic_id(relic_id);
        if normalized_id.is_empty() {
            return Err("relic id must not be empty".to_string());
        }

        if self.find_owned_index(&normalized_id).is_some() {
            self.equipped_id = Some(normalized_id);
            Ok(())
        } else {
            Err(format!("relic '{}' is not owned", relic_id))
        }
    }

    pub fn cycle_next(&mut self) -> Option<RelicSelection> {
        if self.owned.len() < 2 {
            return None;
        }

        let next_index = self
            .equipped_index()
            .map_or(0, |index| (index + 1) % self.owned.len());

        let relic = self.owned[next_index].clone();
        self.equipped_id = Some(normalize_relic_id(&relic.id));
        Some(RelicSelection {
            relic,
            slot: next_index + 1,
            total: self.owned.len(),
        })
    }

    pub fn restore_from_ids(
        &mut self,
        owned_ids: &[String],
        equipped_id: Option<&str>,
        registry: &RelicRegistry,
    ) {
        self.owned.clear();
        self.equipped_id = None;

        for relic_id in owned_ids {
            if let Some(relic) = registry.get(relic_id).cloned() {
                self.add(relic);
            } else {
                eprintln!("[SAVE] Ignoring unknown saved relic '{}'", relic_id);
            }
        }

        if let Some(equipped_id) = equipped_id {
            if let Err(error) = self.equip_id(equipped_id) {
                eprintln!("[SAVE] {}", error);
            }
        }
    }

    pub fn owned_ids(&self) -> Vec<String> {
        self.owned.iter().map(|relic| relic.id.clone()).collect()
    }

    pub fn owned_count(&self) -> usize {
        self.owned.len()
    }

    pub fn equipped_id(&self) -> Option<&str> {
        self.equipped_id.as_deref()
    }

    pub fn current(&self) -> Option<&RelicDefinition> {
        self.equipped_index()
            .and_then(|index| self.owned.get(index))
    }

    pub fn damage(&self, base_damage: f32) -> f32 {
        self.current()
            .map(|relic| base_damage * relic.damage_multiplier)
            .unwrap_or(base_damage)
    }

    pub fn cooldown(&self, base_cooldown: f32) -> f32 {
        self.current()
            .map(|relic| base_cooldown * relic.cooldown_multiplier)
            .unwrap_or(base_cooldown)
            .max(0.0)
    }

    pub fn range(&self, base_range: f32) -> f32 {
        self.current()
            .map(|relic| base_range * relic.range_multiplier)
            .unwrap_or(base_range)
    }

    pub fn hit_stun(&self, base_hit_stun: f32) -> f32 {
        self.current()
            .map(|relic| base_hit_stun + relic.hit_stun_bonus)
            .unwrap_or(base_hit_stun)
            .max(0.0)
    }

    fn equipped_index(&self) -> Option<usize> {
        let equipped_id = self.equipped_id.as_deref()?;
        self.find_owned_index(equipped_id)
    }

    fn find_owned_index(&self, normalized_id: &str) -> Option<usize> {
        self.owned
            .iter()
            .position(|owned| normalize_relic_id(&owned.id) == normalized_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relic_fixture() -> RelicDefinition {
        RelicDefinition {
            id: "ash_splinter".to_string(),
            display_name: "Ash Splinter".to_string(),
            pickup_asset: "pickups/relic_ash_splinter.obj".to_string(),
            family: "Sovereign".to_string(),
            rarity: "Common".to_string(),
            damage_multiplier: 1.2,
            cooldown_multiplier: 1.1,
            range_multiplier: 1.5,
            hit_stun_bonus: 0.05,
            flavor_text: "It remembers the shape of a first wound.".to_string(),
        }
    }

    fn second_relic_fixture() -> RelicDefinition {
        RelicDefinition {
            id: "veil_cinder".to_string(),
            display_name: "Veil Cinder".to_string(),
            pickup_asset: "pickups/relic_veil_cinder.obj".to_string(),
            family: "Moonchild".to_string(),
            rarity: "Rare".to_string(),
            damage_multiplier: 0.9,
            cooldown_multiplier: 0.8,
            range_multiplier: 1.2,
            hit_stun_bonus: 0.12,
            flavor_text: "A coal that refuses to admit it is seen.".to_string(),
        }
    }

    #[test]
    fn no_relic_keeps_base_combat_values() {
        let equipped = EquippedRelic::new();

        assert_eq!(equipped.damage(25.0), 25.0);
        assert_eq!(equipped.cooldown(0.25), 0.25);
        assert_eq!(equipped.range(80.0), 80.0);
        assert_eq!(equipped.hit_stun(0.1), 0.1);
    }

    #[test]
    fn equipped_relic_modifies_combat_values() {
        let mut equipped = EquippedRelic::new();
        equipped.acquire(relic_fixture());

        assert!((equipped.damage(25.0) - 30.0).abs() < 0.001);
        assert!((equipped.cooldown(0.25) - 0.275).abs() < 0.001);
        assert!((equipped.range(80.0) - 120.0).abs() < 0.001);
        assert!((equipped.hit_stun(0.1) - 0.15).abs() < 0.001);
    }

    #[test]
    fn equip_id_switches_to_owned_relic() {
        let mut equipped = EquippedRelic::new();
        equipped.acquire(relic_fixture());
        equipped.acquire(second_relic_fixture());

        equipped.equip_id("Veil Cinder").unwrap();

        assert_eq!(
            equipped.current().map(|relic| relic.id.as_str()),
            Some("veil_cinder")
        );
        assert_eq!(
            equipped.owned_ids(),
            vec!["ash_splinter".to_string(), "veil_cinder".to_string()]
        );
    }

    #[test]
    fn duplicate_pickups_do_not_duplicate_inventory() {
        let mut equipped = EquippedRelic::new();

        let first = equipped.acquire(relic_fixture());
        let duplicate = equipped.acquire(relic_fixture());

        assert!(first.acquired_new);
        assert!(first.equipped);
        assert!(!duplicate.acquired_new);
        assert!(duplicate.equipped);
        assert_eq!(equipped.owned_ids(), vec!["ash_splinter".to_string()]);
        assert_eq!(equipped.owned_count(), 1);
        assert_eq!(equipped.equipped_id(), Some("ash_splinter"));
    }

    #[test]
    fn later_acquisitions_do_not_replace_equipped_relic() {
        let mut equipped = EquippedRelic::new();

        let first = equipped.acquire(relic_fixture());
        let second = equipped.acquire(second_relic_fixture());

        assert!(first.acquired_new);
        assert!(first.equipped);
        assert!(second.acquired_new);
        assert!(!second.equipped);
        assert_eq!(second.slot, 2);
        assert_eq!(second.total, 2);
        assert_eq!(equipped.equipped_id(), Some("ash_splinter"));
        assert_eq!(
            equipped.owned_ids(),
            vec!["ash_splinter".to_string(), "veil_cinder".to_string()]
        );
    }

    #[test]
    fn cycling_requires_at_least_two_owned_relics() {
        let mut equipped = EquippedRelic::new();

        assert_eq!(equipped.cycle_next(), None);
        equipped.acquire(relic_fixture());
        assert_eq!(equipped.cycle_next(), None);
        assert_eq!(equipped.equipped_id(), Some("ash_splinter"));
    }

    #[test]
    fn cycles_through_owned_relics() {
        let mut equipped = EquippedRelic::new();
        equipped.add(relic_fixture());
        equipped.add(second_relic_fixture());

        assert_eq!(equipped.equipped_id(), Some("ash_splinter"));

        let next = equipped.cycle_next();
        assert_eq!(
            next.as_ref().map(|selection| selection.relic.id.as_str()),
            Some("veil_cinder")
        );
        assert_eq!(next.as_ref().map(|selection| selection.slot), Some(2));
        assert_eq!(next.as_ref().map(|selection| selection.total), Some(2));
        assert_eq!(equipped.equipped_id(), Some("veil_cinder"));

        let next = equipped.cycle_next();
        assert_eq!(
            next.as_ref().map(|selection| selection.relic.id.as_str()),
            Some("ash_splinter")
        );
        assert_eq!(next.as_ref().map(|selection| selection.slot), Some(1));
        assert_eq!(next.as_ref().map(|selection| selection.total), Some(2));
        assert_eq!(equipped.equipped_id(), Some("ash_splinter"));
    }

    #[test]
    fn restores_inventory_from_registry_ids() {
        let registry = RelicRegistry::from_definitions(vec![relic_fixture()]).unwrap();
        let mut equipped = EquippedRelic::new();

        equipped.restore_from_ids(
            &["Ash Splinter".to_string(), "missing".to_string()],
            Some("ash_splinter"),
            &registry,
        );

        assert_eq!(equipped.owned_ids(), vec!["ash_splinter".to_string()]);
        assert_eq!(equipped.equipped_id(), Some("ash_splinter"));
    }
}
