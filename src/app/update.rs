use crate::domain::engine::Engine;
use crate::services::{history, session};

use super::message::Message;
use super::state::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SideEffect {
    UpdateDisplay,
    UpdateTabs,
    ToggleScientific(bool),
    ToggleTheme,
    TogglePanel,
    ToggleModePanel,
    RefreshHistory,
    RefreshMemory,
    RefreshPinned,
    ExportedFile(std::path::PathBuf),
    ShowHelp,
    Navigate(crate::ui::keyboard::Direction),
    ActivateButton,
    OpenMenu,
    Quit,
    ResizeWindow,
    Noop,
}

pub fn update(state: &mut AppState, msg: Message) -> Vec<SideEffect> {
    match msg {
        Message::Digit(d) => {
            state.engine_mut().input_digit(d);
            vec![SideEffect::UpdateDisplay]
        }
        Message::Decimal => {
            state.engine_mut().input_decimal();
            vec![SideEffect::UpdateDisplay]
        }
        Message::BinaryOp(op) => {
            state.engine_mut().input_binary_op(op);
            vec![SideEffect::UpdateDisplay]
        }
        Message::UnaryFunc(f) => {
            state.engine_mut().input_unary_func(f);
            vec![SideEffect::UpdateDisplay]
        }
        Message::PostfixOp(op) => {
            state.engine_mut().input_postfix_op(op);
            vec![SideEffect::UpdateDisplay]
        }
        Message::Constant(val, name) => {
            state.engine_mut().input_constant(val, name);
            vec![SideEffect::UpdateDisplay]
        }
        Message::Equals => {
            let ts = state.timestamp();
            let session = state.session_id;
            state.engine_mut().calculate(ts, session);
            history::save_history(&state.engine().history, state.config.history.auto_save);
            vec![SideEffect::UpdateDisplay]
        }
        Message::Clear => {
            state.engine_mut().clear();
            vec![SideEffect::UpdateDisplay]
        }
        Message::Backspace => {
            state.engine_mut().backspace();
            vec![SideEffect::UpdateDisplay]
        }
        Message::ToggleSign => {
            state.engine_mut().toggle_sign();
            vec![SideEffect::UpdateDisplay]
        }
        Message::LeftParen => {
            state.engine_mut().input_left_paren();
            vec![SideEffect::UpdateDisplay]
        }
        Message::RightParen => {
            state.engine_mut().input_right_paren();
            vec![SideEffect::UpdateDisplay]
        }
        Message::EE => {
            state.engine_mut().input_ee();
            vec![SideEffect::UpdateDisplay]
        }
        Message::MemoryClear => {
            state.engine_mut().memory_clear();
            vec![SideEffect::UpdateDisplay]
        }
        Message::MemoryRecall => {
            state.engine_mut().memory_recall();
            vec![SideEffect::UpdateDisplay]
        }
        Message::MemoryAdd => {
            state.engine_mut().memory_add();
            vec![SideEffect::UpdateDisplay]
        }
        Message::MemorySubtract => {
            state.engine_mut().memory_subtract();
            vec![SideEffect::UpdateDisplay]
        }
        Message::MemoryStore => {
            let count = state.engine().memory_slots.len() + 1;
            state.engine_mut().memory_store(format!("M{}", count));
            vec![SideEffect::UpdateDisplay]
        }
        Message::ToggleAngleMode => {
            state.engine_mut().toggle_angle_mode();
            vec![SideEffect::UpdateDisplay]
        }
        Message::Undo => {
            state.engine_mut().undo();
            vec![SideEffect::UpdateDisplay]
        }
        Message::NewTab => {
            let n = state.tabs.len() + 1;
            let settings = state.eval_settings();
            state.tabs.push(Tab {
                engine: Engine::new(settings),
                name: format!("Calc {}", n),
            });
            state.active_tab = state.tabs.len() - 1;
            vec![SideEffect::UpdateTabs, SideEffect::UpdateDisplay]
        }
        Message::CloseTab => {
            if state.tabs.len() <= 1 {
                return vec![SideEffect::Noop];
            }
            state.tabs.remove(state.active_tab);
            if state.active_tab >= state.tabs.len() {
                state.active_tab = state.tabs.len() - 1;
            }
            vec![SideEffect::UpdateTabs, SideEffect::UpdateDisplay]
        }
        Message::CloseTabAt(idx) => {
            if state.tabs.len() <= 1 {
                return vec![SideEffect::Noop];
            }
            if idx < state.tabs.len() {
                state.tabs.remove(idx);
                if state.active_tab >= state.tabs.len() {
                    state.active_tab = state.tabs.len() - 1;
                }
            }
            vec![SideEffect::UpdateTabs, SideEffect::UpdateDisplay]
        }
        Message::SwitchTab(idx) => {
            if idx < state.tabs.len() && idx != state.active_tab {
                state.active_tab = idx;
                vec![SideEffect::UpdateDisplay, SideEffect::UpdateTabs]
            } else {
                vec![SideEffect::Noop]
            }
        }
        Message::NextTab => {
            if state.tabs.len() <= 1 {
                return vec![SideEffect::Noop];
            }
            state.active_tab = (state.active_tab + 1) % state.tabs.len();
            vec![SideEffect::UpdateDisplay, SideEffect::UpdateTabs]
        }
        Message::PrevTab => {
            if state.tabs.len() <= 1 {
                return vec![SideEffect::Noop];
            }
            state.active_tab = if state.active_tab == 0 {
                state.tabs.len() - 1
            } else {
                state.active_tab - 1
            };
            vec![SideEffect::UpdateDisplay, SideEffect::UpdateTabs]
        }
        Message::RenameTab(idx, name) => {
            if idx < state.tabs.len() && !name.is_empty() {
                state.tabs[idx].name = name;
                vec![SideEffect::UpdateTabs]
            } else {
                vec![SideEffect::Noop]
            }
        }
        Message::ToggleScientific => {
            state.scientific_mode = !state.scientific_mode;
            vec![SideEffect::ToggleScientific(state.scientific_mode), SideEffect::ResizeWindow]
        }
        Message::ToggleTheme => {
            vec![SideEffect::ToggleTheme]
        }
        Message::ToggleHistory => {
            if state.panel_visible && state.active_panel == Panel::History {
                state.panel_visible = false;
            } else {
                state.active_panel = Panel::History;
                state.panel_visible = true;
            }
            vec![SideEffect::TogglePanel, SideEffect::RefreshHistory]
        }
        Message::ToggleMemory => {
            if state.panel_visible && state.active_panel == Panel::Memory {
                state.panel_visible = false;
            } else {
                state.active_panel = Panel::Memory;
                state.panel_visible = true;
            }
            vec![SideEffect::TogglePanel, SideEffect::RefreshMemory]
        }
        Message::TogglePinned => {
            if state.panel_visible && state.active_panel == Panel::Pinned {
                state.panel_visible = false;
            } else {
                state.active_panel = Panel::Pinned;
                state.panel_visible = true;
            }
            vec![SideEffect::TogglePanel, SideEffect::RefreshPinned]
        }
        Message::PinResult => {
            let count = state.engine().pinned.len() + 1;
            state.engine_mut().pin_result(format!("Pin {}", count));
            vec![SideEffect::Noop]
        }
        Message::SearchHistory(query) => {
            state.history_search = query;
            vec![SideEffect::RefreshHistory]
        }
        Message::ClearHistory => {
            state.engine_mut().clear_history();
            history::save_history(&state.engine().history, state.config.history.auto_save);
            vec![SideEffect::RefreshHistory]
        }
        Message::ExportHistoryJson => {
            let p = history::export_history_json(&state.engine().history);
            vec![SideEffect::ExportedFile(p)]
        }
        Message::ExportHistoryCsv => {
            let p = history::export_history_csv(&state.engine().history);
            vec![SideEffect::ExportedFile(p)]
        }
        Message::OpenConverter => {
            toggle_mode(state, ModePanel::Converter);
            vec![SideEffect::ToggleModePanel]
        }
        Message::OpenTools => {
            toggle_mode(state, ModePanel::Tools);
            vec![SideEffect::ToggleModePanel]
        }
        Message::OpenNotes => {
            toggle_mode(state, ModePanel::Notes);
            vec![SideEffect::ToggleModePanel]
        }
        Message::CloseMode => {
            if state.mode_panel_visible {
                state.mode_panel_visible = false;
                state.active_mode = None;
                vec![SideEffect::ToggleModePanel]
            } else if state.panel_visible {
                state.panel_visible = false;
                vec![SideEffect::TogglePanel]
            } else {
                state.engine_mut().clear();
                vec![SideEffect::UpdateDisplay]
            }
        }
        Message::ShowHelp => {
            vec![SideEffect::ShowHelp]
        }
        Message::Quit => {
            save_on_exit(state);
            vec![SideEffect::Quit]
        }
        Message::Navigate(dir) => {
            vec![SideEffect::Navigate(dir)]
        }
        Message::Activate => {
            vec![SideEffect::ActivateButton]
        }
        Message::OpenMenu => {
            vec![SideEffect::OpenMenu]
        }
        Message::Noop => {
            vec![SideEffect::Noop]
        }
    }
}

