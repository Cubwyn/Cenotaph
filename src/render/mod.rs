// src/render/mod.rs
// The render module owns all GPU-facing systems: the renderer, pipeline,
// camera, mesh loading, instancing, textures, and asset management.

pub mod renderer;
pub mod pipeline;
pub mod camera;
pub mod mesh;
pub mod instance;
pub mod texture;
pub mod assets;
pub mod lighting;

// Note: Removed module declarations for ui, particles, and effects
// as these files were removed during cleanup

pub use renderer::Renderer;
