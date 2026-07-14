use glam::Vec3;

use crate::data::enemy::EnemyDefinition;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EnemyRuntimeState {
    pub attack_cooldown_remaining: f32,
    pub attack_windup_remaining: f32,
    pub stagger_remaining: f32,
    pub path_waypoint: usize,
}

impl EnemyRuntimeState {
    pub fn tick(&mut self, dt: f32) {
        self.attack_cooldown_remaining = (self.attack_cooldown_remaining - dt).max(0.0);
        self.stagger_remaining = (self.stagger_remaining - dt).max(0.0);
    }

    pub fn stagger(&mut self, duration: f32) {
        self.stagger_remaining = self.stagger_remaining.max(duration.max(0.0));
        self.attack_windup_remaining = 0.0;
    }

    pub fn clear_windup(&mut self) {
        self.attack_windup_remaining = 0.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyBehavior {
    Melee,
    Ranged,
    Flanker,
    Aerial,
}

impl EnemyBehavior {
    pub(crate) fn from_tag(tag: &str) -> Self {
        match tag.trim().to_ascii_lowercase().as_str() {
            "ranged_windup" => Self::Ranged,
            "flanker_lunge" => Self::Flanker,
            "aerial_dive" => Self::Aerial,
            "chase_melee" | "slow_chase_melee" => Self::Melee,
            _ => Self::Melee,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EnemyAiIntent {
    Idle,
    Move { velocity_x: f32, velocity_z: f32 },
    Attack,
}

pub(crate) fn enemy_ai_intent(
    enemy: &EnemyDefinition,
    enemy_pos: Vec3,
    player_pos: Vec3,
) -> EnemyAiIntent {
    let delta = Vec3::new(player_pos.x - enemy_pos.x, 0.0, player_pos.z - enemy_pos.z);
    let distance = delta.length();

    if distance > enemy.activation_range {
        return EnemyAiIntent::Idle;
    }

    match EnemyBehavior::from_tag(&enemy.behavior_tag) {
        EnemyBehavior::Melee => melee_intent(enemy, delta, distance, enemy.move_speed),
        EnemyBehavior::Ranged => ranged_intent(enemy, delta, distance),
        EnemyBehavior::Flanker => flanker_intent(enemy, delta, distance),
        EnemyBehavior::Aerial => melee_intent(enemy, delta, distance, enemy.move_speed * 1.15),
    }
}

pub(crate) fn advance_enemy_attack(
    runtime: &mut EnemyRuntimeState,
    enemy: &EnemyDefinition,
    dt: f32,
) -> bool {
    if runtime.stagger_remaining > 0.0
        || runtime.attack_cooldown_remaining > 0.0
        || enemy.damage <= 0.0
    {
        runtime.attack_windup_remaining = 0.0;
        return false;
    }

    if enemy.attack_windup <= 0.0 {
        runtime.attack_cooldown_remaining = enemy.attack_cooldown;
        return true;
    }

    if runtime.attack_windup_remaining <= 0.0 {
        runtime.attack_windup_remaining = enemy.attack_windup;
    }

    runtime.attack_windup_remaining = (runtime.attack_windup_remaining - dt).max(0.0);
    if runtime.attack_windup_remaining <= f32::EPSILON {
        runtime.attack_windup_remaining = 0.0;
        runtime.attack_cooldown_remaining = enemy.attack_cooldown;
        return true;
    }

    false
}

fn melee_intent(
    enemy: &EnemyDefinition,
    delta: Vec3,
    distance: f32,
    move_speed: f32,
) -> EnemyAiIntent {
    if distance <= enemy.attack_range {
        return EnemyAiIntent::Attack;
    }
    move_toward(delta, distance, move_speed)
}

fn ranged_intent(enemy: &EnemyDefinition, delta: Vec3, distance: f32) -> EnemyAiIntent {
    if distance <= 0.001 {
        return EnemyAiIntent::Attack;
    }

    let comfort_min = enemy.attack_range * 0.35;
    if distance < comfort_min && enemy.move_speed > 0.0 {
        let direction = -delta / distance;
        let speed = enemy.move_speed * 0.75;
        return EnemyAiIntent::Move {
            velocity_x: direction.x * speed,
            velocity_z: direction.z * speed,
        };
    }

    if distance <= enemy.attack_range {
        return EnemyAiIntent::Attack;
    }

    move_toward(delta, distance, enemy.move_speed)
}

fn flanker_intent(enemy: &EnemyDefinition, delta: Vec3, distance: f32) -> EnemyAiIntent {
    if distance <= 0.001 || enemy.move_speed <= 0.0 {
        return EnemyAiIntent::Idle;
    }
    if distance <= enemy.attack_range {
        return EnemyAiIntent::Attack;
    }

    let direction = delta / distance;
    let tangent = Vec3::new(-direction.z, 0.0, direction.x);
    let flank_direction = (direction * 0.55 + tangent * 0.85).normalize_or_zero();
    EnemyAiIntent::Move {
        velocity_x: flank_direction.x * enemy.move_speed,
        velocity_z: flank_direction.z * enemy.move_speed,
    }
}

fn move_toward(delta: Vec3, distance: f32, move_speed: f32) -> EnemyAiIntent {
    if distance <= 0.001 || move_speed <= 0.0 {
        return EnemyAiIntent::Idle;
    }
    let direction = delta / distance;
    EnemyAiIntent::Move {
        velocity_x: direction.x * move_speed,
        velocity_z: direction.z * move_speed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::world::level::ColliderType;

    fn enemy_fixture() -> EnemyDefinition {
        EnemyDefinition {
            id: "ashbound".to_string(),
            display_name: "Ashbound".to_string(),
            role: "grunt".to_string(),
            behavior_tag: "chase_melee".to_string(),
            model_asset: "Cube.obj".to_string(),
            collider_type: ColliderType::Sphere,
            visual_tell: "test silhouette".to_string(),
            health: 35.0,
            damage: 8.0,
            move_speed: 3.0,
            activation_range: 10.0,
            attack_range: 1.5,
            attack_windup: 0.5,
            attack_cooldown: 1.0,
        }
    }

    #[test]
    fn enemy_intent_idles_outside_activation_range() {
        let enemy = enemy_fixture();

        assert_eq!(
            enemy_ai_intent(&enemy, Vec3::ZERO, Vec3::new(12.0, 0.0, 0.0)),
            EnemyAiIntent::Idle
        );
    }

    #[test]
    fn enemy_intent_chases_inside_activation_range() {
        let enemy = enemy_fixture();

        let EnemyAiIntent::Move {
            velocity_x,
            velocity_z,
        } = enemy_ai_intent(&enemy, Vec3::ZERO, Vec3::new(3.0, 0.0, 4.0))
        else {
            panic!("enemy should chase when inside activation range");
        };

        assert!((velocity_x - 1.8).abs() < 0.001);
        assert!((velocity_z - 2.4).abs() < 0.001);
    }

    #[test]
    fn enemy_attack_windup_delays_damage() {
        let enemy = enemy_fixture();
        let mut runtime = EnemyRuntimeState::default();

        assert!(!advance_enemy_attack(&mut runtime, &enemy, 0.2));
        assert!(runtime.attack_windup_remaining > 0.0);
        assert!(!advance_enemy_attack(&mut runtime, &enemy, 0.2));
        assert!(advance_enemy_attack(&mut runtime, &enemy, 0.1));
        assert_eq!(runtime.attack_cooldown_remaining, enemy.attack_cooldown);
    }

    #[test]
    fn enemy_attack_cooldown_blocks_windup() {
        let enemy = enemy_fixture();
        let mut runtime = EnemyRuntimeState {
            attack_cooldown_remaining: 0.25,
            attack_windup_remaining: 0.1,
            stagger_remaining: 0.0,
            path_waypoint: 0,
        };

        assert!(!advance_enemy_attack(&mut runtime, &enemy, 0.1));
        assert_eq!(runtime.attack_windup_remaining, 0.0);
    }

    #[test]
    fn ranged_enemy_holds_at_attack_range() {
        let mut enemy = enemy_fixture();
        enemy.behavior_tag = "ranged_windup".to_string();
        enemy.attack_range = 8.0;

        assert_eq!(
            enemy_ai_intent(&enemy, Vec3::ZERO, Vec3::new(6.0, 0.0, 0.0)),
            EnemyAiIntent::Attack
        );
    }

    #[test]
    fn ranged_enemy_retreats_when_too_close() {
        let mut enemy = enemy_fixture();
        enemy.behavior_tag = "ranged_windup".to_string();
        enemy.attack_range = 8.0;

        let EnemyAiIntent::Move {
            velocity_x,
            velocity_z,
        } = enemy_ai_intent(&enemy, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0))
        else {
            panic!("ranged enemy should retreat when player is too close");
        };

        assert!(velocity_x < 0.0);
        assert_eq!(velocity_z, 0.0);
    }

    #[test]
    fn flanker_enemy_moves_laterally_while_closing() {
        let mut enemy = enemy_fixture();
        enemy.behavior_tag = "flanker_lunge".to_string();

        let EnemyAiIntent::Move {
            velocity_x,
            velocity_z,
        } = enemy_ai_intent(&enemy, Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0))
        else {
            panic!("flanker should move");
        };

        assert!(velocity_x > 0.0);
        assert!(velocity_z > 0.0);
    }

    #[test]
    fn stagger_blocks_attack_windup() {
        let enemy = enemy_fixture();
        let mut runtime = EnemyRuntimeState::default();
        runtime.stagger(0.2);

        assert!(!advance_enemy_attack(&mut runtime, &enemy, 0.5));
        assert_eq!(runtime.attack_windup_remaining, 0.0);
    }
}
