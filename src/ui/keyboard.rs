use gtk::gdk;
use gtk::gdk::keys::constants as key;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::app::message::Message;
use crate::domain::types::*;
use crate::services::config::KeybindingsConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

static KEYMAP: OnceLock<HashMap<String, Message>> = OnceLock::new();

pub fn init_keymap(config: &KeybindingsConfig) {
    let _ = KEYMAP.set(build_keymap(config));
}

pub fn map_key(event: &gdk::EventKey) -> Message {
    let combo = event_to_combo(event);
    if combo.is_empty() {
        return Message::Noop;
    }
    KEYMAP
        .get()
        .and_then(|m| m.get(&combo).cloned())
        .unwrap_or(Message::Noop)
}

fn build_keymap(config: &KeybindingsConfig) -> HashMap<String, Message> {
    let scheme = match config.scheme.as_str() {
        "emacs" => emacs_scheme(),
        _ => default_scheme(),
    };

    let mut map = HashMap::new();
    for (combo, action_name) in &scheme {
        if let Some(msg) = parse_action(action_name) {
            map.insert(combo.clone(), msg);
        }
    }
    for (combo, action_name) in &config.custom {
        if action_name == "none" || action_name == "unbound" {
            map.remove(combo);
        } else if let Some(msg) = parse_action(action_name) {
            map.insert(combo.clone(), msg);
        }
    }
    map
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
    if keyval == key::Escape { return build_combo(ctrl, alt, shift, "Escape"); }
    if keyval == key::BackSpace { return build_combo(ctrl, alt, shift, "BackSpace"); }
    if keyval == key::Delete { return build_combo(ctrl, alt, shift, "Delete"); }
    if keyval == key::ISO_Left_Tab { return build_combo(ctrl, alt, true, "Tab"); }
    if keyval == key::Tab { return build_combo(ctrl, alt, shift, "Tab"); }
    if keyval == key::Left { return build_combo(ctrl, alt, shift, "Left"); }
    if keyval == key::Right { return build_combo(ctrl, alt, shift, "Right"); }
    if keyval == key::Up { return build_combo(ctrl, alt, shift, "Up"); }
    if keyval == key::Down { return build_combo(ctrl, alt, shift, "Down"); }
    if keyval == key::space { return build_combo(ctrl, alt, shift, "space"); }
    if keyval == key::F1 { return build_combo(ctrl, alt, shift, "F1"); }
    if keyval == key::KP_Add { return build_combo(ctrl, alt, false, "+"); }
    if keyval == key::KP_Subtract { return build_combo(ctrl, alt, false, "-"); }
    if keyval == key::KP_Multiply { return build_combo(ctrl, alt, false, "*"); }
    if keyval == key::KP_Divide { return build_combo(ctrl, alt, false, "/"); }
    if keyval == key::KP_Decimal { return build_combo(ctrl, alt, false, "."); }

    if let Some(ch) = keyval.to_unicode() {
        return build_combo(ctrl, alt, false, &ch.to_string());
    }
    String::new()
}

fn build_combo(ctrl: bool, alt: bool, shift: bool, key_name: &str) -> String {
    let mut s = String::new();
    if ctrl { s.push_str("Ctrl+"); }
    if alt { s.push_str("Alt+"); }
    if shift { s.push_str("Shift+"); }
    s.push_str(key_name);
    s
}

