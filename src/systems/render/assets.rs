// src/render/assets.rs
// AssetManager stores GPU-ready mesh data (vertex + index buffers) keyed
// by asset filename. RenderAsset is the GPU representation of a loaded model.

use std::collections::HashMap;

// ── GPU mesh part ─────────────────────────────────────────────────────────────

/// One draw call: an index buffer and the texture it uses.
pub struct RenderAssetMeshPart {
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub texture_name: String,
}

// ── GPU asset ─────────────────────────────────────────────────────────────────

/// A fully GPU-uploaded model: one shared vertex buffer + N mesh parts.
pub struct RenderAsset {
    pub vertex_buffer: wgpu::Buffer,
    pub parts: Vec<RenderAssetMeshPart>,
}

// ── Draw group ────────────────────────────────────────────────────────────────

/// Groups all instances of a single asset for one instanced draw call.
pub struct DrawGroup {
    pub asset_id: String,
    pub num_instances: u32,
    pub instance_buffer: wgpu::Buffer,
}

// ── Manager ───────────────────────────────────────────────────────────────────

pub struct AssetManager {
    assets: HashMap<String, RenderAsset>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: String, asset: RenderAsset) {
        self.assets.insert(id, asset);
    }

    pub fn get(&self, id: &str) -> Option<&RenderAsset> {
        self.assets.get(id)
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }
}
