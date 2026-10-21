//! Bounded reads and same-directory atomic file replacement.
use super::Project;
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};
const MAX_BYTES: u64 = 16 * 1024 * 1024;
pub fn read(path: &Path) -> Result<Project, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Project exceeds 16 MiB".into());
    }
    let project: Project = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    project.validate()?;
    Ok(project)
}
pub fn write(path: &Path, project: &Project) -> Result<(), String> {
    project.validate()?;
    let bytes = serde_json::to_vec_pretty(project).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Project exceeds 16 MiB".into());
    }
    let temporary = path.with_extension(format!("soul-tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
        #[cfg(not(unix))]
        let _ = parent;
        Ok(())
    })()
    .map_err(|e: std::io::Error| e.to_string());
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
