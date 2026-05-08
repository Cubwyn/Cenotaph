// src/engine/mod.rs
// The engine module owns the runtime: GPU state, all subsystems, and the
// per-frame update/render loop.
//
// Sub-modules:
//   state   — EngineState struct + construction + resize + render
//   loader  — disk I/O helpers (textures, prop assets)
//   sync    — instance buffer rebuild (sync_instances)
//   update  — per-frame logic (physics step, gameplay)

pub mod asset_catalog;
pub mod loader;
pub mod state;
pub mod sync;
pub mod update;

pub use state::EngineState;
