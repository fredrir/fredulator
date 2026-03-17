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
///   Escape       clear (AC) / back to calc mode
///   Backspace    delete last entry
///   %            percent
///   !            factorial
///   n            negate (+/-)
///   h/j/k/l      vim navigation (also arrow keys)
///   Space        activate focused button
///   t            toggle theme
///   s            toggle scientific mode
///   q            quit
///   u            undo
///   ;            open menu
///   Tab          next tab
///   Shift+Tab    previous tab
///   Ctrl+t       new tab
///   Ctrl+w       close tab
///   Ctrl+z       undo
///   Ctrl+s       pin current result
///   Ctrl+h       toggle history panel
///   Ctrl+m       toggle memory panel
///   Ctrl+p       toggle pinned panel
///   Ctrl+e       open converter
///   Ctrl+r       open percentage tools
///   Ctrl+n       open math notes
pub fn map_key(event: &gdk::EventKey) -> Action {
    let keyval = event.keyval();
    let state = event.state();
    let ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
    let shift = state.contains(gdk::ModifierType::SHIFT_MASK);

    if keyval == key::Return || keyval == key::KP_Enter {
        return Action::Equals;
    }
    if keyval == key::Escape {
        return Action::BackToCalc;
    }
    if keyval == key::BackSpace || keyval == key::Delete {
        if ctrl {
            return Action::Clear;
        }
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
    if keyval == key::Tab {
        if shift {
            return Action::PrevTab;
        }
        return Action::NextTab;
    }
    if keyval == key::ISO_Left_Tab {
        return Action::PrevTab;
    }

    if let Some(ch) = keyval.to_unicode() {
        if ctrl {
            return match ch {
                'z' => Action::Undo,
                't' => Action::NewTab,
                'w' => Action::CloseTab,
                's' => Action::PinResult,
                'e' => Action::OpenConverter,
                'r' => Action::OpenTools,
                'n' => Action::OpenNotes,
                'm' => Action::ToggleMemory,
                'p' => Action::TogglePinned,
                'h' => Action::ToggleHistory,
                _ => Action::None,
            };
        }

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
            'u' => Action::Undo,
            ' ' => Action::Activate,
            ';' => Action::OpenMenu,
            'S' => Action::MemoryStore,
            _ => Action::None,
        };
    }

    Action::None
}
