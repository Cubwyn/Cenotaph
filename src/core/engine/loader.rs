// src/core/engine/loader.rs
// Disk I/O helpers for the engine.
// These functions upload disk assets to GPU managers without touching
// EngineState directly.

use std::fs;
use std::path::{Path, PathBuf};
use wgpu::util::DeviceExt;

use crate::core::engine::validation::validate_model_geometry;
use crate::systems::render::assets::{AssetManager, RenderAsset, RenderAssetMeshPart};
use crate::systems::render::mesh::try_load_model;
use crate::systems::render::texture::TextureManager;

const ASSETS_DIR: &str = "assets";
const TEXTURES_DIR: &str = "textures";
const BASE_MAP_ASSET_ID: &str = "map_001.glb";

#[derive(Debug, Default)]
pub struct DiskLoadReport {
    pub loaded: usize,
    pub issues: Vec<String>,
}

impl DiskLoadReport {
    pub fn into_result(self, kind: &str) -> Result<usize, String> {
        if self.issues.is_empty() {
            Ok(self.loaded)
        } else {
            Err(format!(
                "{} loading failed with {} issue(s): {}",
                kind,
                self.issues.len(),
                self.issues.join("; ")
            ))
        }
    }
}

/// Upload supported texture files in `textures/` to the texture manager.
pub fn load_textures_from_disk(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    manager: &mut TextureManager,
) -> DiskLoadReport {
    let mut report = DiskLoadReport::default();
    if let Err(error) = fs::create_dir_all(TEXTURES_DIR) {
        report.issues.push(format!(
            "failed to create texture directory '{}': {}",
            TEXTURES_DIR, error
        ));
        return report;
    }
    let entries = match fs::read_dir(TEXTURES_DIR) {
        Ok(entries) => entries,
        Err(error) => {
            report.issues.push(format!(
                "failed to read texture directory '{}': {}",
                TEXTURES_DIR, error
            ));
            return report;
        }
    };

    let mut texture_paths = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_supported_texture(path))
        .collect::<Vec<_>>();
    texture_paths.sort();

    for path in texture_paths {
        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let img_data = match fs::read(&path) {
            Ok(data) => data,
            Err(e) => {
                report.issues.push(format!(
                    "failed to read texture '{}': {}",
                    path.display(),
                    e
                ));
                continue;
            }
        };

        let img = match image::load_from_memory(&img_data) {
            Ok(img) => img,
            Err(e) => {
                report.issues.push(format!(
                    "failed to decode texture '{}': {}",
                    path.display(),
                    e
                ));
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
        report.loaded += 1;
    }
    report
}

/// Upload every prop model under `assets/` except the base map.
pub fn load_prop_assets(device: &wgpu::Device, assets: &mut AssetManager) -> DiskLoadReport {
    let mut report = DiskLoadReport::default();
    let entries = match fs::read_dir(ASSETS_DIR) {
        Ok(entries) => entries,
        Err(error) => {
            report.issues.push(format!(
                "failed to read asset directory '{}': {}",
                ASSETS_DIR, error
            ));
            return report;
        }
    };

    let mut pending: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    pending.sort_by(|left, right| right.cmp(left));

    while let Some(path) = pending.pop() {
        if path.is_dir() {
            if let Ok(sub_entries) = fs::read_dir(&path) {
                let mut children = sub_entries
                    .flatten()
                    .map(|entry| entry.path())
                    .collect::<Vec<_>>();
                children.sort_by(|left, right| right.cmp(left));
                pending.extend(children);
            } else {
                report.issues.push(format!(
                    "failed to read asset directory '{}'",
                    path.display()
                ));
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

        if let Some(path_str) = path.to_str() {
            let model = match try_load_model(path_str) {
                Ok(model) => model,
                Err(error) => {
                    report.issues.push(format!(
                        "failed to load model asset '{}': {}",
                        path.display(),
                        error
                    ));
                    continue;
                }
            };
            let geometry_errors = validate_model_geometry(&model);
            if !geometry_errors.is_empty() {
                report.issues.push(format!(
                    "model asset '{}' failed geometry validation: {}",
                    path.display(),
                    geometry_errors.join("; ")
                ));
                continue;
            }
            let (vertices, parts, _pp, _pi) = model;

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
            report.loaded += 1;
        } else {
            report.issues.push(format!(
                "asset path '{}' is not valid UTF-8",
                path.to_string_lossy()
            ));
        }
    }
    report
}

fn is_supported_texture(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tga"
            )
        })
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
