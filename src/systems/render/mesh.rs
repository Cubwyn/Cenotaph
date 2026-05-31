// src/render/mesh.rs
// Vertex definition and model loading for .glb / .gltf / .obj files.
// Returns both render-ready data (vertices + indexed parts) and
// physics-ready data (point cloud + triangle indices).

use glam::Vec3;

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

// ── Model loading ─────────────────────────────────────────────────────────────

/// Dispatch loader based on file extension.
pub fn load_model(
    path: &str,
) -> (Vec<Vertex>, Vec<RenderMeshPart>, Vec<Vec3>, Vec<[u32; 3]>) {
    println!("[DEBUG] Loading model: {}", path);
    if path.ends_with(".glb") || path.ends_with(".gltf") {
        load_gltf(path)
    } else if path.ends_with(".obj") {
        load_obj(path)
    } else {
        panic!("Unsupported model format: {}", path);
    }
}

/// Load a GLTF / GLB model.
pub fn load_gltf(
    path: &str,
) -> (Vec<Vertex>, Vec<RenderMeshPart>, Vec<Vec3>, Vec<[u32; 3]>) {
    let (document, buffers, _) = gltf::import(path).expect("Failed to load GLB/GLTF");

    let mut vertices = Vec::new();
    let mut parts = Vec::new();
    let mut phys_points = Vec::new();
    let mut phys_indices = Vec::new();
    
    let num_nodes = document.nodes().count();
    println!("[DEBUG] GLTF: {} nodes found", num_nodes);

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
                    uv_iter
                        .into_f32()
                        .enumerate()
                        .for_each(|(i, uv)| {
                            vertices[start_vert as usize + i].tex_coords = uv;
                        });
                }

                if let Some(normal_iter) = reader.read_normals() {
                    normal_iter
                        .enumerate()
                        .for_each(|(i, n)| {
                            vertices[start_vert as usize + i].normal = n;
                        });
                }

                if let Some(idx_iter) = reader.read_indices() {
                    let local: Vec<u32> = idx_iter.into_u32().collect();
                    for chunk in local.chunks(3) {
                        if chunk.len() == 3 {
                            phys_indices.push([
                                chunk[0] + start_vert,
                                chunk[1] + start_vert,
                                chunk[2] + start_vert,
                            ]);
                        }
                    }
                    parts.push(RenderMeshPart {
                        indices: local.into_iter().map(|i| i + start_vert).collect(),
                        texture_name: "default".to_string(),
                    });
                }
            }
        }
    }

    (vertices, parts, phys_points, phys_indices)
}

/// Load an OBJ model.
pub fn load_obj(
    path: &str,
) -> (Vec<Vertex>, Vec<RenderMeshPart>, Vec<Vec3>, Vec<[u32; 3]>) {
    let (models, _) =
        tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS).expect("Failed to load OBJ");

    let mut vertices = Vec::new();
    let mut parts = Vec::new();
    let mut phys_points = Vec::new();
    let mut phys_indices = Vec::new();

    for model in models {
        let mesh = &model.mesh;
        let start_vert = vertices.len() as u32;

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

            let normal = if !mesh.normals.is_empty() {
                [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ]
            } else {
                [0.0, 1.0, 0.0]
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
            texture_name: "default".to_string(),
        });
    }

    (vertices, parts, phys_points, phys_indices)
}
