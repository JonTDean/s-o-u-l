//! Versioned, validated experiment files with atomic replacement on disk.
pub mod maps;
mod storage;
mod validation;
use crate::workbench::{plugin::Workbench, session::Session};
use engine_core::automata::lenia::SavedLenia;
pub use storage::{read, write};
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub version: u32,
    pub selected: usize,
    pub sessions: Vec<Session>,
    pub lenia: Vec<SavedLenia>,
}
impl Project {
    pub fn capture(bench: &Workbench, lenia: Vec<SavedLenia>) -> Result<Self, String> {
        let sessions = serde_json::from_value(
            serde_json::to_value(&bench.sessions).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let project = Self {
            version: 2,
            selected: bench.selected,
            sessions,
            lenia,
        };
        project.validate()?;
        Ok(project)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.version, 1 | 2) {
            return Err("Unsupported project version".into());
        }
        if self.sessions.len() + self.lenia.len() > 8
            || self.lenia.len() > 8
            || self.selected >= self.sessions.len().max(1)
        {
            return Err("Invalid session capacity or selection".into());
        }
        for session in &self.sessions {
            validation::session(session)?;
        }
        for saved in &self.lenia {
            saved.validate()?;
        }
        Ok(())
    }
    pub fn apply_cpu(self, bench: &mut Workbench) {
        bench.sessions = self.sessions;
        bench.selected = self.selected;
        bench.error = None;
    }
}

#[cfg(test)]
mod tests;
