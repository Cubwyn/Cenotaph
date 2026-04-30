// src/physics/engine.rs
// Thin wrapper around rapier3d that owns the simulation state.
// Exposes a minimal interface so the rest of the engine never imports rapier directly.
//
// This module provides a high-level physics interface for the game, handling
// player movement, collision detection, and rigid body simulation. It abstracts
// away the complexity of the underlying rapier3d physics engine while providing
// the necessary functionality for gameplay systems.

use rapier3d::prelude::*;
use rapier3d::geometry::BroadPhaseBvh;
use rapier3d::math::Vec3 as RVec3;
use glam::Vec3;

use crate::config::gameplay::PhysicsConfig;
use crate::world::level::{ColliderType, PropData};

/// Main physics engine that manages all rigid bodies and collision detection.
/// 
/// This struct owns all physics simulation state and provides a clean interface
/// for gameplay systems to interact with the physics world. It handles player
/// movement, collision detection, and prop physics.
pub struct PhysicsEngine {
    /// Set of all rigid bodies in the simulation (player, enemies, dynamic props)
    pub rigid_body_set: RigidBodySet,
    /// Set of all colliders attached to rigid bodies
    pub collider_set: ColliderSet,
    /// Integration parameters controlling simulation accuracy and performance
    pub integration_parameters: IntegrationParameters,
    /// Main physics pipeline that steps the simulation forward
    pub physics_pipeline: PhysicsPipeline,
    /// Manages sleeping bodies and island detection for optimization
    pub island_manager: IslandManager,
    /// Broad-phase collision detection for performance
    pub broad_phase: BroadPhaseBvh,
    /// Narrow-phase collision detection for precise results
    pub narrow_phase: NarrowPhase,
    /// Impulse joints for connecting rigid bodies
    pub impulse_joint_set: ImpulseJointSet,
    /// Multibody joints for complex joint systems
    pub multibody_joint_set: MultibodyJointSet,
    /// Continuous collision detection solver to prevent tunneling
    pub ccd_solver: CCDSolver,
    /// Handle to the player's rigid body for quick access
    pub player_body_handle: RigidBodyHandle,
    /// Handle to the player's collider (currently unused but reserved for future features)
    #[allow(dead_code)]
    pub player_collider_handle: ColliderHandle,
    /// Handles to all prop colliders for cleanup and management
    pub prop_colliders: Vec<ColliderHandle>,
}

impl PhysicsEngine {
    /// Creates a new physics engine with the base map and player setup.
    /// 
    /// # Parameters
    /// - `spawn`: Initial player spawn position [x, y, z]
    /// - `phys_points`: Vertices of the base map for collision detection
    /// - `phys_indices`: Triangle indices defining the base map geometry
    /// - `config`: Physics configuration parameters
    /// 
    /// # Returns
    /// A new PhysicsEngine instance ready for simulation
    pub fn new(
        spawn: [f32; 3],
        phys_points: Vec<Vec3>,
        phys_indices: Vec<[u32; 3]>,
        _config: &PhysicsConfig,
    ) -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        // Build static trimesh collider for the base map
        // This creates a collision mesh from the level geometry so the player
        // can walk on surfaces and collide with walls
        let rapier_points: Vec<RVec3> = phys_points
            .into_iter()
            .map(|p| RVec3::new(p.x, p.y, p.z))
            .collect();
        let map_collider = ColliderBuilder::trimesh(rapier_points, phys_indices)
            .unwrap()
            .build();
        collider_set.insert(map_collider);

        // Player rigid body at spawn position
        // The player is a dynamic body that responds to forces and collisions
        let t = rapier3d::na::Translation3::new(spawn[0], spawn[1], spawn[2]);
        let q = rapier3d::na::UnitQuaternion::identity();
        let pose = rapier3d::na::Isometry3::from_parts(t, q);

        let player_rb = RigidBodyBuilder::dynamic()
            .pose(pose.into())
            .lock_rotations()  // Prevent player from tipping over
            .build();
        let player_body_handle = rigid_body_set.insert(player_rb);

        // Player collider - capsule shape for smooth movement around obstacles
        let player_collider = ColliderBuilder::capsule_y(0.5, 0.3)  // 1.0 height, 0.6 radius
            .restitution(0.0)  // No bouncing
            .friction(0.0)     // Smooth sliding
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

    // ── Prop colliders ────────────────────────────────────────────────────────

