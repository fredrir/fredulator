use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::domain::types::HistoryEntry;
use crate::services::config;

#[derive(Serialize, Deserialize)]
pub struct SessionState {
    pub tabs: Vec<TabState>,
    pub active_tab: usize,
    pub scientific_mode: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TabState {
    pub name: String,
    pub note: String,
    pub history: Vec<HistoryEntry>,
}

pub fn session_path() -> PathBuf {
    config::dir().join("session.json")
}

pub fn save_session(state: &SessionState) {
    let _ = fs::create_dir_all(config::dir());
    if let Ok(json) = serde_json::to_string(state) {
        let _ = fs::write(session_path(), json);
    }
}

pub fn load_session() -> Option<SessionState> {
    let p = session_path();
    let json = fs::read_to_string(p).ok()?;
    serde_json::from_str(&json).ok()
}

fn geometry_path() -> PathBuf {
    config::dir().join("geometry")
}

pub fn save_geometry(x: i32, y: i32, w: i32, h: i32) {
    let _ = fs::create_dir_all(config::dir());
    let _ = fs::write(geometry_path(), format!("{},{},{},{}", x, y, w, h));
}

pub fn load_geometry() -> Option<(i32, i32, i32, i32)> {
    let s = fs::read_to_string(geometry_path()).ok()?;
    let parts: Vec<i32> = s.trim().split(',').filter_map(|p| p.parse().ok()).collect();
    if parts.len() == 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}
