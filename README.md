# Fredulator

A GTK calculator for Linux with vim motions, expression evaluation, and native theme support.

## Features

- **Expression display** - See the full calculation (`2+3×4=`) with proper operator precedence
- **Scientific mode** - Toggle with `Sci` button or `s` key: sin, cos, tan, ln, log, sqrt, power, factorial, constants (pi, e), and more
- **Vim motions** - Navigate buttons with `hjkl`, activate with `Space`
- **4 themes** - Native GTK, Dark, macOS, Windows (cycle with `t`)
- **Memory** - MC, MR, M+, M- for storing values
- **Keyboard-first** - Full numpad and keyboard support, every operation accessible without a mouse

## Keyboard shortcuts

| Key          | Action                  |
| ------------ | ----------------------- |
| `0-9`        | Digits                  |
| `+ - * /`    | Operations              |
| `^`          | Power                   |
| `( )`        | Parentheses             |
| `.`          | Decimal                 |
| `= / Enter`  | Calculate               |
| `%`          | Percent                 |
| `!`          | Factorial               |
| `n`          | Negate (+/-)            |
| `Backspace`  | Delete                  |
| `Escape`     | Clear                   |
| `h j k l`    | Vim navigation          |
| `Arrow keys` | Navigation              |
| `Space`      | Activate focused button |
| `s`          | Toggle scientific mode  |
| `t`          | Cycle theme             |
| `q`          | Quit                    |

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

## Themes

The calculator ships with 4 themes, cycled with `t`:

- **Native** - Respects your system GTK theme. Works with Qt desktop bridges (kvantum, adwaita-qt)
- **Dark** (default) - iOS-inspired dark theme with orange accents
- **macOS** - Apple calculator style
- **Windows** - Microsoft calculator style with blue accents

## Architecture

```
src/
  eval.rs       - Shunting-yard expression evaluator
  engine.rs     - Calculator state machine
  keyboard.rs   - Key mapping with vim motions
  theme.rs      - CSS theme management
  ui.rs         - GTK widget layout
  main.rs       - Event wiring
```

## Dependencies

- GTK 3.22+
- Rust 2021 edition

## License

MIT
