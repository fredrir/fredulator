# Fredulator

A GTK calculator for Linux with vim motions, multi-tab support, smart math, and Fredrik-themed themes.

Fully customizable via `~/.config/fredulator/config.toml` — theme colors, keybindings, layout, number formatting, behavior, history, window settings, and plugin functions.

## Features

- **Expression display** - See the full calculation (`2+3×4=`) with proper operator precedence
- **Smart input** - Live preview shows the result as you type, no `=` needed
- **Scientific mode** - Toggle with `s` key: trig, inverse trig, log, sqrt, power, factorial, constants, memory operations
- **Vim motions** - Navigate buttons with `hjkl`, activate with `Space`
- **Emacs bindings** - Optional `Ctrl+f/b/n/p` navigation via config
- **7 built-in themes + custom** - Fredrik's Void, Frosted Fred, Riced Fredulator, Neon Fredrik, Terminal Fred, Solarized Fred, Native, or fully custom via config
- **Multi-tab** - Run multiple independent calculations with `Ctrl+T` / `Ctrl+W` / `Tab`
- **Calculation history** - Scrollable panel with search, timestamps, export to JSON/CSV (`Ctrl+H`)
- **Visual memory panel** - Store multiple named values (`Ctrl+M`, `S` to store)
- **Pinned calculations** - Save important results with labels (`Ctrl+S` to pin, `Ctrl+P` to view)
- **Undo** - Full undo stack with `u` or `Ctrl+Z`
- **Unit converter** - Length, weight, temperature, speed, volume (`Ctrl+E`)
- **Quick tools** - Tip, discount, and tax calculators (`Ctrl+R`)
- **Math notes** - Multi-line scratchpad with per-line evaluation, natural math input (`Ctrl+N`)
- **Smart math parsing** - `2(3+4)` implicit multiply, `50% of 200 = 100`, function names as text
- **Plugin functions** - Define custom functions in config (e.g., `c2f = "x * 9 / 5 + 32"`)
- **Persistent history** - Save/load calculation history between sessions
- **Keyboard-first** - Every feature accessible without a mouse, all keys remappable
- **Window control** - Always-on-top, transparency, remember position/size, titlebar-less mode

## Configuration

On first run, Fredulator creates `~/.config/fredulator/config.toml` with documented defaults. Edit this file to customize everything:

### Theme

```toml
[theme]
name = "void"           # native, void, frosted, riced, neon, terminal, solarized, custom
accent_color = "#ff9500" # override accent for any theme
background_color = ""    # override background
button_style = "rounded" # rounded, flat, outlined
font = "monospace"       # system, monospace, or any font name
custom_css = ""          # raw CSS appended after theme

# Full custom theme (when name = "custom")
[theme.colors]
window_bg = "#000000"
digit_bg = "#333333"
op_bg = "#ff9500"
# ... 26 color fields total
```

To share themes: copy the `[theme]` and `[theme.colors]` sections between config files.

### Keybindings

```toml
[keybindings]
scheme = "default"  # default (vim hjkl) or emacs (Ctrl+f/b/n/p)

[keybindings.custom]
"Ctrl+Return" = "equals"
"Alt+BackSpace" = "clear"
"x" = "multiply"     # remap any key
"h" = "unbound"      # unbind a key
```

Available actions: `digit_0`..`digit_9`, `add`, `subtract`, `multiply`, `divide`, `power`, `percent`, `factorial`, `equals`, `clear`, `backspace`, `decimal`, `toggle_sign`, `left_paren`, `right_paren`, `navigate_left`, `navigate_right`, `navigate_up`, `navigate_down`, `activate`, `toggle_theme`, `toggle_scientific`, `quit`, `undo`, `new_tab`, `close_tab`, `next_tab`, `prev_tab`, `toggle_history`, `toggle_memory`, `toggle_pinned`, `pin_result`, `memory_store`, `export_history`, `open_converter`, `open_tools`, `open_notes`, `open_menu`, `back_to_calc`

### Layout

```toml
[layout]
button_spacing = 6
grid_padding = 8
button_radius = 12
compact_mode = false    # smaller padding/fonts
show_scientific = false # start with scientific panel open
button_size = "auto"    # auto, small, large
```

### Number Formatting

```toml
[format]
decimal_precision = 10
thousands_separator = ","  # "" (none), ",", ".", " "
scientific_notation = "auto"  # auto, always, never
rounding_mode = "half_up"    # half_up, truncate
```

