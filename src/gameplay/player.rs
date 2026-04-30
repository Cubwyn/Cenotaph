// src/gameplay/player.rs
// Player state — health, stamina, and any persistent stats that survive
// between frames. Movement intent is computed in render::camera and fed
// into physics::engine; only the *results* of that live here.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerStats {
    // Core combat stats
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
}

impl PlayerStats {
    pub fn new() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
        }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn consume_stamina(&mut self, amount: f32) -> bool {
        if self.stamina >= amount {
            self.stamina -= amount;
            true
        } else {
            false
        }
    }

    pub fn regenerate_stamina(&mut self, amount: f32) {
        self.stamina = (self.stamina + amount).min(self.max_stamina);
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    pub fn reset(&mut self) {
        self.health = self.max_health;
        self.stamina = self.max_stamina;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_starts_at_full_health() {
        let p = PlayerStats::new();
        assert_eq!(p.health, p.max_health);
    }

    #[test]
    fn damage_clamps_to_zero() {
        let mut p = PlayerStats::new();
        p.take_damage(9999.0);
        assert_eq!(p.health, 0.0);
        assert!(p.is_dead());
    }

    #[test]
    fn heal_clamps_to_max() {
        let mut p = PlayerStats::new();
        p.take_damage(50.0);
        p.heal(9999.0);
        assert_eq!(p.health, p.max_health);
    }

    #[test]
    fn stamina_consumption_works() {
        let mut p = PlayerStats::new();
        assert!(p.consume_stamina(10.0));
        assert_eq!(p.stamina, 90.0);
        assert!(!p.consume_stamina(100.0));
    }
}
