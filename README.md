# Fredulator

A GTK calculator for Linux with vim motions, multi-tab support, smart math, and Fredrik-themed themes.

## Features

- **Expression display** - See the full calculation (`2+3×4=`) with proper operator precedence
- **Smart input** - Live preview shows the result as you type, no `=` needed
- **Scientific mode** - Toggle with `s` key: trig, inverse trig, log, sqrt, power, factorial, constants, memory operations
- **Vim motions** - Navigate buttons with `hjkl`, activate with `Space`
- **7 themes** - Fredrik's Void, Frosted Fred, Riced Fredulator, Neon Fredrik, Terminal Fred, Solarized Fred, Native (cycle with `t`)
- **Multi-tab** - Run multiple independent calculations with `Ctrl+T` / `Ctrl+W` / `Tab`
- **Calculation history** - Scrollable panel of past results (`Ctrl+H`)
- **Visual memory panel** - Store multiple named values (`Ctrl+M`, `S` to store)
- **Pinned calculations** - Save important results with labels (`Ctrl+S` to pin, `Ctrl+P` to view)
- **Undo** - Full undo stack with `u` or `Ctrl+Z`
- **Unit converter** - Length, weight, temperature, speed, volume (`Ctrl+E`)
- **Quick tools** - Tip, discount, and tax calculators (`Ctrl+R`)
- **Math notes** - Multi-line scratchpad with per-line evaluation, natural math input (`Ctrl+N`)
- **Smart math parsing** - `2(3+4)` implicit multiply, `50% of 200 = 100`, function names as text
- **Keyboard-first** - Every feature accessible without a mouse

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

| Key      | Action                         |
| -------- | ------------------------------ |
| `Ctrl+H` | Toggle history panel           |
| `Ctrl+M` | Toggle memory panel            |
| `Ctrl+P` | Toggle pinned panel            |
| `Ctrl+S` | Pin current result             |
| `S`      | Store value to memory          |
| `Ctrl+E` | Unit converter                 |
| `Ctrl+R` | Quick tools (tip/discount/tax) |
| `Ctrl+N` | Math notes                     |
| `Ctrl+Z` | Undo (alternative)             |

## Themes

7 themes, cycled with `t` or selected from the menu (`≡` / `;`):

- **Native** (default) - Respects your system GTK theme. Works with Qt desktop bridges (kvantum, adwaita-qt)
- **Fredrik's Void** - Deep black with orange accents
- **Frosted Fred** - Glassmorphism with translucent surfaces and soft blue
- **Riced Fredulator** - Catppuccin Mocha palette, inspired by r/unixporn
- **Neon Fredrik** - Cyberpunk with magenta and cyan neon
- **Terminal Fred** - Retro green-on-black terminal aesthetic, monospace
- **Solarized Fred** - Solarized dark with gold and green accents

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
  eval.rs       - Shunting-yard evaluator + text expression parser
  engine.rs     - Calculator state machine (undo, history, memory, pinned)
  keyboard.rs   - Key mapping with vim motions + Ctrl combos
  theme.rs      - CSS theme management (7 themes)
  convert.rs    - Unit conversion engine
  ui.rs         - GTK widget layout (tabs, panels, modes)
  main.rs       - Event wiring
```

## Dependencies

- GTK 3.22+
- Rust 2021 edition

## License

MIT
