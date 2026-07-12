/*
src/core/engine/asset_catalog.rs
Scans the assets/ directory (and subdirectories) for available 3D models.
Generates a JSON catalog that can be used by level editors or referenced
when designing levels.
*/

use serde::{Deserialize, Serialize};
use std::fs;

/// Represents a single model asset found in the assets directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    /// Filename of the model (e.g., "Cube.obj", "terrain.glb")
    pub filename: String,
    /// Relative path from assets/ directory (e.g., "props/Cube.obj")
    pub relative_path: String,
    /// Full path for loading (e.g., "assets/props/Cube.obj")
    pub full_path: String,
    /// File extension indicating format
    pub format: String,
    /// File size in bytes
    pub size_bytes: u64,
}

/// Complete catalog of available 3D models in the assets directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCatalog {
    /// Root directory that was scanned
    pub root_path: String,
    /// List of all discovered model files
    pub models: Vec<AssetEntry>,
    /// List of subdirectories found in assets/
    pub subfolders: Vec<String>,
}

impl AssetCatalog {
    /// Scan the assets/ directory and build a catalog of available models.
    pub fn scan(assets_dir: &str) -> Self {
        let mut models = Vec::new();
        let mut subfolders = Vec::new();
        let root_path = assets_dir.to_string();

        if let Ok(entries) = fs::read_dir(assets_dir) {
            Self::scan_directory(entries, &root_path, &mut models, &mut subfolders);
        }

        // Sort models by filename for easier browsing
        models.sort_by(|a, b| a.filename.cmp(&b.filename));
        subfolders.sort();

        Self {
            root_path,
            models,
            subfolders,
        }
    }

    /// Recursively scan a directory for model files.
    fn scan_directory(
        entries: fs::ReadDir,
        root_path: &str,
        models: &mut Vec<AssetEntry>,
        subfolders: &mut Vec<String>,
    ) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Record subfolder name
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    subfolders.push(name.to_string());
                }
                // Recurse into subdirectory
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    Self::scan_directory(sub_entries, root_path, models, subfolders);
                }
            } else if path.is_file() {
                // Check if it's a supported model format
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext.to_lowercase().as_str(), "obj" | "glb" | "gltf") {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            let full_path = path.to_string_lossy().replace('\\', "/");
                            let relative_path = path
                                .strip_prefix(root_path)
                                .ok()
                                .map(|p| p.to_string_lossy().replace('\\', "/"))
                                .unwrap_or_else(|| filename.to_string());

                            // Get file size
                            let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                            models.push(AssetEntry {
                                filename: filename.to_string(),
                                relative_path,
                                full_path,
                                format: ext.to_lowercase(),
                                size_bytes,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Save the catalog to a JSON file.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize catalog: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Failed to write catalog file: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_serialization() {
        let catalog = AssetCatalog {
            root_path: "assets".to_string(),
            models: vec![AssetEntry {
                filename: "test.obj".to_string(),
                relative_path: "test.obj".to_string(),
                full_path: "assets/test.obj".to_string(),
                format: "obj".to_string(),
                size_bytes: 1024,
            }],
            subfolders: vec![],
        };

        let json = serde_json::to_string(&catalog).unwrap();
        let loaded: AssetCatalog = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.models.len(), 1);
    }
}
