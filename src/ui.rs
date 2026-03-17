use gtk::prelude::*;
use gtk::{Button, Entry, Grid, Window, WindowType};

use crate::engine::Operation;
use crate::keyboard::Direction;

#[derive(Clone, Copy)]
pub enum ButtonAction {
    Digit(char),
    Decimal,
    Operation(Operation),
    Equals,
    Clear,
    ToggleSign,
    Percent,
}

pub struct CalculatorUI {
    pub window: Window,
    pub display: Entry,
    /// 5 rows x 4 cols button grid for vim navigation.
    /// Row 4 cols 0-1 both reference the "0" button (it spans 2 columns).
    pub nav_grid: Vec<Vec<Button>>,
    /// Operation buttons in order: [+, -, ×, ÷] for active-op highlighting.
    pub op_buttons: [Button; 4],
    /// Paired with their calculator action.
    pub action_buttons: Vec<(Button, ButtonAction)>,
}

impl CalculatorUI {
    pub fn navigate(&self, dir: Direction) {
        let (mut col, mut row) = self.find_focused().unwrap_or((0, 0));
        match dir {
            Direction::Left => col = col.saturating_sub(1),
            Direction::Right => col = (col + 1).min(3),
            Direction::Up => row = row.saturating_sub(1),
            Direction::Down => row = (row + 1).min(4),
        }
        self.nav_grid[row][col].grab_focus();
    }

    pub fn activate_focused(&self) {
        if let Some((col, row)) = self.find_focused() {
            self.nav_grid[row][col].clicked();
        }
    }

    fn find_focused(&self) -> Option<(usize, usize)> {
        for (row, buttons) in self.nav_grid.iter().enumerate() {
            for (col, button) in buttons.iter().enumerate() {
                if button.has_focus() {
                    return Some((col, row));
                }
            }
        }
        None
    }
}

pub fn build() -> CalculatorUI {
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Fredulator");
    window.set_default_size(300, 400);
    window.set_resizable(true);
    window.style_context().add_class("main-window");

    // Display
    let display = Entry::new();
    display.style_context().add_class("display-entry");
    display.set_editable(false);
    display.set_text("0");
    display.set_alignment(1.0);

    // Grid
    let grid = Grid::new();
    grid.style_context().add_class("calc-grid");
    grid.set_row_spacing(5);
    grid.set_column_spacing(5);
    grid.set_column_homogeneous(true);
    grid.set_row_homogeneous(true);

    let btn = |label: &str, class: &str| -> Button {
        let b = Button::with_label(label);
        b.style_context().add_class(class);
        b.set_hexpand(true);
        b.set_vexpand(true);
        b.set_can_focus(true);
        b
    };

    // Row 0: AC  +/−  %  ÷
    let ac = btn("AC", "clear-button");
    let plus_minus = btn("+/−", "op-button");
    let pct = btn("%", "op-button");
    let div = btn("÷", "op-button");
    grid.attach(&ac, 0, 0, 1, 1);
    grid.attach(&plus_minus, 1, 0, 1, 1);
    grid.attach(&pct, 2, 0, 1, 1);
    grid.attach(&div, 3, 0, 1, 1);

    // Row 1: 7  8  9  ×
    let d7 = btn("7", "digit-button");
    let d8 = btn("8", "digit-button");
    let d9 = btn("9", "digit-button");
    let mul = btn("×", "op-button");
    grid.attach(&d7, 0, 1, 1, 1);
    grid.attach(&d8, 1, 1, 1, 1);
    grid.attach(&d9, 2, 1, 1, 1);
    grid.attach(&mul, 3, 1, 1, 1);

    // Row 2: 4  5  6  −
    let d4 = btn("4", "digit-button");
    let d5 = btn("5", "digit-button");
    let d6 = btn("6", "digit-button");
    let sub = btn("−", "op-button");
    grid.attach(&d4, 0, 2, 1, 1);
    grid.attach(&d5, 1, 2, 1, 1);
    grid.attach(&d6, 2, 2, 1, 1);
    grid.attach(&sub, 3, 2, 1, 1);

    // Row 3: 1  2  3  +
    let d1 = btn("1", "digit-button");
    let d2 = btn("2", "digit-button");
    let d3 = btn("3", "digit-button");
    let add = btn("+", "op-button");
    grid.attach(&d1, 0, 3, 1, 1);
    grid.attach(&d2, 1, 3, 1, 1);
    grid.attach(&d3, 2, 3, 1, 1);
    grid.attach(&add, 3, 3, 1, 1);

    // Row 4: 0 (span 2)  .  =
    let d0 = btn("0", "digit-button");
    let dot = btn(".", "digit-button");
    let eq = btn("=", "equals-button");
    grid.attach(&d0, 0, 4, 2, 1);
    grid.attach(&dot, 2, 4, 1, 1);
    grid.attach(&eq, 3, 4, 1, 1);

    // Layout
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.set_hexpand(true);
    vbox.set_vexpand(true);
    display.set_hexpand(true);
    vbox.pack_start(&display, false, false, 0);
    vbox.pack_start(&grid, true, true, 0);
    window.add(&vbox);

    // Action buttons
    let action_buttons: Vec<(Button, ButtonAction)> = vec![
        (ac.clone(), ButtonAction::Clear),
        (plus_minus.clone(), ButtonAction::ToggleSign),
        (pct.clone(), ButtonAction::Percent),
        (div.clone(), ButtonAction::Operation(Operation::Divide)),
        (d7.clone(), ButtonAction::Digit('7')),
        (d8.clone(), ButtonAction::Digit('8')),
        (d9.clone(), ButtonAction::Digit('9')),
        (mul.clone(), ButtonAction::Operation(Operation::Multiply)),
        (d4.clone(), ButtonAction::Digit('4')),
        (d5.clone(), ButtonAction::Digit('5')),
        (d6.clone(), ButtonAction::Digit('6')),
        (sub.clone(), ButtonAction::Operation(Operation::Subtract)),
        (d1.clone(), ButtonAction::Digit('1')),
        (d2.clone(), ButtonAction::Digit('2')),
        (d3.clone(), ButtonAction::Digit('3')),
        (add.clone(), ButtonAction::Operation(Operation::Add)),
        (d0.clone(), ButtonAction::Digit('0')),
        (dot.clone(), ButtonAction::Decimal),
        (eq.clone(), ButtonAction::Equals),
    ];

    // Vim navigation grid 
        let nav_grid = vec![
        vec![ac, plus_minus, pct, div.clone()],
        vec![d7, d8, d9, mul.clone()],
        vec![d4, d5, d6, sub.clone()],
        vec![d1, d2, d3, add.clone()],
        vec![d0.clone(), d0, dot, eq],
    ];

    // Op buttons: [+, -, ×, ÷]
    let op_buttons = [add, sub, mul, div];

    CalculatorUI {
        window,
        display,
        nav_grid,
        op_buttons,
        action_buttons,
    }
}
