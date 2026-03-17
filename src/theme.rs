/// Defaults to native Linux theme. Press 't' to toggle the custom dark theme.

use gtk::gdk;
use gtk::prelude::*;
use gtk::{CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION};

const DARK_THEME_CSS: &str = r#"
.main-window {
    background-color: #2b2b2b;
}

.calc-grid {
    margin: 10px;
}

.display-entry {
    font-size: 42px;
    background-color: #151414;
    color: #ffffff;
    border: 1px solid #555;
    padding: 16px;
    margin: 8px;
    border-radius: 8px;
}

button {
    font-size: 16px;
    border-radius: 8px;
    border: none;
    padding: 10px;
    min-height: 40px;
}

.digit-button {
    font-weight: bold;
    background-color: #505050;
    color: #ffffff;
}

.digit-button:hover {
    background-color: #666666;
}

.op-button {
    font-weight: bold;
    background-color: #6e6e6e;
    color: #ffffff;
}

.op-button:hover {
    background-color: #888888;
}

.op-button.active-op {
    background-color: #f39c12;
    color: #333333;
}

.equals-button {
    font-weight: bold;
    background-color: #f39c12;
    color: #ffffff;
}

.equals-button:hover {
    background-color: #f5b041;
}

.clear-button {
    font-weight: bold;
    background-color: #c0392b;
    color: #ffffff;
}

.clear-button:hover {
    background-color: #e74c3c;
}

button:focus {
    box-shadow: inset 0 0 0 2px #4a90d9;
}
"#;

const NATIVE_THEME_CSS: &str = r#"
.calc-grid {
    margin: 10px;
}

.display-entry {
    font-size: 42px;
    padding: 16px;
    margin: 8px;
}

button {
    font-size: 16px;
    padding: 10px;
    min-height: 40px;
}

.op-button.active-op {
    box-shadow: inset 0 0 0 2px currentColor;
}
"#;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Native,
    Dark,
}

pub struct ThemeManager {
    provider: CssProvider,
    current: Theme,
    screen: gdk::Screen,
}

impl ThemeManager {
    pub fn new(screen: gdk::Screen) -> Self {
        let provider = CssProvider::new();
        let mut manager = Self {
            provider,
            current: Theme::Native,
            screen,
        };
        manager.apply();
        manager
    }

    pub fn toggle(&mut self) {
        self.current = match self.current {
            Theme::Native => Theme::Dark,
            Theme::Dark => Theme::Native,
        };
        self.apply();
    }

    fn apply(&mut self) {
        StyleContext::remove_provider_for_screen(&self.screen, &self.provider);

        self.provider = CssProvider::new();
        let css = match self.current {
            Theme::Native => NATIVE_THEME_CSS,
            Theme::Dark => DARK_THEME_CSS,
        };
        self.provider.load_from_data(css.as_bytes()).ok();

        StyleContext::add_provider_for_screen(
            &self.screen,
            &self.provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
