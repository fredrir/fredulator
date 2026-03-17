use crate::domain::engine::{Engine, EvalSettings};
use crate::domain::types::*;
use crate::services::config::Config;

pub struct Tab {
    pub engine: Engine,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    History,
    Memory,
    Pinned,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModePanel {
    Converter,
    Tools,
    Notes,
}

pub struct AppState {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub scientific_mode: bool,
    pub panel_visible: bool,
    pub active_panel: Panel,
    pub mode_panel_visible: bool,
    pub active_mode: Option<ModePanel>,
    pub history_search: String,
    pub convert_category: usize,
    pub session_id: u64,
    pub config: Config,
}

impl AppState {
    pub fn new(config: Config, session_id: u64) -> Self {
        let settings = eval_settings(&config);
        let mut state = Self {
            tabs: Vec::new(),
            active_tab: 0,
            scientific_mode: config.layout.show_scientific,
            panel_visible: false,
            active_panel: Panel::History,
            mode_panel_visible: false,
            active_mode: None,
            history_search: String::new(),
            convert_category: 0,
            session_id,
            config,
        };
        state.tabs.push(Tab {
            engine: Engine::new(settings),
            name: "Calc 1".into(),
        });
        state
    }

    pub fn engine(&self) -> &Engine {
        &self.tabs[self.active_tab].engine
    }

    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.tabs[self.active_tab].engine
    }

    pub fn active_tab_name(&self) -> &str {
        &self.tabs[self.active_tab].name
    }

    pub fn eval_settings(&self) -> EvalSettings {
        eval_settings(&self.config)
    }

    pub fn timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

pub fn eval_settings(config: &Config) -> EvalSettings {
    EvalSettings {
        angle_mode: match config.behavior.angle_mode.as_str() {
            "radians" => AngleMode::Radians,
            _ => AngleMode::Degrees,
        },
        standard_precedence: config.behavior.operator_precedence,
        auto_evaluate: config.behavior.auto_evaluate,
        max_history: config.history.max_entries,
    }
}