fn parse_action(name: &str) -> Option<Message> {
    match name {
        "digit_0" => Some(Message::Digit('0')),
        "digit_1" => Some(Message::Digit('1')),
        "digit_2" => Some(Message::Digit('2')),
        "digit_3" => Some(Message::Digit('3')),
        "digit_4" => Some(Message::Digit('4')),
        "digit_5" => Some(Message::Digit('5')),
        "digit_6" => Some(Message::Digit('6')),
        "digit_7" => Some(Message::Digit('7')),
        "digit_8" => Some(Message::Digit('8')),
        "digit_9" => Some(Message::Digit('9')),
        "decimal" => Some(Message::Decimal),
        "add" => Some(Message::BinaryOp(BinaryOp::Add)),
        "subtract" => Some(Message::BinaryOp(BinaryOp::Subtract)),
        "multiply" => Some(Message::BinaryOp(BinaryOp::Multiply)),
        "divide" => Some(Message::BinaryOp(BinaryOp::Divide)),
        "power" => Some(Message::BinaryOp(BinaryOp::Power)),
        "percent" => Some(Message::PostfixOp(PostfixOp::Percent)),
        "factorial" => Some(Message::PostfixOp(PostfixOp::Factorial)),
        "equals" => Some(Message::Equals),
        "clear" => Some(Message::Clear),
        "backspace" => Some(Message::Backspace),
        "toggle_sign" => Some(Message::ToggleSign),
        "left_paren" => Some(Message::LeftParen),
        "right_paren" => Some(Message::RightParen),
        "navigate_left" => Some(Message::Navigate(Direction::Left)),
        "navigate_right" => Some(Message::Navigate(Direction::Right)),
        "navigate_up" => Some(Message::Navigate(Direction::Up)),
        "navigate_down" => Some(Message::Navigate(Direction::Down)),
        "activate" => Some(Message::Activate),
        "toggle_theme" => Some(Message::ToggleTheme),
        "toggle_scientific" => Some(Message::ToggleScientific),
        "quit" => Some(Message::Quit),
        "undo" => Some(Message::Undo),
        "new_tab" => Some(Message::NewTab),
        "close_tab" => Some(Message::CloseTab),
        "next_tab" => Some(Message::NextTab),
        "prev_tab" => Some(Message::PrevTab),
        "toggle_history" => Some(Message::ToggleHistory),
        "toggle_memory" => Some(Message::ToggleMemory),
        "toggle_pinned" => Some(Message::TogglePinned),
        "pin_result" => Some(Message::PinResult),
        "memory_store" => Some(Message::MemoryStore),
        "open_converter" => Some(Message::OpenConverter),
        "open_tools" => Some(Message::OpenTools),
        "open_notes" => Some(Message::OpenNotes),
        "open_menu" => Some(Message::OpenMenu),
        "back_to_calc" => Some(Message::CloseMode),
        "export_history" => Some(Message::ExportHistoryJson),
        "show_help" => Some(Message::ShowHelp),
        _ => None,
    }
}

fn default_scheme() -> HashMap<String, String> {
    let mut m = HashMap::new();
    for d in '0'..='9' { m.insert(d.to_string(), format!("digit_{}", d)); }
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
    m.insert("?".into(), "show_help".into());
    m.insert("F1".into(), "show_help".into());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scheme_has_digits() {
        let scheme = default_scheme();
        for d in '0'..='9' {
            assert!(scheme.contains_key(&d.to_string()));
        }
    }

    #[test]
    fn default_scheme_has_vim_nav() {
        let scheme = default_scheme();
        assert_eq!(scheme.get("h").unwrap(), "navigate_left");
        assert_eq!(scheme.get("j").unwrap(), "navigate_down");
        assert_eq!(scheme.get("k").unwrap(), "navigate_up");
        assert_eq!(scheme.get("l").unwrap(), "navigate_right");
    }

    #[test]
    fn emacs_scheme_removes_vim_nav() {
        let scheme = emacs_scheme();
        assert!(!scheme.contains_key("h"));
        assert!(!scheme.contains_key("j"));
        assert!(!scheme.contains_key("k"));
        assert!(!scheme.contains_key("l"));
        assert_eq!(scheme.get("Ctrl+f").unwrap(), "navigate_right");
    }

    #[test]
    fn custom_overrides() {
        let config = KeybindingsConfig {
            scheme: "default".into(),
            custom: {
                let mut m = HashMap::new();
                m.insert("x".into(), "multiply".into());
                m.insert("h".into(), "unbound".into());
                m
            },
        };
        let map = build_keymap(&config);
        assert!(matches!(map.get("x"), Some(Message::BinaryOp(BinaryOp::Multiply))));
        assert!(!map.contains_key("h"));
    }

    #[test]
    fn parse_all_actions() {
        let actions = [
            "digit_0", "decimal", "add", "subtract", "multiply", "divide",
            "power", "percent", "factorial", "equals", "clear", "backspace",
            "toggle_sign", "left_paren", "right_paren", "navigate_left",
            "activate", "toggle_theme", "toggle_scientific", "quit", "undo",
            "new_tab", "close_tab", "next_tab", "prev_tab", "toggle_history",
            "toggle_memory", "toggle_pinned", "pin_result", "memory_store",
            "open_converter", "open_tools", "open_notes", "open_menu",
            "back_to_calc", "export_history", "show_help",
        ];
        for a in actions {
            assert!(parse_action(a).is_some(), "Failed to parse: {}", a);
        }
    }

    #[test]
    fn unknown_action_returns_none() {
        assert!(parse_action("nonexistent").is_none());
    }

    #[test]
    fn build_combo_formatting() {
        assert_eq!(build_combo(true, false, false, "t"), "Ctrl+t");
        assert_eq!(build_combo(true, true, false, "x"), "Ctrl+Alt+x");
        assert_eq!(build_combo(false, false, true, "Tab"), "Shift+Tab");
        assert_eq!(build_combo(false, false, false, "5"), "5");
    }
}
