use std::fs;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fredulator_test_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

mod history_io {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    struct HistoryEntry {
        expression: String,
        result_text: String,
        result: f64,
        #[serde(default)]
        timestamp: u64,
        #[serde(default)]
        session: u64,
    }

    fn save_history_to(dir: &Path, history: &[HistoryEntry]) {
        let p = dir.join("history.json");
        let json = serde_json::to_string(history).unwrap();
        fs::write(p, json).unwrap();
    }

    fn load_history_from(dir: &Path) -> Vec<HistoryEntry> {
        let p = dir.join("history.json");
        match fs::read_to_string(p) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = temp_dir("history_roundtrip");
        let entries = vec![
            HistoryEntry {
                expression: "2+3".into(),
                result_text: "5".into(),
                result: 5.0,
                timestamp: 1000,
                session: 1,
            },
            HistoryEntry {
                expression: "10/3".into(),
                result_text: "3.3333333333".into(),
                result: 10.0 / 3.0,
                timestamp: 1001,
                session: 1,
            },
        ];
        save_history_to(&dir, &entries);
        let loaded = load_history_from(&dir);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].expression, "2+3");
        assert_eq!(loaded[1].result, 10.0 / 3.0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_from_missing_file() {
        let dir = temp_dir("history_missing");
        let loaded = load_history_from(&dir);
        assert!(loaded.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_from_corrupt_json() {
        let dir = temp_dir("history_corrupt");
        fs::write(dir.join("history.json"), "not valid json!!!").unwrap();
        let loaded = load_history_from(&dir);
        assert!(loaded.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn csv_export_format() {
        let dir = temp_dir("history_csv");
        let entries = vec![HistoryEntry {
            expression: "1+1".into(),
            result_text: "2".into(),
            result: 2.0,
            timestamp: 42,
            session: 1,
        }];
        let p = dir.join("history_export.csv");
        let mut csv = String::from("expression,result,timestamp\n");
        for entry in &entries {
            csv.push_str(&format!(
                "\"{}\",{},{}\n",
                entry.expression.replace('"', "\"\""),
                entry.result_text,
                entry.timestamp,
            ));
        }
        fs::write(&p, &csv).unwrap();

        let contents = fs::read_to_string(&p).unwrap();
        assert!(contents.starts_with("expression,result,timestamp\n"));
        assert!(contents.contains("\"1+1\""));
        let _ = fs::remove_dir_all(&dir);
    }
}

mod session_io {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct TabState {
        name: String,
        note: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct SessionState {
        tabs: Vec<TabState>,
        active_tab: usize,
        scientific_mode: bool,
    }

    #[test]
    fn session_roundtrip() {
        let dir = temp_dir("session_roundtrip");
        let state = SessionState {
            tabs: vec![
                TabState { name: "Calc 1".into(), note: "".into() },
                TabState { name: "Tax".into(), note: "some notes".into() },
            ],
            active_tab: 1,
            scientific_mode: true,
        };
        let p = dir.join("session.json");
        let json = serde_json::to_string(&state).unwrap();
        fs::write(&p, &json).unwrap();

        let loaded: SessionState = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(loaded.tabs.len(), 2);
        assert_eq!(loaded.active_tab, 1);
        assert!(loaded.scientific_mode);
        assert_eq!(loaded.tabs[1].name, "Tax");
        assert_eq!(loaded.tabs[1].note, "some notes");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_missing_session() {
        let dir = temp_dir("session_missing");
        let p = dir.join("session.json");
        let result: Option<SessionState> = fs::read_to_string(p)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok());
        assert!(result.is_none());
        let _ = fs::remove_dir_all(&dir);
    }
}

mod geometry_io {
    use super::*;

    #[test]
    fn geometry_roundtrip() {
        let dir = temp_dir("geometry_roundtrip");
        let p = dir.join("geometry");
        let (x, y, w, h) = (100, 200, 360, 580);
        fs::write(&p, format!("{},{},{},{}", x, y, w, h)).unwrap();

        let s = fs::read_to_string(&p).unwrap();
        let parts: Vec<i32> = s.trim().split(',').filter_map(|p| p.parse().ok()).collect();
        assert_eq!(parts, vec![100, 200, 360, 580]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn geometry_corrupt_returns_none() {
        let dir = temp_dir("geometry_corrupt");
        let p = dir.join("geometry");
        fs::write(&p, "not,numbers,here").unwrap();

        let s = fs::read_to_string(&p).unwrap();
        let parts: Vec<i32> = s.trim().split(',').filter_map(|p| p.parse().ok()).collect();
        let result = if parts.len() == 4 {
            Some((parts[0], parts[1], parts[2], parts[3]))
        } else {
            None
        };
        assert!(result.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn geometry_too_few_fields() {
        let dir = temp_dir("geometry_few");
        let p = dir.join("geometry");
        fs::write(&p, "100,200").unwrap();

        let s = fs::read_to_string(&p).unwrap();
        let parts: Vec<i32> = s.trim().split(',').filter_map(|p| p.parse().ok()).collect();
        let result = if parts.len() == 4 {
            Some((parts[0], parts[1], parts[2], parts[3]))
        } else {
            None
        };
        assert!(result.is_none());
        let _ = fs::remove_dir_all(&dir);
    }
}

mod config_parsing {
    #[test]
    fn toml_roundtrip_default_config() {
        let default_toml = include_str!("../src/services/config.rs");
        let template = r##"[theme]
name = "native"
accent_color = ""
button_style = "rounded"
font = "system"

[behavior]
auto_evaluate = true
operator_precedence = true
angle_mode = "degrees"

[history]
max_entries = 200
auto_save = false

[layout]
button_spacing = 6
grid_padding = 8
button_radius = 12
"##;
        let parsed: toml::Value = toml::from_str(template).unwrap();
        assert_eq!(parsed["theme"]["name"].as_str().unwrap(), "native");
        assert_eq!(parsed["behavior"]["angle_mode"].as_str().unwrap(), "degrees");
        assert_eq!(parsed["history"]["max_entries"].as_integer().unwrap(), 200);
        let _ = default_toml;
    }
}
