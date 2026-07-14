use std::fmt;
use std::path::{Path, PathBuf};

use crate::core::engine::validation::{validate_project_content, ContentValidationReport};
use crate::developer::commands::{available_levels, DEFAULT_LEVEL_ID};
use crate::game::save::{SaveData, SaveFileHealth, DEFAULT_SAVE_PATH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warning,
    Error,
}

impl CheckStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warning => "WARN",
            Self::Error => "FAIL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub status: CheckStatus,
    pub name: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct ProjectDoctorReport {
    pub checks: Vec<DoctorCheck>,
    pub content: ContentValidationReport,
}

impl ProjectDoctorReport {
    pub fn is_ok(&self) -> bool {
        self.checks
            .iter()
            .all(|check| check.status != CheckStatus::Error)
            && self.content.is_ok()
    }

    pub fn warning_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == CheckStatus::Warning)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == CheckStatus::Error)
            .count()
            + self.content.issues.len()
    }

    pub fn summary(&self) -> String {
        if self.is_ok() {
            format!(
                "project doctor passed: {} check(s), {} warning(s)",
                self.checks.len() + 1,
                self.warning_count()
            )
        } else {
            format!(
                "project doctor failed: {} error(s), {} warning(s)",
                self.error_count(),
                self.warning_count()
            )
        }
    }
}

impl fmt::Display for ProjectDoctorReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "Cenotaph project doctor")?;
        for check in &self.checks {
            writeln!(
                formatter,
                "[{}] {}: {}",
                check.status.label(),
                check.name,
                check.detail
            )?;
        }

        writeln!(
            formatter,
            "[{}] Content: {}",
            if self.content.is_ok() { "PASS" } else { "FAIL" },
            self.content.summary()
        )?;
        for issue in &self.content.issues {
            writeln!(formatter, "       - {}: {}", issue.path, issue.message)?;
        }

        write!(formatter, "{}", self.summary())
    }
}

pub fn run_project_doctor() -> ProjectDoctorReport {
    let root = Path::new(".");
    let mut checks = Vec::new();
    inspect_layout(root, &mut checks);
    inspect_levels(root, &mut checks);
    inspect_save(root, &mut checks);
    inspect_runtime_tree(root, &mut checks);
    inspect_editor_backups(root, &mut checks);
    let content = validate_project_content();
    inspect_transient_files(root, &mut checks);

    ProjectDoctorReport { checks, content }
}

fn inspect_layout(root: &Path, checks: &mut Vec<DoctorCheck>) {
    const REQUIRED_FILES: &[&str] = &["Cargo.toml", "config/tuning.toml", "config/bindings.toml"];
    const REQUIRED_DIRECTORIES: &[&str] = &[
        "assets",
        "data/enemies",
        "data/relics",
        "levels",
        "prefabs",
        "source_assets",
        "textures",
        "tools/level_editor",
    ];

    let missing_files = REQUIRED_FILES
        .iter()
        .filter(|relative| !root.join(relative).is_file())
        .copied()
        .collect::<Vec<_>>();
    let missing_directories = REQUIRED_DIRECTORIES
        .iter()
        .filter(|relative| !root.join(relative).is_dir())
        .copied()
        .collect::<Vec<_>>();

    if missing_files.is_empty() && missing_directories.is_empty() {
        checks.push(check(
            CheckStatus::Pass,
            "Project layout",
            "required runtime, content, source, and tooling paths are present",
        ));
        return;
    }

    let mut details = Vec::new();
    if !missing_files.is_empty() {
        details.push(format!("missing files: {}", missing_files.join(", ")));
    }
    if !missing_directories.is_empty() {
        details.push(format!(
            "missing directories: {}",
            missing_directories.join(", ")
        ));
    }
    checks.push(check(
        CheckStatus::Error,
        "Project layout",
        details.join("; "),
    ));
}

fn inspect_levels(root: &Path, checks: &mut Vec<DoctorCheck>) {
    match available_levels(root) {
        Ok(levels) if levels.iter().any(|level| level == DEFAULT_LEVEL_ID) => checks.push(check(
            CheckStatus::Pass,
            "Playable levels",
            format!(
                "{} level(s) found; default '{}' is available",
                levels.len(),
                DEFAULT_LEVEL_ID
            ),
        )),
        Ok(levels) => checks.push(check(
            CheckStatus::Error,
            "Playable levels",
            format!(
                "{} level(s) found, but default '{}' is missing",
                levels.len(),
                DEFAULT_LEVEL_ID
            ),
        )),
        Err(error) => checks.push(check(CheckStatus::Error, "Playable levels", error)),
    }
}

