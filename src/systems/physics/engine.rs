// src/physics/engine.rs
// Physics engine using Rapier3D for collision detection and response.

use glam::Vec3;
use rapier3d::math::Vec3 as RVec3;
use rapier3d::prelude::*;

use crate::data::config::gameplay::PhysicsConfig;
use crate::data::world::level::{ColliderType, PropData};

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
    #[allow(dead_code)]
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
        phys_points: Vec<Vec3>,
        phys_indices: Vec<[u32; 3]>,
        _config: &PhysicsConfig,
    ) -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        // Apply Y offset used in rendering (124.5) to align physics with visuals
        let map_y_offset = 124.5;

        if !phys_points.is_empty() && !phys_indices.is_empty() {
            // Offset the mesh vertices by the rendering Y offset so physics aligns with visuals
            let rp: Vec<RVec3> = phys_points
                .iter()
                .map(|p| RVec3::new(p.x, p.y + map_y_offset, p.z))
                .collect();

            // Calculate bounds for safety floor
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;
            for p in &rp {
                min_y = min_y.min(p.y);
                max_y = max_y.max(p.y);
                min_x = min_x.min(p.x);
                max_x = max_x.max(p.x);
                min_z = min_z.min(p.z);
                max_z = max_z.max(p.z);
            }

            println!(
                "[DEBUG] Physics mesh bounds: Y=[{:.1}, {:.1}], X=[{:.1}, {:.1}], Z=[{:.1}, {:.1}]",
                min_y, max_y, min_x, max_x, min_z, max_z
            );

            // Create a static rigid body for the level mesh
            let ground_rb = RigidBodyBuilder::fixed().build();
            let ground_rb_handle = rigid_body_set.insert(ground_rb);

            // Create a triangle mesh collider from the actual level geometry
            let ground_collider = ColliderBuilder::trimesh(rp.clone(), phys_indices.to_vec())
                .unwrap()
                .friction(0.6)
                .build();
            collider_set.insert_with_parent(ground_collider, ground_rb_handle, &mut rigid_body_set);

            println!(
                "[DEBUG] Physics: level mesh collider created from {} vertices, {} triangles",
                phys_points.len(),
                phys_indices.len()
            );

            // SAFETY FLOOR: Add a flat floor 2 units below the lowest mesh point.
            // This catches the player if they fall through the mesh due to
            // numerical issues with large triangles and a small player collider.
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
            // Fallback: create ground at Y=125 if no level geometry
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

        // Player rigid body
        let player_rb = RigidBodyBuilder::dynamic()
            .translation(RVec3::new(spawn[0], spawn[1], spawn[2]))
            .lock_rotations()
            .build();
        let player_body_handle = rigid_body_set.insert(player_rb);

        // Player collider - use a slightly larger sphere (0.6 radius) for better
        // numerical stability against large mesh triangles. Friction > 0 gives the
        // player grip on the terrain to walk on slopes and prevents sliding.
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

    pub fn add_prop(&mut self, prop: &PropData, phys_points: &[Vec3], phys_indices: &[[u32; 3]]) {
        let is_dynamic = prop.enemy_type.is_some();
        let rb_builder = if is_dynamic {
            RigidBodyBuilder::dynamic().lock_rotations()
        } else {
            RigidBodyBuilder::fixed()
        };

        let t =
            rapier3d::na::Translation3::new(prop.position[0], prop.position[1], prop.position[2]);
        let q = rapier3d::na::UnitQuaternion::identity();
        let pose = rapier3d::na::Isometry3::from_parts(t, q);
        let rb = rb_builder.pose(pose.into()).build();
        let body_handle = self.rigid_body_set.insert(rb);

        let collider_builder = match prop.collider_type {
            ColliderType::Box => Some(ColliderBuilder::cuboid(
                prop.scale[0] * 0.5,
                prop.scale[1] * 0.5,
                prop.scale[2] * 0.5,
            )),
            ColliderType::Sphere => Some(ColliderBuilder::ball(prop.scale[0] * 0.5)),
            ColliderType::Mesh => {
                if phys_points.is_empty() || phys_indices.is_empty() {
                    eprintln!(
                        "[PHYSICS] Mesh collider for '{}' skipped because mesh data is empty.",
                        prop.asset_id
                    );
                    None
                } else {
                    let rp: Vec<RVec3> = phys_points
                        .iter()
                        .map(|p| RVec3::new(p.x, p.y, p.z))
                        .collect();
                    match ColliderBuilder::trimesh(rp, phys_indices.to_vec()) {
                        Ok(builder) => Some(builder),
                        Err(e) => {
                            eprintln!(
                                "[PHYSICS] Mesh collider for '{}' failed: {}",
                                prop.asset_id, e
                            );
                            None
                        }
                    }
                }
            }
            ColliderType::None => None,
        };

        let collider_handle = collider_builder.map(|builder| {
            let col = builder.sensor(prop.is_hurtbox).build();
            self.collider_set
                .insert_with_parent(col, body_handle, &mut self.rigid_body_set)
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

        // Build a query pipeline that excludes the player's own collider.
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

    pub fn apply_player_movement(
        &mut self,
        intent: [f32; 3],
        is_jumping: bool,
        config: &PhysicsConfig,
        dt: f32,
        speed_multiplier: f32,
    ) {
        // Read state BEFORE taking the mutable borrow on the rigid body.
        let on_ground = self.check_grounded();

        // Update coyote timer:
        //   - If on ground, reset the timer.
        //   - If airborne, count down.
        if on_ground {
            self.coyote_timer = COYOTE_TIME;
        } else {
            self.coyote_timer = (self.coyote_timer - dt).max(0.0);
        }

        // Edge-triggered jump: only jump on the key-down transition.
        let jump_just_pressed = is_jumping && !self.jump_was_pressed;
        self.jump_was_pressed = is_jumping;

        let body = self
            .rigid_body_set
            .get_mut(self.player_body_handle)
            .unwrap();

        body.set_gravity_scale(1.0, true);

        let cur = body.linvel();

        // Allow jumping if:
        //   a) Actually on the ground, OR
        //   b) Within coyote-time window (just stepped off a ledge).
        let can_jump = on_ground || self.coyote_timer > 0.0;

        let effective_speed = config.player_speed * speed_multiplier;

        let mut vel = RVec3::new(
            intent[0] * effective_speed,
            cur.y,
            intent[2] * effective_speed,
        );

        if jump_just_pressed && can_jump {
            vel.y = config.jump_velocity;
            // Clear coyote timer so it's consumed by this jump.
            self.coyote_timer = 0.0;
        }

        body.set_linvel(vel, true);
    }

    pub fn step(&mut self, config: &PhysicsConfig, dt: f32) {
        let gravity = RVec3::new(0.0, config.gravity, 0.0);

        // Use the passed dt for physics simulation to make it frame-rate independent
        let integration_params = IntegrationParameters {
            dt: dt.clamp(0.001, 0.033), // Clamp to prevent extreme jumps in dt
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
            asset_id: "Cube.obj".to_string(),
            position: [1.0, 126.0, -2.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            collider_type: ColliderType::Sphere,
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
}
