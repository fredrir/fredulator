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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_one_tab() {
        let state = AppState::new(Config::default(), 42);
        assert_eq!(state.tabs.len(), 1);
        assert_eq!(state.tabs[0].name, "Calc 1");
    }

    #[test]
    fn new_state_defaults() {
        let state = AppState::new(Config::default(), 100);
        assert_eq!(state.active_tab, 0);
        assert!(!state.panel_visible);
        assert_eq!(state.active_panel, Panel::History);
        assert!(!state.mode_panel_visible);
        assert!(state.active_mode.is_none());
        assert!(state.history_search.is_empty());
        assert_eq!(state.session_id, 100);
    }

    #[test]
    fn scientific_mode_from_config() {
        let mut config = Config::default();
        config.layout.show_scientific = true;
        let state = AppState::new(config, 0);
        assert!(state.scientific_mode);
    }

    #[test]
    fn eval_settings_degrees_default() {
        let config = Config::default();
        let settings = eval_settings(&config);
        assert_eq!(settings.angle_mode, AngleMode::Degrees);
        assert!(settings.standard_precedence);
        assert!(settings.auto_evaluate);
        assert_eq!(settings.max_history, 200);
    }

    #[test]
    fn eval_settings_radians() {
        let mut config = Config::default();
        config.behavior.angle_mode = "radians".into();
        let settings = eval_settings(&config);
        assert_eq!(settings.angle_mode, AngleMode::Radians);
    }

    #[test]
    fn eval_settings_no_precedence() {
        let mut config = Config::default();
        config.behavior.operator_precedence = false;
        let settings = eval_settings(&config);
        assert!(!settings.standard_precedence);
    }

    #[test]
    fn eval_settings_custom_history_limit() {
        let mut config = Config::default();
        config.history.max_entries = 50;
        let settings = eval_settings(&config);
        assert_eq!(settings.max_history, 50);
    }

    #[test]
    fn eval_settings_unknown_angle_mode_defaults_to_degrees() {
        let mut config = Config::default();
        config.behavior.angle_mode = "grads".into();
        let settings = eval_settings(&config);
        assert_eq!(settings.angle_mode, AngleMode::Degrees);
    }

    #[test]
    fn engine_accessor() {
        let state = AppState::new(Config::default(), 0);
        let _ = state.engine().main_display_text();
    }

    #[test]
    fn engine_mut_accessor() {
        let mut state = AppState::new(Config::default(), 0);
        state.engine_mut().input_digit('5');
        assert!(state.engine().main_display_text().contains('5'));
    }
}
