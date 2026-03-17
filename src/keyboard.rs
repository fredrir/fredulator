use gtk::gdk;
use gtk::gdk::keys::constants as key;

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
///   0-9          digits (also numpad)
///   + - * /      operations
///   ^            power
///   ( )          parentheses
///   .            decimal point
///   = / Enter    equals
///   Escape       clear (AC)
///   Backspace    delete last entry
///   %            percent
///   !            factorial
///   n            negate (+/-)
///   h/j/k/l      vim navigation (also arrow keys)
///   Space        activate focused button
///   t            toggle theme
///   s            toggle scientific mode
///   q            quit
pub fn map_key(event: &gdk::EventKey) -> Action {
    let keyval = event.keyval();

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
            '+' => Action::BinaryOp(BinaryOp::Add),
            '-' => Action::BinaryOp(BinaryOp::Subtract),
            '*' => Action::BinaryOp(BinaryOp::Multiply),
            '/' => Action::BinaryOp(BinaryOp::Divide),
            '^' => Action::BinaryOp(BinaryOp::Power),
            '(' => Action::LeftParen,
            ')' => Action::RightParen,
            '.' => Action::Decimal,
            '=' => Action::Equals,
            '%' => Action::PostfixOp(PostfixOp::Percent),
            '!' => Action::PostfixOp(PostfixOp::Factorial),
            'h' => Action::Navigate(Direction::Left),
            'j' => Action::Navigate(Direction::Down),
            'k' => Action::Navigate(Direction::Up),
            'l' => Action::Navigate(Direction::Right),
            'n' => Action::ToggleSign,
            't' => Action::ToggleTheme,
            's' => Action::ToggleScientific,
            'q' => Action::Quit,
            ' ' => Action::Activate,
            _ => Action::None,
        };
    }

    Action::None
}
