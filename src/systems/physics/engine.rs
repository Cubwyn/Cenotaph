//! Rapier-backed world collision, player movement, and dynamic prop bodies.

use glam::Vec3;
use rapier3d::math::Vec3 as RVec3;
use rapier3d::prelude::*;

use crate::data::config::gameplay::PhysicsConfig;
use crate::data::world::level::{ColliderType, PropData, BASE_MAP_Y_OFFSET};

/// How far below the player's centre to cast a ground-detection ray.
/// The player collider is a sphere of radius 0.6, so the ray length is
/// 0.7 = 0.6 (sphere radius) + 0.1 (small margin to avoid missed contacts).
const GROUND_RAY_LENGTH: f32 = 0.7;

/// Maximum time (seconds) after leaving the ground during which a jump
/// is still allowed (coyote time).
const COYOTE_TIME: f32 = 0.1;

pub struct PhysicsEngine {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhaseBvh,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub player_body_handle: RigidBodyHandle,
    pub player_collider_handle: ColliderHandle,
    pub prop_bodies: Vec<RigidBodyHandle>,
    pub prop_colliders: Vec<Option<ColliderHandle>>,
    /// Tracks whether jump key was held last frame (for edge-triggered jumping).
    jump_was_pressed: bool,
    /// Seconds left in the coyote-time window after leaving the ground.
    coyote_timer: f32,
}

impl PhysicsEngine {
    pub fn new(
        spawn: [f32; 3],
        physics_vertices: Vec<Vec3>,
        physics_triangles: Vec<[u32; 3]>,
        _config: &PhysicsConfig,
    ) -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        if !physics_vertices.is_empty() && !physics_triangles.is_empty() {
            // Base-map rendering and collision must share the same authored offset.
            let rapier_vertices: Vec<RVec3> = physics_vertices
                .iter()
                .map(|p| RVec3::new(p.x, p.y + BASE_MAP_Y_OFFSET, p.z))
                .collect();

            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;
            for point in &rapier_vertices {
                min_y = min_y.min(point.y);
                max_y = max_y.max(point.y);
                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
                min_z = min_z.min(point.z);
                max_z = max_z.max(point.z);
            }

            println!(
                "[DEBUG] Physics mesh bounds: Y=[{:.1}, {:.1}], X=[{:.1}, {:.1}], Z=[{:.1}, {:.1}]",
                min_y, max_y, min_x, max_x, min_z, max_z
            );

            let ground_rb = RigidBodyBuilder::fixed().build();
            let ground_rb_handle = rigid_body_set.insert(ground_rb);

            let ground_collider =
                ColliderBuilder::trimesh(rapier_vertices, physics_triangles.clone())
                    .unwrap()
                    .friction(0.6)
                    .build();
            collider_set.insert_with_parent(ground_collider, ground_rb_handle, &mut rigid_body_set);

            println!(
                "[DEBUG] Physics: level mesh collider created from {} vertices, {} triangles",
                physics_vertices.len(),
                physics_triangles.len()
            );

            // Catch rare tunneling beneath large map triangles without affecting play space.
            let safety_floor_y = min_y - 2.0;
            let half_width = ((max_x - min_x) * 0.5 + 10.0).max(100.0);
            let half_depth = ((max_z - min_z) * 0.5 + 10.0).max(100.0);
            let center_x = (min_x + max_x) * 0.5;
            let center_z = (min_z + max_z) * 0.5;

            let safety_rb = RigidBodyBuilder::fixed()
                .translation(RVec3::new(center_x, safety_floor_y, center_z))
                .build();
            let safety_rb_handle = rigid_body_set.insert(safety_rb);
            let safety_collider = ColliderBuilder::cuboid(half_width, 0.5, half_depth)
                .friction(0.5)
                .build();
            collider_set.insert_with_parent(safety_collider, safety_rb_handle, &mut rigid_body_set);

            println!(
                "[DEBUG] Physics: safety floor at Y={}, size {} x {}",
                safety_floor_y,
                half_width * 2.0,
                half_depth * 2.0
            );
        } else {
            // Keep diagnostic levels playable when no map geometry is available.
            let ground_rb = RigidBodyBuilder::fixed()
                .translation(RVec3::new(0.0, 125.0, 0.0))
                .build();
            let ground_rb_handle = rigid_body_set.insert(ground_rb);

            let ground_collider = ColliderBuilder::cuboid(200.0, 0.5, 200.0)
                .friction(2.0)
                .build();
            collider_set.insert_with_parent(ground_collider, ground_rb_handle, &mut rigid_body_set);

            println!("[DEBUG] Physics: fallback floor at Y=125");
        }

