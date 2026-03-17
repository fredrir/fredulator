use std::fs;
use std::path::PathBuf;

use crate::domain::types::HistoryEntry;
use crate::services::config;

pub fn history_path() -> PathBuf {
    config::dir().join("history.json")
}

pub fn save_history(history: &[HistoryEntry], auto_save: bool) {
    if !auto_save {
        return;
    }
    let _ = fs::create_dir_all(config::dir());
    if let Ok(json) = serde_json::to_string(history) {
        let _ = fs::write(history_path(), json);
    }
}

pub fn load_history(auto_save: bool) -> Vec<HistoryEntry> {
    if !auto_save {
        return Vec::new();
    }
    let p = history_path();
    match fs::read_to_string(p) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn export_history_json(history: &[HistoryEntry]) -> PathBuf {
    let _ = fs::create_dir_all(config::dir());
    let p = config::dir().join("history_export.json");
    if let Ok(json) = serde_json::to_string_pretty(history) {
        let _ = fs::write(&p, json);
    }
    p
}

pub fn export_history_csv(history: &[HistoryEntry]) -> PathBuf {
    let _ = fs::create_dir_all(config::dir());
    let p = config::dir().join("history_export.csv");
    let mut s = String::from("expression,result,timestamp\n");
    for entry in history {
        s.push_str(&format!(
            "\"{}\",{},{}\n",
            entry.expression.replace('"', "\"\""),
            entry.result_text,
            entry.timestamp
        ));
    }
    let _ = fs::write(&p, s);
    p
}
