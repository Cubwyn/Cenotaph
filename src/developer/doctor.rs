use std::fmt;
use std::path::{Path, PathBuf};

use crate::core::engine::validation::{validate_project_content, ContentValidationReport};
use crate::data::world::level::LevelData;
use crate::developer::commands::{available_levels, DEFAULT_LEVEL_ID};
use crate::game::save::{SaveData, SaveFileHealth, DEFAULT_SAVE_PATH};
use crate::systems::render::mesh::try_load_model;

const LEVEL_PROP_WARNING_THRESHOLD: usize = 256;
const DYNAMIC_PROP_WARNING_THRESHOLD: usize = 64;
const BASE_MAP_TRIANGLE_WARNING_THRESHOLD: usize = 100_000;

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
    inspect_content_budgets(root, &mut checks);
    inspect_save(root, &mut checks);
    inspect_runtime_tree(root, &mut checks);
    let content = validate_project_content();
    inspect_transient_files(root, &mut checks);

    ProjectDoctorReport { checks, content }
}

fn inspect_content_budgets(root: &Path, checks: &mut Vec<DoctorCheck>) {
    let Ok(level_ids) = available_levels(root) else {
        return;
    };

    let mut peak_props = (0usize, String::new());
    let mut peak_dynamic = (0usize, String::new());
    let mut peak_triangles = (0usize, String::new());
    let mut readable_levels = 0usize;
    let mut readable_maps = 0usize;

    for level_id in level_ids {
        let level_path = root.join("levels").join(format!("{level_id}.json"));
        let path_label = level_path.to_string_lossy();
        let Ok(level) = LevelData::try_load(&path_label) else {
            continue;
        };
        readable_levels += 1;

        let prop_count = level.props.len();
        if peak_props.1.is_empty() || prop_count > peak_props.0 {
            peak_props = (prop_count, level_id.clone());
        }

        let dynamic_count = level
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some() || prop.path_id.is_some())
            .count();
        if peak_dynamic.1.is_empty() || dynamic_count > peak_dynamic.0 {
            peak_dynamic = (dynamic_count, level_id.clone());
        }

        let map_path = root.join(&level.base_map);
        let map_label = map_path.to_string_lossy();
        let Ok(model) = try_load_model(&map_label) else {
            continue;
        };
        readable_maps += 1;
        let triangle_count = model
            .parts
            .iter()
            .map(|part| part.indices.len() / 3)
            .sum::<usize>();
        if peak_triangles.1.is_empty() || triangle_count > peak_triangles.0 {
            peak_triangles = (triangle_count, level_id);
        }
    }

    if readable_levels == 0 {
        checks.push(check(
            CheckStatus::Warning,
            "Content budgets",
            "no readable levels were available for budget checks",
        ));
        return;
    }

    let mut exceeded = Vec::new();
    if peak_props.0 > LEVEL_PROP_WARNING_THRESHOLD {
        exceeded.push(format!(
            "{} has {} props (budget {})",
            peak_props.1, peak_props.0, LEVEL_PROP_WARNING_THRESHOLD
        ));
    }
    if peak_dynamic.0 > DYNAMIC_PROP_WARNING_THRESHOLD {
        exceeded.push(format!(
            "{} has {} dynamic props (budget {})",
            peak_dynamic.1, peak_dynamic.0, DYNAMIC_PROP_WARNING_THRESHOLD
        ));
    }
    if peak_triangles.0 > BASE_MAP_TRIANGLE_WARNING_THRESHOLD {
        exceeded.push(format!(
            "{} has {} base-map triangles (budget {})",
            peak_triangles.1, peak_triangles.0, BASE_MAP_TRIANGLE_WARNING_THRESHOLD
        ));
    }

    if !exceeded.is_empty() {
        checks.push(check(
            CheckStatus::Warning,
            "Content budgets",
            exceeded.join("; "),
        ));
        return;
    }

    let map_detail = if readable_maps == 0 {
        "base-map geometry unavailable".to_string()
    } else {
        format!(
            "{} base-map triangles in {}",
            peak_triangles.0, peak_triangles.1
        )
    };
    checks.push(check(
        CheckStatus::Pass,
        "Content budgets",
        format!(
            "peaks: {} props in {}, {} dynamic props in {}, {map_detail}",
            peak_props.0, peak_props.1, peak_dynamic.0, peak_dynamic.1
        ),
    ));
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
            "required runtime, content, and source paths are present",
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
                    CheckStatus::Warning,
                    "Autosave",
                    format!(
                        "save references removed level '{}' ({}); start a new run to replace this local save",
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

    #[test]
    fn current_project_stays_inside_content_budgets() {
        let mut checks = Vec::new();
        inspect_content_budgets(Path::new("."), &mut checks);

        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Pass, "{}", checks[0].detail);
    }
}
