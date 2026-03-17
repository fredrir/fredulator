use gtk::gdk;
use gtk::gdk::keys::constants as key;

use crate::engine::Operation;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Digit(char),
    Decimal,
    SetOperation(Operation),
    Equals,
    Clear,
    Backspace,
    ToggleSign,
    Percent,
    Navigate(Direction),
    Activate,
    ToggleTheme,
    Quit,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

/// Keyboard layout:
///   0-9          digits
///   + - * /      operations (also numpad variants)
///   .            decimal point
///   = / Enter    equals
///   Escape       clear (AC)
///   Backspace    delete last digit
///   %            percent
///   n            negate (+/-)
///   h/j/k/l      vim navigation (also arrow keys)
///   Space        activate focused button
///   t            toggle theme (native / dark)
///   q            quit
pub fn map_key(event: &gdk::EventKey) -> Action {
    let keyval = event.keyval();

    // Special keys (no unicode representation)
    if keyval == key::Return || keyval == key::KP_Enter {
        return Action::Equals;
    }
    if keyval == key::Escape {
        return Action::Clear;
    }
    if keyval == key::BackSpace || keyval == key::Delete {
        return Action::Backspace;
    }
    if keyval == key::Left {
        return Action::Navigate(Direction::Left);
    }
    if keyval == key::Right {
        return Action::Navigate(Direction::Right);
    }
    if keyval == key::Up {
        return Action::Navigate(Direction::Up);
    }
    if keyval == key::Down {
        return Action::Navigate(Direction::Down);
    }

    if let Some(ch) = keyval.to_unicode() {
        return match ch {
            '0'..='9' => Action::Digit(ch),
            '+' => Action::SetOperation(Operation::Add),
            '-' => Action::SetOperation(Operation::Subtract),
            '*' => Action::SetOperation(Operation::Multiply),
            '/' => Action::SetOperation(Operation::Divide),
            '.' => Action::Decimal,
            '=' => Action::Equals,
            '%' => Action::Percent,
            'h' => Action::Navigate(Direction::Left),
            'j' => Action::Navigate(Direction::Down),
            'k' => Action::Navigate(Direction::Up),
            'l' => Action::Navigate(Direction::Right),
            'n' => Action::ToggleSign,
            't' => Action::ToggleTheme,
            'q' => Action::Quit,
            ' ' => Action::Activate,
            _ => Action::None,
        };
    }

    Action::None
}
