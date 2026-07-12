// src/core/engine/loader.rs
// Disk I/O helpers for the engine.
// These functions upload disk assets to GPU managers without touching
// EngineState directly.

use std::fs;
use std::path::{Path, PathBuf};
use wgpu::util::DeviceExt;

use crate::systems::render::assets::{AssetManager, RenderAsset, RenderAssetMeshPart};
use crate::systems::render::mesh::try_load_model;
use crate::systems::render::texture::TextureManager;

const ASSETS_DIR: &str = "assets";
const TEXTURES_DIR: &str = "textures";
const BASE_MAP_ASSET_ID: &str = "map_001.glb";

/// Upload every PNG/JPG in `textures/` to the texture manager.
pub fn load_textures_from_disk(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    manager: &mut TextureManager,
) {
    fs::create_dir_all(TEXTURES_DIR).unwrap_or_default();
    let Ok(entries) = fs::read_dir(TEXTURES_DIR) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !is_supported_texture(&path) {
            continue;
        }

        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let img_data = match fs::read(&path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read texture file {}: {}",
                    path.display(),
                    e
                );
                continue;
            }
        };

        let img = match image::load_from_memory(&img_data) {
            Ok(img) => img,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to decode texture file {}: {}",
                    path.display(),
                    e
                );
                continue;
            }
        };

        let img_rgba = img.to_rgba8();
        let (w, h) = img_rgba.dimensions();

        let tex_size = wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(&file_name),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * w),
                rows_per_image: Some(h),
            },
            tex_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            label: Some(&file_name),
        });

        manager.insert(&file_name, bind_group);
    }
}

/// Upload every prop model under `assets/` except the base map.
pub fn load_prop_assets(device: &wgpu::Device, assets: &mut AssetManager) {
    let Ok(entries) = fs::read_dir(ASSETS_DIR) else {
        eprintln!("WARNING: Could not read assets/ directory.");
        return;
    };

    let mut found_any = false;
    let mut pending: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();

    while let Some(path) = pending.pop() {
        if path.is_dir() {
            if let Ok(sub_entries) = fs::read_dir(&path) {
                pending.extend(sub_entries.flatten().map(|e| e.path()));
            }
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if !is_supported_model(&path) {
            continue;
        }

        let Some(asset_id) = asset_id_for_path(&path) else {
            continue;
        };

        if asset_id == BASE_MAP_ASSET_ID {
            continue;
        }

        found_any = true;

        if let Some(path_str) = path.to_str() {
            let (vertices, parts, _pp, _pi) = match try_load_model(path_str) {
                Ok(model) => model,
                Err(error) => {
                    eprintln!(
                        "WARNING: Failed to load model asset {}: {}",
                        path.display(),
                        error
                    );
                    continue;
                }
            };

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Asset Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let mut render_parts = Vec::new();
            for part in parts {
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Asset Index Buffer"),
                    contents: bytemuck::cast_slice(&part.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                render_parts.push(RenderAssetMeshPart {
                    index_buffer,
                    num_indices: part.indices.len() as u32,
                    texture_name: part.texture_name,
                });
            }

            assets.insert(
                asset_id,
                RenderAsset {
                    vertex_buffer,
                    parts: render_parts,
                },
            );
        }
    }

    if !found_any {
        eprintln!("WARNING: No prop model files (.obj, .glb, .gltf) found in assets/");
    }
}

fn is_supported_texture(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"))
        .unwrap_or(false)
}

fn is_supported_model(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "glb" | "gltf" | "obj"))
        .unwrap_or(false)
}

fn asset_id_for_path(path: &Path) -> Option<String> {
    path.strip_prefix(ASSETS_DIR)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .or_else(|| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
}
