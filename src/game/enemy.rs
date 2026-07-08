use glam::Vec3;

use crate::data::enemy::EnemyDefinition;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EnemyRuntimeState {
    pub attack_cooldown_remaining: f32,
    pub attack_windup_remaining: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EnemyAiIntent {
    Idle,
    Chase { velocity_x: f32, velocity_z: f32 },
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
    if distance <= enemy.attack_range {
        return EnemyAiIntent::Attack;
    }
    if distance <= 0.001 || enemy.move_speed <= 0.0 {
        return EnemyAiIntent::Idle;
    }

    let direction = delta / distance;
    EnemyAiIntent::Chase {
        velocity_x: direction.x * enemy.move_speed,
        velocity_z: direction.z * enemy.move_speed,
    }
}

pub(crate) fn advance_enemy_attack(
    runtime: &mut EnemyRuntimeState,
    enemy: &EnemyDefinition,
    dt: f32,
) -> bool {
    if runtime.attack_cooldown_remaining > 0.0 || enemy.damage <= 0.0 {
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

        let EnemyAiIntent::Chase {
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
        };

        assert!(!advance_enemy_attack(&mut runtime, &enemy, 0.1));
        assert_eq!(runtime.attack_windup_remaining, 0.0);
    }
}
