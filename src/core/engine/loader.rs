// src/engine/loader.rs
// Disk I/O helpers for the engine.
// These are pure functions — they take device/queue references and return
// GPU resources. They never touch EngineState directly.
//
// This module handles loading assets from disk and uploading them to the GPU.
// It provides functions for loading textures and 3D models, with proper error
// handling and fallback mechanisms to ensure the game can start even with
// missing or corrupted assets.

use wgpu::util::DeviceExt;
use std::fs;

use crate::systems::render::assets::{AssetManager, RenderAsset, RenderAssetMeshPart};
use crate::systems::render::mesh::load_model;
use crate::systems::render::texture::TextureManager;

// ── Texture loading ───────────────────────────────────────────────────────────

/// Scans `textures/` directory and uploads every PNG/JPG to the TextureManager.
/// 
/// This function automatically discovers texture files and uploads them to the GPU
/// as texture resources with appropriate bind groups for shader access. It handles
/// various image formats and provides fallback mechanisms for corrupted files.
/// 
/// # Parameters
/// - `device`: GPU device for creating texture resources
/// - `queue`: GPU command queue for uploading texture data
/// - `layout`: Bind group layout for texture and sampler bindings
/// - `manager`: Texture manager to store the created bind groups
/// 
/// # Process
/// 1. Creates textures/ directory if it doesn't exist
/// 2. Scans for PNG and JPG files
/// 3. Loads each image file into memory
/// 4. Converts to RGBA8 format for GPU compatibility
/// 5. Creates GPU texture with appropriate usage flags
/// 6. Uploads image data to GPU texture
/// 7. Creates texture view and sampler
/// 8. Creates bind group combining texture and sampler
/// 9. Stores bind group in manager with filename as key
/// 
/// # Error Handling
/// - Missing files are skipped with warning messages
/// - Corrupted images are skipped with detailed error reporting
/// - Invalid file names are skipped gracefully
/// - Function continues processing remaining files on errors
pub fn load_textures_from_disk(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    manager: &mut TextureManager,
) {
    // Ensure textures directory exists, create if missing
    std::fs::create_dir_all("textures").unwrap_or_default();
    let Ok(entries) = fs::read_dir("textures") else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else { continue };
        if !matches!(ext, "png" | "jpg") { continue; }

        // Extract filename safely, skip if invalid UTF-8
        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };
        
        // Load image file data with error handling
        let img_data = match fs::read(&path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Warning: Failed to read texture file {}: {}", path.display(), e);
                continue;
            }
        };
        
        // Decode image from memory with error handling
        let img = match image::load_from_memory(&img_data) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Warning: Failed to decode texture file {}: {}", path.display(), e);
                continue;
            }
        };
        
        // Convert to RGBA8 format for GPU compatibility
        let img_rgba = img.to_rgba8();
        let (w, h) = img_rgba.dimensions();

        // Create GPU texture with appropriate parameters
        let tex_size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
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
        
        // Upload image data to GPU texture
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
                bytes_per_row: Some((4 * w + 255) & !255),
                rows_per_image: Some(h),
            },
            tex_size,
        );

        // Create texture view for shader access
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create sampler with repeat addressing and linear filtering
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        
        // Create bind group combining texture and sampler for shader access
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
        
        // Store bind group in manager for later use by shaders
        manager.insert(&file_name, bind_group);
    }
}

// ── Prop asset loading ────────────────────────────────────────────────────────

/// Scans `assets/` directory (including subdirectories) and uploads every GLB/GLTF/OBJ (except the base map) to
/// the AssetManager as GPU vertex + index buffers.
/// 
/// This function loads 3D model files and converts them into GPU-ready vertex and index
/// buffers for rendering. It handles multiple model formats and provides error recovery
/// for corrupted or unsupported files.
/// 
/// # Parameters
/// - `device`: GPU device for creating buffer resources
/// - `assets`: Asset manager to store the created render assets
/// 
/// # Process
/// 1. Scans assets/ directory for model files
/// 2. Skips the base map file (handled separately)
/// 3. Loads each model file using the mesh loader
/// 4. Creates vertex buffer with vertex data
/// 5. Creates index buffer for each mesh part
/// 6. Stores render assets with filename as key
/// 
/// # Supported Formats
/// - GLB (Binary glTF)
/// - GLTF (glTF JSON format)
/// - OBJ (Wavefront OBJ format)
/// 
/// # Error Handling
/// - Missing assets directory prints warning and returns early
/// - Unsupported file formats are skipped
/// - Corrupted model files are caught with panic handling and skipped
/// - Function continues processing remaining files on errors
pub fn load_prop_assets(device: &wgpu::Device, assets: &mut AssetManager) {
    // Check if assets directory exists
    let Ok(entries) = fs::read_dir("assets") else {
        eprintln!("WARNING: Could not read assets/ directory.");
        return;
    };

    // Recursively scan assets directory including subdirectories
    let mut found_any = false;
    let mut pending: Vec<std::path::PathBuf> = entries.flatten().map(|e| e.path()).collect();
    
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            // Add directory contents to pending queue
            if let Ok(sub_entries) = std::fs::read_dir(&path) {
                pending.extend(sub_entries.flatten().map(|e| e.path()));
            }
            continue;
        }
        
        if !path.is_file() { continue; }
        
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else { continue };
        if !matches!(ext, "glb" | "gltf" | "obj") { continue; }

        // Extract filename safely, skip if invalid UTF-8
        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };
        
        // Skip the base map file as it's handled separately by the level system
        if file_name == "map_001.glb" { continue; }

        found_any = true;
        
        // Load model with panic handling to catch parsing errors
        if let Ok((vertices, parts, _pp, _pi)) = std::panic::catch_unwind(
            std::panic::AssertUnwindSafe(|| load_model(path.to_str().unwrap())),
        ) {
            // Create vertex buffer with all vertex data
            let vertex_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Asset Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            
            // Create index buffers for each mesh part
            let mut render_parts = Vec::new();
            for part in parts {
                let index_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
            
            // Store complete render asset in manager using filename as key
            // (allows referencing by filename in level JSON regardless of subfolder)
            assets.insert(file_name, RenderAsset { vertex_buffer, parts: render_parts });
        }
    }

    // Warn if no model files were found, but don't fail - game can still run
    if !found_any {
        eprintln!("WARNING: No prop model files (.obj, .glb, .gltf) found in assets/");
    }
}

