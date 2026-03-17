use gtk::prelude::*;
use gtk::{Button, Grid, Label, Window, WindowType};

use crate::eval::{BinaryOp, PostfixOp, UnaryFunc};
use crate::keyboard::Direction;

#[derive(Clone, Copy)]
pub enum ButtonAction {
    Digit(char),
    Decimal,
    BinaryOp(BinaryOp),
    UnaryFunc(UnaryFunc),
    PostfixOp(PostfixOp),
    Constant(f64, &'static str),
    LeftParen,
    RightParen,
    Equals,
    Clear,
    ToggleSign,
    EE,
    MemoryClear,
    MemoryRecall,
    MemoryAdd,
    MemorySubtract,
    ToggleAngleMode,
}

pub struct NavButton {
    pub button: Button,
    pub col: usize,
    pub row: usize,
    pub scientific: bool,
}

pub struct CalculatorUI {
    pub window: Window,
    pub expr_label: Label,
    pub result_label: Label,
    pub sci_btn: Button,
    pub sci_grid: Grid,
    pub nav_buttons: Vec<NavButton>,
    pub action_buttons: Vec<(Button, ButtonAction)>,
}

pub fn navigate(nav: &[NavButton], dir: Direction, scientific: bool) {
    let visible: Vec<&NavButton> = nav.iter().filter(|b| !b.scientific || scientific).collect();
    let current = visible.iter().find(|b| b.button.has_focus());
    if let Some(cur) = current {
        let (cc, cr) = eff_pos(cur, scientific);
        let target = match dir {
            Direction::Left => visible
                .iter()
                .filter(|b| eff_pos(b, scientific).1 == cr && eff_pos(b, scientific).0 < cc)
                .max_by_key(|b| eff_pos(b, scientific).0),
            Direction::Right => visible
                .iter()
                .filter(|b| eff_pos(b, scientific).1 == cr && eff_pos(b, scientific).0 > cc)
                .min_by_key(|b| eff_pos(b, scientific).0),
            Direction::Up => visible
                .iter()
                .filter(|b| eff_pos(b, scientific).0 == cc && eff_pos(b, scientific).1 < cr)
                .max_by_key(|b| eff_pos(b, scientific).1),
            Direction::Down => visible
                .iter()
                .filter(|b| eff_pos(b, scientific).0 == cc && eff_pos(b, scientific).1 > cr)
                .min_by_key(|b| eff_pos(b, scientific).1),
        };
        if let Some(t) = target {
            t.button.grab_focus();
        }
    } else if let Some(first) = visible.first() {
        first.button.grab_focus();
    }
}

pub fn activate_focused(nav: &[NavButton], scientific: bool) {
    let visible: Vec<&NavButton> = nav.iter().filter(|b| !b.scientific || scientific).collect();
    if let Some(b) = visible.iter().find(|b| b.button.has_focus()) {
        b.button.clicked();
    }
}

fn eff_pos(b: &NavButton, scientific: bool) -> (usize, usize) {
    if b.scientific {
        (b.col, b.row)
    } else if scientific {
        (b.col + 3, b.row)
    } else {
        (b.col, b.row)
    }
}

pub fn build() -> CalculatorUI {
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Fredulator");
    window.set_default_size(320, 540);
    window.set_resizable(true);
    window.style_context().add_class("main-window");

    // --- Display area ---
    let expr_label = Label::new(None);
    expr_label.style_context().add_class("expression-label");
    expr_label.set_xalign(1.0);
    expr_label.set_hexpand(true);
    expr_label.set_selectable(false);
    expr_label.set_visible(false);

    let result_label = Label::new(Some("0"));
    result_label.style_context().add_class("result-label");
    result_label.set_xalign(1.0);
    result_label.set_hexpand(true);
    result_label.set_vexpand(true);
    result_label.set_selectable(false);

    let sci_btn = Button::with_label("Sci");
    sci_btn.style_context().add_class("sci-toggle");
    sci_btn.set_can_focus(false);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header.pack_start(&sci_btn, false, false, 0);
    header.pack_end(&expr_label, true, true, 0);

    let display_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    display_box.style_context().add_class("display-area");
    display_box.pack_start(&header, false, false, 0);
    display_box.pack_start(&result_label, true, true, 0);

    // --- Buttons ---
    let mut action_buttons: Vec<(Button, ButtonAction)> = Vec::new();
    let mut nav_buttons: Vec<NavButton> = Vec::new();

    let mk = |label: &str,
              class: &str,
              action: ButtonAction,
              col: usize,
              row: usize,
              sci: bool,
              actions: &mut Vec<(Button, ButtonAction)>,
              navs: &mut Vec<NavButton>|
     -> Button {
        let b = Button::with_label(label);
        b.style_context().add_class(class);
        b.set_hexpand(true);
        b.set_vexpand(true);
        b.set_can_focus(true);
        actions.push((b.clone(), action));
        navs.push(NavButton {
            button: b.clone(),
            col,
            row,
            scientific: sci,
        });
        b
    };

    // === Scientific grid (3 cols x 6 rows) ===
    let sci_grid = Grid::new();
    sci_grid.style_context().add_class("sci-grid");
    sci_grid.set_row_spacing(6);
    sci_grid.set_column_spacing(6);
    sci_grid.set_column_homogeneous(true);
    sci_grid.set_row_homogeneous(true);

    let sci_btns: Vec<(&str, &str, ButtonAction, usize, usize)> = vec![
        ("(", "function-button", ButtonAction::LeftParen, 0, 0),
        (")", "function-button", ButtonAction::RightParen, 1, 0),
        ("Deg", "function-button", ButtonAction::ToggleAngleMode, 2, 0),
        ("x\u{00b2}", "function-button", ButtonAction::PostfixOp(PostfixOp::Square), 0, 1),
        ("x\u{00b3}", "function-button", ButtonAction::PostfixOp(PostfixOp::Cube), 1, 1),
        ("x\u{02b8}", "function-button", ButtonAction::BinaryOp(BinaryOp::Power), 2, 1),
        ("\u{215f}x", "function-button", ButtonAction::PostfixOp(PostfixOp::Reciprocal), 0, 2),
        ("\u{221a}", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Sqrt), 1, 2),
        ("\u{00b3}\u{221a}", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Cbrt), 2, 2),
        ("sin", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Sin), 0, 3),
        ("cos", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Cos), 1, 3),
        ("tan", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Tan), 2, 3),
        ("ln", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Ln), 0, 4),
        ("log", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Log10), 1, 4),
        ("n!", "function-button", ButtonAction::PostfixOp(PostfixOp::Factorial), 2, 4),
        ("\u{03c0}", "function-button", ButtonAction::Constant(std::f64::consts::PI, "\u{03c0}"), 0, 5),
        ("e", "function-button", ButtonAction::Constant(std::f64::consts::E, "e"), 1, 5),
        ("EE", "function-button", ButtonAction::EE, 2, 5),
    ];

    for (label, class, action, col, row) in sci_btns {
        let b = mk(label, class, action, col, row, true, &mut action_buttons, &mut nav_buttons);
        sci_grid.attach(&b, col as i32, row as i32, 1, 1);
    }

    // === Main grid (4 cols x 6 rows) ===
    let main_grid = Grid::new();
    main_grid.style_context().add_class("calc-grid");
    main_grid.set_row_spacing(6);
    main_grid.set_column_spacing(6);
    main_grid.set_column_homogeneous(true);
    main_grid.set_row_homogeneous(true);

    // Row 0: memory (low emphasis)
    let mem_btns: Vec<(&str, ButtonAction, usize)> = vec![
        ("MC", ButtonAction::MemoryClear, 0),
        ("MR", ButtonAction::MemoryRecall, 1),
        ("M+", ButtonAction::MemoryAdd, 2),
        ("M\u{2212}", ButtonAction::MemorySubtract, 3),
    ];
    for (label, action, col) in mem_btns {
        let b = mk(label, "memory-button", action, col, 0, false, &mut action_buttons, &mut nav_buttons);
        main_grid.attach(&b, col as i32, 0, 1, 1);
    }

    // Row 1: AC +/- % / (util buttons + operator)
    let b = mk("AC", "clear-button", ButtonAction::Clear, 0, 1, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 0, 1, 1, 1);
    let b = mk("+/\u{2212}", "util-button", ButtonAction::ToggleSign, 1, 1, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 1, 1, 1, 1);
    let b = mk("%", "util-button", ButtonAction::PostfixOp(PostfixOp::Percent), 2, 1, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 2, 1, 1, 1);
    let b = mk("\u{00f7}", "op-button", ButtonAction::BinaryOp(BinaryOp::Divide), 3, 1, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 1, 1, 1);

    // Row 2: 7 8 9 x
    for (i, d) in ['7', '8', '9'].iter().enumerate() {
        let b = mk(&d.to_string(), "digit-button", ButtonAction::Digit(*d), i, 2, false, &mut action_buttons, &mut nav_buttons);
        main_grid.attach(&b, i as i32, 2, 1, 1);
    }
    let b = mk("\u{00d7}", "op-button", ButtonAction::BinaryOp(BinaryOp::Multiply), 3, 2, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 2, 1, 1);

    // Row 3: 4 5 6 -
    for (i, d) in ['4', '5', '6'].iter().enumerate() {
        let b = mk(&d.to_string(), "digit-button", ButtonAction::Digit(*d), i, 3, false, &mut action_buttons, &mut nav_buttons);
        main_grid.attach(&b, i as i32, 3, 1, 1);
    }
    let b = mk("\u{2212}", "op-button", ButtonAction::BinaryOp(BinaryOp::Subtract), 3, 3, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 3, 1, 1);

    // Row 4: 1 2 3 +
    for (i, d) in ['1', '2', '3'].iter().enumerate() {
        let b = mk(&d.to_string(), "digit-button", ButtonAction::Digit(*d), i, 4, false, &mut action_buttons, &mut nav_buttons);
        main_grid.attach(&b, i as i32, 4, 1, 1);
    }
    let b = mk("+", "op-button", ButtonAction::BinaryOp(BinaryOp::Add), 3, 4, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 4, 1, 1);

    // Row 5: 0 (span 2) . =
    let d0 = mk("0", "digit-button", ButtonAction::Digit('0'), 0, 5, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&d0, 0, 5, 2, 1);
    nav_buttons.push(NavButton { button: d0, col: 1, row: 5, scientific: false });
    let b = mk(".", "digit-button", ButtonAction::Decimal, 2, 5, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 2, 5, 1, 1);
    let b = mk("=", "equals-button", ButtonAction::Equals, 3, 5, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 5, 1, 1);

    // --- Layout ---
    let grid_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    grid_box.pack_start(&sci_grid, true, true, 0);
    grid_box.pack_start(&main_grid, true, true, 0);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&display_box, false, false, 0);
    vbox.pack_start(&grid_box, true, true, 0);

    window.add(&vbox);

    CalculatorUI {
        window,
        expr_label,
        result_label,
        sci_btn,
        sci_grid,
        nav_buttons,
        action_buttons,
    }
}
