/// Press 't' to cycle themes.
/// Native mode respects your system GTK theme.

use gtk::gdk;
use gtk::prelude::*;
use gtk::{CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION};

// Shared layout
const BASE_CSS: &str = r#"
.display-area { padding: 12px 16px; min-height: 100px; }
.expression-label { font-size: 16px; padding: 4px 4px; }
.result-label { font-size: 48px; padding: 8px 4px; font-weight: 300; }
.result-label.result-medium { font-size: 36px; }
.result-label.result-small { font-size: 24px; }
.preview-label { font-size: 14px; padding: 2px 4px; font-style: italic; }
.calc-grid { margin: 4px 8px 8px 8px; }
.sci-grid { margin: 4px 0 8px 8px; }
button { font-size: 18px; padding: 10px; min-height: 44px; border-radius: 12px; }
.op-button { font-size: 22px; font-weight: bold; }
.equals-button { font-size: 22px; font-weight: bold; }
.memory-button { font-size: 13px; min-height: 32px; }
.function-button { font-size: 14px; }
.tab-bar { padding: 4px 8px 0 8px; }
.tab-button { font-size: 13px; padding: 4px 12px; min-height: 28px; border-radius: 8px 8px 0 0; border: none; }
.tab-add { font-size: 16px; padding: 2px 10px; min-height: 28px; border-radius: 8px; border: none; }
.menu-button { font-size: 18px; padding: 4px 10px; min-height: 28px; border-radius: 8px; border: none; }
.menu-item { font-size: 14px; padding: 8px 16px; min-height: 28px; border-radius: 6px; border: none; }
.menu-item:hover { opacity: 0.85; }
.menu-header { font-size: 11px; font-weight: bold; padding: 8px 16px 4px 16px; }
.panel-container { min-width: 200px; padding: 0 4px; }
.panel-tab { font-size: 12px; font-weight: bold; padding: 6px 10px; min-height: 24px; border-radius: 6px; border: none; }
.panel-item { font-size: 13px; padding: 8px 12px; border-radius: 6px; border: none; min-height: 20px; }
.panel-item-expr { font-size: 11px; }
.panel-item-result { font-size: 14px; font-weight: bold; }
.panel-item-label { font-size: 11px; font-style: italic; }
.panel-empty { font-size: 13px; padding: 24px 12px; font-style: italic; }
.converter-panel { padding: 12px; }
.converter-panel entry { font-size: 18px; padding: 8px; min-height: 36px; border-radius: 8px; }
.converter-panel label { font-size: 14px; padding: 4px; }
.converter-cat-btn { font-size: 13px; padding: 6px 12px; min-height: 28px; border-radius: 8px; border: none; }
.converter-result { font-size: 28px; font-weight: bold; padding: 12px; }
.converter-swap { font-size: 18px; padding: 6px 16px; border-radius: 8px; border: none; }
.tools-panel { padding: 12px; }
.tools-panel entry { font-size: 18px; padding: 8px; min-height: 36px; border-radius: 8px; }
.tools-panel label { font-size: 14px; padding: 4px; }
.tools-result { font-size: 24px; font-weight: bold; padding: 8px; }
.tools-pct-btn { font-size: 14px; padding: 8px 12px; min-height: 32px; border-radius: 8px; border: none; }
.notes-panel { padding: 12px; }
.notes-panel textview { font-size: 16px; padding: 8px; border-radius: 8px; }
.notes-panel textview text { font-family: monospace; }
.notes-result { font-size: 14px; font-family: monospace; padding: 8px; }
.mode-header { font-size: 16px; font-weight: bold; padding: 8px 4px; }
.back-button { font-size: 14px; padding: 4px 12px; min-height: 28px; border-radius: 8px; border: none; }
"#;

