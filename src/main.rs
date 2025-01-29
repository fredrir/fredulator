use gtk::gdk;
use gtk::prelude::*;
use gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
use gtk::{Button, CssProvider, Entry, Grid, StyleContext, Window, WindowType};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    None,
}

#[derive(Debug)]
struct CalculatorState {
    current_value: f64,
    buffer: String,
    current_op: Operation,
}

impl CalculatorState {
    fn new() -> Self {
        Self {
            current_value: 0.0,
            buffer: String::new(),
            current_op: Operation::None,
        }
    }

    fn clear(&mut self) {
        self.current_value = 0.0;
        self.buffer.clear();
        self.current_op = Operation::None;
    }

    fn set_operation(&mut self, op: Operation) {
        if !self.buffer.is_empty() {
            self.current_value = self.buffer.parse().unwrap_or(0.0);
            self.buffer.clear();
        }
        self.current_op = op;
    }

    fn input_digit(&mut self, digit: char) {
        self.buffer.push(digit);
    }

    fn calculate(&mut self) -> f64 {
        let new_val = if self.buffer.is_empty() {
            0.0
        } else {
            self.buffer.parse().unwrap_or(0.0)
        };

        self.buffer.clear();

        self.current_value = match self.current_op {
            Operation::Add => self.current_value + new_val,
            Operation::Subtract => self.current_value - new_val,
            Operation::Multiply => self.current_value * new_val,
            Operation::Divide => {
                if new_val.abs() < f64::EPSILON {
                    0.0
                } else {
                    self.current_value / new_val
                }
            }
            Operation::None => new_val,
        };

        self.current_op = Operation::None;
        self.current_value
    }

    fn toggle_sign(&mut self) {
        if !self.buffer.is_empty() {
            if let Ok(val) = self.buffer.parse::<f64>() {
                self.buffer = (-val).to_string();
            }
        } else {
            self.current_value = -self.current_value;
        }
    }

    fn percent(&mut self) {
        if !self.buffer.is_empty() {
            if let Ok(val) = self.buffer.parse::<f64>() {
                self.buffer = (val / 100.0).to_string();
            }
        } else {
            self.current_value /= 100.0;
        }
    }
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    // (Optional) Load CSS
    let css_provider = CssProvider::new();
    css_provider.load_from_path("styles.css").ok();
    StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &css_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Fredulator");
    window.set_default_size(300, 400);
    window.set_resizable(true);
    window.style_context().add_class("main-window");

    let calc_state = Rc::new(RefCell::new(CalculatorState::new()));

    // Display Entry
    let display = Entry::new();
    display.style_context().add_class("display-entry");
    display.set_editable(false);
    display.set_text("0");

    // Grid
    let grid = Grid::new();
    grid.style_context().add_class("calc-grid");
    grid.set_row_spacing(5);
    grid.set_column_spacing(5);
    grid.set_column_homogeneous(true);
    grid.set_row_homogeneous(true);

    fn create_button(label: &str, style_class: &str) -> Button {
        let button = Button::with_label(label);
        button.style_context().add_class(style_class);
        button
    }

    fn attach_button(grid: &Grid, button: &Button, left: i32, top: i32, width: i32, height: i32) {
        button.set_hexpand(true);
        button.set_vexpand(true);
        grid.attach(button, left, top, width, height);
    }

    let ac_button = create_button("AC", "clear-button");
    attach_button(&grid, &ac_button, 0, 0, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        ac_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.clear();
            display_clone.set_text("0");
        });
    }

    let plus_minus_button = create_button("+/-", "op-button");
    attach_button(&grid, &plus_minus_button, 1, 0, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        plus_minus_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.toggle_sign();
            if !state.buffer.is_empty() {
                display_clone.set_text(&state.buffer);
            } else {
                display_clone.set_text(&state.current_value.to_string());
            }
        });
    }

    let percent_button = create_button("%", "op-button");
    attach_button(&grid, &percent_button, 2, 0, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        percent_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.percent();
            if !state.buffer.is_empty() {
                display_clone.set_text(&state.buffer);
            } else {
                display_clone.set_text(&state.current_value.to_string());
            }
        });
    }

    let divide_button = create_button("/", "op-button");
    attach_button(&grid, &divide_button, 3, 0, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        divide_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.set_operation(Operation::Divide);
            display_clone.set_text(&state.current_value.to_string());
        });
    }

    for (col, digit) in ["7", "8", "9"].iter().enumerate() {
        let button = create_button(digit, "digit-button");
        attach_button(&grid, &button, col as i32, 1, 1, 1);

        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        let d = digit.to_string();
        button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.input_digit(d.chars().next().unwrap());
            display_clone.set_text(&state.buffer);
        });
    }

    let multiply_button = create_button("×", "op-button");
    attach_button(&grid, &multiply_button, 3, 1, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        multiply_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.set_operation(Operation::Multiply);
            display_clone.set_text(&state.current_value.to_string());
        });
    }

    for (col, digit) in ["4", "5", "6"].iter().enumerate() {
        let button = create_button(digit, "digit-button");
        attach_button(&grid, &button, col as i32, 2, 1, 1);

        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        let d = digit.to_string();
        button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.input_digit(d.chars().next().unwrap());
            display_clone.set_text(&state.buffer);
        });
    }

    let subtract_button = create_button("-", "op-button");
    attach_button(&grid, &subtract_button, 3, 2, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        subtract_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.set_operation(Operation::Subtract);
            display_clone.set_text(&state.current_value.to_string());
        });
    }

    for (col, digit) in ["1", "2", "3"].iter().enumerate() {
        let button = create_button(digit, "digit-button");
        attach_button(&grid, &button, col as i32, 3, 1, 1);

        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        let d = digit.to_string();
        button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.input_digit(d.chars().next().unwrap());
            display_clone.set_text(&state.buffer);
        });
    }

    let add_button = create_button("+", "op-button");
    attach_button(&grid, &add_button, 3, 3, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        add_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.set_operation(Operation::Add);
            display_clone.set_text(&state.current_value.to_string());
        });
    }

    //
    // Fifth row: 0  ., =
    //
    let zero_button = create_button("0", "digit-button");
    attach_button(&grid, &zero_button, 0, 4, 2, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        zero_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.input_digit('0');
            display_clone.set_text(&state.buffer);
        });
    }

    let decimal_button = create_button(".", "digit-button");
    attach_button(&grid, &decimal_button, 2, 4, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        decimal_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            if !state.buffer.contains('.') {
                state.input_digit('.');
            }
            display_clone.set_text(&state.buffer);
        });
    }

    let equals_button = create_button("=", "equals-button");
    attach_button(&grid, &equals_button, 3, 4, 1, 1);
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();
        equals_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            let result = state.calculate();
            display_clone.set_text(&result.to_string());
        });
    }

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.set_hexpand(true);
    vbox.set_vexpand(true);

    display.set_hexpand(true);

    vbox.pack_start(&display, false, false, 0);
    vbox.pack_start(&grid, true, true, 0);

    window.add(&vbox);

    window.connect_delete_event(|win, _| {
        unsafe {
            win.destroy();
        }
        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all();
    gtk::main();
}
