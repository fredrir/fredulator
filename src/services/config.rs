use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        })
        .join("fredulator")
}

pub fn path() -> PathBuf {
    dir().join("config.toml")
}

pub fn load() -> Config {
    let p = path();
    if !p.exists() {
        let _ = fs::create_dir_all(dir());
        let _ = fs::write(&p, generate_default_config());
    }
    match fs::read_to_string(&p) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: ThemeConfig,
    pub keybindings: KeybindingsConfig,
    pub layout: LayoutConfig,
    pub format: FormatConfig,
    pub behavior: BehaviorConfig,
    pub history: HistoryConfig,
    pub input: InputConfig,
    pub feedback: FeedbackConfig,
    pub window: WindowConfig,
    pub plugins: PluginsConfig,
    pub session: SessionConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            keybindings: KeybindingsConfig::default(),
            layout: LayoutConfig::default(),
            format: FormatConfig::default(),
            behavior: BehaviorConfig::default(),
            history: HistoryConfig::default(),
            input: InputConfig::default(),
            feedback: FeedbackConfig::default(),
            window: WindowConfig::default(),
            plugins: PluginsConfig::default(),
            session: SessionConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionConfig {
    pub restore_session: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            restore_session: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: String,
    pub accent_color: String,
    pub background_color: String,
    pub button_style: String,
    pub font: String,
    pub custom_css: String,
    pub colors: ThemeColors,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "native".into(),
            accent_color: String::new(),
            background_color: String::new(),
            button_style: "rounded".into(),
            font: "system".into(),
            custom_css: String::new(),
            colors: ThemeColors::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeColors {
    pub window_bg: String,
    pub display_bg: String,
    pub display_fg: String,
    pub display_secondary: String,
    pub display_preview: String,
    pub digit_bg: String,
    pub digit_fg: String,
    pub op_bg: String,
    pub op_fg: String,
    pub equals_bg: String,
    pub equals_fg: String,
    pub clear_bg: String,
    pub clear_fg: String,
    pub util_bg: String,
    pub util_fg: String,
    pub function_bg: String,
    pub function_fg: String,
    pub memory_bg: String,
    pub memory_fg: String,
    pub panel_bg: String,
    pub panel_fg: String,
    pub panel_accent: String,
    pub tab_bg: String,
    pub tab_fg: String,
    pub tab_active_bg: String,
    pub tab_active_fg: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            window_bg: "#000000".into(),
            display_bg: "#000000".into(),
            display_fg: "#ffffff".into(),
            display_secondary: "#8e8e93".into(),
            display_preview: "#636366".into(),
            digit_bg: "#333333".into(),
            digit_fg: "#ffffff".into(),
            op_bg: "#ff9500".into(),
            op_fg: "#ffffff".into(),
            equals_bg: "#ff9500".into(),
            equals_fg: "#ffffff".into(),
            clear_bg: "#a5a5a5".into(),
            clear_fg: "#000000".into(),
            util_bg: "#a5a5a5".into(),
            util_fg: "#000000".into(),
            function_bg: "#1c1c1e".into(),
            function_fg: "#ebebf5".into(),
            memory_bg: "#1c1c1e".into(),
            memory_fg: "#8e8e93".into(),
            panel_bg: "#0a0a0a".into(),
            panel_fg: "#ebebf5".into(),
            panel_accent: "#ff9500".into(),
            tab_bg: "#1c1c1e".into(),
            tab_fg: "#8e8e93".into(),
            tab_active_bg: "#2c2c2e".into(),
            tab_active_fg: "#ff9500".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindingsConfig {
    pub scheme: String,
    pub custom: HashMap<String, String>,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            scheme: "default".into(),
            custom: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    pub button_spacing: u32,
    pub grid_padding: u32,
    pub button_radius: u32,
    pub compact_mode: bool,
    pub show_scientific: bool,
    pub show_memory_row: bool,
    pub button_size: String,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            button_spacing: 6,
            grid_padding: 8,
            button_radius: 12,
            compact_mode: false,
            show_scientific: false,
            show_memory_row: true,
            button_size: "auto".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FormatConfig {
    pub decimal_precision: u32,
    pub thousands_separator: String,
    pub scientific_notation: String,
    pub rounding_mode: String,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            decimal_precision: 10,
            thousands_separator: String::new(),
            scientific_notation: "auto".into(),
            rounding_mode: "half_up".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub auto_evaluate: bool,
    pub operator_precedence: bool,
    pub angle_mode: String,
    pub percentage_behavior: String,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_evaluate: true,
            operator_precedence: true,
            angle_mode: "degrees".into(),
            percentage_behavior: "divide_100".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryConfig {
    pub max_entries: usize,
    pub auto_save: bool,
    pub show_timestamps: bool,
    pub group_by_session: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 200,
            auto_save: false,
            show_timestamps: false,
            group_by_session: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub smart_parsing: bool,
    pub expression_mode: String,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            smart_parsing: true,
            expression_mode: "inline".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeedbackConfig {
    pub animations: bool,
    pub button_press_style: String,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            animations: true,
            button_press_style: "instant".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub always_on_top: bool,
    pub opacity: f64,
    pub remember_geometry: bool,
    pub compact_mode: bool,
    pub default_width: i32,
    pub default_height: i32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            always_on_top: false,
            opacity: 1.0,
            remember_geometry: false,
            compact_mode: false,
            default_width: 360,
            default_height: 580,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    pub functions: HashMap<String, String>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }
}

fn generate_default_config() -> String {
    r##"# Fredulator Configuration
# ~/.config/fredulator/config.toml
# Changes take effect on restart.

# -- Theme ---------------------------------------------------------
[theme]
# Built-in: native, void, frosted, riced, neon, terminal, solarized, custom
name = "native"
# Override accent color for any theme (leave empty for theme default)
accent_color = ""
# Override background color for any theme
background_color = ""
# Button corners: rounded, flat, outlined
button_style = "rounded"
# Font: system, monospace, or any installed font name
font = "system"
# Raw CSS appended after all other styles (advanced)
custom_css = ""

# Full color scheme (only used when name = "custom")
# To export/import themes: copy this [theme.colors] section
[theme.colors]
window_bg = "#000000"
display_bg = "#000000"
display_fg = "#ffffff"
display_secondary = "#8e8e93"
display_preview = "#636366"
digit_bg = "#333333"
digit_fg = "#ffffff"
op_bg = "#ff9500"
op_fg = "#ffffff"
equals_bg = "#ff9500"
equals_fg = "#ffffff"
clear_bg = "#a5a5a5"
clear_fg = "#000000"
util_bg = "#a5a5a5"
util_fg = "#000000"
function_bg = "#1c1c1e"
function_fg = "#ebebf5"
memory_bg = "#1c1c1e"
memory_fg = "#8e8e93"
panel_bg = "#0a0a0a"
panel_fg = "#ebebf5"
panel_accent = "#ff9500"
tab_bg = "#1c1c1e"
tab_fg = "#8e8e93"
tab_active_bg = "#2c2c2e"
tab_active_fg = "#ff9500"

# -- Keybindings ---------------------------------------------------
[keybindings]
# Base scheme: default (vim hjkl navigation), emacs (Ctrl+f/b/n/p)
scheme = "default"

# Override or add keybindings: "KeyCombo" = "action"
#
# Modifiers: Ctrl, Alt, Shift (Shift only for special keys)
# Keys: a-z, 0-9, symbols, Return, Escape, BackSpace, Delete,
#        Tab, Left, Right, Up, Down, space
#
# Actions:
#   digit_0..digit_9, decimal, add, subtract, multiply, divide,
#   power, percent, factorial, equals, clear, backspace,
#   toggle_sign, left_paren, right_paren,
#   navigate_left, navigate_right, navigate_up, navigate_down,
#   activate, toggle_theme, toggle_scientific, quit, undo,
#   new_tab, close_tab, next_tab, prev_tab,
#   toggle_history, toggle_memory, toggle_pinned,
#   pin_result, memory_store, export_history,
#   open_converter, open_tools, open_notes, open_menu, back_to_calc
[keybindings.custom]
# "Ctrl+Return" = "equals"
# "Alt+BackSpace" = "clear"

# -- Layout --------------------------------------------------------
[layout]
button_spacing = 6
grid_padding = 8
button_radius = 12
compact_mode = false
# Start with scientific panel visible
show_scientific = false
show_memory_row = true
# auto, small, large
button_size = "auto"

# -- Number Formatting ---------------------------------------------
[format]
# Max decimal places (0 = integers only, up to 20)
decimal_precision = 10
# Thousands separator: "" (none), ",", ".", " "
thousands_separator = ""
# auto, always, never
scientific_notation = "auto"
# half_up, truncate
rounding_mode = "half_up"

# -- Behavior ------------------------------------------------------
[behavior]
# Show live preview result as you type
auto_evaluate = true
# true = standard math precedence (2+3*4=14)
# false = left-to-right like basic calculators (2+3*4=20)
operator_precedence = true
# Default angle mode: degrees, radians
angle_mode = "degrees"
# divide_100 or of_previous
percentage_behavior = "divide_100"

# -- History -------------------------------------------------------
[history]
max_entries = 200
# Save/load history between sessions
auto_save = false
# Show timestamps in history panel
show_timestamps = false
# Group history entries by session
group_by_session = false

# -- Input ---------------------------------------------------------
[input]
# 2(3+4) auto-multiply, "50% of 200"
smart_parsing = true
# inline or step_by_step
expression_mode = "inline"

# -- Feedback ------------------------------------------------------
[feedback]
# Enable/disable transition animations
animations = true
# instant or animated
button_press_style = "instant"

# -- Window --------------------------------------------------------
[window]
always_on_top = false
# 0.1 to 1.0 (requires compositor)
opacity = 1.0
# Remember position and size between sessions
remember_geometry = false
# Titlebar-less mode
compact_mode = false
default_width = 360
default_height = 580

# -- Plugins -------------------------------------------------------
# Custom functions: name = "expression using x"
# Available in math notes and as plugin_<name> in expressions
[plugins.functions]
# double = "x * 2"
# half = "x / 2"
# c2f = "x * 9 / 5 + 32"
# f2c = "(x - 32) * 5 / 9"
"##
    .to_string()
}
