use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

#[cfg(unix)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

pub fn ensure_relative_symlink(link_path: &Path, target_path: &Path) -> AppResult<()> {
    let link_parent = link_path.parent().ok_or_else(|| {
        AppError::System(format!(
            "Failed to create symlink for '{}': missing parent directory",
            link_path.display()
        ))
    })?;

    std::fs::create_dir_all(link_parent).map_err(|e| {
        AppError::System(format!(
            "Failed to create directory '{}': {}",
            link_parent.display(),
            e
        ))
    })?;

    let relative_target = relative_path(link_parent, target_path);
    ensure_symlink(link_path, &relative_target)
}

pub fn ensure_symlink(link_path: &Path, link_target: &Path) -> AppResult<()> {
    let link_parent = link_path.parent().ok_or_else(|| {
        AppError::System(format!(
            "Failed to create symlink for '{}': missing parent directory",
            link_path.display()
        ))
    })?;

    std::fs::create_dir_all(link_parent).map_err(|e| {
        AppError::System(format!(
            "Failed to create directory '{}': {}",
            link_parent.display(),
            e
        ))
    })?;

    if let Ok(metadata) = std::fs::symlink_metadata(link_path) {
        if metadata.file_type().is_symlink() {
            if let Ok(existing_target) = std::fs::read_link(link_path) {
                if existing_target == link_target {
                    return Ok(());
                }
            }
        }

        std::fs::remove_file(link_path).map_err(|e| {
            AppError::System(format!(
                "Failed to replace config link '{}': {}",
                link_path.display(),
                e
            ))
        })?;
    }

    create_file_symlink(link_target, link_path).map_err(|e| {
        AppError::System(format!(
            "Failed to create symlink '{}' -> '{}': {}",
            link_path.display(),
            link_target.display(),
            e
        ))
    })
}

pub fn path_exists_or_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

fn relative_path(from_dir: &Path, to_path: &Path) -> PathBuf {
    let from_components = from_dir.components().collect::<Vec<_>>();
    let to_components = to_path.components().collect::<Vec<_>>();

    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(from, to)| from == to)
        .count();

    let mut relative = PathBuf::new();
    for _ in common_len..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[common_len..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        relative.push(".");
    }

    relative
}
