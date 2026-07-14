/*
src/core/engine/asset_catalog.rs
Scans project content folders for editor-visible assets.

The runtime still only loads a small subset directly, but the standalone editor
needs to see source files too: model sources, textures, dialogue/data files,
audio, materials, levels, and config.
*/

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a single asset or source file found in a project content folder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetEntry {
    /// File name, e.g. "Cube.obj" or "opening_dialogue.toml".
    pub filename: String,
    /// Root folder that produced the entry, e.g. "assets" or "textures".
    pub root_path: String,
    /// Relative path from the root folder, e.g. "props/Cube.obj".
    pub relative_path: String,
    /// Full project-relative path, e.g. "assets/props/Cube.obj".
    pub full_path: String,
    /// Lowercase extension indicating file format.
    pub format: String,
    /// Editor-facing broad kind: model, texture, dialogue, audio, material, etc.
    pub kind: String,
    /// True when current runtime systems can load/use this file directly.
    pub runtime_supported: bool,
    /// File size in bytes.
    pub size_bytes: u64,
}

/// Complete catalog of editor-visible project assets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetCatalog {
    /// Root directories that were scanned.
    pub root_path: String,
    /// All discovered supported files.
    pub assets: Vec<AssetEntry>,
    /// All discovered model/source-model files.
    pub models: Vec<AssetEntry>,
    pub textures: Vec<AssetEntry>,
    pub audio: Vec<AssetEntry>,
    pub materials: Vec<AssetEntry>,
    pub dialogue: Vec<AssetEntry>,
    pub data: Vec<AssetEntry>,
    pub levels: Vec<AssetEntry>,
    pub config: Vec<AssetEntry>,
    /// List of subdirectories found below scanned roots.
    pub subfolders: Vec<String>,
}

impl AssetCatalog {
    /// Scan a single directory. This remains for existing call sites/tests.
    pub fn scan(root: &str) -> Self {
        Self::scan_roots(&[root])
    }

    /// Scan the project roots the editor should understand.
    pub fn scan_project() -> Self {
        Self::scan_roots(&[
            "assets",
            "textures",
            "source_assets",
            "data",
            "prefabs",
            "levels",
            "config",
        ])
    }

    /// Scan several roots and build a grouped catalog.
    pub fn scan_roots(roots: &[&str]) -> Self {
        let mut assets = Vec::new();
        let mut subfolders = Vec::new();

        for root in roots {
            let root_path = Path::new(root);
            if let Ok(entries) = fs::read_dir(root_path) {
                Self::scan_directory(entries, root_path, &mut assets, &mut subfolders);
            }
        }

        assets.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then(left.full_path.cmp(&right.full_path))
        });
        subfolders.sort();
        subfolders.dedup();

        let grouped = |kind: &str| {
            assets
                .iter()
                .filter(|asset| asset.kind == kind)
                .cloned()
                .collect::<Vec<_>>()
        };

        Self {
            root_path: roots.join(";"),
            models: grouped("model"),
            textures: grouped("texture"),
            audio: grouped("audio"),
            materials: grouped("material"),
            dialogue: grouped("dialogue"),
            data: grouped("data"),
            levels: grouped("level"),
            config: grouped("config"),
            assets,
            subfolders,
        }
    }

    pub fn runtime_models(&self) -> Vec<AssetEntry> {
        self.models
            .iter()
            .filter(|asset| asset.runtime_supported)
            .cloned()
            .collect()
    }

    fn scan_directory(
        entries: fs::ReadDir,
        root_path: &Path,
        assets: &mut Vec<AssetEntry>,
        subfolders: &mut Vec<String>,
    ) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with('.'))
                {
                    continue;
                }
                if let Some(relative) = relative_path(root_path, &path) {
                    subfolders.push(format!(
                        "{}/{}",
                        root_path.to_string_lossy().replace('\\', "/"),
                        relative
                    ));
                }
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    Self::scan_directory(sub_entries, root_path, assets, subfolders);
                }
                continue;
            }

            if !path.is_file() {
                continue;
            }

            let Some(ext) = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
            else {
                continue;
            };
            let Some((kind, runtime_supported)) = classify_asset(root_path, &path, &ext) else {
                continue;
            };
            let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            let full_path = path.to_string_lossy().replace('\\', "/");
            let relative_path = relative_path(root_path, &path).unwrap_or_else(|| filename.into());
            let size_bytes = fs::metadata(&path)
                .map(|metadata| metadata.len())
                .unwrap_or(0);

            assets.push(AssetEntry {
                filename: filename.to_string(),
                root_path: root_path.to_string_lossy().replace('\\', "/"),
                relative_path,
                full_path,
                format: ext,
                kind: kind.to_string(),
                runtime_supported,
                size_bytes,
            });
        }
    }
}

