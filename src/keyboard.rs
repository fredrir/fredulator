use gtk::gdk;
use gtk::gdk::keys::constants as key;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::eval::{BinaryOp, PostfixOp};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Digit(char),
    Decimal,
    BinaryOp(BinaryOp),
    PostfixOp(PostfixOp),
    Equals,
    Clear,
    Backspace,
    ToggleSign,
    LeftParen,
    RightParen,
    Navigate(Direction),
    Activate,
    ToggleTheme,
    ToggleScientific,
    Quit,
    Undo,
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    ToggleHistory,
    ToggleMemory,
    TogglePinned,
    PinResult,
    OpenConverter,
    OpenTools,
    OpenNotes,
    OpenMenu,
    MemoryStore,
    BackToCalc,
    ExportHistory,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

static KEYMAP: OnceLock<HashMap<String, Action>> = OnceLock::new();

fn get_keymap() -> &'static HashMap<String, Action> {
    KEYMAP.get_or_init(|| {
        let cfg = crate::config::get();
        let mut map = HashMap::new();

        let scheme = match cfg.keybindings.scheme.as_str() {
            "emacs" => emacs_scheme(),
            _ => default_scheme(),
        };

        for (combo, action_name) in &scheme {
            if let Some(action) = parse_action(action_name) {
                map.insert(combo.clone(), action);
            }
        }

        for (combo, action_name) in &cfg.keybindings.custom {
            if action_name == "none" || action_name == "unbound" {
                map.remove(combo);
            } else if let Some(action) = parse_action(action_name) {
                map.insert(combo.clone(), action);
            }
        }

        map
    })
}

pub fn map_key(event: &gdk::EventKey) -> Action {
    let combo = event_to_combo(event);
    if combo.is_empty() {
        return Action::None;
    }
    get_keymap().get(&combo).copied().unwrap_or(Action::None)
}

fn event_to_combo(event: &gdk::EventKey) -> String {
    let keyval = event.keyval();
    let state = event.state();
    let ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
    let alt = state.contains(gdk::ModifierType::MOD1_MASK);
    let shift = state.contains(gdk::ModifierType::SHIFT_MASK);

    if keyval == key::Return || keyval == key::KP_Enter {
        return build_combo(ctrl, alt, shift, "Return");
    }
    if keyval == key::Escape {
        return build_combo(ctrl, alt, shift, "Escape");
    }
    if keyval == key::BackSpace {
        return build_combo(ctrl, alt, shift, "BackSpace");
    }
    if keyval == key::Delete {
        return build_combo(ctrl, alt, shift, "Delete");
    }
    if keyval == key::ISO_Left_Tab {
        return build_combo(ctrl, alt, true, "Tab");
    }
    if keyval == key::Tab {
        return build_combo(ctrl, alt, shift, "Tab");
    }
    if keyval == key::Left {
        return build_combo(ctrl, alt, shift, "Left");
    }
    if keyval == key::Right {
        return build_combo(ctrl, alt, shift, "Right");
    }
    if keyval == key::Up {
        return build_combo(ctrl, alt, shift, "Up");
    }
    if keyval == key::Down {
        return build_combo(ctrl, alt, shift, "Down");
    }
    if keyval == key::space {
        return build_combo(ctrl, alt, shift, "space");
    }
    if keyval == key::KP_Add {
        return build_combo(ctrl, alt, false, "+");
    }
    if keyval == key::KP_Subtract {
        return build_combo(ctrl, alt, false, "-");
    }
    if keyval == key::KP_Multiply {
        return build_combo(ctrl, alt, false, "*");
    }
    if keyval == key::KP_Divide {
        return build_combo(ctrl, alt, false, "/");
    }
    if keyval == key::KP_Decimal {
        return build_combo(ctrl, alt, false, ".");
    }

    if let Some(ch) = keyval.to_unicode() {
        return build_combo(ctrl, alt, false, &ch.to_string());
    }

    String::new()
}

fn build_combo(ctrl: bool, alt: bool, shift: bool, key_name: &str) -> String {
    let mut s = String::new();
    if ctrl {
        s.push_str("Ctrl+");
    }
    if alt {
        s.push_str("Alt+");
    }
    if shift {
        s.push_str("Shift+");
    }
    s.push_str(key_name);
    s
}

