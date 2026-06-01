// src/render/mod.rs
// The render module owns all GPU-facing systems: the pipeline,
// camera, mesh loading, instancing, textures, HUD, and asset management.

pub mod pipeline;
pub mod camera;
pub mod mesh;
pub mod instance;
pub mod texture;
pub mod assets;
pub mod lighting;
pub mod hud;