        let player_rb = RigidBodyBuilder::dynamic()
            .translation(RVec3::new(spawn[0], spawn[1], spawn[2]))
            .lock_rotations()
            .build();
        let player_body_handle = rigid_body_set.insert(player_rb);

        // Radius and friction are deliberately generous for stable traversal over
        // large map triangles and authored slopes.
        let player_collider = ColliderBuilder::ball(0.6)
            .restitution(0.0)
            .friction(0.6)
            .build();
        let player_collider_handle = collider_set.insert_with_parent(
            player_collider,
            player_body_handle,
            &mut rigid_body_set,
        );

        Self {
            rigid_body_set,
            collider_set,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            player_body_handle,
            player_collider_handle,
            prop_bodies: Vec::new(),
            prop_colliders: Vec::new(),
            jump_was_pressed: false,
            coyote_timer: 0.0,
        }
    }

    pub fn add_prop(
        &mut self,
        prop: &PropData,
        physics_vertices: &[Vec3],
        physics_triangles: &[[u32; 3]],
    ) {
        let is_dynamic = prop.enemy_type.is_some() || prop.path_id.is_some();
        let rb_builder = if is_dynamic {
            RigidBodyBuilder::dynamic().lock_rotations()
        } else {
            RigidBodyBuilder::fixed()
        };

        let translation =
            rapier3d::na::Translation3::new(prop.position[0], prop.position[1], prop.position[2]);
        let rotation = prop.rotation_radians();
        let rendered_rotation =
            glam::Quat::from_euler(glam::EulerRot::XYZ, rotation[0], rotation[1], rotation[2]);
        let rapier_rotation =
            rapier3d::na::UnitQuaternion::new_normalize(rapier3d::na::Quaternion::new(
                rendered_rotation.w,
                rendered_rotation.x,
                rendered_rotation.y,
                rendered_rotation.z,
            ));
        let pose = rapier3d::na::Isometry3::from_parts(translation, rapier_rotation);
        let rigid_body = rb_builder.pose(pose.into()).build();
        let body_handle = self.rigid_body_set.insert(rigid_body);

        let collider_builder = match prop.collider_type {
            ColliderType::Box => Some(ColliderBuilder::cuboid(
                prop.scale[0] * 0.5,
                prop.scale[1] * 0.5,
                prop.scale[2] * 0.5,
            )),
            ColliderType::Sphere => Some(ColliderBuilder::ball(prop.scale[0] * 0.5)),
            ColliderType::Mesh => {
                if physics_vertices.is_empty() || physics_triangles.is_empty() {
                    eprintln!(
                        "[PHYSICS] Mesh collider for '{}' skipped because mesh data is empty.",
                        prop.asset_id
                    );
                    None
                } else {
                    let scaled_vertices: Vec<RVec3> = physics_vertices
                        .iter()
                        .map(|p| {
                            RVec3::new(
                                p.x * prop.scale[0],
                                p.y * prop.scale[1],
                                p.z * prop.scale[2],
                            )
                        })
                        .collect();
                    match ColliderBuilder::trimesh(scaled_vertices, physics_triangles.to_vec()) {
                        Ok(builder) => Some(builder),
                        Err(error) => {
                            eprintln!(
                                "[PHYSICS] Mesh collider for '{}' failed: {}",
                                prop.asset_id, error
                            );
                            None
                        }
                    }
                }
            }
            ColliderType::None => None,
        };

        let collider_handle = collider_builder.map(|builder| {
            let collider = builder.sensor(prop.is_hurtbox).build();
            self.collider_set
                .insert_with_parent(collider, body_handle, &mut self.rigid_body_set)
        });
        self.prop_bodies.push(body_handle);
        self.prop_colliders.push(collider_handle);
    }

    pub fn remove_prop(&mut self, index: usize) {
        if index >= self.prop_bodies.len() {
            return;
        }

        let body_handle = self.prop_bodies.remove(index);
        self.rigid_body_set.remove(
            body_handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );

        if index < self.prop_colliders.len() {
            self.prop_colliders.remove(index);
        }
    }

    pub fn get_player_pos(&self) -> [f32; 3] {
        let body = self.rigid_body_set.get(self.player_body_handle).unwrap();
        let t = body.translation();
        [t.x, t.y, t.z]
    }

    pub fn get_player_velocity(&self) -> [f32; 3] {
        let body = self.rigid_body_set.get(self.player_body_handle).unwrap();
        let velocity = body.linvel();
        [velocity.x, velocity.y, velocity.z]
    }

    pub fn is_player_grounded(&self) -> bool {
        self.check_grounded()
    }

    pub fn get_prop_pos(&self, index: usize) -> Option<[f32; 3]> {
        let handle = *self.prop_bodies.get(index)?;
        let body = self.rigid_body_set.get(handle)?;
        let t = body.translation();
        Some([t.x, t.y, t.z])
    }

    pub fn set_prop_horizontal_velocity(&mut self, index: usize, x: f32, z: f32) {
        let Some(handle) = self.prop_bodies.get(index).copied() else {
            return;
        };
        let Some(body) = self.rigid_body_set.get_mut(handle) else {
            return;
        };

        let current = body.linvel();
        body.set_linvel(RVec3::new(x, current.y, z), true);
    }

    /// Returns `true` if there is ground directly beneath the player, by casting
    /// a short ray downward using a QueryPipeline built from the broad-phase.
    fn check_grounded(&self) -> bool {
        let body = self.rigid_body_set.get(self.player_body_handle).unwrap();
        let pos = body.translation();

        let ray = Ray::new(pos, RVec3::new(0.0, -1.0, 0.0));

        let filter = QueryFilter::default().exclude_collider(self.player_collider_handle);
        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            filter,
        );

        query_pipeline
            .cast_ray(&ray, GROUND_RAY_LENGTH, true)
            .is_some()
    }

    /// Returns the nearest solid world obstruction along a weapon ray.
    ///
    /// This intentionally ignores the player, sensors, and dynamic bodies
    /// (enemies), leaving combat target selection to the gameplay layer while
    /// still preventing hitscan shots from passing through walls and level mesh.
    pub fn weapon_obstruction_distance(
        &self,
        origin: Vec3,
        dir: Vec3,
        max_range: f32,
    ) -> Option<f32> {
        let dir_len_sq = dir.length_squared();
        let max_range = max_range.max(0.0);
        if dir_len_sq <= f32::EPSILON || max_range <= 0.0 {
            return None;
        }

        let dir = dir / dir_len_sq.sqrt();
        let ray = Ray::new(
            RVec3::new(origin.x, origin.y, origin.z),
            RVec3::new(dir.x, dir.y, dir.z),
        );
        let filter = QueryFilter::only_fixed()
            .exclude_sensors()
            .exclude_collider(self.player_collider_handle);
        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            filter,
        );

        query_pipeline
            .cast_ray(&ray, max_range, true)
            .map(|(_, distance)| distance.max(0.0))
    }

    pub fn apply_player_movement(
        &mut self,
        intent: [f32; 3],
        is_jumping: bool,
        config: &PhysicsConfig,
        dt: f32,
        speed_multiplier: f32,
    ) -> bool {
        // Query grounding before borrowing the body mutably.
        let on_ground = self.check_grounded();

        if on_ground {
            self.coyote_timer = COYOTE_TIME;
        } else {
            self.coyote_timer = (self.coyote_timer - dt).max(0.0);
        }

        let jump_just_pressed = is_jumping && !self.jump_was_pressed;
        self.jump_was_pressed = is_jumping;

        let body = self
            .rigid_body_set
            .get_mut(self.player_body_handle)
            .unwrap();

        body.set_gravity_scale(1.0, true);

        let current_velocity = body.linvel();

        let can_jump = on_ground || self.coyote_timer > 0.0;

        let effective_speed = config.player_speed * speed_multiplier;

        let mut vel = RVec3::new(
            intent[0] * effective_speed,
            current_velocity.y,
            intent[2] * effective_speed,
        );

        if jump_just_pressed && can_jump {
            vel.y = config.jump_velocity;
            self.coyote_timer = 0.0;
        }

        body.set_linvel(vel, true);
        jump_just_pressed && can_jump
    }

    pub fn step(&mut self, config: &PhysicsConfig, dt: f32) {
        let gravity = RVec3::new(0.0, config.gravity, 0.0);

        // Bound unusually long frames so one simulation step cannot tunnel far.
        let integration_params = IntegrationParameters {
            dt: dt.clamp(0.001, 0.033),
            ..Default::default()
        };

        self.physics_pipeline.step(
            gravity,
            &integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enemy_prop() -> PropData {
        PropData {
            id: None,
            display_name: None,
            asset_id: "Cube.obj".to_string(),
            position: [1.0, 126.0, -2.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            collider_type: ColliderType::Sphere,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id: None,
            resource_value: 0,
            anchor_id: None,
            enemy_type: Some("ashbound".to_string()),
            enemy_health: 40.0,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            loot_table_id: None,
            path_id: None,
            dialogue_id: None,
            event_id: None,
        }
    }

    fn wall_prop() -> PropData {
        PropData {
            id: None,
            display_name: None,
            asset_id: "Wall.obj".to_string(),
            position: [0.0, 126.0, -5.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [4.0, 4.0, 1.0],
            collider_type: ColliderType::Box,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id: None,
            resource_value: 0,
            anchor_id: None,
            enemy_type: None,
            enemy_health: 0.0,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            loot_table_id: None,
            path_id: None,
            dialogue_id: None,
            event_id: None,
        }
    }

    fn hurtbox_sensor_prop() -> PropData {
        PropData {
            id: None,
            display_name: None,
            asset_id: "Warning.obj".to_string(),
            position: [0.0, 126.0, -5.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [2.0, 2.0, 2.0],
            collider_type: ColliderType::Sphere,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: true,
            item_id: None,
            resource_value: 0,
            anchor_id: None,
            enemy_type: None,
            enemy_health: 0.0,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            loot_table_id: None,
            path_id: None,
            dialogue_id: None,
            event_id: None,
        }
    }

    #[test]
    fn dynamic_prop_position_and_velocity_are_exposed() {
        let mut engine = PhysicsEngine::new(
            [0.0, 126.0, 0.0],
            Vec::new(),
            Vec::new(),
            &PhysicsConfig::default(),
        );
        let prop = enemy_prop();

        engine.add_prop(&prop, &[], &[]);
        assert_eq!(engine.get_prop_pos(0), Some(prop.position));

        engine.set_prop_horizontal_velocity(0, 2.0, -1.0);
        let body = engine.rigid_body_set.get(engine.prop_bodies[0]).unwrap();
        let velocity = body.linvel();
        assert_eq!(velocity.x, 2.0);
        assert_eq!(velocity.z, -1.0);
    }

    #[test]
    fn weapon_obstruction_distance_hits_static_walls() {
        let mut engine = PhysicsEngine::new(
            [0.0, 126.0, 0.0],
            Vec::new(),
            Vec::new(),
            &PhysicsConfig::default(),
        );
        engine.add_prop(&wall_prop(), &[], &[]);
        engine.step(&PhysicsConfig::default(), 0.016);

        let distance = engine
            .weapon_obstruction_distance(
                Vec3::new(0.0, 126.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
                20.0,
            )
            .expect("static wall should block weapon ray");

        assert!(
            (4.45..=4.55).contains(&distance),
            "expected wall face around 4.5m, got {distance}"
        );
    }

    #[test]
    fn rotated_box_collider_matches_rendered_orientation() {
        let mut engine = PhysicsEngine::new(
            [0.0, 126.0, 0.0],
            Vec::new(),
            Vec::new(),
            &PhysicsConfig::default(),
        );
        let mut wall = wall_prop();
        wall.rotation = [0.0, 90.0, 0.0];
        engine.add_prop(&wall, &[], &[]);
        engine.step(&PhysicsConfig::default(), 0.016);

        let distance = engine
            .weapon_obstruction_distance(
                Vec3::new(0.0, 126.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
                20.0,
            )
            .expect("rotated wall should block weapon ray");

        assert!(
            (2.95..=3.05).contains(&distance),
            "expected rotated wall face around 3.0m, got {distance}"
        );
    }

    #[test]
    fn physics_rotation_order_matches_render_transform() {
        let mut engine = PhysicsEngine::new(
            [0.0, 126.0, 0.0],
            Vec::new(),
            Vec::new(),
            &PhysicsConfig::default(),
        );
        let mut wall = wall_prop();
        wall.rotation = [20.0, 35.0, 10.0];
        engine.add_prop(&wall, &[], &[]);

        let body = engine.rigid_body_set.get(engine.prop_bodies[0]).unwrap();
        let actual = body.position().rotation * RVec3::new(1.0, 0.0, 0.0);
        let radians = wall.rotation_radians();
        let expected =
            glam::Quat::from_euler(glam::EulerRot::XYZ, radians[0], radians[1], radians[2])
                * Vec3::X;

        assert!((actual.x - expected.x).abs() < 0.0001);
        assert!((actual.y - expected.y).abs() < 0.0001);
        assert!((actual.z - expected.z).abs() < 0.0001);
    }

    #[test]
    fn weapon_obstruction_distance_works_before_any_props_are_added() {
        let mut engine = PhysicsEngine::new(
            [0.0, 126.0, 0.0],
            Vec::new(),
            Vec::new(),
            &PhysicsConfig::default(),
        );
        engine.step(&PhysicsConfig::default(), 0.016);

        let distance = engine
            .weapon_obstruction_distance(
                Vec3::new(0.0, 128.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
                10.0,
            )
            .expect("fallback floor should block downward weapon ray immediately");

        assert!(
            (2.45..=2.55).contains(&distance),
            "expected fallback floor top around 2.5m, got {distance}"
        );
    }

    #[test]
    fn weapon_obstruction_distance_ignores_dynamic_enemies() {
        let mut engine = PhysicsEngine::new(
            [0.0, 126.0, 0.0],
            Vec::new(),
            Vec::new(),
            &PhysicsConfig::default(),
        );
        let mut enemy = enemy_prop();
        enemy.position = [0.0, 126.0, -5.0];
        engine.add_prop(&enemy, &[], &[]);
        engine.step(&PhysicsConfig::default(), 0.016);

        assert_eq!(
            engine.weapon_obstruction_distance(
                Vec3::new(0.0, 126.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
                20.0,
            ),
            None
        );
    }

    #[test]
    fn weapon_obstruction_distance_ignores_sensor_hurtboxes() {
        let mut engine = PhysicsEngine::new(
            [0.0, 126.0, 0.0],
            Vec::new(),
            Vec::new(),
            &PhysicsConfig::default(),
        );
        engine.add_prop(&hurtbox_sensor_prop(), &[], &[]);
        engine.step(&PhysicsConfig::default(), 0.016);

        assert_eq!(
            engine.weapon_obstruction_distance(
                Vec3::new(0.0, 126.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
                20.0,
            ),
            None
        );
    }
}