### Behavior

```toml
[behavior]
auto_evaluate = true          # live preview
operator_precedence = true    # false = left-to-right like basic calcs
angle_mode = "degrees"        # degrees, radians
percentage_behavior = "divide_100"
```

### History

```toml
[history]
max_entries = 200
auto_save = false        # persist between sessions
show_timestamps = false
group_by_session = false
```

History can be exported via the JSON/CSV buttons in the history panel or with `Ctrl+Shift+E`. Exports are saved to `~/.config/fredulator/`.

### Window

```toml
[window]
always_on_top = false
opacity = 1.0            # 0.1-1.0, requires compositor
remember_geometry = false # save/restore position and size
compact_mode = false      # titlebar-less
default_width = 360
default_height = 580
```

### Plugins

```toml
[plugins.functions]
double = "x * 2"
c2f = "x * 9 / 5 + 32"
f2c = "(x - 32) * 5 / 9"
```

Plugin functions are available in Math Notes as `funcname(value)`.

## Keyboard Shortcuts

### Calculator

| Key          | Action                     |
| ------------ | -------------------------- |
| `0-9`        | Digits                     |
| `+ - * /`    | Operations                 |
| `^`          | Power                      |
| `( )`        | Parentheses                |
| `.`          | Decimal                    |
| `= / Enter`  | Calculate                  |
| `%`          | Percent                    |
| `!`          | Factorial                  |
| `n`          | Negate (+/-)               |
| `Backspace`  | Delete last                |
| `Escape`     | Clear / back to calculator |
| `h j k l`    | Vim navigation             |
| `Arrow keys` | Navigation                 |
| `Space`      | Activate focused button    |
| `u`          | Undo                       |
| `s`          | Toggle scientific mode     |
| `t`          | Cycle theme                |
| `q`          | Quit                       |
| `;`          | Open menu                  |

### Tabs

| Key         | Action       |
| ----------- | ------------ |
| `Ctrl+T`    | New tab      |
| `Ctrl+W`    | Close tab    |
| `Tab`       | Next tab     |
| `Shift+Tab` | Previous tab |

### Panels & Modes

| Key            | Action                         |
| -------------- | ------------------------------ |
| `Ctrl+H`       | Toggle history panel           |
| `Ctrl+M`       | Toggle memory panel            |
| `Ctrl+P`       | Toggle pinned panel            |
| `Ctrl+S`       | Pin current result             |
| `S`            | Store value to memory          |
| `Ctrl+E`       | Unit converter                 |
| `Ctrl+R`       | Quick tools (tip/discount/tax) |
| `Ctrl+N`       | Math notes                     |
| `Ctrl+Z`       | Undo (alternative)             |
| `Ctrl+Shift+E` | Export history                 |

All keybindings can be remapped in `~/.config/fredulator/config.toml`.

## Themes

7 built-in themes, cycled with `t` or selected from the menu, plus fully custom themes:

- **Native** (default) - Respects your system GTK theme
- **Fredrik's Void** - Deep black with orange accents
- **Frosted Fred** - Glassmorphism with translucent surfaces and soft blue
- **Riced Fredulator** - Catppuccin Mocha palette
- **Neon Fredrik** - Cyberpunk with magenta and cyan neon
- **Terminal Fred** - Retro green-on-black terminal aesthetic, monospace
- **Solarized Fred** - Solarized dark with gold and green accents
- **Custom** - Define all 26 color fields in config

## Install

### From source

```bash
git clone https://github.com/fredrir/fredulator.git
cd fredulator
cargo build --release
sudo cp target/release/fredulator /usr/local/bin/
sudo cp fredulator.desktop /usr/share/applications/
```

### Arch Linux (AUR)

```bash
yay -S fredulator-git
```

## Architecture

```
src/
  config.rs     - TOML config system (~/.config/fredulator/config.toml)
  eval.rs       - Shunting-yard evaluator + text expression parser + plugin functions
  engine.rs     - Calculator state machine (undo, history, memory, pinned)
  keyboard.rs   - Configurable keybinding system (default/vim/emacs + custom)
  theme.rs      - CSS theme management (7 built-in + custom colors + overrides)
  convert.rs    - Unit conversion engine
  ui.rs         - GTK widget layout (tabs, panels, modes, history search/export)
  main.rs       - Event wiring + window config + history persistence
```

## Dependencies

- GTK 3.22+
- Rust 2021 edition

## License

MIT
