use glam::Vec3;

use crate::core::engine::state::EngineState;
use crate::data::world::level::LevelPathData;
use crate::game::enemy::{advance_enemy_attack, enemy_ai_intent, EnemyAiIntent, EnemyRuntimeState};

impl EngineState {
    pub(crate) fn ensure_enemy_runtime_matches_props(&mut self) {
        self.enemy_runtime.truncate(self.level_data.props.len());
        for prop in self.level_data.props.iter().skip(self.enemy_runtime.len()) {
            self.enemy_runtime.push(EnemyRuntimeState::for_max_health(
                prop.enemy_type.as_ref().map_or(0.0, |_| prop.enemy_health),
            ));
        }
    }

    pub(crate) fn update_enemy_ai(&mut self, dt: f32) {
        self.ensure_enemy_runtime_matches_props();

        for runtime in &mut self.enemy_runtime {
            runtime.tick(dt);
        }

        let player_pos = self.physics.get_player_pos();
        let player_v = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
        let enemies: Vec<_> = self
            .level_data
            .props
            .iter()
            .enumerate()
            .filter(|(_, prop)| prop.enemy_type.is_some() && prop.enemy_health > 0.0)
            .filter_map(|(index, prop)| {
                let enemy_type = prop.enemy_type.as_deref()?;
                let enemy = self.enemy_registry.get(enemy_type)?;
                Some((index, enemy.clone()))
            })
            .collect();

        for (index, enemy) in enemies {
            if self.player.is_dead || self.player.health.is_depleted() {
                self.physics.set_prop_horizontal_velocity(index, 0.0, 0.0);
                continue;
            }

            let Some(enemy_pos) = self
                .physics
                .get_prop_pos(index)
                .or_else(|| self.level_data.props.get(index).map(|prop| prop.position))
            else {
                continue;
            };
            let enemy_v = Vec3::new(enemy_pos[0], enemy_pos[1], enemy_pos[2]);

            match enemy_ai_intent(&enemy, enemy_v, player_v) {
                EnemyAiIntent::Idle => {
                    let velocity = self
                        .path_follow_velocity(index, enemy_pos, enemy.move_speed)
                        .unwrap_or((0.0, 0.0));
                    self.physics
                        .set_prop_horizontal_velocity(index, velocity.0, velocity.1);
                    if let Some(runtime) = self.enemy_runtime.get_mut(index) {
                        runtime.attack_windup_remaining = 0.0;
                    }
                }
                EnemyAiIntent::Move {
                    velocity_x,
                    velocity_z,
                } => {
                    self.physics
                        .set_prop_horizontal_velocity(index, velocity_x, velocity_z);
                    if let Some(runtime) = self.enemy_runtime.get_mut(index) {
                        runtime.clear_windup();
                    }
                }
                EnemyAiIntent::Attack => {
                    self.physics.set_prop_horizontal_velocity(index, 0.0, 0.0);
                    let should_damage = self
                        .enemy_runtime
                        .get_mut(index)
                        .is_some_and(|runtime| advance_enemy_attack(runtime, &enemy, dt));
                    if !should_damage {
                        continue;
                    }

                    let damage = self.cycle.enemy_damage(enemy.damage);
                    let source = format!("{} attack", enemy.display_name);
                    if self.apply_player_damage(&source, damage) {
                        break;
                    }
                }
            }
        }
    }

    pub(crate) fn update_non_enemy_path_followers(&mut self) {
        self.ensure_enemy_runtime_matches_props();
        let followers: Vec<_> = self
            .level_data
            .props
            .iter()
            .enumerate()
            .filter(|(_, prop)| prop.enemy_type.is_none() && prop.path_id.is_some())
            .filter_map(|(index, prop)| {
                self.physics
                    .get_prop_pos(index)
                    .or(Some(prop.position))
                    .map(|position| (index, position))
            })
            .collect();

        for (index, position) in followers {
            let velocity = self
                .path_follow_velocity(index, position, 1.0)
                .unwrap_or((0.0, 0.0));
            self.physics
                .set_prop_horizontal_velocity(index, velocity.0, velocity.1);
        }
    }

    pub(crate) fn path_follow_velocity(
        &mut self,
        prop_index: usize,
        prop_position: [f32; 3],
        base_speed: f32,
    ) -> Option<(f32, f32)> {
        let path_id = self
            .level_data
            .props
            .get(prop_index)?
            .path_id
            .as_deref()?
            .to_string();
        let path = self
            .level_data
            .paths
            .iter()
            .find(|path| path.id == path_id)?
            .clone();
        let runtime = self.enemy_runtime.get_mut(prop_index)?;
        path_velocity_for_runtime(runtime, &path, prop_position, base_speed)
    }

    pub(crate) fn sync_dynamic_prop_positions_from_physics(&mut self) -> bool {
        let mut changed = false;

        for (index, prop) in self.level_data.props.iter_mut().enumerate() {
            if prop.enemy_type.is_none() && prop.path_id.is_none() {
                continue;
            }
            let Some(position) = self.physics.get_prop_pos(index) else {
                continue;
            };

            let current = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
            let next = Vec3::new(position[0], position[1], position[2]);
            if current.distance_squared(next) > 0.000001 {
                prop.position = position;
                changed = true;
            }
        }

        changed
    }
}

pub(crate) fn path_velocity_for_runtime(
    runtime: &mut EnemyRuntimeState,
    path: &LevelPathData,
    prop_position: [f32; 3],
    base_speed: f32,
) -> Option<(f32, f32)> {
    if path.waypoints.len() < 2 || base_speed <= 0.0 {
        return None;
    }

    runtime.path_waypoint = runtime.path_waypoint.min(path.waypoints.len() - 1);
    let mut target = Vec3::new(
        path.waypoints[runtime.path_waypoint][0],
        path.waypoints[runtime.path_waypoint][1],
        path.waypoints[runtime.path_waypoint][2],
    );
    let position = Vec3::new(prop_position[0], prop_position[1], prop_position[2]);
    let mut delta = Vec3::new(target.x - position.x, 0.0, target.z - position.z);

    if delta.length() <= 0.35 {
        if runtime.path_waypoint + 1 < path.waypoints.len() {
            runtime.path_waypoint += 1;
        } else if path.looped {
            runtime.path_waypoint = 0;
        } else {
            return Some((0.0, 0.0));
        }

        target = Vec3::new(
            path.waypoints[runtime.path_waypoint][0],
            path.waypoints[runtime.path_waypoint][1],
            path.waypoints[runtime.path_waypoint][2],
        );
        delta = Vec3::new(target.x - position.x, 0.0, target.z - position.z);
    }

    let distance = delta.length();
    if distance <= 0.001 {
        return Some((0.0, 0.0));
    }

    let speed = base_speed * path.speed_multiplier.max(0.0);
    let direction = delta / distance;
    Some((direction.x * speed, direction.z * speed))
}