fn parse_action(name: &str) -> Option<Action> {
    match name {
        "digit_0" => Some(Action::Digit('0')),
        "digit_1" => Some(Action::Digit('1')),
        "digit_2" => Some(Action::Digit('2')),
        "digit_3" => Some(Action::Digit('3')),
        "digit_4" => Some(Action::Digit('4')),
        "digit_5" => Some(Action::Digit('5')),
        "digit_6" => Some(Action::Digit('6')),
        "digit_7" => Some(Action::Digit('7')),
        "digit_8" => Some(Action::Digit('8')),
        "digit_9" => Some(Action::Digit('9')),
        "decimal" => Some(Action::Decimal),
        "add" => Some(Action::BinaryOp(BinaryOp::Add)),
        "subtract" => Some(Action::BinaryOp(BinaryOp::Subtract)),
        "multiply" => Some(Action::BinaryOp(BinaryOp::Multiply)),
        "divide" => Some(Action::BinaryOp(BinaryOp::Divide)),
        "power" => Some(Action::BinaryOp(BinaryOp::Power)),
        "percent" => Some(Action::PostfixOp(PostfixOp::Percent)),
        "factorial" => Some(Action::PostfixOp(PostfixOp::Factorial)),
        "equals" => Some(Action::Equals),
        "clear" => Some(Action::Clear),
        "backspace" => Some(Action::Backspace),
        "toggle_sign" => Some(Action::ToggleSign),
        "left_paren" => Some(Action::LeftParen),
        "right_paren" => Some(Action::RightParen),
        "navigate_left" => Some(Action::Navigate(Direction::Left)),
        "navigate_right" => Some(Action::Navigate(Direction::Right)),
        "navigate_up" => Some(Action::Navigate(Direction::Up)),
        "navigate_down" => Some(Action::Navigate(Direction::Down)),
        "activate" => Some(Action::Activate),
        "toggle_theme" => Some(Action::ToggleTheme),
        "toggle_scientific" => Some(Action::ToggleScientific),
        "quit" => Some(Action::Quit),
        "undo" => Some(Action::Undo),
        "new_tab" => Some(Action::NewTab),
        "close_tab" => Some(Action::CloseTab),
        "next_tab" => Some(Action::NextTab),
        "prev_tab" => Some(Action::PrevTab),
        "toggle_history" => Some(Action::ToggleHistory),
        "toggle_memory" => Some(Action::ToggleMemory),
        "toggle_pinned" => Some(Action::TogglePinned),
        "pin_result" => Some(Action::PinResult),
        "memory_store" => Some(Action::MemoryStore),
        "open_converter" => Some(Action::OpenConverter),
        "open_tools" => Some(Action::OpenTools),
        "open_notes" => Some(Action::OpenNotes),
        "open_menu" => Some(Action::OpenMenu),
        "back_to_calc" => Some(Action::BackToCalc),
        "export_history" => Some(Action::ExportHistory),
        _ => None,
    }
}

fn default_scheme() -> HashMap<String, String> {
    let mut m = HashMap::new();

    for d in '0'..='9' {
        m.insert(d.to_string(), format!("digit_{}", d));
    }

    m.insert("+".into(), "add".into());
    m.insert("-".into(), "subtract".into());
    m.insert("*".into(), "multiply".into());
    m.insert("/".into(), "divide".into());
    m.insert("^".into(), "power".into());
    m.insert("%".into(), "percent".into());
    m.insert("!".into(), "factorial".into());
    m.insert(".".into(), "decimal".into());
    m.insert("=".into(), "equals".into());
    m.insert("Return".into(), "equals".into());
    m.insert("Escape".into(), "back_to_calc".into());
    m.insert("BackSpace".into(), "backspace".into());
    m.insert("Delete".into(), "backspace".into());
    m.insert("Ctrl+BackSpace".into(), "clear".into());
    m.insert("Ctrl+Delete".into(), "clear".into());
    m.insert("(".into(), "left_paren".into());
    m.insert(")".into(), "right_paren".into());
    m.insert("n".into(), "toggle_sign".into());

    m.insert("h".into(), "navigate_left".into());
    m.insert("j".into(), "navigate_down".into());
    m.insert("k".into(), "navigate_up".into());
    m.insert("l".into(), "navigate_right".into());
    m.insert("Left".into(), "navigate_left".into());
    m.insert("Right".into(), "navigate_right".into());
    m.insert("Up".into(), "navigate_up".into());
    m.insert("Down".into(), "navigate_down".into());

    m.insert("space".into(), "activate".into());
    m.insert("t".into(), "toggle_theme".into());
    m.insert("s".into(), "toggle_scientific".into());
    m.insert("q".into(), "quit".into());
    m.insert("u".into(), "undo".into());
    m.insert(";".into(), "open_menu".into());
    m.insert("S".into(), "memory_store".into());

    m.insert("Ctrl+z".into(), "undo".into());
    m.insert("Ctrl+t".into(), "new_tab".into());
    m.insert("Ctrl+w".into(), "close_tab".into());
    m.insert("Ctrl+s".into(), "pin_result".into());
    m.insert("Ctrl+e".into(), "open_converter".into());
    m.insert("Ctrl+r".into(), "open_tools".into());
    m.insert("Ctrl+n".into(), "open_notes".into());
    m.insert("Ctrl+m".into(), "toggle_memory".into());
    m.insert("Ctrl+p".into(), "toggle_pinned".into());
    m.insert("Ctrl+h".into(), "toggle_history".into());

    m.insert("Tab".into(), "next_tab".into());
    m.insert("Shift+Tab".into(), "prev_tab".into());

    m.insert("Ctrl+Shift+e".into(), "export_history".into());

    m
}

fn emacs_scheme() -> HashMap<String, String> {
    let mut m = default_scheme();

    m.remove("h");
    m.remove("j");
    m.remove("k");
    m.remove("l");

    m.insert("Ctrl+f".into(), "navigate_right".into());
    m.insert("Ctrl+b".into(), "navigate_left".into());
    m.insert("Ctrl+p".into(), "navigate_up".into());
    m.insert("Ctrl+n".into(), "navigate_down".into());

    m.insert("Ctrl+g".into(), "clear".into());
    m.insert("Ctrl+/".into(), "undo".into());

    m.insert("Alt+p".into(), "toggle_pinned".into());
    m.insert("Alt+n".into(), "open_notes".into());

    m
}