fn relative_path(root_path: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root_path)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn classify_asset(root_path: &Path, path: &Path, ext: &str) -> Option<(&'static str, bool)> {
    let root = root_path.to_string_lossy().replace('\\', "/");
    let full_path = path.to_string_lossy().replace('\\', "/");

    if matches!(ext, "obj" | "glb" | "gltf") {
        return Some(("model", root == "assets"));
    }
    if matches!(ext, "fbx" | "dae" | "blend" | "stl" | "ply") {
        return Some(("model", false));
    }
    if matches!(ext, "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tga") {
        return Some(("texture", root == "textures"));
    }
    if matches!(ext, "wav" | "ogg" | "mp3" | "flac") {
        return Some(("audio", false));
    }
    if matches!(ext, "mtl" | "mat" | "material") {
        return Some(("material", false));
    }
    if root == "levels" && ext == "json" {
        return Some(("level", true));
    }
    if root == "config" && ext == "toml" {
        return Some(("config", true));
    }
    if full_path.to_ascii_lowercase().contains("dialog")
        && matches!(
            ext,
            "json" | "toml" | "ron" | "yaml" | "yml" | "csv" | "txt" | "md"
        )
    {
        return Some(("dialogue", false));
    }
    if matches!(
        ext,
        "json" | "toml" | "ron" | "yaml" | "yml" | "csv" | "txt" | "md"
    ) {
        return Some(("data", root == "data" && matches!(ext, "json" | "toml")));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_serialization() {
        let catalog = AssetCatalog {
            root_path: "assets".to_string(),
            assets: vec![AssetEntry {
                filename: "test.obj".to_string(),
                root_path: "assets".to_string(),
                relative_path: "test.obj".to_string(),
                full_path: "assets/test.obj".to_string(),
                format: "obj".to_string(),
                kind: "model".to_string(),
                runtime_supported: true,
                size_bytes: 1024,
            }],
            models: vec![],
            textures: vec![],
            audio: vec![],
            materials: vec![],
            dialogue: vec![],
            data: vec![],
            levels: vec![],
            config: vec![],
            subfolders: vec![],
        };

        let json = serde_json::to_string(&catalog).unwrap();
        let loaded: AssetCatalog = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.assets.len(), 1);
        assert_eq!(loaded.assets[0].kind, "model");
    }

    #[test]
    fn asset_classifier_separates_runtime_and_source_formats() {
        assert_eq!(
            classify_asset(Path::new("assets"), Path::new("assets/prop.glb"), "glb"),
            Some(("model", true))
        );
        assert_eq!(
            classify_asset(
                Path::new("source_assets"),
                Path::new("source_assets/maps/prop.obj"),
                "obj"
            ),
            Some(("model", false))
        );
        assert_eq!(
            classify_asset(Path::new("assets"), Path::new("assets/source.fbx"), "fbx"),
            Some(("model", false))
        );
        assert_eq!(
            classify_asset(
                Path::new("textures"),
                Path::new("textures/albedo.webp"),
                "webp"
            ),
            Some(("texture", true))
        );
        assert_eq!(
            classify_asset(
                Path::new("data"),
                Path::new("data/dialogue/intro.toml"),
                "toml"
            ),
            Some(("dialogue", false))
        );
    }
}
