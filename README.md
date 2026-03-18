# Fredulator

**A customizable GTK calculator with VIM motions**

Vim motions, multi-tab workspaces, live expression preview, and 7 built-in themes, all from the keyboard without touching the mouse. Built for Linux with GTK 3.

> 📸 _Screenshots coming soon — contributions welcome_

---

## Contents

- [Why Fredulator?](#why-fredulator)
- [Install](#install)
- [Key shortcuts](#key-shortcuts)
- [Features](#features)
- [Themes](#themes)
- [Configuration](#configuration)
- [Architecture](#architecture)

---

## Why Fredulator?

Most calculators are either too simple or too complex. Fredulator hits the middle:

- **See the whole expression** — `2 + 3 × 4 =` displayed as you type, not just the running total
- **Live preview** — result appears before you press `=`
- **Navigate without a mouse** — `hjkl` moves between buttons, `Tab` switches tabs, everything is reachable by keyboard
- **Multiple workspaces** — independent tabs, each with their own history and memory
- **Customizable to the bone** — keybindings, themes, number formatting, behavior, plugins, all in one TOML file

---

## Install

### Arch Linux (AUR) — recommended

```bash
yay -S fredulator-git
```

### From source

```bash
git clone https://github.com/fredrir/fredulator.git
cd fredulator
cargo build --release
sudo install -Dm755 target/release/fredulator /usr/local/bin/fredulator
sudo cp fredulator.desktop /usr/share/applications/
```

**Requirements:** GTK 3.22+, Rust 2021 edition

---

## Key shortcuts

The most useful shortcuts to know:

| Key                   | Action                           |
| --------------------- | -------------------------------- |
| `hjkl` / arrows       | Navigate buttons                 |
| `Space`               | Activate focused button          |
| `Tab` / `Shift+Tab`   | Next / previous tab              |
| `g` + `t` / `g` + `T` | Next / previous tab (chord)      |
| `Ctrl+T` / `Ctrl+W`   | New / close tab                  |
| `s`                   | Toggle scientific mode           |
| `t`                   | Cycle theme                      |
| `u` / `Ctrl+Z`        | Undo                             |
| `Ctrl+H/M/P`          | History / Memory / Pinned panels |
| `Ctrl+E/R/N`          | Converter / Tools / Notes        |
| `?` / `F1`            | Full shortcut reference          |
| `Ctrl+Q`              | Quit                             |

<details>
<summary>Full keyboard reference</summary>

### Calculator

| Key         | Action                |
| ----------- | --------------------- |
| `0–9`       | Digits                |
| `+ − * /`   | Arithmetic            |
| `^`         | Power                 |
| `( )`       | Parentheses           |
| `.`         | Decimal               |
| `= / Enter` | Calculate             |
| `%`         | Percent               |
| `!`         | Factorial             |
| `n`         | Negate (+/−)          |
| `Backspace` | Delete last character |
| `Escape`    | Clear / close panel   |

### Tabs

| Key          | Action               |
| ------------ | -------------------- |
| `Ctrl+T`     | New tab              |
| `Ctrl+W`     | Close tab            |
| `Tab`        | Next tab             |
| `Shift+Tab`  | Previous tab         |
| `g` + `t`    | Next tab (chord)     |
| `g` + `T`    | Previous tab (chord) |
| Click        | Switch to tab        |
| Double-click | Rename tab           |
| Right-click  | Delete / rename tab  |

### Panels & modes

| Key            | Action                             |
| -------------- | ---------------------------------- |
| `Ctrl+H`       | History (search, export JSON/CSV)  |
| `Ctrl+M`       | Memory panel                       |
| `Ctrl+P`       | Pinned results                     |
| `Ctrl+S`       | Pin current result                 |
| `S`            | Store value to memory              |
| `Ctrl+Shift+E` | Export history                     |
| `Ctrl+E`       | Unit converter                     |
| `Ctrl+R`       | Quick tools (tip / discount / tax) |
| `Ctrl+N`       | Math notes (per-line evaluation)   |

</details>

---

## Features

### Calculator core

- **Expression display** — see the full calculation as you type: `2 + 3 × (4 − 1)`
- **Live preview** — result shown inline before pressing `=`
- **Smart parsing** — `2(3+4)` implicit multiply, `50% of 200`, function names as text (`sin(45)`)
- **Scientific mode** — trig, inverse trig, log, sqrt, power, factorial, memory ops (`s` to toggle)
- **Undo stack** — full history with `u` or `Ctrl+Z`

### Workspaces

- **Multi-tab** — independent calculations, each with their own engine state
- **Persistent sessions** — tabs and history survive app restarts (opt-in)
- **Calculation history** — scrollable panel with search and export to JSON/CSV
- **Memory panel** — store multiple named values with `S`
- **Pinned results** — save important calculations with `Ctrl+S`

### Tools

- **Unit converter** — length, weight, temperature, speed, volume
- **Quick tools** — tip calculator, discount, tax (slides in from the right)
- **Math notes** — multi-line scratchpad, each line auto-evaluates

### Customisation

- **7 built-in themes** with instant preview — plus fully custom via config
- **Remappable keybindings** — default (vim) or emacs scheme, override any key
- **Number formatting** — decimal precision, thousands separator, scientific notation
- **Plugin functions** — define custom functions in config: `c2f = "x * 9 / 5 + 32"`
- **Window control** — always-on-top, opacity, remember position/size

---

## Themes

Cycle with `t` or pick from the menu (with colour preview dots).

| Theme         | Vibe                                              |
| ------------- | ------------------------------------------------- |
| **Native**    | Follows your system GTK theme                     |
| **Void**      | Deep black, orange accents, the default dark mode |
| **Frosted**   | Translucent surfaces, soft blue, glassmorphism    |
| **Riced**     | Catppuccin Mocha palette                          |
| **Neon**      | Cyberpunk magenta and cyan                        |
| **Terminal**  | Retro green-on-black, monospace font              |
| **Solarized** | Solarized dark with gold and green                |
| **Custom**    | All 26 colour fields via config                   |

Custom theme example:

```toml
[theme]
name = "custom"
accent_color = "#ff6ac1"

[theme.colors]
window_bg = "#1a1a2e"
digit_bg = "#16213e"
op_bg = "#ff6ac1"
# ... 23 more fields
```

---

## Configuration

Config lives at `~/.config/fredulator/config.toml` and is created with documented defaults on first run.

### Keybindings

```toml
[keybindings]
scheme = "default"   # or "emacs" for Ctrl+f/b/n/p navigation

[keybindings.custom]
"Ctrl+Return" = "equals"
"x"           = "multiply"
"h"           = "unbound"     # unbind a key
```

Available actions: `digit_0`–`digit_9`, `add`, `subtract`, `multiply`, `divide`, `power`, `percent`, `factorial`, `equals`, `clear`, `backspace`, `decimal`, `toggle_sign`, `left_paren`, `right_paren`, `navigate_left/right/up/down`, `activate`, `toggle_theme`, `toggle_scientific`, `quit`, `undo`, `new_tab`, `close_tab`, `next_tab`, `prev_tab`, `toggle_history/memory/pinned`, `pin_result`, `memory_store`, `export_history`, `open_converter/tools/notes`, `open_menu`, `back_to_calc`, `show_help`

### Number formatting

```toml
[format]
decimal_precision    = 10
thousands_separator  = ","       # "", ",", ".", " "
scientific_notation  = "auto"    # auto | always | never
```

### Behaviour

```toml
[behavior]
auto_evaluate       = true       # live preview
operator_precedence = true       # false = left-to-right (basic calculator mode)
angle_mode          = "degrees"  # degrees | radians
```

### Plugin functions

```toml
[plugins.functions]
double = "x * 2"
c2f    = "x * 9 / 5 + 32"
f2c    = "(x - 32) * 5 / 9"
```

Use in Math Notes as `c2f(100)`. All other config options (window, history, session, layout) are documented in the generated config file.

---

## Architecture

Elm-style unidirectional data flow, no async, no threads, single-threaded GTK event loop.

```
UI event → Message → update(state, msg) → SideEffects → GTK rendering
```

```
src/
  main.rs          Signal wiring, SideEffect rendering
  app/             State coordination (message, state, update)
  domain/          Pure logic — no GTK, no IO (eval, engine, convert, types)
  ui/              GTK widgets (builder, keyboard, navigation)
  services/        Persistence (config, theme, history, session)
```

---

## License

MIT
