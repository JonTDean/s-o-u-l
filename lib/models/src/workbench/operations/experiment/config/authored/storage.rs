//! Bounded portable rule files, published atomically without replacing a file.
use super::composition::Program;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
pub const MAX_BYTES: usize = 128 * 1024;
pub fn parse(text: &str) -> Result<Program, String> {
    if text.len() > MAX_BYTES {
        return Err("Rule file exceeds 128 KiB".into());
    }
    let program: Program = serde_json::from_str(text).map_err(|e| e.to_string())?;
    program.validate()?;
    Ok(program)
}
pub fn read(path: &Path) -> Result<Program, String> {
    let mut text = String::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_BYTES as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    parse(&text)
}
pub fn write(path: &Path, program: &Program) -> Result<(), String> {
    program.validate()?;
    let bytes = serde_json::to_vec_pretty(program).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("Rule file exceeds 128 KiB".into());
    }
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temporary = path.with_extension(format!("rule-{}-{seq}", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::hard_link(&temporary, path)
    })()
    .map_err(|e: std::io::Error| e.to_string());
    drop(file);
    let _ = fs::remove_file(temporary);
    result
}
