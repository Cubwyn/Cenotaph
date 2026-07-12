use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleModifier {
    FirstAscent,
    LeanHarvest,
    HostileEchoes,
    ScarredRelics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleState {
    pub number: u32,
    pub modifier: CycleModifier,
}

impl CycleState {
    pub fn new(number: u32) -> Self {
        let number = number.max(1);
        Self {
            number,
            modifier: CycleModifier::for_cycle(number),
        }
    }

    pub fn advance(&mut self) {
        *self = Self::new(self.number.saturating_add(1));
    }

    pub fn resource_reward(&self, base_amount: u32) -> u32 {
        if base_amount == 0 {
            return 0;
        }

        ((base_amount as f32) * self.resource_multiplier())
            .round()
            .max(1.0) as u32
    }

    pub fn enemy_damage(&self, base_damage: f32) -> f32 {
        base_damage * self.enemy_damage_multiplier()
    }

    pub fn relic_damage(&self, base_damage: f32) -> f32 {
        base_damage * self.relic_damage_multiplier()
    }

    pub fn reward_relic_id(&self, enemy_type: &str) -> &'static str {
        match enemy_type.trim().to_ascii_lowercase().as_str() {
            "ashbound" | "burdened" => "ash_splinter",
            "censer" | "harpy" => "veil_cinder",
            "chainrunner" => "chain_sigil",
            _ => "ash_splinter",
        }
    }

    fn resource_multiplier(self) -> f32 {
        match self.modifier {
            CycleModifier::FirstAscent => 1.0,
            CycleModifier::LeanHarvest => 0.75,
            CycleModifier::HostileEchoes => 1.15,
            CycleModifier::ScarredRelics => 1.0,
        }
    }

    fn enemy_damage_multiplier(self) -> f32 {
        match self.modifier {
            CycleModifier::FirstAscent => 1.0,
            CycleModifier::LeanHarvest => 1.0,
            CycleModifier::HostileEchoes => 1.25,
            CycleModifier::ScarredRelics => 1.1,
        }
    }

    fn relic_damage_multiplier(self) -> f32 {
        match self.modifier {
            CycleModifier::FirstAscent => 1.0,
            CycleModifier::LeanHarvest => 1.0,
            CycleModifier::HostileEchoes => 1.0,
            CycleModifier::ScarredRelics => 1.2,
        }
    }
}

impl CycleModifier {
    pub fn display_label(self) -> &'static str {
        match self {
            Self::FirstAscent => "FIRST ASCENT",
            Self::LeanHarvest => "LEAN HARVEST",
            Self::HostileEchoes => "HOSTILE ECHOES",
            Self::ScarredRelics => "SCARRED RELICS",
        }
    }

    fn for_cycle(number: u32) -> Self {
        match (number.saturating_sub(1)) % 4 {
            0 => Self::FirstAscent,
            1 => Self::LeanHarvest,
            2 => Self::HostileEchoes,
            _ => Self::ScarredRelics,
        }
    }
}

impl Default for CycleState {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_rotate_modifiers() {
        assert_eq!(CycleState::new(1).modifier, CycleModifier::FirstAscent);
        assert_eq!(CycleState::new(2).modifier, CycleModifier::LeanHarvest);
        assert_eq!(CycleState::new(3).modifier, CycleModifier::HostileEchoes);
        assert_eq!(CycleState::new(4).modifier, CycleModifier::ScarredRelics);
        assert_eq!(CycleState::new(5).modifier, CycleModifier::FirstAscent);
    }

    #[test]
    fn modifiers_have_player_facing_labels() {
        assert_eq!(CycleModifier::FirstAscent.display_label(), "FIRST ASCENT");
        assert_eq!(CycleModifier::LeanHarvest.display_label(), "LEAN HARVEST");
        assert_eq!(
            CycleModifier::HostileEchoes.display_label(),
            "HOSTILE ECHOES"
        );
        assert_eq!(
            CycleModifier::ScarredRelics.display_label(),
            "SCARRED RELICS"
        );
    }

    #[test]
    fn modifiers_affect_runtime_values() {
        let lean = CycleState::new(2);
        let hostile = CycleState::new(3);
        let scarred = CycleState::new(4);

        assert_eq!(lean.resource_reward(20), 15);
        assert!((hostile.enemy_damage(10.0) - 12.5).abs() < 0.001);
        assert!((scarred.relic_damage(10.0) - 12.0).abs() < 0.001);
    }

    #[test]
    fn enemies_have_deterministic_test_reward_relics() {
        let cycle = CycleState::new(1);

        assert_eq!(cycle.reward_relic_id("Ashbound"), "ash_splinter");
        assert_eq!(cycle.reward_relic_id("Burdened"), "ash_splinter");
        assert_eq!(cycle.reward_relic_id("Censer"), "veil_cinder");
        assert_eq!(cycle.reward_relic_id("Chainrunner"), "chain_sigil");
        assert_eq!(cycle.reward_relic_id("Harpy"), "veil_cinder");
    }
}
