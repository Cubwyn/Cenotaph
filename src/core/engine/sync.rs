// src/engine/sync.rs
// Instance buffer synchronisation.
// Rebuilds the per-asset GPU instance buffers from the current LevelData props.
// Called after any prop is added, moved, or removed.

use std::collections::HashMap;

use glam::{Quat, Vec3};
use wgpu::util::DeviceExt;

use crate::core::engine::state::EngineState;
use crate::data::world::level::BrushGeometryData;
use crate::systems::render::assets::{DrawGroup, RenderAsset, RenderAssetMeshPart};
use crate::systems::render::instance::{Instance, InstanceRaw};
use crate::systems::render::mesh::Vertex;

impl EngineState {
    /// Rebuild all `active_draw_groups` from `self.level_data.props`.
    /// Must be called whenever the prop list changes.
    pub fn sync_instances(&mut self) {
        let mut groupings: HashMap<String, Vec<Instance>> = HashMap::new();

        for (index, prop) in self.level_data.props.iter().enumerate() {
            let rotation = prop.rotation_radians();
            let instance = Instance {
                position: Vec3::new(prop.position[0], prop.position[1], prop.position[2]),
                rotation: Quat::from_euler(
                    glam::EulerRot::XYZ,
                    rotation[0],
                    rotation[1],
                    rotation[2],
                ),
                scale: Vec3::new(prop.scale[0], prop.scale[1], prop.scale[2]),
            };
            let asset_id = if let Some(geometry) = prop.brush_geometry.as_ref() {
                let asset_id = format!("__brush_geometry_{}", index);
                if let Some(asset) = build_brush_render_asset(&self.device, geometry) {
                    self.assets.insert(asset_id.clone(), asset);
                    asset_id
                } else {
                    prop.asset_id.clone()
                }
            } else {
                prop.asset_id.clone()
            };
            groupings.entry(asset_id).or_default().push(instance);
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

fn build_brush_render_asset(
    device: &wgpu::Device,
    geometry: &BrushGeometryData,
) -> Option<RenderAsset> {
    if geometry.vertices.len() < 3 || geometry.faces.is_empty() {
        return None;
    }

    let mut indices = Vec::with_capacity(geometry.faces.len() * 3);
    let mut normals = vec![Vec3::ZERO; geometry.vertices.len()];
    for face in &geometry.faces {
        let [a, b, c] = *face;
        let vertex_count = geometry.vertices.len() as u32;
        if a >= vertex_count || b >= vertex_count || c >= vertex_count {
            continue;
        }

        let pa = Vec3::from_array(geometry.vertices[a as usize]);
        let pb = Vec3::from_array(geometry.vertices[b as usize]);
        let pc = Vec3::from_array(geometry.vertices[c as usize]);
        let normal = (pb - pa).cross(pc - pa);
        if normal.length_squared() <= 0.000001 {
            continue;
        }

        let normal = normal.normalize();
        normals[a as usize] += normal;
        normals[b as usize] += normal;
        normals[c as usize] += normal;
        indices.extend([a, b, c]);
    }

    if indices.is_empty() {
        return None;
    }

    let vertices: Vec<Vertex> = geometry
        .vertices
        .iter()
        .enumerate()
        .map(|(index, position)| {
            let normal = if normals[index].length_squared() > 0.000001 {
                normals[index].normalize()
            } else {
                Vec3::Y
            };
            Vertex {
                position: *position,
                tex_coords: [position[0], position[2]],
                normal: normal.to_array(),
            }
        })
        .collect();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Brush Geometry Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Brush Geometry Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Some(RenderAsset {
        vertex_buffer,
        parts: vec![RenderAssetMeshPart {
            index_buffer,
            num_indices: indices.len() as u32,
            texture_name: "default".to_string(),
        }],
    })
}
