// src/render/lighting.rs
// Basic lighting and fog system for Cenotaph's underground environments

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub intensity: f32,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FogUniform {
    pub density: f32,
    pub _padding1: [u32; 3],
    pub color: [f32; 3],
    pub _padding2: u32,
}

pub struct LightingSystem {
    pub light_bind_group: wgpu::BindGroup,
    pub fog_bind_group: wgpu::BindGroup,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub fog_bind_group_layout: wgpu::BindGroupLayout,
    light_buffer: wgpu::Buffer,
    fog_buffer: wgpu::Buffer,
}

impl LightingSystem {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        // Default light: dim ambient light for underground caverns
        let light_data = LightUniform {
            position: [0.0, 10.0, 0.0],
            _padding: 0,
            color: [0.1, 0.1, 0.15],
            intensity: 1.0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let fog_data = FogUniform {
            density: 0.05,
            _padding1: [0; 3],
            color: [0.1, 0.1, 0.15],
            _padding2: 0,
        };

        let fog_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fog Buffer"),
            contents: bytemuck::cast_slice(&[fog_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind groups
        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("light_bind_group_layout"),
        });

        let fog_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("fog_bind_group_layout"),
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("light_bind_group"),
        });

        let fog_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &fog_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: fog_buffer.as_entire_binding(),
            }],
            label: Some("fog_bind_group"),
        });

        Self {
            light_bind_group,
            fog_bind_group,
            light_bind_group_layout,
            fog_bind_group_layout,
            light_buffer,
            fog_buffer,
        }
    }

    pub fn update_light(&self, queue: &wgpu::Queue, position: [f32; 3], color: [f32; 3], intensity: f32) {
        let light_data = LightUniform {
            position,
            _padding: 0,
            color,
            intensity,
        };
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[light_data]));
    }

    pub fn update_fog(&self, queue: &wgpu::Queue, density: f32, color: [f32; 3]) {
        let fog_data = FogUniform {
            density,
            _padding1: [0; 3],
            color,
            _padding2: 0,
        };
        queue.write_buffer(&self.fog_buffer, 0, bytemuck::cast_slice(&[fog_data]));
    }

    pub fn get_light_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.light_bind_group_layout
    }

    pub fn get_fog_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.fog_bind_group_layout
    }
}