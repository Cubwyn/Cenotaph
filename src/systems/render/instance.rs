// src/systems/render/instance.rs
// Per-instance transform data uploaded to the GPU as a vertex buffer.

use glam::{Mat4, Quat, Vec3};

/// CPU-side instance transform.
pub struct Instance {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub tint: [f32; 4],
    /// x = UV scale, y = emissive strength; z/w are reserved.
    pub material: [f32; 4],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let model = Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        InstanceRaw {
            model: model.to_cols_array_2d(),
            tint: self.tint,
            material: self.material,
        }
    }
}

/// GPU-side instance data (column-major 4×4 matrix).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub tint: [f32; 4],
    pub material: [f32; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[[f32; 4]; 4]>() + mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
