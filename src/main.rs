mod engine;
mod keyboard;
mod theme;
mod ui;

use crate::engine::Engine;
use crate::keyboard::Action;
use crate::theme::ThemeManager;
use crate::ui::ButtonAction;

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    gtk::init().expect("Failed to initialize GTK");

    let screen = gtk::gdk::Screen::default().expect("Failed to get default screen");
    let theme = Rc::new(RefCell::new(ThemeManager::new(screen)));
    let engine = Rc::new(RefCell::new(Engine::new()));
    let ui = ui::build();

    // Wire button click handlers
    for (button, action) in &ui.action_buttons {
        let eng = engine.clone();
        let display = ui.display.clone();
        let op_btns = ui.op_buttons.clone();
        let action = *action;
        button.connect_clicked(move |_| {
            let mut e = eng.borrow_mut();
            match action {
                ButtonAction::Digit(d) => e.input_digit(d),
                ButtonAction::Decimal => e.input_decimal(),
                ButtonAction::Operation(op) => e.set_operation(op),
                ButtonAction::Equals => e.calculate(),
                ButtonAction::Clear => e.clear(),
                ButtonAction::ToggleSign => e.toggle_sign(),
                ButtonAction::Percent => e.percent(),
            }
            display.set_text(&e.display_text());
            update_op_highlight(&op_btns, e.active_op());
        });
    }

    {
        let eng = engine.clone();
        let display = ui.display.clone();
        let op_btns = ui.op_buttons.clone();
        let theme = theme.clone();
        let window = ui.window.clone();
        let ui_ref = Rc::new(ui);
        let ui_for_keys = ui_ref.clone();

        ui_ref.window.connect_key_press_event(move |_, event| {
            let action = keyboard::map_key(event);

            match action {
                Action::Navigate(dir) => {
                    ui_for_keys.navigate(dir);
                }
                Action::Activate => {
                    ui_for_keys.activate_focused();
                }
                Action::ToggleTheme => {
                    theme.borrow_mut().toggle();
                }
                Action::Quit => {
                    window.close();
                }
                Action::None => {
                    return gtk::Inhibit(false);
                }
                calc_action => {
                    let mut e = eng.borrow_mut();
                    match calc_action {
                        Action::Digit(d) => e.input_digit(d),
                        Action::Decimal => e.input_decimal(),
                        Action::SetOperation(op) => e.set_operation(op),
                        Action::Equals => e.calculate(),
                        Action::Clear => e.clear(),
                        Action::Backspace => e.backspace(),
                        Action::ToggleSign => e.toggle_sign(),
                        Action::Percent => e.percent(),
                        _ => unreachable!(),
                    }
                    display.set_text(&e.display_text());
                    update_op_highlight(&op_btns, e.active_op());
                }
            }

            gtk::Inhibit(true)
        });

        ui_ref.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            gtk::Inhibit(false)
        });

        ui_ref.window.show_all();
    }

    gtk::main();
}

fn update_op_highlight(buttons: &[gtk::Button; 4], active: engine::Operation) {
    use engine::Operation;
    let ops = [
        Operation::Add,
        Operation::Subtract,
        Operation::Multiply,
        Operation::Divide,
    ];
    for (btn, op) in buttons.iter().zip(ops.iter()) {
        if *op == active {
            btn.style_context().add_class("active-op");
        } else {
            btn.style_context().remove_class("active-op");
        }
    }
}
