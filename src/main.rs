mod engine;
mod eval;
mod keyboard;
mod theme;
mod ui;

use crate::engine::Engine;
use crate::eval::AngleMode;
use crate::keyboard::Action;
use crate::theme::ThemeManager;
use crate::ui::ButtonAction;

use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn main() {
    gtk::init().expect("Failed to initialize GTK");

    let screen = gtk::gdk::Screen::default().expect("Failed to get default screen");
    let theme = Rc::new(RefCell::new(ThemeManager::new(screen)));
    let engine = Rc::new(RefCell::new(Engine::new()));
    let sci_mode = Rc::new(Cell::new(false));

    let calc_ui = ui::build();

    // Wire Sci toggle button
    {
        let sci_grid = calc_ui.sci_grid.clone();
        let window = calc_ui.window.clone();
        let sci_mode = sci_mode.clone();
        calc_ui.sci_btn.connect_clicked(move |_| {
            let mode = !sci_mode.get();
            sci_mode.set(mode);
            toggle_scientific(&sci_grid, &window, mode);
        });
    }

    // Wire calculator button click handlers
    for (button, action) in &calc_ui.action_buttons {
        let eng = engine.clone();
        let expr = calc_ui.expr_label.clone();
        let result = calc_ui.result_label.clone();
        let action = *action;

        button.connect_clicked(move |btn| {
            let mut e = eng.borrow_mut();
            match action {
                ButtonAction::Digit(d) => e.input_digit(d),
                ButtonAction::Decimal => e.input_decimal(),
                ButtonAction::BinaryOp(op) => e.input_binary_op(op),
                ButtonAction::UnaryFunc(f) => e.input_unary_func(f),
                ButtonAction::PostfixOp(op) => e.input_postfix_op(op),
                ButtonAction::Constant(val, name) => e.input_constant(val, name),
                ButtonAction::LeftParen => e.input_left_paren(),
                ButtonAction::RightParen => e.input_right_paren(),
                ButtonAction::Equals => e.calculate(),
                ButtonAction::Clear => e.clear(),
                ButtonAction::ToggleSign => e.toggle_sign(),
                ButtonAction::EE => e.input_ee(),
                ButtonAction::MemoryClear => e.memory_clear(),
                ButtonAction::MemoryRecall => e.memory_recall(),
                ButtonAction::MemoryAdd => e.memory_add(),
                ButtonAction::MemorySubtract => e.memory_subtract(),
                ButtonAction::ToggleAngleMode => {
                    e.toggle_angle_mode();
                    btn.set_label(match e.angle_mode() {
                        AngleMode::Degrees => "Deg",
                        AngleMode::Radians => "Rad",
                    });
                }
            }
            update_display(&e, &expr, &result);
        });
    }

    // Wire keyboard events
    {
        let eng = engine.clone();
        let expr = calc_ui.expr_label.clone();
        let result = calc_ui.result_label.clone();
        let theme = theme.clone();
        let window = calc_ui.window.clone();
        let sci_grid = calc_ui.sci_grid.clone();
        let sci_mode = sci_mode.clone();
        let nav_buttons = Rc::new(calc_ui.nav_buttons);
        let nav_ref = nav_buttons.clone();
        let ui_window = calc_ui.window.clone();

        ui_window.connect_key_press_event(move |_, event| {
            let action = keyboard::map_key(event);

            match action {
                Action::Navigate(dir) => {
                    ui::navigate(&nav_ref, dir, sci_mode.get());
                    return gtk::Inhibit(true);
                }
                Action::Activate => {
                    ui::activate_focused(&nav_ref, sci_mode.get());
                    return gtk::Inhibit(true);
                }
                Action::ToggleTheme => {
                    theme.borrow_mut().toggle();
                    return gtk::Inhibit(true);
                }
                Action::ToggleScientific => {
                    let mode = !sci_mode.get();
                    sci_mode.set(mode);
                    toggle_scientific(&sci_grid, &window, mode);
                    return gtk::Inhibit(true);
                }
                Action::Quit => {
                    window.close();
                    return gtk::Inhibit(true);
                }
                Action::None => return gtk::Inhibit(false),
                _ => {}
            }

            let mut e = eng.borrow_mut();
            match action {
                Action::Digit(d) => e.input_digit(d),
                Action::Decimal => e.input_decimal(),
                Action::BinaryOp(op) => e.input_binary_op(op),
                Action::PostfixOp(op) => e.input_postfix_op(op),
                Action::Equals => e.calculate(),
                Action::Clear => e.clear(),
                Action::Backspace => e.backspace(),
                Action::ToggleSign => e.toggle_sign(),
                Action::LeftParen => e.input_left_paren(),
                Action::RightParen => e.input_right_paren(),
                _ => unreachable!(),
            }
            update_display(&e, &expr, &result);

            gtk::Inhibit(true)
        });

        calc_ui.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            gtk::Inhibit(false)
        });

        calc_ui.window.show_all();
        calc_ui.sci_grid.hide();
    }

    gtk::main();
}

fn update_display(engine: &Engine, expr: &gtk::Label, result: &gtk::Label) {
    let main_text = engine.main_display_text();

    // Auto-size: shrink font for long content
    let ctx = result.style_context();
    ctx.remove_class("result-medium");
    ctx.remove_class("result-small");
    if main_text.len() > 14 {
        ctx.add_class("result-small");
    } else if main_text.len() > 8 {
        ctx.add_class("result-medium");
    }

    result.set_text(&main_text);

    if engine.show_secondary() {
        expr.set_text(&engine.secondary_display_text());
        expr.set_visible(true);
    } else {
        expr.set_text("");
        expr.set_visible(false);
    }
}

fn toggle_scientific(sci_grid: &gtk::Grid, window: &gtk::Window, mode: bool) {
    if mode {
        sci_grid.show_all();
        window.resize(540, 540);
    } else {
        sci_grid.hide();
        window.resize(320, 540);
    }
}
