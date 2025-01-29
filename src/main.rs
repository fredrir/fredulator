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
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let css_provider = CssProvider::new();

    css_provider
        .load_from_path("styles.css")
        .expect("Failed to load CSS");

    StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &css_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Fredulator");
    window.set_default_size(250, 300);

    window.set_widget_name("main-window");

    let calc_state = Rc::new(RefCell::new(CalculatorState::new()));

    let display = Entry::new();
    display.style_context().add_class("display-entry");
    display.set_editable(false);
    display.set_text("0");

    let grid = Grid::new();
    grid.set_row_spacing(5);
    grid.set_column_spacing(5);

    fn create_button(label: &str, grid: &Grid, row: i32, col: i32, style_class: &str) -> Button {
        let button = Button::with_label(label);
        grid.attach(&button, col, row, 1, 1);

        button.style_context().add_class(style_class);
        button
    }

    let digits = [
        ("7", 1, 0),
        ("8", 1, 1),
        ("9", 1, 2),
        ("4", 2, 0),
        ("5", 2, 1),
        ("6", 2, 2),
        ("1", 3, 0),
        ("2", 3, 1),
        ("3", 3, 2),
        ("0", 4, 0),
    ];
    for &(label, row, col) in &digits {
        let button = create_button(label, &grid, row, col, "digit-button");
        let label_string = label.to_string();

        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();

        button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            let digit = label_string.chars().next().unwrap();
            state.input_digit(digit);
            display_clone.set_text(&state.buffer);
        });
    }

    let operations = [
        ("+", Operation::Add, 1, 3),
        ("-", Operation::Subtract, 2, 3),
        ("*", Operation::Multiply, 3, 3),
        ("/", Operation::Divide, 4, 3),
    ];
    for &(label, op, row, col) in &operations {
        let button = create_button(label, &grid, row, col, "op-button");

        let op_clone = op;

        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();

        button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.set_operation(op_clone);
            display_clone.set_text(&state.current_value.to_string());
        });
    }

    // Create "=" button
    let equals_button = create_button("=", &grid, 4, 2, "equals-button");
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();

        equals_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            let result = state.calculate();
            display_clone.set_text(&result.to_string());
        });
    }

    // Create "C" (clear) button
    let clear_button = create_button("C", &grid, 4, 1, "clear-button");
    {
        let display_clone = display.clone();
        let calc_state_clone = calc_state.clone();

        clear_button.connect_clicked(move |_| {
            let mut state = calc_state_clone.borrow_mut();
            state.clear();
            display_clone.set_text("0");
        });
    }

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
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

    // Show everything
    window.show_all();

    // Start the main GTK loop
    gtk::main();
}
