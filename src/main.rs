mod convert;
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

struct Tab {
    engine: Engine,
    button: gtk::Button,
    name: String,
}

fn main() {
    gtk::init().expect("Failed to initialize GTK");

    let screen = gtk::gdk::Screen::default().expect("Failed to get default screen");
    let theme = Rc::new(RefCell::new(ThemeManager::new(screen)));
    let sci_mode = Rc::new(Cell::new(false));
    let current_mode = Rc::new(RefCell::new("calculator".to_string()));
    let conv_category = Rc::new(Cell::new(0usize));

    // Multi-tab engine system
    let tabs: Rc<RefCell<Vec<Tab>>> = Rc::new(RefCell::new(Vec::new()));
    let active_tab: Rc<Cell<usize>> = Rc::new(Cell::new(0));

    let calc_ui = ui::build();

    // Create initial tab
    {
        let mut t = tabs.borrow_mut();
        let btn = gtk::Button::with_label("Calc 1");
        btn.style_context().add_class("tab-button");
        btn.style_context().add_class("active");
        btn.set_can_focus(false);
        calc_ui.tab_bar.pack_start(&btn, false, false, 0);
        // Reorder so tab buttons come before + and menu
        calc_ui.tab_bar.reorder_child(&btn, 0);
        t.push(Tab {
            engine: Engine::new(),
            button: btn,
            name: "Calc 1".into(),
        });
    }

    // Helper: get current engine
    let get_engine = {
        let tabs = tabs.clone();
        let active = active_tab.clone();
        move |f: &dyn Fn(&mut Engine)| {
            let mut t = tabs.borrow_mut();
            let idx = active.get();
            if let Some(tab) = t.get_mut(idx) {
                f(&mut tab.engine);
            }
        }
    };
    let _ = get_engine; // used below via closures

    // ==================== TAB MANAGEMENT ====================
    // Add tab button
    {
        let tabs = tabs.clone();
        let active = active_tab.clone();
        let tab_bar = calc_ui.tab_bar.clone();
        let expr = calc_ui.expr_label.clone();
        let result = calc_ui.result_label.clone();
        let preview = calc_ui.preview_label.clone();

        calc_ui.tab_add_btn.connect_clicked(move |_| {
            let mut t = tabs.borrow_mut();
            let n = t.len() + 1;
            let name = format!("Calc {}", n);
            let btn = gtk::Button::with_label(&name);
            btn.style_context().add_class("tab-button");
            btn.set_can_focus(false);
            tab_bar.pack_start(&btn, false, false, 0);
            tab_bar.reorder_child(&btn, t.len() as i32);
            btn.show();

            // Deactivate old tab
            let old_idx = active.get();
            if let Some(old) = t.get(old_idx) {
                old.button.style_context().remove_class("active");
            }

            t.push(Tab {
                engine: Engine::new(),
                button: btn,
                name,
            });
            let new_idx = t.len() - 1;
            active.set(new_idx);
            t[new_idx].button.style_context().add_class("active");

            update_display(&t[new_idx].engine, &expr, &result, &preview);
        });
    }

    // ==================== PANEL TAB SWITCHING ====================
    {
        let stack = calc_ui.panel_stack.clone();
        let h_btn = calc_ui.panel_history_btn.clone();
        let m_btn = calc_ui.panel_memory_btn.clone();
        let p_btn = calc_ui.panel_pinned_btn.clone();

        let switch_panel = move |name: &str| {
            stack.set_visible_child_name(name);
            h_btn.style_context().remove_class("active");
            m_btn.style_context().remove_class("active");
            p_btn.style_context().remove_class("active");
            match name {
                "history" => h_btn.style_context().add_class("active"),
                "memory" => m_btn.style_context().add_class("active"),
                "pinned" => p_btn.style_context().add_class("active"),
                _ => {}
            }
        };

        let sw = switch_panel.clone();
        calc_ui.panel_history_btn.connect_clicked(move |_| {
            sw("history");
        });
        let sw = switch_panel.clone();
        calc_ui.panel_memory_btn.connect_clicked(move |_| {
            sw("memory");
        });
        let sw = switch_panel;
        calc_ui.panel_pinned_btn.connect_clicked(move |_| {
            sw("pinned");
        });
    }

    // ==================== MENU ACTIONS ====================
    {
        let sci_grid = calc_ui.sci_grid.clone();
        let window = calc_ui.window.clone();
        let sci_mode_c = sci_mode.clone();
        let popover = calc_ui.menu_popover.clone();
        calc_ui.menu_sci_btn.connect_clicked(move |_| {
            popover.popdown();
            let mode = !sci_mode_c.get();
            sci_mode_c.set(mode);
            toggle_scientific(&sci_grid, &window, mode);
        });
    }
    {
        let mode_stack = calc_ui.mode_stack.clone();
        let current_mode = current_mode.clone();
        let popover = calc_ui.menu_popover.clone();
        calc_ui.menu_notes_btn.connect_clicked(move |_| {
            popover.popdown();
            mode_stack.set_visible_child_name("notes");
            *current_mode.borrow_mut() = "notes".into();
        });
    }
    {
        let mode_stack = calc_ui.mode_stack.clone();
        let current_mode = current_mode.clone();
        let popover = calc_ui.menu_popover.clone();
        calc_ui.menu_converter_btn.connect_clicked(move |_| {
            popover.popdown();
            mode_stack.set_visible_child_name("converter");
            *current_mode.borrow_mut() = "converter".into();
        });
    }
    {
        let mode_stack = calc_ui.mode_stack.clone();
        let current_mode = current_mode.clone();
        let popover = calc_ui.menu_popover.clone();
        calc_ui.menu_tools_btn.connect_clicked(move |_| {
            popover.popdown();
            mode_stack.set_visible_child_name("tools");
            *current_mode.borrow_mut() = "tools".into();
        });
    }

    // Theme buttons in menu
    for (btn, idx) in &calc_ui.menu_theme_btns {
        let theme = theme.clone();
        let popover = calc_ui.menu_popover.clone();
        let theme_val = theme::Theme::ALL[*idx];
        btn.connect_clicked(move |_| {
            popover.popdown();
            theme.borrow_mut().set_theme(theme_val);
        });
    }

    // Back buttons
    {
        let mode_stack = calc_ui.mode_stack.clone();
        let current_mode = current_mode.clone();
        calc_ui.conv_back_btn.connect_clicked(move |_| {
            mode_stack.set_visible_child_name("calculator");
            *current_mode.borrow_mut() = "calculator".into();
        });
    }
    {
        let mode_stack = calc_ui.mode_stack.clone();
        let current_mode = current_mode.clone();
        calc_ui.tools_back_btn.connect_clicked(move |_| {
            mode_stack.set_visible_child_name("calculator");
            *current_mode.borrow_mut() = "calculator".into();
        });
    }
    {
        let mode_stack = calc_ui.mode_stack.clone();
        let current_mode = current_mode.clone();
        calc_ui.notes_back_btn.connect_clicked(move |_| {
            mode_stack.set_visible_child_name("calculator");
            *current_mode.borrow_mut() = "calculator".into();
        });
    }

    // ==================== CONVERTER LOGIC ====================
    {
        let entry = calc_ui.conv_value_entry.clone();
        let from_combo = calc_ui.conv_from_combo.clone();
        let to_combo = calc_ui.conv_to_combo.clone();
        let result_lbl = calc_ui.conv_result_label.clone();
        let cat = conv_category.clone();

        let do_convert = move || {
            let val: f64 = entry.text().parse().unwrap_or(0.0);
            let category = convert::Category::ALL[cat.get()];
            let from = from_combo.active_text().map(|s| s.to_string()).unwrap_or_default();
            let to = to_combo.active_text().map(|s| s.to_string()).unwrap_or_default();
            if !from.is_empty() && !to.is_empty() {
                let result = convert::convert(category, &from, &to, val);
                result_lbl.set_text(&eval::format_number(result));
            }
        };

        let dc = do_convert.clone();
        calc_ui.conv_value_entry.connect_changed(move |_| dc());
        let dc = do_convert.clone();
        calc_ui.conv_from_combo.connect_changed(move |_| dc());
        let dc = do_convert;
        calc_ui.conv_to_combo.connect_changed(move |_| dc());
    }

    // Category switching
    for (i, btn) in calc_ui.conv_cat_btns.iter().enumerate() {
        let cat = conv_category.clone();
        let from_combo = calc_ui.conv_from_combo.clone();
        let to_combo = calc_ui.conv_to_combo.clone();
        let result_lbl = calc_ui.conv_result_label.clone();
        let entry = calc_ui.conv_value_entry.clone();
        let all_btns: Vec<gtk::Button> = calc_ui.conv_cat_btns.clone();

        btn.connect_clicked(move |_| {
            cat.set(i);
            for b in &all_btns {
                b.style_context().remove_class("active");
            }
            all_btns[i].style_context().add_class("active");

            from_combo.remove_all();
            to_combo.remove_all();
            let category = convert::Category::ALL[i];
            for (abbr, _name) in category.units() {
                from_combo.append_text(abbr);
                to_combo.append_text(abbr);
            }
            from_combo.set_active(Some(0));
            to_combo.set_active(Some(1));

            let val: f64 = entry.text().parse().unwrap_or(1.0);
            let units = category.units();
            if units.len() >= 2 {
                let result = convert::convert(category, units[0].0, units[1].0, val);
                result_lbl.set_text(&eval::format_number(result));
            }
        });
    }

    // Swap button
    {
        let from = calc_ui.conv_from_combo.clone();
        let to = calc_ui.conv_to_combo.clone();
        calc_ui.conv_swap_btn.connect_clicked(move |_| {
            let f = from.active();
            let t = to.active();
            from.set_active(t);
            to.set_active(f);
        });
    }

    // ==================== TOOLS LOGIC ====================
    // Tip calculator
    {
        let amount_entry = calc_ui.tip_amount_entry.clone();
        let _custom_entry = calc_ui.tip_custom_entry.clone();
        let result_lbl = calc_ui.tip_result_label.clone();

        let calc_tip = move |pct: f64| {
            let amount: f64 = amount_entry.text().parse().unwrap_or(0.0);
            let tip = amount * pct / 100.0;
            result_lbl.set_text(&format!("Tip: {:.2}  |  Total: {:.2}", tip, amount + tip));
        };

        for (btn, pct) in &calc_ui.tip_pct_btns {
            let ct = calc_tip.clone();
            let pct = *pct;
            btn.connect_clicked(move |_| ct(pct));
        }

        let ct = calc_tip;
        calc_ui.tip_custom_entry.connect_changed(move |entry| {
            let pct: f64 = entry.text().parse().unwrap_or(0.0);
            ct(pct);
        });
    }

    // Discount calculator
    {
        let price_entry = calc_ui.discount_price_entry.clone();
        let pct_entry = calc_ui.discount_pct_entry.clone();
        let result_lbl = calc_ui.discount_result_label.clone();

        let calc_disc = move || {
            let price: f64 = price_entry.text().parse().unwrap_or(0.0);
            let pct: f64 = pct_entry.text().parse().unwrap_or(0.0);
            let savings = price * pct / 100.0;
            result_lbl.set_text(&format!(
                "Save: {:.2}  |  Final: {:.2}",
                savings,
                price - savings
            ));
        };

        let cd = calc_disc.clone();
        calc_ui.discount_price_entry.connect_changed(move |_| cd());
        let cd = calc_disc;
        calc_ui.discount_pct_entry.connect_changed(move |_| cd());
    }

    // Tax calculator
    {
        let amount_entry = calc_ui.tax_amount_entry.clone();
        let rate_entry = calc_ui.tax_rate_entry.clone();
        let result_lbl = calc_ui.tax_result_label.clone();

        let calc_tax = move || {
            let amount: f64 = amount_entry.text().parse().unwrap_or(0.0);
            let rate: f64 = rate_entry.text().parse().unwrap_or(0.0);
            let tax = amount * rate / 100.0;
            result_lbl.set_text(&format!(
                "Tax: {:.2}  |  Total: {:.2}",
                tax,
                amount + tax
            ));
        };

        let ct = calc_tax.clone();
        calc_ui.tax_amount_entry.connect_changed(move |_| ct());
        let ct = calc_tax;
        calc_ui.tax_rate_entry.connect_changed(move |_| ct());
    }

    // ==================== MATH NOTES ====================
    {
        let result_lbl = calc_ui.notes_result_label.clone();
        let textview = calc_ui.notes_textview.clone();

        if let Some(buf) = textview.buffer() {
            buf.connect_changed(move |buf| {
                let text = buf
                    .text(&buf.start_iter(), &buf.end_iter(), false)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let mut results = Vec::new();
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                        results.push(String::new());
                        continue;
                    }
                    match eval::parse_expression(line) {
                        Ok(tokens) if !tokens.is_empty() => {
                            match eval::evaluate(&tokens, AngleMode::Degrees) {
                                Ok(val) => results.push(format!("= {}", eval::format_number(val))),
                                Err(e) => results.push(format!("  {}", e)),
                            }
                        }
                        _ => results.push(String::new()),
                    }
                }
                result_lbl.set_text(&results.join("\n"));
            });
        }
    }

    // ==================== CALCULATOR BUTTONS ====================
    for (button, action) in &calc_ui.action_buttons {
        let tabs_c = tabs.clone();
        let active_c = active_tab.clone();
        let expr = calc_ui.expr_label.clone();
        let result = calc_ui.result_label.clone();
        let preview = calc_ui.preview_label.clone();
        let action = *action;

        button.connect_clicked(move |btn| {
            let mut t = tabs_c.borrow_mut();
            let idx = active_c.get();
            if let Some(tab) = t.get_mut(idx) {
                let e = &mut tab.engine;
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
                update_display(e, &expr, &result, &preview);
            }
        });
    }

    // ==================== KEYBOARD EVENTS ====================
    {
        let tabs_c = tabs.clone();
        let active_c = active_tab.clone();
        let expr = calc_ui.expr_label.clone();
        let result = calc_ui.result_label.clone();
        let preview = calc_ui.preview_label.clone();
        let theme_c = theme.clone();
        let window = calc_ui.window.clone();
        let sci_grid = calc_ui.sci_grid.clone();
        let sci_mode_c = sci_mode.clone();
        let nav_buttons = Rc::new(calc_ui.nav_buttons);
        let nav_ref = nav_buttons.clone();
        let panel_revealer = calc_ui.panel_revealer.clone();
        let panel_stack = calc_ui.panel_stack.clone();
        let mode_stack = calc_ui.mode_stack.clone();
        let current_mode_c = current_mode.clone();
        let tab_bar = calc_ui.tab_bar.clone();
        let menu_popover = calc_ui.menu_popover.clone();
        let history_list = calc_ui.history_list.clone();
        let memory_list = calc_ui.memory_list.clone();
        let pinned_list = calc_ui.pinned_list.clone();
        let p_history_btn = calc_ui.panel_history_btn.clone();
        let p_memory_btn = calc_ui.panel_memory_btn.clone();
        let p_pinned_btn = calc_ui.panel_pinned_btn.clone();
        let angle_btn = calc_ui.angle_btn.clone();

        calc_ui.window.connect_key_press_event(move |_, event| {
            let action = keyboard::map_key(event);

            match action {
                Action::Navigate(dir) => {
                    if *current_mode_c.borrow() == "calculator" {
                        ui::navigate(&nav_ref, dir, sci_mode_c.get());
                    }
                    return gtk::Inhibit(true);
                }
                Action::Activate => {
                    if *current_mode_c.borrow() == "calculator" {
                        ui::activate_focused(&nav_ref, sci_mode_c.get());
                    }
                    return gtk::Inhibit(true);
                }
                Action::ToggleTheme => {
                    theme_c.borrow_mut().toggle();
                    return gtk::Inhibit(true);
                }
                Action::ToggleScientific => {
                    let mode = !sci_mode_c.get();
                    sci_mode_c.set(mode);
                    toggle_scientific(&sci_grid, &window, mode);
                    return gtk::Inhibit(true);
                }
                Action::Quit => {
                    window.close();
                    return gtk::Inhibit(true);
                }
                Action::Undo => {
                    let mut t = tabs_c.borrow_mut();
                    let idx = active_c.get();
                    if let Some(tab) = t.get_mut(idx) {
                        tab.engine.undo();
                        update_display(&tab.engine, &expr, &result, &preview);
                    }
                    return gtk::Inhibit(true);
                }
                Action::NewTab => {
                    let mut t = tabs_c.borrow_mut();
                    let n = t.len() + 1;
                    let name = format!("Calc {}", n);
                    let btn = gtk::Button::with_label(&name);
                    btn.style_context().add_class("tab-button");
                    btn.set_can_focus(false);
                    tab_bar.pack_start(&btn, false, false, 0);
                    tab_bar.reorder_child(&btn, t.len() as i32);
                    btn.show();

                    let old_idx = active_c.get();
                    if let Some(old) = t.get(old_idx) {
                        old.button.style_context().remove_class("active");
                    }
                    t.push(Tab {
                        engine: Engine::new(),
                        button: btn,
                        name,
                    });
                    let new_idx = t.len() - 1;
                    active_c.set(new_idx);
                    t[new_idx].button.style_context().add_class("active");
                    update_display(&t[new_idx].engine, &expr, &result, &preview);
                    return gtk::Inhibit(true);
                }
                Action::CloseTab => {
                    let mut t = tabs_c.borrow_mut();
                    if t.len() <= 1 {
                        return gtk::Inhibit(true);
                    }
                    let idx = active_c.get();
                    let removed = t.remove(idx);
                    tab_bar.remove(&removed.button);
                    let new_idx = if idx >= t.len() { t.len() - 1 } else { idx };
                    active_c.set(new_idx);
                    for (i, tab) in t.iter().enumerate() {
                        if i == new_idx {
                            tab.button.style_context().add_class("active");
                        } else {
                            tab.button.style_context().remove_class("active");
                        }
                    }
                    update_display(&t[new_idx].engine, &expr, &result, &preview);
                    return gtk::Inhibit(true);
                }
                Action::NextTab => {
                    let t = tabs_c.borrow();
                    if t.len() <= 1 {
                        return gtk::Inhibit(true);
                    }
                    let old = active_c.get();
                    let new_idx = (old + 1) % t.len();
                    t[old].button.style_context().remove_class("active");
                    t[new_idx].button.style_context().add_class("active");
                    active_c.set(new_idx);
                    update_display(&t[new_idx].engine, &expr, &result, &preview);
                    return gtk::Inhibit(true);
                }
                Action::PrevTab => {
                    let t = tabs_c.borrow();
                    if t.len() <= 1 {
                        return gtk::Inhibit(true);
                    }
                    let old = active_c.get();
                    let new_idx = if old == 0 { t.len() - 1 } else { old - 1 };
                    t[old].button.style_context().remove_class("active");
                    t[new_idx].button.style_context().add_class("active");
                    active_c.set(new_idx);
                    update_display(&t[new_idx].engine, &expr, &result, &preview);
                    return gtk::Inhibit(true);
                }
                Action::ToggleHistory => {
                    toggle_panel(&panel_revealer, &panel_stack, "history", &p_history_btn, &p_memory_btn, &p_pinned_btn);
                    refresh_history(&tabs_c.borrow(), active_c.get(), &history_list);
                    return gtk::Inhibit(true);
                }
                Action::ToggleMemory => {
                    toggle_panel(&panel_revealer, &panel_stack, "memory", &p_history_btn, &p_memory_btn, &p_pinned_btn);
                    refresh_memory(&tabs_c.borrow(), active_c.get(), &memory_list);
                    return gtk::Inhibit(true);
                }
                Action::TogglePinned => {
                    toggle_panel(&panel_revealer, &panel_stack, "pinned", &p_history_btn, &p_memory_btn, &p_pinned_btn);
                    refresh_pinned(&tabs_c.borrow(), active_c.get(), &pinned_list);
                    return gtk::Inhibit(true);
                }
                Action::PinResult => {
                    let mut t = tabs_c.borrow_mut();
                    let idx = active_c.get();
                    if let Some(tab) = t.get_mut(idx) {
                        let count = tab.engine.pinned.len() + 1;
                        tab.engine.pin_result(format!("Pin {}", count));
                    }
                    return gtk::Inhibit(true);
                }
                Action::MemoryStore => {
                    let mut t = tabs_c.borrow_mut();
                    let idx = active_c.get();
                    if let Some(tab) = t.get_mut(idx) {
                        let count = tab.engine.memory_slots.len() + 1;
                        tab.engine.memory_store(format!("M{}", count));
                    }
                    return gtk::Inhibit(true);
                }
                Action::OpenConverter => {
                    mode_stack.set_visible_child_name("converter");
                    *current_mode_c.borrow_mut() = "converter".into();
                    return gtk::Inhibit(true);
                }
                Action::OpenTools => {
                    mode_stack.set_visible_child_name("tools");
                    *current_mode_c.borrow_mut() = "tools".into();
                    return gtk::Inhibit(true);
                }
                Action::OpenNotes => {
                    mode_stack.set_visible_child_name("notes");
                    *current_mode_c.borrow_mut() = "notes".into();
                    return gtk::Inhibit(true);
                }
                Action::OpenMenu => {
                    menu_popover.popup();
                    return gtk::Inhibit(true);
                }
                Action::BackToCalc => {
                    let mode = current_mode_c.borrow().clone();
                    if mode != "calculator" {
                        mode_stack.set_visible_child_name("calculator");
                        *current_mode_c.borrow_mut() = "calculator".into();
                        return gtk::Inhibit(true);
                    }
                    // In calculator mode, Escape = clear
                    let mut t = tabs_c.borrow_mut();
                    let idx = active_c.get();
                    if let Some(tab) = t.get_mut(idx) {
                        tab.engine.clear();
                        update_display(&tab.engine, &expr, &result, &preview);
                    }
                    return gtk::Inhibit(true);
                }
                Action::None => {
                    // In non-calculator modes, don't intercept keys
                    if *current_mode_c.borrow() != "calculator" {
                        return gtk::Inhibit(false);
                    }
                    return gtk::Inhibit(false);
                }
                _ => {
                    // For calculator input actions, only process in calculator mode
                    if *current_mode_c.borrow() != "calculator" {
                        return gtk::Inhibit(false);
                    }
                }
            }

            let mut t = tabs_c.borrow_mut();
            let idx = active_c.get();
            if let Some(tab) = t.get_mut(idx) {
                let e = &mut tab.engine;
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
                    _ => {}
                }
                update_display(e, &expr, &result, &preview);

                // Update angle button label if it exists
                if let Some(ref abtn) = angle_btn {
                    abtn.set_label(match e.angle_mode() {
                        AngleMode::Degrees => "Deg",
                        AngleMode::Radians => "Rad",
                    });
                }
            }

            gtk::Inhibit(true)
        });

        calc_ui.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            gtk::Inhibit(false)
        });

        calc_ui.window.show_all();
        calc_ui.sci_grid.hide();
        calc_ui.panel_revealer.set_reveal_child(false);
    }

    gtk::main();
}

