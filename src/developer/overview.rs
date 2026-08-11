//! Read-only orientation tools for the project tree.
//!
//! This intentionally reports file counts rather than loading content. Use
//! `validate` when you need to know whether the content is valid; use this
//! command when you need to know where the project currently lives.

use std::path::Path;

#[derive(Debug, Clone, Copy)]
struct ContentArea {
    label: &'static str,
    relative_path: &'static str,
    extension: &'static str,
}

const CONTENT_AREAS: &[ContentArea] = &[
    ContentArea {
        label: "Playable levels",
        relative_path: "levels",
        extension: "json",
    },
    ContentArea {
        label: "Reusable prefabs",
        relative_path: "prefabs",
        extension: "json",
    },
    ContentArea {
        label: "Enemy definitions",
        relative_path: "data/enemies",
        extension: "toml",
    },
    ContentArea {
        label: "Relic definitions",
        relative_path: "data/relics",
        extension: "toml",
    },
    ContentArea {
        label: "Runtime models",
        relative_path: "assets",
        extension: "model",
    },
    ContentArea {
        label: "Runtime textures",
        relative_path: "textures",
        extension: "texture",
    },
];

pub fn render_project_overview(project_root: impl AsRef<Path>) -> Result<String, String> {
    let root = project_root.as_ref();
    let mut output = String::from("Cenotaph project map\n\nContent\n");

    for area in CONTENT_AREAS {
        let path = root.join(area.relative_path);
        let count = count_matching_files(&path, area.extension)?;
        output.push_str(&format!(
            "- {:<20} {:>4}  ({})\n",
            area.label, count, area.relative_path
        ));
    }

    output.push_str(
        "\nWhere to make changes\n\
- New enemy or relic: data/enemies/ or data/relics/\n\
- New map or encounter: levels/ (reuse prefabs/ when possible)\n\
- Shared tuning or controls: config/\n\
- Runtime behavior: src/game/ or src/core/engine/\n\
- Rendering, input, physics, or audio plumbing: src/systems/\n\
- Developer checks and project diagnostics: src/developer/ and scripts/\n\
\nSafe loop\n\
1. Make one small content or code change.\n\
2. Run cargo content to confirm the project shape.\n\
3. Run cargo validate-content to catch broken references.\n\
4. Run powershell -NoProfile -ExecutionPolicy Bypass -File scripts/project_check.ps1.\n\
5. Use the manual smoke checklist before treating a runtime change as done.\n",
    );

    Ok(output)
}

fn count_matching_files(root: &Path, extension: &str) -> Result<usize, String> {
    if !root.exists() {
        return Ok(0);
    }

    let entries = std::fs::read_dir(root)
        .map_err(|error| format!("failed to read '{}': {}", root.display(), error))?;
    let mut count = 0;

    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to inspect '{}': {}", root.display(), error))?;
        let path = entry.path();
        if path.is_dir() {
            count += count_matching_files(&path, extension)?;
        } else if path.is_file() && matches_extension(&path, extension) {
            count += 1;
        }
    }

    Ok(count)
}

fn matches_extension(path: &Path, category: &str) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };

    match category {
        "model" => matches!(
            extension.to_ascii_lowercase().as_str(),
            "obj" | "gltf" | "glb"
        ),
        "texture" => matches!(
            extension.to_ascii_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tga"
        ),
        expected => extension.eq_ignore_ascii_case(expected),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn overview_counts_nested_content_and_ignores_other_files() {
        let root = unique_temp_dir("overview");
        std::fs::create_dir_all(root.join("levels/nested")).unwrap();
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("levels/a.json"), "{}").unwrap();
        std::fs::write(root.join("levels/nested/b.JSON"), "{}").unwrap();
        std::fs::write(root.join("levels/notes.txt"), "ignore").unwrap();
        std::fs::write(root.join("assets/shape.obj"), "placeholder").unwrap();

        let report = render_project_overview(&root).unwrap();

        assert!(report.lines().any(|line| line.contains("Playable levels")
            && line.contains("2")
            && line.contains("(levels)")));
        assert!(report.lines().any(|line| line.contains("Runtime models")
            && line.contains("1")
            && line.contains("(assets)")));
        assert!(!report.contains("notes.txt"));
        std::fs::remove_dir_all(root).ok();
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "cenotaph_{}_{}_{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
