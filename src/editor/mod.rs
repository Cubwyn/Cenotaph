// src/editor/mod.rs
// In-world level editor — compiled ONLY when the `editor` feature is enabled.
//
// To build without the editor (ship build):
//   cargo build --release --no-default-features
//
// To build with the editor (dev build, the default):
//   cargo build
//   cargo run

pub mod state;

pub use state::EditorState;

use std::fs;

/// Scans the `assets/` directory and returns a list of placeable model filenames.
pub fn scan_assets_folder() -> Vec<String> {
    let Ok(entries) = fs::read_dir("assets") else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if matches!(ext.as_str(), "glb" | "gltf" | "obj") {
                path.file_name()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}
