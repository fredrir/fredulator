/// Press 't' to cycle. Native mode respects your system GTK theme,
/// which also works with Qt-bridged themes (kvantum, adwaita-qt).

use gtk::gdk;
use gtk::prelude::*;
use gtk::{CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION};

const NATIVE_CSS: &str = r#"
.display-area { padding: 12px 16px; min-height: 100px; }
.expression-label { font-size: 16px; padding: 4px 4px; }
.result-label { font-size: 48px; padding: 8px 4px; font-weight: 300; }
.result-label.result-medium { font-size: 36px; }
.result-label.result-small { font-size: 24px; }
.calc-grid { margin: 4px 8px 8px 8px; }
.sci-grid { margin: 4px 0 8px 8px; }
button { font-size: 18px; padding: 10px; min-height: 48px; border-radius: 12px; }
.op-button { font-size: 22px; font-weight: bold; }
.equals-button { font-size: 22px; font-weight: bold; }
.memory-button { font-size: 13px; min-height: 32px; }
.function-button { font-size: 14px; }
.sci-toggle { font-size: 12px; padding: 2px 8px; min-height: 24px; border: none; }
"#;

const DARK_CSS: &str = r#"
.main-window { background-color: #000000; }

.display-area {
    padding: 12px 16px;
    min-height: 100px;
}
.expression-label {
    font-size: 16px;
    color: #8e8e93;
    padding: 4px 4px;
}
.result-label {
    font-size: 48px;
    color: #ffffff;
    font-weight: 300;
    padding: 8px 4px;
}
.result-label.result-medium { font-size: 36px; }
.result-label.result-small { font-size: 24px; }
.sci-toggle {
    background-color: transparent;
    color: #636366;
    font-size: 12px;
    border: none;
    padding: 2px 10px;
    min-height: 24px;
}
.sci-toggle:hover { color: #ffffff; }

.calc-grid { margin: 4px 8px 8px 8px; }
.sci-grid { margin: 4px 0 8px 8px; }

button {
    font-size: 18px;
    border-radius: 100px;
    border: none;
    padding: 10px;
    min-height: 48px;
}
button:focus { box-shadow: inset 0 0 0 2px #0a84ff; }

.digit-button {
    background-color: #333333;
    color: #ffffff;
    font-size: 22px;
}
.digit-button:hover { background-color: #444444; }
.digit-button:active { background-color: #555555; }

.op-button {
    background-color: #ff9500;
    color: #ffffff;
    font-size: 26px;
    font-weight: bold;
}
.op-button:hover { background-color: #ffaa33; }
.op-button:active { background-color: #cc7700; }

.equals-button {
    background-color: #ff9500;
    color: #ffffff;
    font-size: 26px;
    font-weight: bold;
}
.equals-button:hover { background-color: #ffaa33; }
.equals-button:active { background-color: #cc7700; }

/* Top row: AC */
.clear-button {
    background-color: #a5a5a5;
    color: #000000;
    font-size: 18px;
}
.clear-button:hover { background-color: #b8b8b8; }
.clear-button:active { background-color: #8e8e8e; }

/* Top row: +/- %*/
.util-button {
    background-color: #a5a5a5;
    color: #000000;
    font-size: 18px;
}
.util-button:hover { background-color: #b8b8b8; }
.util-button:active { background-color: #8e8e8e; }

.memory-button {
    background-color: #1c1c1e;
    color: #8e8e93;
    font-size: 13px;
    min-height: 32px;
    border-radius: 8px;
}
.memory-button:hover { background-color: #2c2c2e; color: #ffffff; }

/* Scientific functions*/
.function-button {
    background-color: #1c1c1e;
    color: #ebebf5;
    font-size: 14px;
    border-radius: 100px;
}
.function-button:hover { background-color: #2c2c2e; }
.function-button:active { background-color: #3a3a3c; }
"#;

const MACOS_CSS: &str = r#"
.main-window { background-color: #1c1c1e; }

.display-area { padding: 12px 16px; min-height: 100px; }
.expression-label { font-size: 16px; color: #98989d; padding: 4px 4px; }
.result-label { font-size: 48px; color: #ffffff; font-weight: 200; padding: 8px 4px; }
.result-label.result-medium { font-size: 36px; }
.result-label.result-small { font-size: 24px; }
.sci-toggle {
    background-color: transparent; color: #636366;
    font-size: 12px; border: none; padding: 2px 10px; min-height: 24px;
}
.sci-toggle:hover { color: #ffffff; }

.calc-grid { margin: 4px 8px 8px 8px; }
.sci-grid { margin: 4px 0 8px 8px; }

button { font-size: 18px; border-radius: 100px; border: none; padding: 10px; min-height: 48px; }
button:focus { box-shadow: inset 0 0 0 2px #0a84ff; }

.digit-button { background-color: #505050; color: #ffffff; font-size: 22px; }
.digit-button:hover { background-color: #616161; }

.op-button { background-color: #ff9f0a; color: #ffffff; font-size: 26px; font-weight: bold; }
.op-button:hover { background-color: #ffb340; }

.equals-button { background-color: #ff9f0a; color: #ffffff; font-size: 26px; font-weight: bold; }
.equals-button:hover { background-color: #ffb340; }

.clear-button { background-color: #a5a5a5; color: #000000; font-size: 18px; }
.clear-button:hover { background-color: #b8b8b8; }

.util-button { background-color: #a5a5a5; color: #000000; font-size: 18px; }
.util-button:hover { background-color: #b8b8b8; }

.memory-button {
    background-color: #2c2c2e; color: #98989d; font-size: 13px;
    min-height: 32px; border-radius: 8px;
}
.memory-button:hover { background-color: #3a3a3c; color: #ffffff; }

.function-button { background-color: #2c2c2e; color: #ebebf5; font-size: 14px; }
.function-button:hover { background-color: #3a3a3c; }
"#;

const WINDOWS_CSS: &str = r#"
.main-window { background-color: #202020; }

.display-area { padding: 12px 16px; min-height: 100px; }
.expression-label { font-size: 14px; color: #999999; padding: 4px 4px; }
.result-label { font-size: 48px; color: #ffffff; font-weight: 300; padding: 8px 4px; }
.result-label.result-medium { font-size: 36px; }
.result-label.result-small { font-size: 24px; }
.sci-toggle {
    background-color: transparent; color: #666666;
    font-size: 12px; border: none; padding: 2px 10px; min-height: 24px;
}
.sci-toggle:hover { color: #ffffff; }

.calc-grid { margin: 2px 8px 8px 8px; }
.sci-grid { margin: 2px 0 8px 8px; }

button { font-size: 16px; border-radius: 4px; border: none; padding: 10px; min-height: 48px; }
button:focus { box-shadow: inset 0 0 0 2px #4cc2ff; }

.digit-button { background-color: #3b3b3b; color: #ffffff; font-size: 20px; }
.digit-button:hover { background-color: #4a4a4a; }

.op-button { background-color: #323232; color: #ffffff; font-size: 22px; }
.op-button:hover { background-color: #444444; }

.equals-button {
    background-color: #4cc2ff; color: #000000; font-size: 22px; font-weight: bold;
}
.equals-button:hover { background-color: #60ccff; }

.clear-button { background-color: #323232; color: #ffffff; font-size: 16px; }
.clear-button:hover { background-color: #444444; }

.util-button { background-color: #323232; color: #ffffff; font-size: 16px; }
.util-button:hover { background-color: #444444; }

.memory-button {
    background-color: #202020; color: #888888; font-size: 13px;
    min-height: 32px; border-radius: 4px; border: none;
}
.memory-button:hover { background-color: #333333; color: #ffffff; }

.function-button {
    background-color: #2a2a2a; color: #cccccc; font-size: 14px; border-radius: 4px;
}
.function-button:hover { background-color: #3a3a3a; }
"#;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Native,
    Dark,
    MacOS,
    Windows,
}

impl Theme {
    pub fn next(self) -> Self {
        match self {
            Self::Native => Self::Dark,
            Self::Dark => Self::MacOS,
            Self::MacOS => Self::Windows,
            Self::Windows => Self::Native,
        }
    }
}

pub struct ThemeManager {
    provider: CssProvider,
    current: Theme,
    screen: gdk::Screen,
}

impl ThemeManager {
    pub fn new(screen: gdk::Screen) -> Self {
        let provider = CssProvider::new();
        let mut m = Self {
            provider,
            current: Theme::Native,
            screen,
        };
        m.apply();
        m
    }

    pub fn toggle(&mut self) {
        self.current = self.current.next();
        self.apply();
    }

    fn apply(&mut self) {
        StyleContext::remove_provider_for_screen(&self.screen, &self.provider);
        self.provider = CssProvider::new();
        let css = match self.current {
            Theme::Native => NATIVE_CSS,
            Theme::Dark => DARK_CSS,
            Theme::MacOS => MACOS_CSS,
            Theme::Windows => WINDOWS_CSS,
        };
        self.provider.load_from_data(css.as_bytes()).ok();
        StyleContext::add_provider_for_screen(
            &self.screen,
            &self.provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
