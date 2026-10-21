//! Named, portable specimen checkpoints reuse the validated project format.
use super::{Project, read, write};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Names are bounded portable filenames, never paths.
pub fn path(root: &Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty()
        || name.len() > 64
        || name.starts_with('.')
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_ ".contains(c))
        || name.trim() != name
    {
        return Err("Use 1–64 letters, digits, spaces, hyphens or underscores".into());
    }
    let stem = name.split('.').next().unwrap().to_ascii_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (1..=9).any(|n| stem == format!("COM{n}") || stem == format!("LPT{n}"))
    {
        return Err("Reserved filename".into());
    }
    Ok(root.join(format!("{name}.json")))
}

pub fn entries(root: &Path) -> Result<Vec<String>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if entry.file_type().map_err(|e| e.to_string())?.is_file()
            && p.extension().is_some_and(|s| s == "json")
            && let Some(name) = p.file_stem().and_then(|s| s.to_str())
            && path(root, name).is_ok()
        {
            names.push(name.to_owned());
        }
    }
    names.sort();
    Ok(names)
}

/// Import, duplicate and export validate before writing and never replace an entry.
pub fn copy(source: &Path, destination: &Path) -> Result<(), String> {
    let project = read(source)?;
    validate(&project)?;
    create(destination, &project)
}

pub fn validate(project: &Project) -> Result<(), String> {
    project.validate()?;
    if project.sessions.len() + project.lenia.len() != 1 {
        return Err("A specimen must contain exactly one channel".into());
    }
    if project.sessions.iter().any(|s| !s.paused) || project.lenia.iter().any(|s| !s.paused) {
        return Err("Pause the specimen before saving".into());
    }
    Ok(())
}

pub fn create(destination: &Path, project: &Project) -> Result<(), String> {
    validate(project)?;
    if destination.exists() {
        return Err("Name already exists; choose a new name".into());
    }
    if let Some(parent) = destination.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    // Publish without replacing a concurrently created entry.
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temporary = destination.with_extension(format!("specimen-{}-{id}", std::process::id()));
    write(&temporary, project)?;
    let result = fs::hard_link(&temporary, destination).map_err(|e| e.to_string());
    let _ = fs::remove_file(temporary);
    result
}

/// Stable per-user location; override permits isolated tests and portable installations.
pub fn default_root() -> PathBuf {
    std::env::var_os("SOUL_LIBRARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs_next::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("soul")
                .join("specimens")
        })
}

#[cfg(test)]
mod tests;

pub mod orbium;