// Fredrik's void
const VOID_CSS: &str = r#"
.main-window { background-color: #000000; }
.display-area { background-color: #000000; }
.expression-label { color: #8e8e93; }
.result-label { color: #ffffff; }
.preview-label { color: #636366; }

.tab-bar { background-color: #000000; }
.tab-button { background-color: #1c1c1e; color: #8e8e93; }
.tab-button.active { background-color: #2c2c2e; color: #ff9500; }
.tab-add { background-color: transparent; color: #636366; }
.tab-add:hover { color: #ff9500; }
.menu-button { background-color: transparent; color: #636366; }
.menu-button:hover { color: #ff9500; }

.menu-item { background-color: #1c1c1e; color: #ebebf5; }
.menu-item:hover { background-color: #2c2c2e; }
.menu-header { color: #636366; }

button { border: none; }
button:focus { box-shadow: inset 0 0 0 2px #ff9500; }
.digit-button { background-color: #333333; color: #ffffff; font-size: 22px; border-radius: 100px; }
.digit-button:hover { background-color: #444444; }
.digit-button:active { background-color: #555555; }
.op-button { background-color: #ff9500; color: #ffffff; font-size: 26px; border-radius: 100px; }
.op-button:hover { background-color: #ffaa33; }
.op-button:active { background-color: #cc7700; }
.equals-button { background-color: #ff9500; color: #ffffff; font-size: 26px; border-radius: 100px; }
.equals-button:hover { background-color: #ffaa33; }
.clear-button { background-color: #a5a5a5; color: #000000; border-radius: 100px; }
.clear-button:hover { background-color: #b8b8b8; }
.util-button { background-color: #a5a5a5; color: #000000; border-radius: 100px; }
.util-button:hover { background-color: #b8b8b8; }
.memory-button { background-color: #1c1c1e; color: #8e8e93; border-radius: 8px; }
.memory-button:hover { background-color: #2c2c2e; color: #ffffff; }
.function-button { background-color: #1c1c1e; color: #ebebf5; border-radius: 100px; }
.function-button:hover { background-color: #2c2c2e; }

.panel-container { background-color: #0a0a0a; }
.panel-tab { background-color: #1c1c1e; color: #8e8e93; }
.panel-tab.active { background-color: #ff9500; color: #ffffff; }
.panel-item { background-color: #1c1c1e; color: #ebebf5; }
.panel-item:hover { background-color: #2c2c2e; }
.panel-item-expr { color: #8e8e93; }
.panel-item-result { color: #ff9500; }
.panel-item-label { color: #636366; }
.panel-empty { color: #636366; }

.converter-panel { background-color: #000000; }
.converter-panel entry { background-color: #1c1c1e; color: #ffffff; border: 1px solid #333333; }
.converter-panel label { color: #ebebf5; }
.converter-cat-btn { background-color: #1c1c1e; color: #8e8e93; }
.converter-cat-btn.active { background-color: #ff9500; color: #ffffff; }
.converter-result { color: #ff9500; }
.converter-swap { background-color: #333333; color: #ffffff; }
.converter-swap:hover { background-color: #444444; }

.tools-panel { background-color: #000000; }
.tools-panel entry { background-color: #1c1c1e; color: #ffffff; border: 1px solid #333333; }
.tools-panel label { color: #ebebf5; }
.tools-result { color: #ff9500; }
.tools-pct-btn { background-color: #1c1c1e; color: #ebebf5; }
.tools-pct-btn.active, .tools-pct-btn:active { background-color: #ff9500; color: #ffffff; }

.notes-panel { background-color: #000000; }
.notes-panel textview { background-color: #1c1c1e; color: #ffffff; border: 1px solid #333333; }
.notes-panel textview text { background-color: #1c1c1e; color: #ffffff; }
.notes-result { background-color: #0a0a0a; color: #ff9500; }

.mode-header { color: #ffffff; }
.back-button { background-color: #333333; color: #ffffff; }
.back-button:hover { background-color: #444444; }
"#;

// Frosted Fred
const FROSTED_CSS: &str = r#"
.main-window { background-color: #1a1a2e; }
.display-area { background-color: rgba(255,255,255,0.05); border-radius: 16px; margin: 8px; }
.expression-label { color: rgba(255,255,255,0.5); }
.result-label { color: #ffffff; }
.preview-label { color: rgba(255,255,255,0.3); }

.tab-bar { background-color: transparent; }
.tab-button { background-color: rgba(255,255,255,0.06); color: rgba(255,255,255,0.5); }
.tab-button.active { background-color: rgba(255,255,255,0.12); color: #7eb8ff; }
.tab-add { background-color: transparent; color: rgba(255,255,255,0.3); }
.tab-add:hover { color: #7eb8ff; }
.menu-button { background-color: transparent; color: rgba(255,255,255,0.4); }
.menu-button:hover { color: #7eb8ff; }

.menu-item { background-color: rgba(255,255,255,0.08); color: rgba(255,255,255,0.9); }
.menu-item:hover { background-color: rgba(255,255,255,0.14); }
.menu-header { color: rgba(255,255,255,0.4); }

button { border: none; border-radius: 16px; }
button:focus { box-shadow: inset 0 0 0 2px rgba(126,184,255,0.6); }
.digit-button { background-color: rgba(255,255,255,0.1); color: #ffffff; font-size: 22px; }
.digit-button:hover { background-color: rgba(255,255,255,0.16); }
.op-button { background-color: rgba(126,184,255,0.25); color: #7eb8ff; font-size: 26px; }
.op-button:hover { background-color: rgba(126,184,255,0.35); }
.equals-button { background-color: rgba(126,184,255,0.35); color: #ffffff; font-size: 26px; }
.equals-button:hover { background-color: rgba(126,184,255,0.45); }
.clear-button { background-color: rgba(255,255,255,0.15); color: #ffffff; }
.clear-button:hover { background-color: rgba(255,255,255,0.22); }
.util-button { background-color: rgba(255,255,255,0.15); color: #ffffff; }
.util-button:hover { background-color: rgba(255,255,255,0.22); }
.memory-button { background-color: rgba(255,255,255,0.05); color: rgba(255,255,255,0.5); border-radius: 10px; }
.memory-button:hover { background-color: rgba(255,255,255,0.1); color: #ffffff; }
.function-button { background-color: rgba(255,255,255,0.06); color: rgba(255,255,255,0.85); }
.function-button:hover { background-color: rgba(255,255,255,0.12); }

.panel-container { background-color: rgba(255,255,255,0.03); }
.panel-tab { background-color: rgba(255,255,255,0.06); color: rgba(255,255,255,0.5); }
.panel-tab.active { background-color: rgba(126,184,255,0.25); color: #7eb8ff; }
.panel-item { background-color: rgba(255,255,255,0.06); color: rgba(255,255,255,0.9); }
.panel-item:hover { background-color: rgba(255,255,255,0.1); }
.panel-item-expr { color: rgba(255,255,255,0.4); }
.panel-item-result { color: #7eb8ff; }
.panel-item-label { color: rgba(255,255,255,0.35); }
.panel-empty { color: rgba(255,255,255,0.3); }

.converter-panel { background-color: transparent; }
.converter-panel entry { background-color: rgba(255,255,255,0.08); color: #ffffff; border: 1px solid rgba(255,255,255,0.12); border-radius: 12px; }
.converter-panel label { color: rgba(255,255,255,0.85); }
.converter-cat-btn { background-color: rgba(255,255,255,0.06); color: rgba(255,255,255,0.5); }
.converter-cat-btn.active { background-color: rgba(126,184,255,0.25); color: #7eb8ff; }
.converter-result { color: #7eb8ff; }
.converter-swap { background-color: rgba(255,255,255,0.1); color: #ffffff; }

.tools-panel { background-color: transparent; }
.tools-panel entry { background-color: rgba(255,255,255,0.08); color: #ffffff; border: 1px solid rgba(255,255,255,0.12); border-radius: 12px; }
.tools-panel label { color: rgba(255,255,255,0.85); }
.tools-result { color: #7eb8ff; }
.tools-pct-btn { background-color: rgba(255,255,255,0.08); color: rgba(255,255,255,0.8); }
.tools-pct-btn.active { background-color: rgba(126,184,255,0.25); color: #7eb8ff; }

.notes-panel { background-color: transparent; }
.notes-panel textview { background-color: rgba(255,255,255,0.06); color: #ffffff; border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; }
.notes-panel textview text { background-color: transparent; color: #ffffff; }
.notes-result { background-color: rgba(255,255,255,0.04); color: #7eb8ff; }

.mode-header { color: #ffffff; }
.back-button { background-color: rgba(255,255,255,0.1); color: #ffffff; }
"#;

// Riced Fredulator
const RICED_CSS: &str = r#"
.main-window { background-color: #1e1e2e; }
.display-area { background-color: #1e1e2e; }
.expression-label { color: #6c7086; }
.result-label { color: #cdd6f4; }
.preview-label { color: #45475a; }

.tab-bar { background-color: #181825; }
.tab-button { background-color: #181825; color: #6c7086; }
.tab-button.active { background-color: #313244; color: #cba6f7; }
.tab-add { background-color: transparent; color: #45475a; }
.tab-add:hover { color: #cba6f7; }
.menu-button { background-color: transparent; color: #6c7086; }
.menu-button:hover { color: #cba6f7; }

.menu-item { background-color: #313244; color: #cdd6f4; }
.menu-item:hover { background-color: #45475a; }
.menu-header { color: #6c7086; }

button { border: none; }
button:focus { box-shadow: inset 0 0 0 2px #cba6f7; }
.digit-button { background-color: #313244; color: #cdd6f4; font-size: 22px; border-radius: 12px; }
.digit-button:hover { background-color: #45475a; }
.op-button { background-color: #cba6f7; color: #1e1e2e; font-size: 26px; border-radius: 12px; }
.op-button:hover { background-color: #b4befe; }
.equals-button { background-color: #a6e3a1; color: #1e1e2e; font-size: 26px; border-radius: 12px; }
.equals-button:hover { background-color: #94e2d5; }
.clear-button { background-color: #f38ba8; color: #1e1e2e; border-radius: 12px; }
.clear-button:hover { background-color: #eba0ac; }
.util-button { background-color: #45475a; color: #cdd6f4; border-radius: 12px; }
.util-button:hover { background-color: #585b70; }
.memory-button { background-color: #181825; color: #6c7086; border-radius: 8px; }
.memory-button:hover { background-color: #313244; color: #cdd6f4; }
.function-button { background-color: #181825; color: #bac2de; border-radius: 12px; }
.function-button:hover { background-color: #313244; }

.panel-container { background-color: #11111b; }
.panel-tab { background-color: #181825; color: #6c7086; }
.panel-tab.active { background-color: #cba6f7; color: #1e1e2e; }
.panel-item { background-color: #1e1e2e; color: #cdd6f4; }
.panel-item:hover { background-color: #313244; }
.panel-item-expr { color: #6c7086; }
.panel-item-result { color: #cba6f7; }
.panel-item-label { color: #45475a; }
.panel-empty { color: #45475a; }

.converter-panel { background-color: #1e1e2e; }
.converter-panel entry { background-color: #313244; color: #cdd6f4; border: 1px solid #45475a; }
.converter-panel label { color: #cdd6f4; }
.converter-cat-btn { background-color: #181825; color: #6c7086; }
.converter-cat-btn.active { background-color: #cba6f7; color: #1e1e2e; }
.converter-result { color: #a6e3a1; }
.converter-swap { background-color: #313244; color: #cdd6f4; }

.tools-panel { background-color: #1e1e2e; }
.tools-panel entry { background-color: #313244; color: #cdd6f4; border: 1px solid #45475a; }
.tools-panel label { color: #cdd6f4; }
.tools-result { color: #a6e3a1; }
.tools-pct-btn { background-color: #313244; color: #bac2de; }
.tools-pct-btn.active { background-color: #cba6f7; color: #1e1e2e; }

.notes-panel { background-color: #1e1e2e; }
.notes-panel textview { background-color: #313244; color: #cdd6f4; border: 1px solid #45475a; }
.notes-panel textview text { background-color: #313244; color: #cdd6f4; }
.notes-result { background-color: #181825; color: #a6e3a1; }

.mode-header { color: #cdd6f4; }
.back-button { background-color: #313244; color: #cdd6f4; }
"#;

//Neon Fredrik
const NEON_CSS: &str = r#"
.main-window { background-color: #0a0a1a; }
.display-area { background-color: #0a0a1a; }
.expression-label { color: #4a4a6a; }
.result-label { color: #00ffff; }
.preview-label { color: #1a1a3a; }

.tab-bar { background-color: #050510; }
.tab-button { background-color: #0f0f2a; color: #4a4a6a; }
.tab-button.active { background-color: #1a1a3a; color: #ff0080; }
.tab-add { background-color: transparent; color: #2a2a4a; }
.tab-add:hover { color: #00ffff; }
.menu-button { background-color: transparent; color: #4a4a6a; }
.menu-button:hover { color: #ff0080; }

.menu-item { background-color: #0f0f2a; color: #c0c0e0; }
.menu-item:hover { background-color: #1a1a3a; }
.menu-header { color: #4a4a6a; }

button { border: none; }
button:focus { box-shadow: inset 0 0 0 2px #ff0080; }
.digit-button { background-color: #1a1a3a; color: #e0e0ff; font-size: 22px; border-radius: 8px; }
.digit-button:hover { background-color: #2a2a4a; }
.op-button { background-color: #ff0080; color: #ffffff; font-size: 26px; border-radius: 8px; }
.op-button:hover { background-color: #ff3399; }
.equals-button { background-color: #00ffff; color: #0a0a1a; font-size: 26px; font-weight: bold; border-radius: 8px; }
.equals-button:hover { background-color: #33ffff; }
.clear-button { background-color: #2a2a4a; color: #ff0080; border-radius: 8px; }
.clear-button:hover { background-color: #3a3a5a; }
.util-button { background-color: #2a2a4a; color: #c0c0e0; border-radius: 8px; }
.util-button:hover { background-color: #3a3a5a; }
.memory-button { background-color: #0f0f2a; color: #4a4a8a; border-radius: 6px; }
.memory-button:hover { background-color: #1a1a3a; color: #00ffff; }
.function-button { background-color: #0f0f2a; color: #8080b0; border-radius: 8px; }
.function-button:hover { background-color: #1a1a3a; }

.panel-container { background-color: #050510; }
.panel-tab { background-color: #0f0f2a; color: #4a4a6a; }
.panel-tab.active { background-color: #ff0080; color: #ffffff; }
.panel-item { background-color: #0f0f2a; color: #c0c0e0; }
.panel-item:hover { background-color: #1a1a3a; }
.panel-item-expr { color: #4a4a6a; }
.panel-item-result { color: #00ffff; }
.panel-item-label { color: #2a2a4a; }
.panel-empty { color: #2a2a4a; }

.converter-panel { background-color: #0a0a1a; }
.converter-panel entry { background-color: #0f0f2a; color: #e0e0ff; border: 1px solid #2a2a4a; }
.converter-panel label { color: #c0c0e0; }
.converter-cat-btn { background-color: #0f0f2a; color: #4a4a8a; }
.converter-cat-btn.active { background-color: #ff0080; color: #ffffff; }
.converter-result { color: #00ffff; }
.converter-swap { background-color: #1a1a3a; color: #00ffff; }

.tools-panel { background-color: #0a0a1a; }
.tools-panel entry { background-color: #0f0f2a; color: #e0e0ff; border: 1px solid #2a2a4a; }
.tools-panel label { color: #c0c0e0; }
.tools-result { color: #00ffff; }
.tools-pct-btn { background-color: #0f0f2a; color: #8080b0; }
.tools-pct-btn.active { background-color: #ff0080; color: #ffffff; }

.notes-panel { background-color: #0a0a1a; }
.notes-panel textview { background-color: #0f0f2a; color: #e0e0ff; border: 1px solid #2a2a4a; }
.notes-panel textview text { background-color: #0f0f2a; color: #e0e0ff; }
.notes-result { background-color: #050510; color: #00ffff; }

.mode-header { color: #00ffff; }
.back-button { background-color: #1a1a3a; color: #00ffff; }
"#;

// Terminal Fred
const TERMINAL_CSS: &str = r#"
.main-window { background-color: #0a0a0a; }
.display-area { background-color: #0a0a0a; }
.expression-label { color: #338833; font-family: monospace; }
.result-label { color: #00ff00; font-family: monospace; font-weight: bold; }
.preview-label { color: #1a3a1a; font-family: monospace; }

.tab-bar { background-color: #050505; }
.tab-button { background-color: #0a0a0a; color: #338833; font-family: monospace; }
.tab-button.active { background-color: #1a1a1a; color: #00ff00; }
.tab-add { background-color: transparent; color: #1a3a1a; font-family: monospace; }
.tab-add:hover { color: #00ff00; }
.menu-button { background-color: transparent; color: #338833; font-family: monospace; }
.menu-button:hover { color: #00ff00; }

.menu-item { background-color: #111111; color: #00cc00; font-family: monospace; }
.menu-item:hover { background-color: #1a1a1a; }
.menu-header { color: #338833; font-family: monospace; }

button { border: none; font-family: monospace; }
button:focus { box-shadow: inset 0 0 0 2px #00ff00; }
.digit-button { background-color: #1a1a1a; color: #00ff00; font-size: 22px; border-radius: 4px; }
.digit-button:hover { background-color: #222222; }
.op-button { background-color: #003300; color: #00ff00; font-size: 26px; border-radius: 4px; }
.op-button:hover { background-color: #004400; }
.equals-button { background-color: #00aa00; color: #000000; font-size: 26px; font-weight: bold; border-radius: 4px; }
.equals-button:hover { background-color: #00cc00; }
.clear-button { background-color: #331111; color: #ff4444; border-radius: 4px; }
.clear-button:hover { background-color: #442222; }
.util-button { background-color: #1a1a1a; color: #00cc00; border-radius: 4px; }
.util-button:hover { background-color: #222222; }
.memory-button { background-color: #0a0a0a; color: #338833; border-radius: 4px; }
.memory-button:hover { background-color: #1a1a1a; color: #00ff00; }
.function-button { background-color: #111111; color: #00aa00; border-radius: 4px; }
.function-button:hover { background-color: #1a1a1a; }

.panel-container { background-color: #050505; }
.panel-tab { background-color: #0a0a0a; color: #338833; font-family: monospace; }
.panel-tab.active { background-color: #003300; color: #00ff00; }
.panel-item { background-color: #0a0a0a; color: #00cc00; font-family: monospace; }
.panel-item:hover { background-color: #1a1a1a; }
.panel-item-expr { color: #338833; }
.panel-item-result { color: #00ff00; }
.panel-item-label { color: #1a3a1a; }
.panel-empty { color: #1a3a1a; font-family: monospace; }

.converter-panel { background-color: #0a0a0a; }
.converter-panel entry { background-color: #111111; color: #00ff00; border: 1px solid #003300; font-family: monospace; }
.converter-panel label { color: #00cc00; font-family: monospace; }
.converter-cat-btn { background-color: #0a0a0a; color: #338833; font-family: monospace; }
.converter-cat-btn.active { background-color: #003300; color: #00ff00; }
.converter-result { color: #00ff00; font-family: monospace; }
.converter-swap { background-color: #1a1a1a; color: #00ff00; font-family: monospace; }

.tools-panel { background-color: #0a0a0a; }
.tools-panel entry { background-color: #111111; color: #00ff00; border: 1px solid #003300; font-family: monospace; }
.tools-panel label { color: #00cc00; font-family: monospace; }
.tools-result { color: #00ff00; font-family: monospace; }
.tools-pct-btn { background-color: #111111; color: #00aa00; font-family: monospace; }
.tools-pct-btn.active { background-color: #003300; color: #00ff00; }

.notes-panel { background-color: #0a0a0a; }
.notes-panel textview { background-color: #0a0a0a; color: #00ff00; border: 1px solid #003300; }
.notes-panel textview text { background-color: #0a0a0a; color: #00ff00; }
.notes-result { background-color: #050505; color: #00ff00; font-family: monospace; }

.mode-header { color: #00ff00; font-family: monospace; }
.back-button { background-color: #1a1a1a; color: #00ff00; font-family: monospace; }
"#;

// Solarized Fred
const SOLARIZED_CSS: &str = r#"
.main-window { background-color: #002b36; }
.display-area { background-color: #002b36; }
.expression-label { color: #586e75; }
.result-label { color: #fdf6e3; }
.preview-label { color: #073642; }

.tab-bar { background-color: #001e27; }
.tab-button { background-color: #002b36; color: #586e75; }
.tab-button.active { background-color: #073642; color: #b58900; }
.tab-add { background-color: transparent; color: #073642; }
.tab-add:hover { color: #b58900; }
.menu-button { background-color: transparent; color: #586e75; }
.menu-button:hover { color: #b58900; }

.menu-item { background-color: #073642; color: #93a1a1; }
.menu-item:hover { background-color: #0a4a5a; }
.menu-header { color: #586e75; }

button { border: none; }
button:focus { box-shadow: inset 0 0 0 2px #268bd2; }
.digit-button { background-color: #073642; color: #eee8d5; font-size: 22px; border-radius: 10px; }
.digit-button:hover { background-color: #0a4a5a; }
.op-button { background-color: #b58900; color: #002b36; font-size: 26px; border-radius: 10px; }
.op-button:hover { background-color: #cb9a00; }
.equals-button { background-color: #859900; color: #002b36; font-size: 26px; font-weight: bold; border-radius: 10px; }
.equals-button:hover { background-color: #97ad00; }
.clear-button { background-color: #dc322f; color: #fdf6e3; border-radius: 10px; }
.clear-button:hover { background-color: #e8524f; }
.util-button { background-color: #073642; color: #93a1a1; border-radius: 10px; }
.util-button:hover { background-color: #0a4a5a; }
.memory-button { background-color: #002b36; color: #586e75; border-radius: 8px; }
.memory-button:hover { background-color: #073642; color: #93a1a1; }
.function-button { background-color: #073642; color: #839496; border-radius: 10px; }
.function-button:hover { background-color: #0a4a5a; }

.panel-container { background-color: #001e27; }
.panel-tab { background-color: #002b36; color: #586e75; }
.panel-tab.active { background-color: #b58900; color: #002b36; }
.panel-item { background-color: #002b36; color: #93a1a1; }
.panel-item:hover { background-color: #073642; }
.panel-item-expr { color: #586e75; }
.panel-item-result { color: #b58900; }
.panel-item-label { color: #073642; }
.panel-empty { color: #073642; }

.converter-panel { background-color: #002b36; }
.converter-panel entry { background-color: #073642; color: #eee8d5; border: 1px solid #586e75; }
.converter-panel label { color: #93a1a1; }
.converter-cat-btn { background-color: #002b36; color: #586e75; }
.converter-cat-btn.active { background-color: #b58900; color: #002b36; }
.converter-result { color: #859900; }
.converter-swap { background-color: #073642; color: #93a1a1; }

.tools-panel { background-color: #002b36; }
.tools-panel entry { background-color: #073642; color: #eee8d5; border: 1px solid #586e75; }
.tools-panel label { color: #93a1a1; }
.tools-result { color: #859900; }
.tools-pct-btn { background-color: #073642; color: #839496; }
.tools-pct-btn.active { background-color: #b58900; color: #002b36; }

.notes-panel { background-color: #002b36; }
.notes-panel textview { background-color: #073642; color: #eee8d5; border: 1px solid #586e75; }
.notes-panel textview text { background-color: #073642; color: #eee8d5; }
.notes-result { background-color: #001e27; color: #859900; }

.mode-header { color: #eee8d5; }
.back-button { background-color: #073642; color: #93a1a1; }
"#;

// Native
const NATIVE_CSS: &str = "";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Native,
    Void,
    Frosted,
    Riced,
    Neon,
    Terminal,
    Solarized,
}

impl Theme {
    pub fn name(self) -> &'static str {
        match self {
            Self::Native => "Native",
            Self::Void => "Fredrik's Void",
            Self::Frosted => "Frosted Fred",
            Self::Riced => "Riced Fredulator",
            Self::Neon => "Neon Fredrik",
            Self::Terminal => "Terminal Fred",
            Self::Solarized => "Solarized Fred",
        }
    }

    pub const ALL: &'static [Theme] = &[
        Self::Native,
        Self::Void,
        Self::Frosted,
        Self::Riced,
        Self::Neon,
        Self::Terminal,
        Self::Solarized,
    ];

    pub fn next(self) -> Self {
        let all = Self::ALL;
        let idx = all.iter().position(|&t| t == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
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

    pub fn current(&self) -> Theme {
        self.current
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.current = theme;
        self.apply();
    }

    pub fn toggle(&mut self) {
        self.current = self.current.next();
        self.apply();
    }

    fn apply(&mut self) {
        StyleContext::remove_provider_for_screen(&self.screen, &self.provider);
        self.provider = CssProvider::new();
        let theme_css = match self.current {
            Theme::Native => NATIVE_CSS,
            Theme::Void => VOID_CSS,
            Theme::Frosted => FROSTED_CSS,
            Theme::Riced => RICED_CSS,
            Theme::Neon => NEON_CSS,
            Theme::Terminal => TERMINAL_CSS,
            Theme::Solarized => SOLARIZED_CSS,
        };
        let full_css = format!("{}\n{}", BASE_CSS, theme_css);
        self.provider.load_from_data(full_css.as_bytes()).ok();
        StyleContext::add_provider_for_screen(
            &self.screen,
            &self.provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
