use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

const STALE_LOCK_AGE: Duration = Duration::from_secs(30);
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn write_file_staged(path: impl AsRef<Path>, bytes: &[u8]) -> Result<(), String> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create '{}': {}", parent.display(), error))?;

    let write_lock = WriteLock::acquire(path)?;
    recover_interrupted_write_inner(path)?;
    let (temp_path, mut temp_file) = create_staging_file(path)?;
    let recovery_path = sidecar_path(path, "recover");

    let result = (|| {
        temp_file
            .write_all(bytes)
            .map_err(|error| format!("failed to stage data for '{}': {}", path.display(), error))?;
        temp_file.flush().map_err(|error| {
            format!(
                "failed to flush staged data for '{}': {}",
                path.display(),
                error
            )
        })?;
        temp_file.sync_all().map_err(|error| {
            format!(
                "failed to sync staged data for '{}': {}",
                path.display(),
                error
            )
        })?;
        drop(temp_file);

        if path.is_file() {
            std::fs::copy(path, &recovery_path).map_err(|error| {
                format!(
                    "failed to preserve '{}' before replacement: {}",
                    path.display(),
                    error
                )
            })?;
            sync_file(&recovery_path)?;
        }

        match std::fs::rename(&temp_path, path) {
            Ok(()) => Ok(()),
            Err(rename_error) if path.exists() => {
                std::fs::remove_file(path).map_err(|remove_error| {
                    format!(
                        "failed to replace '{}' after rename error '{}': {}",
                        path.display(),
                        rename_error,
                        remove_error
                    )
                })?;
                std::fs::rename(&temp_path, path).map_err(|error| {
                    format!(
                        "failed to move staged data into '{}': {}",
                        path.display(),
                        error
                    )
                })
            }
            Err(error) => Err(format!(
                "failed to move staged data into '{}': {}",
                path.display(),
                error
            )),
        }
    })();

    match result {
        Ok(()) => {
            std::fs::remove_file(&recovery_path).ok();
            drop(write_lock);
            Ok(())
        }
        Err(error) => {
            std::fs::remove_file(&temp_path).ok();
            if !path.exists() && recovery_path.is_file() {
                if let Err(restore_error) = std::fs::rename(&recovery_path, path) {
                    drop(write_lock);
                    return Err(format!(
                        "{}; recovery copy could not be restored: {}",
                        error, restore_error
                    ));
                }
            }
            drop(write_lock);
            Err(error)
        }
    }
}

pub fn recover_interrupted_write(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    let lock_path = sidecar_path(path, "lock");
    if lock_path.exists() {
        if lock_is_stale(&lock_path) {
            std::fs::remove_file(&lock_path).map_err(|error| {
                format!(
                    "failed to clear stale write lock '{}': {}",
                    lock_path.display(),
                    error
                )
            })?;
        } else if !path.exists() {
            return Err(format!(
                "'{}' is currently being replaced by another process",
                path.display()
            ));
        } else {
            return Ok(());
        }
    }
    recover_interrupted_write_inner(path)
}

fn recover_interrupted_write_inner(path: &Path) -> Result<(), String> {
    let recovery_path = sidecar_path(path, "recover");
    if path.exists() {
        if recovery_path.exists() {
            std::fs::remove_file(&recovery_path).map_err(|error| {
                format!(
                    "failed to remove stale recovery file '{}': {}",
                    recovery_path.display(),
                    error
                )
            })?;
        }
    } else if recovery_path.exists() {
        std::fs::rename(&recovery_path, path).map_err(|error| {
            format!(
                "failed to recover interrupted write for '{}': {}",
                path.display(),
                error
            )
        })?;
    }
    Ok(())
}

fn create_staging_file(path: &Path) -> Result<(PathBuf, std::fs::File), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("data");
    for _ in 0..128 {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp_path = parent.join(format!(
            ".{}.tmp.{}.{}",
            file_name,
            std::process::id(),
            sequence
        ));
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create staging file '{}': {}",
                    temp_path.display(),
                    error
                ))
            }
        }
    }
    Err(format!(
        "failed to allocate a unique staging file in '{}'",
        parent.display()
    ))
}

fn sync_file(path: &Path) -> Result<(), String> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("failed to sync '{}': {}", path.display(), error))
}

fn sidecar_path(path: &Path, suffix: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("data");
    path.with_file_name(format!(".{}.{}", file_name, suffix))
}

fn lock_is_stale(path: &Path) -> bool {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age >= STALE_LOCK_AGE)
}

struct WriteLock {
    path: PathBuf,
}

impl WriteLock {
    fn acquire(target: &Path) -> Result<Self, String> {
        let path = sidecar_path(target, "lock");
        for _ in 0..2 {
            match std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
            {
                Ok(mut file) => {
                    if let Err(error) = writeln!(file, "pid={}", std::process::id()) {
                        drop(file);
                        std::fs::remove_file(&path).ok();
                        return Err(format!(
                            "failed to initialize write lock '{}': {}",
                            path.display(),
                            error
                        ));
                    }
                    if let Err(error) = file.sync_all() {
                        drop(file);
                        std::fs::remove_file(&path).ok();
                        return Err(format!(
                            "failed to sync write lock '{}': {}",
                            path.display(),
                            error
                        ));
                    }
                    return Ok(Self { path });
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::AlreadyExists
                        && lock_is_stale(&path) =>
                {
                    std::fs::remove_file(&path).map_err(|remove_error| {
                        format!(
                            "failed to clear stale write lock '{}': {}",
                            path.display(),
                            remove_error
                        )
                    })?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err(format!(
                        "another process is already writing '{}'",
                        target.display()
                    ));
                }
                Err(error) => {
                    return Err(format!(
                        "failed to create write lock '{}': {}",
                        path.display(),
                        error
                    ));
                }
            }
        }
        Err(format!(
            "failed to acquire write lock for '{}'",
            target.display()
        ))
    }
}

impl Drop for WriteLock {
    fn drop(&mut self) {
        std::fs::remove_file(&self.path).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_write_replaces_content_and_cleans_sidecars() {
        let path = unique_path("staged_write");
        write_file_staged(&path, b"first").unwrap();
        write_file_staged(&path, b"second").unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"second");
        assert!(!sidecar_path(&path, "lock").exists());
        assert!(!sidecar_path(&path, "recover").exists());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn missing_target_is_restored_from_recovery_sidecar() {
        let path = unique_path("interrupted_write");
        let recovery = sidecar_path(&path, "recover");
        std::fs::write(&recovery, "known good").unwrap();

        recover_interrupted_write(&path).unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "known good");
        assert!(!recovery.exists());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn active_writer_is_not_overridden() {
        let path = unique_path("active_write");
        let lock = sidecar_path(&path, "lock");
        std::fs::write(&lock, "pid=test").unwrap();

        let error = recover_interrupted_write(&path).unwrap_err();

        assert!(error.contains("currently being replaced"));
        std::fs::remove_file(lock).ok();
    }

    fn unique_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "cenotaph_{}_{}_{}",
            label,
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