fn update_display(engine: &Engine, expr: &gtk::Label, result: &gtk::Label, preview: &gtk::Label) {
    let main_text = engine.main_display_text();

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

    // Auto-eval preview
    if let Some(preview_text) = engine.auto_eval() {
        preview.set_text(&format!("\u{2248} {}", preview_text));
        preview.set_visible(true);
    } else {
        preview.set_text("");
        preview.set_visible(false);
    }
}

fn toggle_scientific(sci_grid: &gtk::Grid, window: &gtk::Window, mode: bool) {
    if mode {
        sci_grid.show_all();
        window.resize(580, 580);
    } else {
        sci_grid.hide();
        window.resize(360, 580);
    }
}

fn toggle_panel(
    revealer: &gtk::Revealer,
    stack: &gtk::Stack,
    name: &str,
    h_btn: &gtk::Button,
    m_btn: &gtk::Button,
    p_btn: &gtk::Button,
) {
    let currently_visible = revealer.reveals_child();
    let current_name = stack
        .visible_child_name()
        .map(|s| s.to_string())
        .unwrap_or_default();

    if currently_visible && current_name == name {
        revealer.set_reveal_child(false);
    } else {
        stack.set_visible_child_name(name);
        revealer.set_reveal_child(true);
        h_btn.style_context().remove_class("active");
        m_btn.style_context().remove_class("active");
        p_btn.style_context().remove_class("active");
        match name {
            "history" => h_btn.style_context().add_class("active"),
            "memory" => m_btn.style_context().add_class("active"),
            "pinned" => p_btn.style_context().add_class("active"),
            _ => {}
        }
    }
}

