// src/systems/render/mod.rs
// The render module owns all GPU-facing systems: the pipeline,
// camera, mesh loading, instancing, textures, HUD, and asset management.

pub mod assets;
pub mod camera;
pub mod hud;
pub mod instance;
pub mod lighting;
pub mod mesh;
pub mod particles;
pub mod pipeline;
pub mod texture;
