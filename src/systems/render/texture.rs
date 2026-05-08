// src/render/texture.rs
// TextureManager holds GPU bind groups keyed by filename.
// Missing textures fall back to a magenta/black checkerboard so broken
// assets are immediately visible without crashing.

use std::collections::HashMap;

pub struct TextureManager {
    textures: HashMap<String, wgpu::BindGroup>,
    fallback: wgpu::BindGroup,
}

impl TextureManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let fallback = Self::create_fallback(device, queue, layout);
        Self {
            textures: HashMap::new(),
            fallback,
        }
    }

    pub fn insert(&mut self, name: &str, bind_group: wgpu::BindGroup) {
        self.textures.insert(name.to_string(), bind_group);
    }

    /// Returns the bind group for `name`, or the fallback checkerboard.
    pub fn get(&self, name: &str) -> &wgpu::BindGroup {
        self.textures.get(name).unwrap_or(&self.fallback)
    }

    // ── Fallback texture ──────────────────────────────────────────────────────

    fn create_fallback(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        // 4x4 light gray/dark gray checkerboard
        let light = [200u8, 200, 200, 255];
        let dark = [100u8, 100, 100, 255];
        let mut data = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                if (x + y) % 2 == 0 {
                    data.extend_from_slice(&light);
                } else {
                    data.extend_from_slice(&dark);
                }
            }
        }

        let size = wgpu::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fallback Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(16),
                rows_per_image: Some(4),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Fallback Texture Bind Group"),
        })
    }
}