    /// Adds a prop to the physics simulation with appropriate collision shape.
    /// 
    /// # Parameters
    /// - `prop`: Prop data containing position, scale, and collision type
    /// - `phys_points`: Vertices for mesh colliders (if applicable)
    /// - `phys_indices`: Triangle indices for mesh colliders (if applicable)
    /// 
    /// # Behavior
    /// - Dynamic props (enemies) get dynamic rigid bodies
    /// - Static props get fixed rigid bodies
    /// - Collision shape is determined by the prop's collider_type field
    pub fn add_prop(
        &mut self,
        prop: &PropData,
        phys_points: &[Vec3],
        phys_indices: &[[u32; 3]],
    ) {
        // Determine if this prop should be dynamic (enemies) or static (decorations)
        let is_dynamic = prop.enemy_type.is_some();
        let rb_builder = if is_dynamic {
            RigidBodyBuilder::dynamic()
        } else {
            RigidBodyBuilder::fixed()
        };

        // Set up the rigid body position and orientation
        let t = rapier3d::na::Translation3::new(
            prop.position[0],
            prop.position[1],
            prop.position[2],
        );
        let q = rapier3d::na::UnitQuaternion::identity();
        let pose = rapier3d::na::Isometry3::from_parts(t, q);
        let rb = rb_builder.pose(pose.into()).build();
        let handle = self.rigid_body_set.insert(rb);

        // Create appropriate collision shape based on prop type
        let collider_builder = match prop.collider_type {
            ColliderType::Box => Some(ColliderBuilder::cuboid(
                prop.scale[0] * 0.5,  // Half-extents for box collider
                prop.scale[1] * 0.5,
                prop.scale[2] * 0.5,
            )),
            ColliderType::Sphere => Some(ColliderBuilder::ball(prop.scale[0] * 0.5)),
            ColliderType::Mesh => {
                // Create a mesh collider from the prop's geometry
                let rp: Vec<RVec3> = phys_points
                    .iter()
                    .map(|p| RVec3::new(p.x, p.y, p.z))
                    .collect();
                Some(ColliderBuilder::trimesh(rp, phys_indices.to_vec()).unwrap())
            }
            ColliderType::None => None,  // No collision for this prop
        };

        if let Some(builder) = collider_builder {
            // Build the collider and attach it to the rigid body
            #[allow(unused_mut)]
            let col = builder.sensor(prop.is_hurtbox).build();
            let col_handle =
                self.collider_set
                    .insert_with_parent(col, handle, &mut self.rigid_body_set);
            self.prop_colliders.push(col_handle);
        }
    }

    // ── Player queries ────────────────────────────────────────────────────────

    /// Returns the current player position in world coordinates.
    /// 
    /// # Returns
    /// An array `[x, y, z]` representing the player's position in world space.
    /// 
    /// # Panics
    /// This function will panic if the player body handle is invalid.
    /// This should never happen in normal operation as the player body
    /// is created during physics initialization and never removed.
    /// 
    /// # Usage
    /// This is used by the camera system, rendering, and gameplay logic
    /// to determine where the player is located in the world.
    pub fn get_player_pos(&self) -> [f32; 3] {
        let body = self.rigid_body_set.get(self.player_body_handle).unwrap();
        let t = body.translation();
        [t.x, t.y, t.z]
    }

    // ── Player movement ───────────────────────────────────────────────────────

    /// Applies movement input to the player character.
    /// 
    /// # Parameters
    /// - `intent`: Movement vector [forward/back, up/down, left/right]
    /// - `is_jumping`: Whether the player is attempting to jump
    /// - `is_edit_mode`: Whether the editor is active (affects movement behavior)
    /// - `config`: Physics configuration for movement parameters
    /// 
    /// # Behavior
    /// - In edit mode: Player flies with no gravity, faster movement
    /// - In game mode: Normal physics-based movement with gravity and jumping
    /// - Movement is applied as velocity changes rather than position changes
    pub fn apply_player_movement(
        &mut self,
        intent: [f32; 3],
        is_jumping: bool,
        is_edit_mode: bool,
        config: &PhysicsConfig,
    ) {
        let body = self
            .rigid_body_set
            .get_mut(self.player_body_handle)
            .unwrap();

        if is_edit_mode {
            // Editor mode: Disable gravity and allow free flying
            body.set_gravity_scale(0.0, true);
            let fly_speed = config.player_speed * 2.0;  // Faster in editor
            body.set_linvel(
                RVec3::new(
                    intent[0] * fly_speed,
                    intent[1] * fly_speed,
                    intent[2] * fly_speed,
                ),
                true,
            );
        } else {
            // Game mode: Normal physics with gravity
            body.set_gravity_scale(1.0, true);
            let cur = body.linvel();
            let grounded = cur.y.abs() < 0.1;  // Check if player is on ground
            
            // Calculate new velocity based on input
            let mut vel = RVec3::new(
                intent[0] * config.player_speed,
                cur.y,  // Preserve vertical velocity (gravity effect)
                intent[2] * config.player_speed,
            );
            
            // Apply jump if player is grounded and jumping
            if is_jumping && grounded {
                vel.y = config.jump_velocity;
            }
            body.set_linvel(vel, true);
        }
    }

    // ── Editor helpers ────────────────────────────────────────────────────────

    /// Zero out the player body's velocity and disable gravity.
    /// Called every frame while the editor is active so the body stays frozen.
    /// 
    /// # Usage
    /// This is used in the level editor to prevent the player from falling
    /// or moving when not actively controlling the character, allowing for
    /// precise editing and camera positioning.
    pub fn freeze_player(&mut self) {
        let body = self
            .rigid_body_set
            .get_mut(self.player_body_handle)
            .unwrap();
        body.set_gravity_scale(0.0, false);
        body.set_linvel(RVec3::new(0.0, 0.0, 0.0), false);
        body.set_angvel(RVec3::new(0.0, 0.0, 0.0), false);
    }

    // ── Simulation step ───────────────────────────────────────────────────────

    /// Advances the physics simulation by one time step.
    /// 
    /// # Parameters
    /// - `config`: Physics configuration containing gravity settings
    /// 
    /// # Process
    /// This method runs the complete physics pipeline including:
    /// 1. Gravity application
    /// 2. Collision detection (broad and narrow phase)
    /// 3. Constraint solving
    /// 4. Integration (position/velocity updates)
    /// 
    /// # Timing
    /// This should be called once per frame with a consistent time step
    /// for stable simulation. The time step is controlled by the integration
    /// parameters set during initialization.
    pub fn step(&mut self, config: &PhysicsConfig) {
        let gravity = RVec3::new(0.0, config.gravity, 0.0);
        self.physics_pipeline.step(
            gravity,
            &self.integration_parameters,
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