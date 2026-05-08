// src/engine/sync.rs
// Instance buffer synchronisation.
// Rebuilds the per-asset GPU instance buffers from the current LevelData props.
// Called after any prop is added, moved, or removed.

use std::collections::HashMap;

use glam::{Quat, Vec3};
use wgpu::util::DeviceExt;

use crate::core::engine::state::EngineState;
use crate::systems::render::assets::DrawGroup;
use crate::systems::render::instance::{Instance, InstanceRaw};

impl EngineState {
    /// Rebuild all `active_draw_groups` from `self.level_data.props`.
    /// Must be called whenever the prop list changes.
    pub fn sync_instances(&mut self) {
        let mut groupings: HashMap<String, Vec<Instance>> = HashMap::new();

        for prop in &self.level_data.props {
            let instance = Instance {
                position: Vec3::new(prop.position[0], prop.position[1], prop.position[2]),
                rotation: Quat::from_euler(
                    glam::EulerRot::XYZ,
                    prop.rotation[0],
                    prop.rotation[1],
                    prop.rotation[2],
                ),
                scale: Vec3::new(prop.scale[0], prop.scale[1], prop.scale[2]),
            };
            groupings
                .entry(prop.asset_id.clone())
                .or_default()
                .push(instance);
        }

        self.active_draw_groups.clear();
        for (asset_id, instances) in groupings {
            let raw: Vec<InstanceRaw> = instances.iter().map(Instance::to_raw).collect();
            let instance_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Draw Group Instance Buffer"),
                        contents: bytemuck::cast_slice(&raw),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            self.active_draw_groups.push(DrawGroup {
                asset_id,
                num_instances: instances.len() as u32,
                instance_buffer,
            });
        }
    }
}