fn refresh_history(tabs: &[Tab], active: usize, list: &gtk::Box) {
    for child in list.children() {
        list.remove(&child);
    }
    if let Some(tab) = tabs.get(active) {
        if tab.engine.history.is_empty() {
            let empty = gtk::Label::new(Some("No calculations yet"));
            empty.style_context().add_class("panel-empty");
            list.pack_start(&empty, false, false, 0);
        } else {
            for entry in tab.engine.history.iter().rev() {
                let item = gtk::Box::new(gtk::Orientation::Vertical, 2);
                item.style_context().add_class("panel-item");
                item.set_margin_bottom(2);

                let expr_lbl = gtk::Label::new(Some(&entry.expression));
                expr_lbl.style_context().add_class("panel-item-expr");
                expr_lbl.set_xalign(1.0);
                expr_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);

                let res_lbl = gtk::Label::new(Some(&format!("= {}", entry.result_text)));
                res_lbl.style_context().add_class("panel-item-result");
                res_lbl.set_xalign(1.0);

                item.pack_start(&expr_lbl, false, false, 0);
                item.pack_start(&res_lbl, false, false, 0);
                list.pack_start(&item, false, false, 0);
            }
        }
    }
    list.show_all();
}

fn refresh_memory(tabs: &[Tab], active: usize, list: &gtk::Box) {
    for child in list.children() {
        list.remove(&child);
    }
    if let Some(tab) = tabs.get(active) {
        // Show quick memory
        if tab.engine.has_memory() {
            let item = gtk::Box::new(gtk::Orientation::Vertical, 2);
            item.style_context().add_class("panel-item");
            item.set_margin_bottom(2);
            let lbl = gtk::Label::new(Some("Quick Memory (M+/M-)"));
            lbl.style_context().add_class("panel-item-label");
            lbl.set_xalign(0.0);
            item.pack_start(&lbl, false, false, 0);
            list.pack_start(&item, false, false, 0);
        }

        if tab.engine.memory_slots.is_empty() && !tab.engine.has_memory() {
            let empty = gtk::Label::new(Some("No stored values\n\nPress S to store current value\nUse M+/M- in scientific mode"));
            empty.style_context().add_class("panel-empty");
            list.pack_start(&empty, false, false, 0);
        } else {
            for slot in &tab.engine.memory_slots {
                let item = gtk::Box::new(gtk::Orientation::Vertical, 2);
                item.style_context().add_class("panel-item");
                item.set_margin_bottom(2);

                let lbl = gtk::Label::new(Some(&slot.label));
                lbl.style_context().add_class("panel-item-label");
                lbl.set_xalign(0.0);

                let val = gtk::Label::new(Some(&eval::format_number(slot.value)));
                val.style_context().add_class("panel-item-result");
                val.set_xalign(1.0);

                item.pack_start(&lbl, false, false, 0);
                item.pack_start(&val, false, false, 0);
                list.pack_start(&item, false, false, 0);
            }
        }
    }
    list.show_all();
}