fn toggle_mode(state: &mut AppState, mode: ModePanel) {
    if state.active_mode == Some(mode) && state.mode_panel_visible {
        state.mode_panel_visible = false;
        state.active_mode = None;
    } else {
        state.active_mode = Some(mode);
        state.mode_panel_visible = true;
    }
}

pub fn save_on_exit(state: &AppState) {
    if state.config.session.restore_session {
        let tab_states: Vec<session::TabState> = state
            .tabs
            .iter()
            .map(|tab| session::TabState {
                name: tab.name.clone(),
                note: tab.engine.note.clone(),
                history: tab.engine.history.clone(),
            })
            .collect();
        let ss = session::SessionState {
            tabs: tab_states,
            active_tab: state.active_tab,
            scientific_mode: state.scientific_mode,
        };
        session::save_session(&ss);
    }
}

pub fn restore_session(state: &mut AppState) {
    if !state.config.session.restore_session {
        return;
    }
    if let Some(ss) = session::load_session() {
        state.tabs.clear();
        let settings = state.eval_settings();
        for ts in &ss.tabs {
            let mut engine = Engine::new(settings);
            for entry in &ts.history {
                engine.history.push(entry.clone());
            }
            engine.note = ts.note.clone();
            state.tabs.push(Tab { engine, name: ts.name.clone() });
        }
        if state.tabs.is_empty() {
            state.tabs.push(Tab {
                engine: Engine::new(settings),
                name: "Calc 1".into(),
            });
        }
        state.active_tab = ss.active_tab.min(state.tabs.len() - 1);
        state.scientific_mode = ss.scientific_mode;
    } else {
        let loaded = history::load_history(state.config.history.auto_save);
        for entry in loaded {
            state.engine_mut().history.push(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::config::Config;

    fn test_state() -> AppState {
        AppState::new(Config::default(), 12345)
    }

    #[test]
    fn digit_updates_display() {
        let mut s = test_state();
        let effects = update(&mut s, Message::Digit('5'));
        assert_eq!(effects, vec![SideEffect::UpdateDisplay]);
        assert_eq!(s.engine().main_display_text(), "5");
    }

    #[test]
    fn calculation_flow() {
        let mut s = test_state();
        update(&mut s, Message::Digit('2'));
        update(&mut s, Message::BinaryOp(crate::domain::types::BinaryOp::Add));
        update(&mut s, Message::Digit('3'));
        update(&mut s, Message::Equals);
        assert_eq!(s.engine().main_display_text(), "5");
        assert_eq!(s.engine().history.len(), 1);
    }

    #[test]
    fn tab_management() {
        let mut s = test_state();
        assert_eq!(s.tabs.len(), 1);

        update(&mut s, Message::NewTab);
        assert_eq!(s.tabs.len(), 2);
        assert_eq!(s.active_tab, 1);
        assert_eq!(s.tabs[1].name, "Calc 2");

        update(&mut s, Message::SwitchTab(0));
        assert_eq!(s.active_tab, 0);

        update(&mut s, Message::CloseTab);
        assert_eq!(s.tabs.len(), 1);
    }

    #[test]
    fn cannot_close_last_tab() {
        let mut s = test_state();
        let effects = update(&mut s, Message::CloseTab);
        assert_eq!(effects, vec![SideEffect::Noop]);
        assert_eq!(s.tabs.len(), 1);
    }

    #[test]
    fn next_prev_tab() {
        let mut s = test_state();
        update(&mut s, Message::NewTab);
        update(&mut s, Message::NewTab);
        assert_eq!(s.active_tab, 2);

        update(&mut s, Message::NextTab);
        assert_eq!(s.active_tab, 0);

        update(&mut s, Message::PrevTab);
        assert_eq!(s.active_tab, 2);
    }

    #[test]
    fn rename_tab() {
        let mut s = test_state();
        update(&mut s, Message::RenameTab(0, "Budget".into()));
        assert_eq!(s.tabs[0].name, "Budget");
    }

    #[test]
    fn rename_empty_string_ignored() {
        let mut s = test_state();
        update(&mut s, Message::RenameTab(0, "".into()));
        assert_eq!(s.tabs[0].name, "Calc 1");
    }

    #[test]
    fn toggle_scientific() {
        let mut s = test_state();
        assert!(!s.scientific_mode);
        let effects = update(&mut s, Message::ToggleScientific);
        assert!(s.scientific_mode);
        assert!(effects.contains(&SideEffect::ToggleScientific(true)));
    }

    #[test]
    fn panel_toggle() {
        let mut s = test_state();
        assert!(!s.panel_visible);

        update(&mut s, Message::ToggleHistory);
        assert!(s.panel_visible);
        assert_eq!(s.active_panel, Panel::History);

        update(&mut s, Message::ToggleMemory);
        assert!(s.panel_visible);
        assert_eq!(s.active_panel, Panel::Memory);

        update(&mut s, Message::ToggleMemory);
        assert!(!s.panel_visible);
    }

    #[test]
    fn mode_panel_toggle() {
        let mut s = test_state();
        assert!(!s.mode_panel_visible);

        update(&mut s, Message::OpenConverter);
        assert!(s.mode_panel_visible);
        assert_eq!(s.active_mode, Some(ModePanel::Converter));

        update(&mut s, Message::OpenConverter);
        assert!(!s.mode_panel_visible);
        assert_eq!(s.active_mode, None);
    }

    #[test]
    fn close_mode_priority() {
        let mut s = test_state();

        update(&mut s, Message::OpenConverter);
        let effects = update(&mut s, Message::CloseMode);
        assert!(!s.mode_panel_visible);
        assert!(effects.contains(&SideEffect::ToggleModePanel));

        update(&mut s, Message::ToggleHistory);
        let effects = update(&mut s, Message::CloseMode);
        assert!(!s.panel_visible);
        assert!(effects.contains(&SideEffect::TogglePanel));

        let effects = update(&mut s, Message::CloseMode);
        assert!(effects.contains(&SideEffect::UpdateDisplay));
    }

    #[test]
    fn history_search() {
        let mut s = test_state();
        update(&mut s, Message::SearchHistory("test".into()));
        assert_eq!(s.history_search, "test");
    }

    #[test]
    fn pin_result() {
        let mut s = test_state();
        update(&mut s, Message::Digit('5'));
        update(&mut s, Message::Equals);
        update(&mut s, Message::PinResult);
        assert_eq!(s.engine().pinned.len(), 1);
    }

    #[test]
    fn memory_store_and_recall() {
        let mut s = test_state();
        update(&mut s, Message::Digit('4'));
        update(&mut s, Message::Digit('2'));
        update(&mut s, Message::Equals);
        update(&mut s, Message::MemoryStore);
        assert_eq!(s.engine().memory_slots.len(), 1);
        assert_eq!(s.engine().memory_slots[0].value, 42.0);
    }

    #[test]
    fn undo() {
        let mut s = test_state();
        update(&mut s, Message::Digit('5'));
        update(&mut s, Message::Digit('3'));
        update(&mut s, Message::Undo);
        assert_eq!(s.engine().main_display_text(), "5");
    }

    #[test]
    fn quit_returns_quit_effect() {
        let mut s = test_state();
        let effects = update(&mut s, Message::Quit);
        assert!(effects.contains(&SideEffect::Quit));
    }
}
