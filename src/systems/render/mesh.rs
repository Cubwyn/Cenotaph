// src/systems/render/mesh.rs
// Vertex definition and model loading for .glb / .gltf / .obj files.
// Returns both render-ready data (vertices + indexed parts) and
// physics-ready data (point cloud + triangle indices).

use glam::Vec3;
use std::path::Path;

// ── Vertex ────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// ── Mesh part ─────────────────────────────────────────────────────────────────

/// A single draw call's worth of index data, tagged with a texture name.
pub struct RenderMeshPart {
    pub indices: Vec<u32>,
    pub texture_name: String,
}

pub type ModelData = (Vec<Vertex>, Vec<RenderMeshPart>, Vec<Vec3>, Vec<[u32; 3]>);

// ── Model loading ─────────────────────────────────────────────────────────────

pub fn try_load_model(path: &str) -> Result<ModelData, String> {
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    match extension.as_deref() {
        Some("glb" | "gltf") => load_gltf(path),
        Some("obj") => load_obj(path),
        Some(ext) => Err(format!("Unsupported model format '.{}': {}", ext, path)),
        None => Err(format!("Model path has no extension: {}", path)),
    }
}

#[cfg(test)]
pub fn empty_model() -> ModelData {
    (
        vec![Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

/// Load a GLTF / GLB model.
pub fn load_gltf(path: &str) -> Result<ModelData, String> {
    let (document, buffers, _) =
        gltf::import(path).map_err(|e| format!("Failed to load GLB/GLTF '{}': {}", path, e))?;

    let mut vertices = Vec::new();
    let mut parts = Vec::new();
    let mut phys_points = Vec::new();
    let mut phys_indices = Vec::new();

    for node in document.nodes() {
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buf| Some(&buffers[buf.index()]));
                let start_vert = vertices.len() as u32;

                if let Some(pos_iter) = reader.read_positions() {
                    for pos in pos_iter {
                        phys_points.push(Vec3::from_slice(&pos));
                        vertices.push(Vertex {
                            position: pos,
                            tex_coords: [0.0, 0.0],
                            normal: [0.0, 1.0, 0.0],
                        });
                    }
                }

                if let Some(uv_iter) = reader.read_tex_coords(0) {
                    uv_iter.into_f32().enumerate().for_each(|(i, uv)| {
                        vertices[start_vert as usize + i].tex_coords = uv;
                    });
                }

                if let Some(normal_iter) = reader.read_normals() {
                    normal_iter.enumerate().for_each(|(i, n)| {
                        vertices[start_vert as usize + i].normal = n;
                    });
                }

                let primitive_vertex_count = vertices.len() as u32 - start_vert;
                let local: Vec<u32> = reader
                    .read_indices()
                    .map(|indices| indices.into_u32().collect())
                    .unwrap_or_else(|| (0..primitive_vertex_count).collect());
                for chunk in local.chunks_exact(3) {
                    phys_indices.push([
                        chunk[0] + start_vert,
                        chunk[1] + start_vert,
                        chunk[2] + start_vert,
                    ]);
                }
                let texture_name = primitive
                    .material()
                    .pbr_metallic_roughness()
                    .base_color_texture()
                    .and_then(|info| match info.texture().source().source() {
                        gltf::image::Source::Uri { uri, .. } => texture_name_from_reference(uri),
                        gltf::image::Source::View { .. } => None,
                    })
                    .unwrap_or_else(|| "default".to_string());
                parts.push(RenderMeshPart {
                    indices: local.into_iter().map(|index| index + start_vert).collect(),
                    texture_name,
                });
            }
        }
    }

    Ok((vertices, parts, phys_points, phys_indices))
}

/// Load an OBJ model.
pub fn load_obj(path: &str) -> Result<ModelData, String> {
    let (models, materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)
        .map_err(|e| format!("Failed to load OBJ '{}': {}", path, e))?;
    let materials = materials.unwrap_or_default();

    let mut vertices = Vec::new();
    let mut parts = Vec::new();
    let mut phys_points = Vec::new();
    let mut phys_indices = Vec::new();

    for model in models {
        let mesh = &model.mesh;
        let start_vert = vertices.len() as u32;
        let generated_normals = mesh
            .normals
            .is_empty()
            .then(|| generate_normals(&mesh.positions, &mesh.indices));

        for i in 0..mesh.positions.len() / 3 {
            let pos = [
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            ];
            phys_points.push(Vec3::from_slice(&pos));

            let tex_coords = if !mesh.texcoords.is_empty() {
                [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]]
            } else {
                [0.0, 0.0]
            };

            let normal = if mesh.normals.len() >= (i + 1) * 3 {
                [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ]
            } else {
                generated_normals
                    .as_ref()
                    .and_then(|normals| normals.get(i))
                    .copied()
                    .unwrap_or([0.0, 1.0, 0.0])
            };

            vertices.push(Vertex {
                position: pos,
                tex_coords,
                normal,
            });
        }

        let local: Vec<u32> = mesh.indices.iter().map(|&i| i + start_vert).collect();
        for chunk in local.chunks(3) {
            if chunk.len() == 3 {
                phys_indices.push([chunk[0], chunk[1], chunk[2]]);
            }
        }
        parts.push(RenderMeshPart {
            indices: local,
            texture_name: mesh
                .material_id
                .and_then(|index| materials.get(index))
                .and_then(|material| material.diffuse_texture.as_deref())
                .and_then(texture_name_from_reference)
                .unwrap_or_else(|| "default".to_string()),
        });
    }

    Ok((vertices, parts, phys_points, phys_indices))
}

fn texture_name_from_reference(reference: &str) -> Option<String> {
    let normalized = reference.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.starts_with("data:") {
        return None;
    }
    if let Some((_, relative)) = normalized.rsplit_once("textures/") {
        return (!relative.is_empty()).then(|| relative.to_string());
    }
    normalized
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

fn generate_normals(positions: &[f32], indices: &[u32]) -> Vec<[f32; 3]> {
    let vertex_count = positions.len() / 3;
    let mut normals = vec![Vec3::ZERO; vertex_count];

    for triangle in indices.chunks_exact(3) {
        let [a, b, c] = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if a >= vertex_count || b >= vertex_count || c >= vertex_count {
            continue;
        }

        let point = |index: usize| {
            Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            )
        };
        let normal = (point(b) - point(a)).cross(point(c) - point(a));
        if normal.is_finite() && normal.length_squared() > 0.000001 {
            normals[a] += normal;
            normals[b] += normal;
            normals[c] += normal;
        }
    }

    normals
        .into_iter()
        .map(|normal| {
            if normal.length_squared() > 0.000001 {
                normal.normalize().to_array()
            } else {
                [0.0, 1.0, 0.0]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_surface_normals_for_untextured_obj_geometry() {
        let positions = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let normals = generate_normals(&positions, &[0, 2, 1]);

        assert_eq!(normals.len(), 3);
        assert!(normals.iter().all(|normal| normal[1] > 0.99));
    }

    #[test]
    fn degenerate_geometry_gets_a_finite_fallback_normal() {
        let normals = generate_normals(&[0.0; 9], &[0, 1, 2]);

        assert_eq!(normals, vec![[0.0, 1.0, 0.0]; 3]);
    }
}
