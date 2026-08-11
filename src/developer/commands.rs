use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::data::world::level;

pub const DEFAULT_LEVEL_ID: &str = "movement_test";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectCommand {
    Play { level_id: String },
    Continue,
    Validate,
    Doctor,
    ListLevels,
    Overview,
    Help,
}

pub fn parse_args<I>(args: I) -> Result<ProjectCommand, String>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args
        .into_iter()
        .map(|argument| {
            argument
                .into_string()
                .map_err(|_| "command arguments must be valid UTF-8".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();

    let Some(first) = args.next() else {
        return Ok(ProjectCommand::Play {
            level_id: DEFAULT_LEVEL_ID.to_string(),
        });
    };

    let command = match first.as_str() {
        "help" | "-h" | "--help" => ProjectCommand::Help,
        "validate" | "validate-content" | "--validate" => ProjectCommand::Validate,
        "doctor" | "diagnose" => ProjectCommand::Doctor,
        "levels" | "list-levels" => ProjectCommand::ListLevels,
        "content" | "overview" | "map" => ProjectCommand::Overview,
        "continue" | "resume" => ProjectCommand::Continue,
        "play" => {
            let level_id = args.next().unwrap_or_else(|| DEFAULT_LEVEL_ID.to_string());
            validate_level_id(&level_id)?;
            ProjectCommand::Play { level_id }
        }
        argument if argument.starts_with('-') => {
            return Err(format!(
                "unknown option '{}'. Run `cargo run -- help` for available commands.",
                argument
            ));
        }
        level_id => {
            validate_level_id(level_id)?;
            ProjectCommand::Play {
                level_id: level_id.to_string(),
            }
        }
    };

    if let Some(extra) = args.next() {
        return Err(format!(
            "unexpected argument '{}'. Run `cargo run -- help` for command usage.",
            extra
        ));
    }

    Ok(command)
}

pub fn validate_level_id(level_id: &str) -> Result<(), String> {
    level::validate_level_id(level_id)
}

pub fn available_levels(project_root: impl AsRef<Path>) -> Result<Vec<String>, String> {
    let levels_dir = project_root.as_ref().join("levels");
    let entries = std::fs::read_dir(&levels_dir).map_err(|error| {
        format!(
            "failed to read level directory '{}': {}",
            levels_dir.display(),
            error
        )
    })?;

    let mut levels = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .filter_map(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .filter(|stem| validate_level_id(stem).is_ok())
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>();
    levels.sort();
    levels.dedup();
    Ok(levels)
}

pub fn resolve_level_path(
    project_root: impl AsRef<Path>,
    level_id: &str,
) -> Result<PathBuf, String> {
    validate_level_id(level_id)?;
    let project_root = project_root.as_ref();
    let path = project_root
        .join("levels")
        .join(format!("{}.json", level_id));
    if path.is_file() {
        return Ok(path);
    }

    let available = available_levels(project_root)?;
    let available_text = if available.is_empty() {
        "no levels were found".to_string()
    } else {
        format!("available levels: {}", available.join(", "))
    };
    Err(format!(
        "level '{}' does not exist at '{}'; {}",
        level_id,
        path.display(),
        available_text
    ))
}

pub fn help_text() -> &'static str {
    r#"Cenotaph project commands

USAGE
    cargo run                         Play movement_test
    cargo run -- play [level-id]      Play a level
    cargo run -- <level-id>           Backward-compatible play shorthand
    cargo run -- continue             Resume the latest valid autosave
    cargo run -- validate             Validate authored project content
    cargo run -- doctor               Check project layout, content, and save health
    cargo run -- levels               List playable level ids
    cargo run -- content              Show the project content map and safe change loop
    cargo run -- help                 Show this help

Level ids may contain letters, numbers, '-' and '_' only."#
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn no_arguments_plays_default_level() {
        assert_eq!(
            parse_args(Vec::<OsString>::new()).unwrap(),
            ProjectCommand::Play {
                level_id: DEFAULT_LEVEL_ID.to_string()
            }
        );
    }

    #[test]
    fn explicit_and_shorthand_play_commands_match() {
        let explicit = parse_args(args(&["play", "foundation_test"])).unwrap();
        let shorthand = parse_args(args(&["foundation_test"])).unwrap();
        assert_eq!(explicit, shorthand);
    }

    #[test]
    fn content_aliases_select_project_overview() {
        for alias in ["content", "overview", "map"] {
            assert_eq!(
                parse_args(args(&[alias])).unwrap(),
                ProjectCommand::Overview
            );
        }
    }

    #[test]
    fn rejects_traversal_and_extra_arguments() {
        assert!(parse_args(args(&["play", "../secret"])).is_err());
        assert!(parse_args(args(&["doctor", "extra"])).is_err());
    }

    #[test]
    fn level_discovery_is_sorted_and_ignores_non_json_files() {
        let root = unique_temp_dir("commands_levels");
        let levels = root.join("levels");
        std::fs::create_dir_all(&levels).unwrap();
        std::fs::write(levels.join("zeta.json"), "{}").unwrap();
        std::fs::write(levels.join("alpha.JSON"), "{}").unwrap();
        std::fs::write(levels.join("notes.txt"), "ignore").unwrap();

        assert_eq!(
            available_levels(&root).unwrap(),
            vec!["alpha".to_string(), "zeta".to_string()]
        );
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
