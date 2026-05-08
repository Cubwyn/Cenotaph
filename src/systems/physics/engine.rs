// src/physics/engine.rs
// Simplified physics engine for debugging

use rapier3d::prelude::*;
use rapier3d::math::Vec3 as RVec3;
use glam::Vec3;

use crate::data::config::gameplay::PhysicsConfig;
use crate::data::world::level::{ColliderType, PropData};

pub struct PhysicsEngine {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
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
    pub prop_colliders: Vec<ColliderHandle>,
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

        // Create level geometry collider from the actual level mesh data
        // Apply Y offset used in rendering (124.5) to align physics with visuals
        let map_y_offset = 124.5;
        
        if !phys_points.is_empty() && !phys_indices.is_empty() {
            // Offset the mesh vertices by the rendering Y offset so physics aligns with visuals
            let rp: Vec<RVec3> = phys_points
                .iter()
                .map(|p| RVec3::new(p.x, p.y + map_y_offset, p.z))
                .collect();
            
            // Create a static rigid body for the level mesh
            let ground_rb = RigidBodyBuilder::fixed().build();
            let ground_rb_handle = rigid_body_set.insert(ground_rb);
            
            // Create a triangle mesh collider from the actual level geometry
            let ground_collider = ColliderBuilder::trimesh(rp, phys_indices.to_vec())
                .unwrap()
                .friction(0.5)
                .build();
            collider_set.insert_with_parent(ground_collider, ground_rb_handle, &mut rigid_body_set);
            
            println!("[DEBUG] Physics: level mesh collider created from {} vertices, {} triangles", 
                phys_points.len(), phys_indices.len());
        } else {
            // Fallback: create ground at Y=125 if no level geometry
            let ground_rb = RigidBodyBuilder::fixed()
                .translation(RVec3::new(0.0, 125.0, 0.0))
                .build();
            let ground_rb_handle = rigid_body_set.insert(ground_rb);
            
            let ground_collider = ColliderBuilder::cuboid(200.0, 0.1, 200.0)
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

        // Player collider - sphere for better collision detection
        let player_collider = ColliderBuilder::ball(0.5)
            .restitution(0.0)
            .friction(0.0)
            .build();
        let player_collider_handle = collider_set.insert_with_parent(
            player_collider,
            player_body_handle,
            &mut rigid_body_set,
        );

        Self {
            rigid_body_set,
            collider_set,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            player_body_handle,
            player_collider_handle,
            prop_colliders: Vec::new(),
        }
    }

    pub fn add_prop(
        &mut self,
        prop: &PropData,
        phys_points: &[Vec3],
        phys_indices: &[[u32; 3]],
    ) {
        let is_dynamic = prop.enemy_type.is_some();
        let rb_builder = if is_dynamic {
            RigidBodyBuilder::dynamic()
        } else {
            RigidBodyBuilder::fixed()
        };

        let t = rapier3d::na::Translation3::new(
            prop.position[0],
            prop.position[1],
            prop.position[2],
        );
        let q = rapier3d::na::UnitQuaternion::identity();
        let pose = rapier3d::na::Isometry3::from_parts(t, q);
        let rb = rb_builder.pose(pose.into()).build();
        let handle = self.rigid_body_set.insert(rb);

        let collider_builder = match prop.collider_type {
            ColliderType::Box => Some(ColliderBuilder::cuboid(
                prop.scale[0] * 0.5,
                prop.scale[1] * 0.5,
                prop.scale[2] * 0.5,
            )),
            ColliderType::Sphere => Some(ColliderBuilder::ball(prop.scale[0] * 0.5)),
            ColliderType::Mesh => {
                let rp: Vec<RVec3> = phys_points
                    .iter()
                    .map(|p| RVec3::new(p.x, p.y, p.z))
                    .collect();
                Some(ColliderBuilder::trimesh(rp, phys_indices.to_vec()).unwrap())
            }
            ColliderType::None => None,
        };

        if let Some(builder) = collider_builder {
            #[allow(unused_mut)]
            let col = builder.sensor(prop.is_hurtbox).build();
            let col_handle =
                self.collider_set
                    .insert_with_parent(col, handle, &mut self.rigid_body_set);
            self.prop_colliders.push(col_handle);
        }
    }

    pub fn get_player_pos(&self) -> [f32; 3] {
        let body = self.rigid_body_set.get(self.player_body_handle).unwrap();
        let t = body.translation();
        [t.x, t.y, t.z]
    }

    pub fn apply_player_movement(
        &mut self,
        intent: [f32; 3],
        is_jumping: bool,
        config: &PhysicsConfig,
        dt: f32,
    ) {
        let body = self
            .rigid_body_set
            .get_mut(self.player_body_handle)
            .unwrap();

        body.set_gravity_scale(1.0, true);
        
        let cur = body.linvel();
        
        // Grounded check: if vertical velocity is near zero, we're on the ground
        // When touching ground, gravity is counteracted by normal force, resulting in ~0 Y velocity
        let grounded = cur.y.abs() < 0.5;
        
        let mut vel = RVec3::new(
            intent[0] * config.player_speed,
            cur.y,
            intent[2] * config.player_speed,
        );
        
        if is_jumping && grounded {
            vel.y = config.jump_velocity;
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