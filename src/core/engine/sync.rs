// src/engine/sync.rs
// Instance buffer synchronisation.
// Rebuilds buffers after structural edits and streams transforms for movement.

use std::collections::HashMap;

use glam::{Quat, Vec3};
use wgpu::util::DeviceExt;

use crate::core::engine::state::EngineState;
use crate::data::config::visuals::VisualConfig;
use crate::data::world::level::BrushGeometryData;
use crate::systems::render::assets::{DrawGroup, RenderAsset, RenderAssetMeshPart};
use crate::systems::render::instance::{Instance, InstanceRaw};
use crate::systems::render::mesh::Vertex;

impl EngineState {
    /// Rebuild all `active_draw_groups` from `self.level_data.props`.
    /// Must be called whenever the prop list changes.
    pub fn sync_instances(&mut self) {
        let mut groupings: HashMap<(String, Option<String>), Vec<Instance>> = HashMap::new();

        for (index, prop) in self.level_data.props.iter().enumerate() {
            let instance = instance_for_prop(prop, &self.config_data.visuals);
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
            let texture_override = texture_override_for_prop(&self.config_data.visuals, prop);
            groupings
                .entry((asset_id, texture_override))
                .or_default()
                .push(instance);
        }

        self.active_draw_groups.clear();
        for ((asset_id, texture_override), instances) in groupings {
            let raw: Vec<InstanceRaw> = instances.iter().map(Instance::to_raw).collect();
            let instance_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Draw Group Instance Buffer"),
                        contents: bytemuck::cast_slice(&raw),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
            self.active_draw_groups.push(DrawGroup {
                asset_id,
                texture_override,
                num_instances: instances.len() as u32,
                instance_buffer,
            });
        }
    }

    /// Stream transforms into existing buffers without allocating GPU resources.
    pub fn sync_dynamic_instances(&mut self) {
        let mut groupings: HashMap<(String, Option<String>), Vec<InstanceRaw>> = HashMap::new();
        for (index, prop) in self.level_data.props.iter().enumerate() {
            let asset_id = if prop.brush_geometry.is_some() {
                format!("__brush_geometry_{}", index)
            } else {
                prop.asset_id.clone()
            };
            let texture_override = texture_override_for_prop(&self.config_data.visuals, prop);
            groupings
                .entry((asset_id, texture_override))
                .or_default()
                .push(instance_for_prop(prop, &self.config_data.visuals).to_raw());
        }

        let layout_matches = groupings.len() == self.active_draw_groups.len()
            && self.active_draw_groups.iter().all(|group| {
                groupings
                    .get(&(group.asset_id.clone(), group.texture_override.clone()))
                    .is_some_and(|instances| instances.len() as u32 == group.num_instances)
            });
        if !layout_matches {
            self.sync_instances();
            return;
        }

        for group in &self.active_draw_groups {
            let instances = &groupings[&(group.asset_id.clone(), group.texture_override.clone())];
            self.queue
                .write_buffer(&group.instance_buffer, 0, bytemuck::cast_slice(instances));
        }
    }
}

fn instance_for_prop(
    prop: &crate::data::world::level::PropData,
    visuals: &VisualConfig,
) -> Instance {
    let rotation = prop.rotation_radians();
    let material = prop.surface_material.as_ref();
    let profile = visuals.profile_for(&prop.asset_id, prop.enemy_type.as_deref(), prop.is_hurtbox);
    let tint = prop_tint(visuals, prop);
    let material_tint = material.map_or([1.0; 3], |material| material.tint);
    let visual_role = prop_visual_role(visuals, prop);
    let default_emissive = match visual_role as u32 {
        1 => 0.07,
        3 => 0.04,
        4 => 0.16,
        _ => 0.0,
    };
    let phase = (prop.position[0] * 0.73 + prop.position[1] * 0.19 + prop.position[2] * 0.41)
        .sin()
        .abs()
        * std::f32::consts::TAU;
    Instance {
        position: Vec3::from_array(prop.position),
        rotation: Quat::from_euler(glam::EulerRot::XYZ, rotation[0], rotation[1], rotation[2]),
        scale: Vec3::from_array(prop.scale),
        tint: [
            tint[0] * material_tint[0],
            tint[1] * material_tint[1],
            tint[2] * material_tint[2],
            1.0,
        ],
        material: [
            material.map_or(profile.uv_scale, |material| material.uv_scale),
            material.map_or(profile.emissive.max(default_emissive), |material| {
                material.emissive.max(default_emissive)
            }),
            profile.animation_role.max(visual_role),
            phase,
        ],
    }
}

fn texture_override_for_prop(
    visuals: &VisualConfig,
    prop: &crate::data::world::level::PropData,
) -> Option<String> {
    if let Some(texture) = prop
        .surface_material
        .as_ref()
        .and_then(|material| material.texture.clone())
    {
        return Some(texture);
    }

    visuals
        .profile_for(&prop.asset_id, prop.enemy_type.as_deref(), prop.is_hurtbox)
        .texture
        .clone()
}

fn prop_visual_role(visuals: &VisualConfig, prop: &crate::data::world::level::PropData) -> f32 {
    let configured = visuals
        .profile_for(&prop.asset_id, prop.enemy_type.as_deref(), prop.is_hurtbox)
        .animation_role;
    if configured > 0.0 {
        return configured;
    }
    let asset = prop.asset_id.to_ascii_lowercase();
    if asset.contains("resource_shard") || asset.contains("relic_") {
        1.0
    } else if prop.enemy_type.is_some() {
        2.0
    } else if asset.contains("anchor") || asset.contains("transition_gate") {
        3.0
    } else if prop.is_hurtbox || asset.contains("hurtbox") {
        4.0
    } else {
        0.0
    }
}

fn prop_tint(visuals: &VisualConfig, prop: &crate::data::world::level::PropData) -> [f32; 4] {
    if let Some(color) = prop.light_color {
        return [color[0], color[1], color[2], 1.0];
    }

    let rgb = visuals
        .profile_for(&prop.asset_id, prop.enemy_type.as_deref(), prop.is_hurtbox)
        .tint;
    [rgb[0], rgb[1], rgb[2], 1.0]
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
