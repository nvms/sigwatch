use crate::layout::LayoutDef;
use crate::watch::Watchpoint;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub pid: u32,
    pub watches: Vec<Watchpoint>,
    pub layouts: Vec<LayoutDef>,
    pub poll_rate_ms: u64,
}

impl Session {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            watches: Vec::new(),
            layouts: Vec::new(),
            poll_rate_ms: 100,
        }
    }
}

pub fn save(session: &Session, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(session)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load(path: &str) -> Result<Session> {
    let content = std::fs::read_to_string(path)?;
    let session: Session = serde_json::from_str(&content)?;
    Ok(session)
}

pub fn default_session_dir() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|d| d.join("sigwatch").join("sessions"))
}

pub fn auto_save_path(pid: u32) -> Option<std::path::PathBuf> {
    default_session_dir().map(|d| d.join(format!("{pid}.json")))
}

pub fn ensure_session_dir() -> Result<std::path::PathBuf> {
    let dir = default_session_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine data directory"))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_roundtrip() {
        let session = Session::new(1234);
        let json = serde_json::to_string(&session).unwrap();
        let loaded: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.pid, 1234);
        assert_eq!(loaded.poll_rate_ms, 100);
        assert!(loaded.watches.is_empty());
    }

    #[test]
    fn test_auto_save_path() {
        let path = auto_save_path(1234);
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_str().unwrap().contains("1234.json"));
    }
}