fn refresh_pinned(tabs: &[Tab], active: usize, list: &gtk::Box) {
    for child in list.children() {
        list.remove(&child);
    }
    if let Some(tab) = tabs.get(active) {
        if tab.engine.pinned.is_empty() {
            let empty = gtk::Label::new(Some("No pinned results\n\nPress Ctrl+S to pin"));
            empty.style_context().add_class("panel-empty");
            list.pack_start(&empty, false, false, 0);
        } else {
            for pin in &tab.engine.pinned {
                let item = gtk::Box::new(gtk::Orientation::Vertical, 2);
                item.style_context().add_class("panel-item");
                item.set_margin_bottom(2);

                let lbl = gtk::Label::new(Some(&pin.label));
                lbl.style_context().add_class("panel-item-label");
                lbl.set_xalign(0.0);

                let expr = gtk::Label::new(Some(&pin.expression));
                expr.style_context().add_class("panel-item-expr");
                expr.set_xalign(1.0);
                expr.set_ellipsize(gtk::pango::EllipsizeMode::End);

                let val = gtk::Label::new(Some(&format!("= {}", eval::format_number(pin.result))));
                val.style_context().add_class("panel-item-result");
                val.set_xalign(1.0);

                item.pack_start(&lbl, false, false, 0);
                item.pack_start(&expr, false, false, 0);
                item.pack_start(&val, false, false, 0);
                list.pack_start(&item, false, false, 0);
            }
        }
    }
    list.show_all();
}
