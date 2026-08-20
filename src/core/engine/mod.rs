// src/engine/mod.rs
// The engine module owns the runtime: GPU state, all subsystems, and the
// per-frame update/render loop.
//
// Sub-modules:
//   state        — EngineState struct + construction + resize + render
//   level_loader — level loading, preparation, and save-state restoration
//   hud_state    — HUD state assembly (read-only mapping to display structs)
//   loader       — disk I/O helpers (textures, prop assets)
//   sync         — instance buffer rebuild (sync_instances)
//   update       — per-frame logic (physics step, gameplay)

pub mod combat;
pub mod hud_state;
pub mod level_loader;
pub mod loader;
pub mod level_events;
pub mod state;
pub mod sync;
pub mod update;
pub mod validation;