fn inspect_save(root: &Path, checks: &mut Vec<DoctorCheck>) {
    let save_path = root.join(DEFAULT_SAVE_PATH);
    match SaveData::inspect_path(&save_path) {
        SaveFileHealth::Missing => checks.push(check(
            CheckStatus::Pass,
            "Autosave",
            "no autosave exists yet; a new run can start normally",
        )),
        SaveFileHealth::Healthy(save) => {
            let level_path = root
                .join("levels")
                .join(format!("{}.json", save.level_name));
            if level_path.is_file() {
                checks.push(check(
                    CheckStatus::Pass,
                    "Autosave",
                    format!(
                        "healthy save for '{}' at cycle {}",
                        save.level_name, save.cycle_number
                    ),
                ));
            } else {
                checks.push(check(
                    CheckStatus::Error,
                    "Autosave",
                    format!(
                        "save references missing level '{}' ({})",
                        save.level_name,
                        level_path.display()
                    ),
                ));
            }
        }
        SaveFileHealth::Recoverable {
            backup,
            primary_error,
        } => checks.push(check(
            CheckStatus::Warning,
            "Autosave",
            format!(
                "primary is damaged ({primary_error}); valid backup for '{}' will be recovered on continue",
                backup.level_name
            ),
        )),
        SaveFileHealth::Invalid {
            primary_error,
            backup_error,
        } => {
            let backup_detail = backup_error
                .map(|error| format!("; backup is also invalid ({error})"))
                .unwrap_or_else(|| "; no valid backup exists".to_string());
            checks.push(check(
                CheckStatus::Error,
                "Autosave",
                format!("primary is invalid ({primary_error}){backup_detail}"),
            ));
        }
    }
}

fn inspect_runtime_tree(root: &Path, checks: &mut Vec<DoctorCheck>) {
    let source_extensions = ["blend", "kra", "psd", "xcf"];
    let mut misplaced = Vec::new();
    for relative in ["assets", "levels", "textures"] {
        collect_matching_files(&root.join(relative), &source_extensions, &mut misplaced);
    }

    if misplaced.is_empty() {
        checks.push(check(
            CheckStatus::Pass,
            "Runtime tree",
            "editable source formats are kept outside runtime content directories",
        ));
    } else {
        checks.push(check(
            CheckStatus::Warning,
            "Runtime tree",
            format!(
                "move source-only file(s) into source_assets: {}",
                display_paths(&misplaced)
            ),
        ));
    }
}

fn inspect_editor_backups(root: &Path, checks: &mut Vec<DoctorCheck>) {
    let backup_dirs = [
        root.join("levels/.editor_backups"),
        root.join("prefabs/.editor_backups"),
    ];
    let backup_count = backup_dirs
        .iter()
        .map(|directory| count_files(directory))
        .sum::<usize>();
    let status = if backup_count > 100 {
        CheckStatus::Warning
    } else {
        CheckStatus::Pass
    };
    let detail = if backup_count > 100 {
        format!(
            "{} editor backups found; archive or prune old copies when they are no longer useful",
            backup_count
        )
    } else {
        format!("{} editor backup file(s) retained", backup_count)
    };
    checks.push(check(status, "Editor backups", detail));
}

fn inspect_transient_files(root: &Path, checks: &mut Vec<DoctorCheck>) {
    let mut files = Vec::new();
    for relative in ["levels", "prefabs", "save"] {
        collect_transient_files(&root.join(relative), &mut files);
    }
    files.sort();

    if files.is_empty() {
        checks.push(check(
            CheckStatus::Pass,
            "Pending writes",
            "no abandoned staging, lock, or recovery sidecars found",
        ));
    } else {
        checks.push(check(
            CheckStatus::Warning,
            "Pending writes",
            format!(
                "inspect abandoned write sidecar(s): {}",
                display_paths(&files)
            ),
        ));
    }
}

fn collect_transient_files(path: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_transient_files(&path, files);
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with('.')
            && (name.contains(".tmp.") || name.ends_with(".lock") || name.ends_with(".recover"))
        {
            files.push(path);
        }
    }
}

fn collect_matching_files(path: &Path, extensions: &[&str], matches: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_matching_files(&path, extensions, matches);
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extensions
                    .iter()
                    .any(|candidate| extension.eq_ignore_ascii_case(candidate))
            })
        {
            matches.push(path);
        }
    }
    matches.sort();
}

fn count_files(path: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .map(|path| {
            if path.is_dir() {
                count_files(&path)
            } else {
                usize::from(path.is_file())
            }
        })
        .sum()
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .take(5)
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .chain((paths.len() > 5).then(|| format!("and {} more", paths.len() - 5)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn check(status: CheckStatus, name: impl Into<String>, detail: impl Into<String>) -> DoctorCheck {
    DoctorCheck {
        status,
        name: name.into(),
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_summary_counts_warnings_and_content_errors() {
        let report = ProjectDoctorReport {
            checks: vec![
                check(CheckStatus::Pass, "One", "ok"),
                check(CheckStatus::Warning, "Two", "careful"),
            ],
            content: ContentValidationReport::default(),
        };

        assert!(report.is_ok());
        assert_eq!(report.warning_count(), 1);
        assert!(report.summary().contains("1 warning"));
    }

    #[test]
    fn path_display_is_bounded() {
        let paths = (0..7)
            .map(|index| PathBuf::from(format!("assets/source_{index}.blend")))
            .collect::<Vec<_>>();
        let display = display_paths(&paths);
        assert!(display.contains("and 2 more"));
        assert!(!display.contains("source_6"));
    }
}
