mod app;
mod domain;
mod services;
mod ui;

use crate::app::message::Message;
use crate::app::state::{AppState, ModePanel, Panel};
use crate::app::update::{self, SideEffect};
use crate::domain::types::{AngleMode, ConvertCategory};
use crate::services::theme::{Theme, ThemeManager};
use crate::ui::builder::{ButtonAction, CalculatorUI};
use crate::ui::navigation::NavButton;

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const HELP_TEXT: &str = "Calculator:  0-9 digits, + - * / ^ operators\n\
  = / Enter    Calculate\n\
  Backspace    Delete last\n\
  Escape       Clear / close panel\n\
  n            Negate (+/-)\n\
  h j k l      Vim navigation\n\
  Space        Activate button\n\
  s            Toggle scientific\n\
  t            Cycle theme\n\
  u            Undo\n\
  ;            Open menu\n\
  S            Store to memory\n\n\
Tabs:  Ctrl+T new, Ctrl+W close, Tab/Shift+Tab switch\n\n\
Panels:  Ctrl+H history, Ctrl+M memory, Ctrl+P pinned\n\
  Ctrl+S pin result, Ctrl+Shift+E export\n\n\
Modes:  Ctrl+E converter, Ctrl+R tools, Ctrl+N notes\n\n\
  q quit, ? help";

fn main() {
    gtk::init().expect("Failed to initialize GTK");

    let config = services::config::load();

    ui::keyboard::init_keymap(&config.keybindings);

    let session_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let state = Rc::new(RefCell::new(AppState::new(config, session_id)));

    update::restore_session(&mut state.borrow_mut());

    let screen = gtk::gdk::Screen::default().expect("Failed to get default screen");
    let theme_mgr = {
        let s = state.borrow();
        ThemeManager::new(screen, &s.config.theme, &s.config.layout, &s.config.feedback)
    };
    let theme_mgr = Rc::new(RefCell::new(theme_mgr));

    let mut calc_ui = {
        let s = state.borrow();
        ui::builder::build(&s.config)
    };

    let nav_buttons = Rc::new(std::mem::take(&mut calc_ui.nav_buttons));

    rebuild_tab_bar(&state, &calc_ui);

    wire_action_buttons(&state, &calc_ui, &theme_mgr, &nav_buttons);
    wire_panel_buttons(&state, &calc_ui, &theme_mgr, &nav_buttons);
    wire_menu_buttons(&state, &calc_ui, &theme_mgr, &nav_buttons);
    wire_converter(&state, &calc_ui);
    wire_tools(&calc_ui);
    wire_notes(&calc_ui, &state);
    wire_keyboard(&state, &calc_ui, &theme_mgr, &nav_buttons);
    wire_window_close(&state, &calc_ui);

    let wcfg = &state.borrow().config.window.clone();
    if wcfg.always_on_top {
        calc_ui.window.set_keep_above(true);
    }
    if wcfg.opacity < 1.0 && wcfg.opacity > 0.0 {
        calc_ui.window.set_opacity(wcfg.opacity);
    }
    if wcfg.compact_mode {
        calc_ui.window.set_decorated(false);
    }
    if wcfg.remember_geometry {
        if let Some((x, y, w, h)) = services::session::load_geometry() {
            calc_ui.window.move_(x, y);
            calc_ui.window.resize(w, h);
        }
    }

    {
        let s = state.borrow();
        if s.scientific_mode {
            calc_ui.menu_sci_btn.style_context().add_class("active");
            calc_ui.menu_basic_btn.style_context().remove_class("active");
        }
    }

    calc_ui.window.show_all();

    {
        let s = state.borrow();
        if !s.scientific_mode {
            calc_ui.sci_grid.hide();
        }
    }
    calc_ui.panel_revealer.set_reveal_child(false);
    calc_ui.mode_panel_revealer.set_reveal_child(false);

    update_display(&state.borrow(), &calc_ui);

    gtk::main();
}

fn update_display(state: &AppState, calc_ui: &CalculatorUI) {
    let engine = state.engine();
    let main_text = engine.main_display_text();

    let ctx = calc_ui.result_label.style_context();
    ctx.remove_class("result-medium");
    ctx.remove_class("result-small");
    if main_text.len() > 12 {
        ctx.add_class("result-small");
    } else if main_text.len() > 7 {
        ctx.add_class("result-medium");
    }

    calc_ui.result_label.set_text(&main_text);

    if engine.show_secondary() {
        calc_ui.expr_label.set_text(&engine.secondary_display_text());
        calc_ui.expr_label.set_opacity(1.0);
    } else {
        calc_ui.expr_label.set_text(" ");
        calc_ui.expr_label.set_opacity(0.0);
    }

    if let Some(preview_text) = engine.auto_eval() {
        calc_ui.preview_label.set_text(&format!("\u{2248} {}", preview_text));
        calc_ui.preview_label.set_opacity(1.0);
    } else {
        calc_ui.preview_label.set_text(" ");
        calc_ui.preview_label.set_opacity(0.0);
    }
}

fn rebuild_tab_bar(state: &Rc<RefCell<AppState>>, calc_ui: &CalculatorUI) {
    for child in calc_ui.tab_bar.children() {
        if child != calc_ui.tab_add_btn && child.style_context().has_class("tab-button") {
            calc_ui.tab_bar.remove(&child);
        }
    }

    let s = state.borrow();
    for (i, tab) in s.tabs.iter().enumerate() {
        let btn = gtk::Button::with_label(&tab.name);
        btn.style_context().add_class("tab-button");
        btn.set_can_focus(false);
        if i == s.active_tab {
            btn.style_context().add_class("active");
        }
        calc_ui.tab_bar.pack_start(&btn, false, false, 0);
        calc_ui.tab_bar.reorder_child(&btn, i as i32);
        btn.show();

        {
            let state_c = state.clone();
            let calc_ui_tab_bar = calc_ui.tab_bar.clone();
            let calc_ui_expr = calc_ui.expr_label.clone();
            let calc_ui_result = calc_ui.result_label.clone();
            let calc_ui_preview = calc_ui.preview_label.clone();
            let calc_ui_angle = calc_ui.angle_btn.clone();
            let idx = i;
            btn.connect_clicked(move |_| {
                let effects = {
                    let mut st = state_c.borrow_mut();
                    update::update(&mut st, Message::SwitchTab(idx))
                };
                for eff in effects {
                    match eff {
                        SideEffect::UpdateDisplay => {
                            let st = state_c.borrow();
                            let engine = st.engine();
                            let main_text = engine.main_display_text();
                            let ctx = calc_ui_result.style_context();
                            ctx.remove_class("result-medium");
                            ctx.remove_class("result-small");
                            if main_text.len() > 12 { ctx.add_class("result-small"); }
                            else if main_text.len() > 7 { ctx.add_class("result-medium"); }
                            calc_ui_result.set_text(&main_text);
                            if engine.show_secondary() {
                                calc_ui_expr.set_text(&engine.secondary_display_text());
                                calc_ui_expr.set_opacity(1.0);
                            } else {
                                calc_ui_expr.set_text(" ");
                                calc_ui_expr.set_opacity(0.0);
                            }
                            if let Some(preview_text) = engine.auto_eval() {
                                calc_ui_preview.set_text(&format!("\u{2248} {}", preview_text));
                                calc_ui_preview.set_opacity(1.0);
                            } else {
                                calc_ui_preview.set_text(" ");
                                calc_ui_preview.set_opacity(0.0);
                            }
                            if let Some(ref abtn) = calc_ui_angle {
                                abtn.set_label(match engine.angle_mode() {
                                    AngleMode::Degrees => "Deg",
                                    AngleMode::Radians => "Rad",
                                });
                            }
                        }
                        SideEffect::UpdateTabs => {
                            let st = state_c.borrow();
                            for child in calc_ui_tab_bar.children() {
                                if child.style_context().has_class("tab-button") {
                                    child.style_context().remove_class("active");
                                }
                            }
                            let tab_buttons: Vec<_> = calc_ui_tab_bar.children().into_iter()
                                .filter(|c| c.style_context().has_class("tab-button"))
                                .collect();
                            if let Some(active_btn) = tab_buttons.get(st.active_tab) {
                                active_btn.style_context().add_class("active");
                            }
                        }
                        _ => {}
                    }
                }
            });
        }

        {
            let state_c = state.clone();
            let idx = i;
            btn.connect_button_press_event(move |clicked_btn, event| {
                if event.event_type() != gtk::gdk::EventType::DoubleButtonPress {
                    return gtk::Inhibit(false);
                }
                let current_name = {
                    let st = state_c.borrow();
                    st.tabs.get(idx).map(|t| t.name.clone()).unwrap_or_default()
                };

                let popover = gtk::Popover::new(Some(clicked_btn));
                let entry = gtk::Entry::new();
                entry.set_text(&current_name);
                entry.set_margin_top(4);
                entry.set_margin_bottom(4);
                entry.set_margin_start(4);
                entry.set_margin_end(4);
                popover.add(&entry);
                entry.show();
                popover.popup();
                entry.grab_focus();

                let state_inner = state_c.clone();
                let btn_inner = clicked_btn.clone();
                let popover_inner = popover.clone();
                entry.connect_activate(move |e| {
                    let new_name = e.text().to_string();
                    if !new_name.is_empty() {
                        let mut st = state_inner.borrow_mut();
                        update::update(&mut st, Message::RenameTab(idx, new_name.clone()));
                        btn_inner.set_label(&new_name);
                    }
                    popover_inner.popdown();
                });

                gtk::Inhibit(true)
            });
        }
    }
}

fn wire_action_buttons(
    state: &Rc<RefCell<AppState>>,
    calc_ui: &CalculatorUI,
    _theme_mgr: &Rc<RefCell<ThemeManager>>,
    _nav_buttons: &Rc<Vec<NavButton>>,
) {
    for (button, action) in &calc_ui.action_buttons {
        let state_c = state.clone();
        let action = *action;
        let calc_ui_expr = calc_ui.expr_label.clone();
        let calc_ui_result = calc_ui.result_label.clone();
        let calc_ui_preview = calc_ui.preview_label.clone();
        let calc_ui_angle = calc_ui.angle_btn.clone();
        let calc_ui_window = calc_ui.window.clone();
        let calc_ui_sci_grid = calc_ui.sci_grid.clone();
        let calc_ui_menu_basic = calc_ui.menu_basic_btn.clone();
        let calc_ui_menu_sci = calc_ui.menu_sci_btn.clone();

        button.connect_clicked(move |btn| {
            let msg = match action {
                ButtonAction::Digit(d) => Message::Digit(d),
                ButtonAction::Decimal => Message::Decimal,
                ButtonAction::BinaryOp(op) => Message::BinaryOp(op),
                ButtonAction::UnaryFunc(f) => Message::UnaryFunc(f),
                ButtonAction::PostfixOp(op) => Message::PostfixOp(op),
                ButtonAction::Constant(val, name) => Message::Constant(val, name),
                ButtonAction::LeftParen => Message::LeftParen,
                ButtonAction::RightParen => Message::RightParen,
                ButtonAction::Equals => Message::Equals,
                ButtonAction::Clear => Message::Clear,
                ButtonAction::ToggleSign => Message::ToggleSign,
                ButtonAction::EE => Message::EE,
                ButtonAction::MemoryClear => Message::MemoryClear,
                ButtonAction::MemoryRecall => Message::MemoryRecall,
                ButtonAction::MemoryAdd => Message::MemoryAdd,
                ButtonAction::MemorySubtract => Message::MemorySubtract,
                ButtonAction::ToggleAngleMode => Message::ToggleAngleMode,
            };

            let effects = {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, msg)
            };

            for eff in effects {
                match eff {
                    SideEffect::UpdateDisplay => {
                        let s = state_c.borrow();
                        let engine = s.engine();
                        let main_text = engine.main_display_text();
                        let ctx = calc_ui_result.style_context();
                        ctx.remove_class("result-medium");
                        ctx.remove_class("result-small");
                        if main_text.len() > 12 { ctx.add_class("result-small"); }
                        else if main_text.len() > 7 { ctx.add_class("result-medium"); }
                        calc_ui_result.set_text(&main_text);
                        if engine.show_secondary() {
                            calc_ui_expr.set_text(&engine.secondary_display_text());
                            calc_ui_expr.set_opacity(1.0);
                        } else {
                            calc_ui_expr.set_text(" ");
                            calc_ui_expr.set_opacity(0.0);
                        }
                        if let Some(preview_text) = engine.auto_eval() {
                            calc_ui_preview.set_text(&format!("\u{2248} {}", preview_text));
                            calc_ui_preview.set_opacity(1.0);
                        } else {
                            calc_ui_preview.set_text(" ");
                            calc_ui_preview.set_opacity(0.0);
                        }
                        if let Some(ref abtn) = calc_ui_angle {
                            abtn.set_label(match engine.angle_mode() {
                                AngleMode::Degrees => "Deg",
                                AngleMode::Radians => "Rad",
                            });
                        }
                    }
                    SideEffect::ToggleScientific(mode) => {
                        if mode {
                            calc_ui_sci_grid.show_all();
                            calc_ui_menu_sci.style_context().add_class("active");
                            calc_ui_menu_basic.style_context().remove_class("active");
                        } else {
                            calc_ui_sci_grid.hide();
                            calc_ui_menu_basic.style_context().add_class("active");
                            calc_ui_menu_sci.style_context().remove_class("active");
                        }
                    }
                    SideEffect::ResizeWindow => {
                        let s = state_c.borrow();
                        if s.scientific_mode {
                            calc_ui_window.resize(580, s.config.window.default_height);
                        } else {
                            calc_ui_window.resize(s.config.window.default_width, s.config.window.default_height);
                        }
                    }
                    _ => {}
                }
            }

            if matches!(action, ButtonAction::ToggleAngleMode) {
                let s = state_c.borrow();
                btn.set_label(match s.engine().angle_mode() {
                    AngleMode::Degrees => "Deg",
                    AngleMode::Radians => "Rad",
                });
            }
        });
    }
}

fn wire_panel_buttons(
    state: &Rc<RefCell<AppState>>,
    calc_ui: &CalculatorUI,
    _theme_mgr: &Rc<RefCell<ThemeManager>>,
    _nav_buttons: &Rc<Vec<NavButton>>,
) {
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
        calc_ui.panel_history_btn.connect_clicked(move |_| sw("history"));
        let sw = switch_panel.clone();
        calc_ui.panel_memory_btn.connect_clicked(move |_| sw("memory"));
        let sw = switch_panel;
        calc_ui.panel_pinned_btn.connect_clicked(move |_| sw("pinned"));
    }

    {
        let state_c = state.clone();
        let history_list = calc_ui.history_list.clone();
        calc_ui.history_clear_btn.connect_clicked({
            let state_c = state_c.clone();
            move |_| {
                {
                    let mut s = state_c.borrow_mut();
                    update::update(&mut s, Message::ClearHistory);
                }
                let s = state_c.borrow();
                refresh_history(&s.engine().history, &history_list, &s.history_search, s.config.history.show_timestamps);
            }
        });
    }

    {
        let state_c = state.clone();
        calc_ui.history_export_json_btn.connect_clicked({
            let state_c = state_c.clone();
            move |btn| {
                let effects = {
                    let mut s = state_c.borrow_mut();
                    update::update(&mut s, Message::ExportHistoryJson)
                };
                for eff in effects {
                    if let SideEffect::ExportedFile(p) = eff {
                        eprintln!("Exported: {}", p.display());
                        btn.set_label("Saved!");
                        let btn_c = btn.clone();
                        gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
                            btn_c.set_label("JSON");
                            gtk::glib::Continue(false)
                        });
                    }
                }
            }
        });
    }

    {
        let state_c = state.clone();
        calc_ui.history_export_csv_btn.connect_clicked({
            let state_c = state_c.clone();
            move |btn| {
                let effects = {
                    let mut s = state_c.borrow_mut();
                    update::update(&mut s, Message::ExportHistoryCsv)
                };
                for eff in effects {
                    if let SideEffect::ExportedFile(p) = eff {
                        eprintln!("Exported: {}", p.display());
                        btn.set_label("Saved!");
                        let btn_c = btn.clone();
                        gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
                            btn_c.set_label("CSV");
                            gtk::glib::Continue(false)
                        });
                    }
                }
            }
        });
    }

    {
        let state_c = state.clone();
        let history_list = calc_ui.history_list.clone();
        calc_ui.history_search_entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, Message::SearchHistory(query));
            }
            let s = state_c.borrow();
            refresh_history(&s.engine().history, &history_list, &s.history_search, s.config.history.show_timestamps);
        });
    }
}

fn wire_menu_buttons(
    state: &Rc<RefCell<AppState>>,
    calc_ui: &CalculatorUI,
    theme_mgr: &Rc<RefCell<ThemeManager>>,
    _nav_buttons: &Rc<Vec<NavButton>>,
) {
    {
        let state_c = state.clone();
        let popover = calc_ui.menu_popover.clone();
        let sci_grid = calc_ui.sci_grid.clone();
        let window = calc_ui.window.clone();
        let basic_btn = calc_ui.menu_basic_btn.clone();
        let sci_btn = calc_ui.menu_sci_btn.clone();
        calc_ui.menu_basic_btn.connect_clicked(move |_| {
            popover.popdown();
            let is_sci = state_c.borrow().scientific_mode;
            if is_sci {
                let effects = {
                    let mut s = state_c.borrow_mut();
                    update::update(&mut s, Message::ToggleScientific)
                };
                for eff in effects {
                    match eff {
                        SideEffect::ToggleScientific(mode) => {
                            if mode { sci_grid.show_all(); sci_btn.style_context().add_class("active"); basic_btn.style_context().remove_class("active"); }
                            else { sci_grid.hide(); basic_btn.style_context().add_class("active"); sci_btn.style_context().remove_class("active"); }
                        }
                        SideEffect::ResizeWindow => {
                            let s = state_c.borrow();
                            if s.scientific_mode { window.resize(580, s.config.window.default_height); }
                            else { window.resize(s.config.window.default_width, s.config.window.default_height); }
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    {
        let state_c = state.clone();
        let popover = calc_ui.menu_popover.clone();
        let sci_grid = calc_ui.sci_grid.clone();
        let window = calc_ui.window.clone();
        let basic_btn = calc_ui.menu_basic_btn.clone();
        let sci_btn = calc_ui.menu_sci_btn.clone();
        calc_ui.menu_sci_btn.connect_clicked(move |_| {
            popover.popdown();
            let is_sci = state_c.borrow().scientific_mode;
            if !is_sci {
                let effects = {
                    let mut s = state_c.borrow_mut();
                    update::update(&mut s, Message::ToggleScientific)
                };
                for eff in effects {
                    match eff {
                        SideEffect::ToggleScientific(mode) => {
                            if mode { sci_grid.show_all(); sci_btn.style_context().add_class("active"); basic_btn.style_context().remove_class("active"); }
                            else { sci_grid.hide(); basic_btn.style_context().add_class("active"); sci_btn.style_context().remove_class("active"); }
                        }
                        SideEffect::ResizeWindow => {
                            let s = state_c.borrow();
                            if s.scientific_mode { window.resize(580, s.config.window.default_height); }
                            else { window.resize(s.config.window.default_width, s.config.window.default_height); }
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    {
        let popover = calc_ui.menu_popover.clone();
        let window = calc_ui.window.clone();
        calc_ui.menu_help_btn.connect_clicked(move |_| {
            popover.popdown();
            show_help_dialog(&window);
        });
    }

    {
        let state_c = state.clone();
        let popover = calc_ui.menu_popover.clone();
        let mode_panel_revealer = calc_ui.mode_panel_revealer.clone();
        let mode_panel_stack = calc_ui.mode_panel_stack.clone();
        calc_ui.menu_notes_btn.connect_clicked(move |_| {
            popover.popdown();
            let _effects = {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, Message::OpenNotes)
            };
            let s = state_c.borrow();
            mode_panel_revealer.set_reveal_child(s.mode_panel_visible);
            if s.mode_panel_visible {
                mode_panel_stack.set_visible_child_name("notes");
            }
        });
    }

    {
        let state_c = state.clone();
        let popover = calc_ui.menu_popover.clone();
        let mode_panel_revealer = calc_ui.mode_panel_revealer.clone();
        let mode_panel_stack = calc_ui.mode_panel_stack.clone();
        calc_ui.menu_converter_btn.connect_clicked(move |_| {
            popover.popdown();
            {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, Message::OpenConverter);
            }
            let s = state_c.borrow();
            mode_panel_revealer.set_reveal_child(s.mode_panel_visible);
            if s.mode_panel_visible {
                mode_panel_stack.set_visible_child_name("converter");
            }
        });
    }

    {
        let state_c = state.clone();
        let popover = calc_ui.menu_popover.clone();
        let mode_panel_revealer = calc_ui.mode_panel_revealer.clone();
        let mode_panel_stack = calc_ui.mode_panel_stack.clone();
        calc_ui.menu_tools_btn.connect_clicked(move |_| {
            popover.popdown();
            {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, Message::OpenTools);
            }
            let s = state_c.borrow();
            mode_panel_revealer.set_reveal_child(s.mode_panel_visible);
            if s.mode_panel_visible {
                mode_panel_stack.set_visible_child_name("tools");
            }
        });
    }

    for (btn, idx) in &calc_ui.menu_theme_btns {
        let state_c = state.clone();
        let theme_mgr_c = theme_mgr.clone();
        let popover = calc_ui.menu_popover.clone();
        let theme_val = Theme::ALL[*idx];
        let all_btns: Vec<(gtk::Button, usize)> = calc_ui.menu_theme_btns.clone();
        let current_idx = *idx;
        btn.connect_clicked(move |_| {
            popover.popdown();
            let s = state_c.borrow();
            theme_mgr_c.borrow_mut().set_theme(theme_val, &s.config.theme, &s.config.layout, &s.config.feedback);
            for (b, i) in &all_btns {
                if *i == current_idx {
                    b.style_context().add_class("menu-item-active");
                } else {
                    b.style_context().remove_class("menu-item-active");
                }
            }
        });
    }

    {
        let state_c = state.clone();
        let mode_panel_revealer = calc_ui.mode_panel_revealer.clone();
        calc_ui.conv_back_btn.connect_clicked(move |_| {
            let _effects = {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, Message::CloseMode)
            };
            mode_panel_revealer.set_reveal_child(false);
        });
    }
    {
        let state_c = state.clone();
        let mode_panel_revealer = calc_ui.mode_panel_revealer.clone();
        calc_ui.tools_back_btn.connect_clicked(move |_| {
            let _effects = {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, Message::CloseMode)
            };
            mode_panel_revealer.set_reveal_child(false);
        });
    }
    {
        let state_c = state.clone();
        let mode_panel_revealer = calc_ui.mode_panel_revealer.clone();
        calc_ui.notes_back_btn.connect_clicked(move |_| {
            let _effects = {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, Message::CloseMode)
            };
            mode_panel_revealer.set_reveal_child(false);
        });
    }
}

fn wire_converter(_state: &Rc<RefCell<AppState>>, calc_ui: &CalculatorUI) {
    let conv_category = Rc::new(std::cell::Cell::new(0usize));

    {
        let entry = calc_ui.conv_value_entry.clone();
        let from_combo = calc_ui.conv_from_combo.clone();
        let to_combo = calc_ui.conv_to_combo.clone();
        let result_lbl = calc_ui.conv_result_label.clone();
        let cat = conv_category.clone();

        let do_convert = move || {
            let val: f64 = entry.text().parse().unwrap_or(0.0);
            let category = ConvertCategory::ALL[cat.get()];
            let from = from_combo.active_text().map(|s| s.to_string()).unwrap_or_default();
            let to = to_combo.active_text().map(|s| s.to_string()).unwrap_or_default();
            if !from.is_empty() && !to.is_empty() {
                let result = domain::convert::convert(category, &from, &to, val);
                result_lbl.set_text(&domain::types::format_number_default(result));
            }
        };

        let dc = do_convert.clone();
        calc_ui.conv_value_entry.connect_changed(move |_| dc());
        let dc = do_convert.clone();
        calc_ui.conv_from_combo.connect_changed(move |_| dc());
        let dc = do_convert;
        calc_ui.conv_to_combo.connect_changed(move |_| dc());
    }

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
            let category = ConvertCategory::ALL[i];
            for (abbr, _name) in category.units() {
                from_combo.append_text(abbr);
                to_combo.append_text(abbr);
            }
            from_combo.set_active(Some(0));
            to_combo.set_active(Some(1));

            let val: f64 = entry.text().parse().unwrap_or(1.0);
            let units = category.units();
            if units.len() >= 2 {
                let result = domain::convert::convert(category, units[0].0, units[1].0, val);
                result_lbl.set_text(&domain::types::format_number_default(result));
            }
        });
    }

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
}

fn wire_tools(calc_ui: &CalculatorUI) {
    {
        let amount_entry = calc_ui.tip_amount_entry.clone();
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

    {
        let price_entry = calc_ui.discount_price_entry.clone();
        let pct_entry = calc_ui.discount_pct_entry.clone();
        let result_lbl = calc_ui.discount_result_label.clone();

        let calc_disc = move || {
            let price: f64 = price_entry.text().parse().unwrap_or(0.0);
            let pct: f64 = pct_entry.text().parse().unwrap_or(0.0);
            let savings = price * pct / 100.0;
            result_lbl.set_text(&format!("Save: {:.2}  |  Final: {:.2}", savings, price - savings));
        };

        let cd = calc_disc.clone();
        calc_ui.discount_price_entry.connect_changed(move |_| cd());
        let cd = calc_disc;
        calc_ui.discount_pct_entry.connect_changed(move |_| cd());
    }

    {
        let amount_entry = calc_ui.tax_amount_entry.clone();
        let rate_entry = calc_ui.tax_rate_entry.clone();
        let result_lbl = calc_ui.tax_result_label.clone();

        let calc_tax = move || {
            let amount: f64 = amount_entry.text().parse().unwrap_or(0.0);
            let rate: f64 = rate_entry.text().parse().unwrap_or(0.0);
            let tax = amount * rate / 100.0;
            result_lbl.set_text(&format!("Tax: {:.2}  |  Total: {:.2}", tax, amount + tax));
        };

        let ct = calc_tax.clone();
        calc_ui.tax_amount_entry.connect_changed(move |_| ct());
        let ct = calc_tax;
        calc_ui.tax_rate_entry.connect_changed(move |_| ct());
    }
}

fn wire_notes(calc_ui: &CalculatorUI, state: &Rc<RefCell<AppState>>) {
    let result_lbl = calc_ui.notes_result_label.clone();
    let textview = calc_ui.notes_textview.clone();
    let state_c = state.clone();

    if let Some(buf) = textview.buffer() {
        buf.connect_changed(move |buf| {
            let text = buf
                .text(&buf.start_iter(), &buf.end_iter(), false)
                .map(|s| s.to_string())
                .unwrap_or_default();

            let plugins = {
                let s = state_c.borrow();
                s.config.plugins.functions.clone()
            };

            let mut results = Vec::new();
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                    results.push(String::new());
                    continue;
                }
                match domain::eval::parse_expression(line, &plugins) {
                    Ok(tokens) if !tokens.is_empty() => {
                        match domain::eval::evaluate(&tokens, AngleMode::Degrees, true) {
                            Ok(val) => results.push(format!("= {}", domain::types::format_number_default(val))),
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

fn wire_keyboard(
    state: &Rc<RefCell<AppState>>,
    calc_ui: &CalculatorUI,
    theme_mgr: &Rc<RefCell<ThemeManager>>,
    nav_buttons: &Rc<Vec<NavButton>>,
) {
    let state_c = state.clone();
    let theme_mgr_c = theme_mgr.clone();
    let nav_c = nav_buttons.clone();
    let expr = calc_ui.expr_label.clone();
    let result_l = calc_ui.result_label.clone();
    let preview = calc_ui.preview_label.clone();
    let sci_grid = calc_ui.sci_grid.clone();
    let window = calc_ui.window.clone();
    let menu_basic_btn = calc_ui.menu_basic_btn.clone();
    let menu_sci_btn = calc_ui.menu_sci_btn.clone();
    let panel_revealer = calc_ui.panel_revealer.clone();
    let panel_stack = calc_ui.panel_stack.clone();
    let mode_panel_revealer = calc_ui.mode_panel_revealer.clone();
    let mode_panel_stack = calc_ui.mode_panel_stack.clone();
    let menu_popover = calc_ui.menu_popover.clone();
    let history_list = calc_ui.history_list.clone();
    let memory_list = calc_ui.memory_list.clone();
    let pinned_list = calc_ui.pinned_list.clone();
    let p_history_btn = calc_ui.panel_history_btn.clone();
    let p_memory_btn = calc_ui.panel_memory_btn.clone();
    let p_pinned_btn = calc_ui.panel_pinned_btn.clone();
    let angle_btn = calc_ui.angle_btn.clone();
    let tab_bar = calc_ui.tab_bar.clone();

    calc_ui.window.connect_key_press_event(move |_, event| {
        let msg = ui::keyboard::map_key(event);
        if matches!(msg, Message::Noop) {
            return gtk::Inhibit(false);
        }

        let effects = {
            let mut s = state_c.borrow_mut();
            update::update(&mut s, msg)
        };

        for eff in effects {
            match eff {
                SideEffect::UpdateDisplay => {
                    let s = state_c.borrow();
                    let engine = s.engine();
                    let main_text = engine.main_display_text();
                    let ctx = result_l.style_context();
                    ctx.remove_class("result-medium");
                    ctx.remove_class("result-small");
                    if main_text.len() > 12 { ctx.add_class("result-small"); }
                    else if main_text.len() > 7 { ctx.add_class("result-medium"); }
                    result_l.set_text(&main_text);
                    if engine.show_secondary() {
                        expr.set_text(&engine.secondary_display_text());
                        expr.set_opacity(1.0);
                    } else {
                        expr.set_text(" ");
                        expr.set_opacity(0.0);
                    }
                    if let Some(preview_text) = engine.auto_eval() {
                        preview.set_text(&format!("\u{2248} {}", preview_text));
                        preview.set_opacity(1.0);
                    } else {
                        preview.set_text(" ");
                        preview.set_opacity(0.0);
                    }
                    if let Some(ref abtn) = angle_btn {
                        abtn.set_label(match engine.angle_mode() {
                            AngleMode::Degrees => "Deg",
                            AngleMode::Radians => "Rad",
                        });
                    }
                }
                SideEffect::UpdateTabs => {
                    for child in tab_bar.children() {
                        if child.style_context().has_class("tab-button") {
                            tab_bar.remove(&child);
                        }
                    }
                    let s = state_c.borrow();
                    for (i, tab) in s.tabs.iter().enumerate() {
                        let btn = gtk::Button::with_label(&tab.name);
                        btn.style_context().add_class("tab-button");
                        btn.set_can_focus(false);
                        if i == s.active_tab {
                            btn.style_context().add_class("active");
                        }
                        tab_bar.pack_start(&btn, false, false, 0);
                        tab_bar.reorder_child(&btn, i as i32);
                        btn.show();

                        let state_inner = state_c.clone();
                        let expr_inner = expr.clone();
                        let result_inner = result_l.clone();
                        let preview_inner = preview.clone();
                        let angle_inner = angle_btn.clone();
                        let tab_bar_inner = tab_bar.clone();
                        let idx = i;
                        btn.connect_clicked(move |_| {
                            let effects = {
                                let mut st = state_inner.borrow_mut();
                                update::update(&mut st, Message::SwitchTab(idx))
                            };
                            for eff in effects {
                                match eff {
                                    SideEffect::UpdateDisplay => {
                                        let st = state_inner.borrow();
                                        let engine = st.engine();
                                        let main_text = engine.main_display_text();
                                        let ctx = result_inner.style_context();
                                        ctx.remove_class("result-medium");
                                        ctx.remove_class("result-small");
                                        if main_text.len() > 12 { ctx.add_class("result-small"); }
                                        else if main_text.len() > 7 { ctx.add_class("result-medium"); }
                                        result_inner.set_text(&main_text);
                                        if engine.show_secondary() {
                                            expr_inner.set_text(&engine.secondary_display_text());
                                            expr_inner.set_opacity(1.0);
                                        } else {
                                            expr_inner.set_text(" ");
                                            expr_inner.set_opacity(0.0);
                                        }
                                        if let Some(preview_text) = engine.auto_eval() {
                                            preview_inner.set_text(&format!("\u{2248} {}", preview_text));
                                            preview_inner.set_opacity(1.0);
                                        } else {
                                            preview_inner.set_text(" ");
                                            preview_inner.set_opacity(0.0);
                                        }
                                        if let Some(ref abtn) = angle_inner {
                                            abtn.set_label(match engine.angle_mode() {
                                                AngleMode::Degrees => "Deg",
                                                AngleMode::Radians => "Rad",
                                            });
                                        }
                                    }
                                    SideEffect::UpdateTabs => {
                                        let st = state_inner.borrow();
                                        for child in tab_bar_inner.children() {
                                            if child.style_context().has_class("tab-button") {
                                                child.style_context().remove_class("active");
                                            }
                                        }
                                        let tab_buttons: Vec<_> = tab_bar_inner.children().into_iter()
                                            .filter(|c| c.style_context().has_class("tab-button"))
                                            .collect();
                                        if let Some(active_btn) = tab_buttons.get(st.active_tab) {
                                            active_btn.style_context().add_class("active");
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        });

                        let state_rename = state_c.clone();
                        let idx_rename = i;
                        btn.connect_button_press_event(move |clicked_btn, event| {
                            if event.event_type() != gtk::gdk::EventType::DoubleButtonPress {
                                return gtk::Inhibit(false);
                            }
                            let current_name = {
                                let st = state_rename.borrow();
                                st.tabs.get(idx_rename).map(|t| t.name.clone()).unwrap_or_default()
                            };
                            let popover = gtk::Popover::new(Some(clicked_btn));
                            let entry = gtk::Entry::new();
                            entry.set_text(&current_name);
                            entry.set_margin_top(4);
                            entry.set_margin_bottom(4);
                            entry.set_margin_start(4);
                            entry.set_margin_end(4);
                            popover.add(&entry);
                            entry.show();
                            popover.popup();
                            entry.grab_focus();
                            let state_entry = state_rename.clone();
                            let btn_entry = clicked_btn.clone();
                            let popover_entry = popover.clone();
                            entry.connect_activate(move |e| {
                                let new_name = e.text().to_string();
                                if !new_name.is_empty() {
                                    let mut st = state_entry.borrow_mut();
                                    update::update(&mut st, Message::RenameTab(idx_rename, new_name.clone()));
                                    btn_entry.set_label(&new_name);
                                }
                                popover_entry.popdown();
                            });
                            gtk::Inhibit(true)
                        });
                    }
                }
                SideEffect::ToggleScientific(mode) => {
                    if mode {
                        sci_grid.show_all();
                        menu_sci_btn.style_context().add_class("active");
                        menu_basic_btn.style_context().remove_class("active");
                    } else {
                        sci_grid.hide();
                        menu_basic_btn.style_context().add_class("active");
                        menu_sci_btn.style_context().remove_class("active");
                    }
                }
                SideEffect::ResizeWindow => {
                    let s = state_c.borrow();
                    if s.scientific_mode {
                        window.resize(580, s.config.window.default_height);
                    } else {
                        window.resize(s.config.window.default_width, s.config.window.default_height);
                    }
                }
                SideEffect::ToggleTheme => {
                    let s = state_c.borrow();
                    theme_mgr_c.borrow_mut().toggle(&s.config.theme, &s.config.layout, &s.config.feedback);
                }
                SideEffect::TogglePanel => {
                    let s = state_c.borrow();
                    panel_revealer.set_reveal_child(s.panel_visible);
                    if s.panel_visible {
                        p_history_btn.style_context().remove_class("active");
                        p_memory_btn.style_context().remove_class("active");
                        p_pinned_btn.style_context().remove_class("active");
                        match s.active_panel {
                            Panel::History => {
                                panel_stack.set_visible_child_name("history");
                                p_history_btn.style_context().add_class("active");
                            }
                            Panel::Memory => {
                                panel_stack.set_visible_child_name("memory");
                                p_memory_btn.style_context().add_class("active");
                            }
                            Panel::Pinned => {
                                panel_stack.set_visible_child_name("pinned");
                                p_pinned_btn.style_context().add_class("active");
                            }
                        }
                    }
                }
                SideEffect::ToggleModePanel => {
                    let s = state_c.borrow();
                    mode_panel_revealer.set_reveal_child(s.mode_panel_visible);
                    if s.mode_panel_visible {
                        if let Some(mode) = s.active_mode {
                            let name = match mode {
                                ModePanel::Converter => "converter",
                                ModePanel::Tools => "tools",
                                ModePanel::Notes => "notes",
                            };
                            mode_panel_stack.set_visible_child_name(name);
                        }
                    }
                }
                SideEffect::RefreshHistory => {
                    let s = state_c.borrow();
                    refresh_history(&s.engine().history, &history_list, &s.history_search, s.config.history.show_timestamps);
                }
                SideEffect::RefreshMemory => {
                    let s = state_c.borrow();
                    refresh_memory(&s.engine().memory_slots, s.engine().has_memory(), &memory_list);
                }
                SideEffect::RefreshPinned => {
                    let s = state_c.borrow();
                    refresh_pinned(&s.engine().pinned, &pinned_list);
                }
                SideEffect::ExportedFile(path) => {
                    eprintln!("Exported: {}", path.display());
                }
                SideEffect::ShowHelp => {
                    show_help_dialog(&window);
                }
                SideEffect::Navigate(dir) => {
                    let (mode_open, sci) = {
                        let s = state_c.borrow();
                        (s.mode_panel_visible, s.scientific_mode)
                    };
                    if !mode_open {
                        ui::navigation::navigate(&nav_c, dir, sci);
                    }
                }
                SideEffect::ActivateButton => {
                    let (mode_open, sci) = {
                        let s = state_c.borrow();
                        (s.mode_panel_visible, s.scientific_mode)
                    };
                    if !mode_open {
                        ui::navigation::activate_focused(&nav_c, sci);
                    }
                }
                SideEffect::OpenMenu => {
                    menu_popover.popup();
                }
                SideEffect::Quit => {
                    window.close();
                }
                SideEffect::Noop => {}
            }
        }

        gtk::Inhibit(true)
    });
}

fn wire_window_close(state: &Rc<RefCell<AppState>>, calc_ui: &CalculatorUI) {
    let state_c = state.clone();
    let window = calc_ui.window.clone();
    calc_ui.window.connect_delete_event(move |_, _| {
        if state_c.borrow().config.window.remember_geometry {
            let (x, y) = window.position();
            let (w, h) = window.size();
            services::session::save_geometry(x, y, w, h);
        }
        {
            let s = state_c.borrow();
            update::save_on_exit(&s);
        }
        gtk::main_quit();
        gtk::Inhibit(false)
    });

    let state_c = state.clone();
    calc_ui.tab_add_btn.connect_clicked({
        let state_c = state_c.clone();
        let tab_bar = calc_ui.tab_bar.clone();
        let expr = calc_ui.expr_label.clone();
        let result_l = calc_ui.result_label.clone();
        let preview = calc_ui.preview_label.clone();
        let angle_btn = calc_ui.angle_btn.clone();
        move |_| {
            {
                let mut s = state_c.borrow_mut();
                update::update(&mut s, Message::NewTab);
            }
            for child in tab_bar.children() {
                if child.style_context().has_class("tab-button") {
                    tab_bar.remove(&child);
                }
            }
            let s = state_c.borrow();
            for (i, tab) in s.tabs.iter().enumerate() {
                let btn = gtk::Button::with_label(&tab.name);
                btn.style_context().add_class("tab-button");
                btn.set_can_focus(false);
                if i == s.active_tab {
                    btn.style_context().add_class("active");
                }
                tab_bar.pack_start(&btn, false, false, 0);
                tab_bar.reorder_child(&btn, i as i32);
                btn.show();

                let state_inner = state_c.clone();
                let expr_inner = expr.clone();
                let result_inner = result_l.clone();
                let preview_inner = preview.clone();
                let angle_inner = angle_btn.clone();
                let tab_bar_inner = tab_bar.clone();
                let idx = i;
                btn.connect_clicked(move |_| {
                    let effects = {
                        let mut st = state_inner.borrow_mut();
                        update::update(&mut st, Message::SwitchTab(idx))
                    };
                    for eff in effects {
                        match eff {
                            SideEffect::UpdateDisplay => {
                                let st = state_inner.borrow();
                                let engine = st.engine();
                                let main_text = engine.main_display_text();
                                let ctx = result_inner.style_context();
                                ctx.remove_class("result-medium");
                                ctx.remove_class("result-small");
                                if main_text.len() > 12 { ctx.add_class("result-small"); }
                                else if main_text.len() > 7 { ctx.add_class("result-medium"); }
                                result_inner.set_text(&main_text);
                                if engine.show_secondary() {
                                    expr_inner.set_text(&engine.secondary_display_text());
                                    expr_inner.set_opacity(1.0);
                                } else {
                                    expr_inner.set_text(" ");
                                    expr_inner.set_opacity(0.0);
                                }
                                if let Some(preview_text) = engine.auto_eval() {
                                    preview_inner.set_text(&format!("\u{2248} {}", preview_text));
                                    preview_inner.set_opacity(1.0);
                                } else {
                                    preview_inner.set_text(" ");
                                    preview_inner.set_opacity(0.0);
                                }
                                if let Some(ref abtn) = angle_inner {
                                    abtn.set_label(match engine.angle_mode() {
                                        AngleMode::Degrees => "Deg",
                                        AngleMode::Radians => "Rad",
                                    });
                                }
                            }
                            SideEffect::UpdateTabs => {
                                let st = state_inner.borrow();
                                for child in tab_bar_inner.children() {
                                    if child.style_context().has_class("tab-button") {
                                        child.style_context().remove_class("active");
                                    }
                                }
                                let tab_buttons: Vec<_> = tab_bar_inner.children().into_iter()
                                    .filter(|c| c.style_context().has_class("tab-button"))
                                    .collect();
                                if let Some(active_btn) = tab_buttons.get(st.active_tab) {
                                    active_btn.style_context().add_class("active");
                                }
                            }
                            _ => {}
                        }
                    }
                });

                let state_rename = state_c.clone();
                let idx_rename = i;
                btn.connect_button_press_event(move |clicked_btn, event| {
                    if event.event_type() != gtk::gdk::EventType::DoubleButtonPress {
                        return gtk::Inhibit(false);
                    }
                    let current_name = {
                        let st = state_rename.borrow();
                        st.tabs.get(idx_rename).map(|t| t.name.clone()).unwrap_or_default()
                    };
                    let popover = gtk::Popover::new(Some(clicked_btn));
                    let entry = gtk::Entry::new();
                    entry.set_text(&current_name);
                    entry.set_margin_top(4);
                    entry.set_margin_bottom(4);
                    entry.set_margin_start(4);
                    entry.set_margin_end(4);
                    popover.add(&entry);
                    entry.show();
                    popover.popup();
                    entry.grab_focus();
                    let state_entry = state_rename.clone();
                    let btn_entry = clicked_btn.clone();
                    let popover_entry = popover.clone();
                    entry.connect_activate(move |e| {
                        let new_name = e.text().to_string();
                        if !new_name.is_empty() {
                            let mut st = state_entry.borrow_mut();
                            update::update(&mut st, Message::RenameTab(idx_rename, new_name.clone()));
                            btn_entry.set_label(&new_name);
                        }
                        popover_entry.popdown();
                    });
                    gtk::Inhibit(true)
                });
            }

            let engine = s.engine();
            let main_text = engine.main_display_text();
            let ctx = result_l.style_context();
            ctx.remove_class("result-medium");
            ctx.remove_class("result-small");
            if main_text.len() > 12 { ctx.add_class("result-small"); }
            else if main_text.len() > 7 { ctx.add_class("result-medium"); }
            result_l.set_text(&main_text);
            if engine.show_secondary() {
                expr.set_text(&engine.secondary_display_text());
                expr.set_opacity(1.0);
            } else {
                expr.set_text(" ");
                expr.set_opacity(0.0);
            }
            if let Some(preview_text) = engine.auto_eval() {
                preview.set_text(&format!("\u{2248} {}", preview_text));
                preview.set_opacity(1.0);
            } else {
                preview.set_text(" ");
                preview.set_opacity(0.0);
            }
        }
    });
}

fn show_help_dialog(window: &gtk::Window) {
    let dialog = gtk::Dialog::with_buttons(
        Some("Keyboard Shortcuts"),
        Some(window),
        gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
        &[("Close", gtk::ResponseType::Close)],
    );
    dialog.set_default_size(380, 440);
    let content = dialog.content_area();
    let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    let label = gtk::Label::new(Some(HELP_TEXT));
    label.set_xalign(0.0);
    label.set_yalign(0.0);
    label.set_margin_top(12);
    label.set_margin_bottom(12);
    label.set_margin_start(16);
    label.set_margin_end(16);
    label.set_selectable(true);
    scroll.add(&label);
    content.pack_start(&scroll, true, true, 0);
    dialog.show_all();
    dialog.run();
    unsafe { dialog.destroy(); }
}

fn format_timestamp(ts: u64) -> String {
    let secs = ts % 60;
    let mins = (ts / 60) % 60;
    let hours = (ts / 3600) % 24;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn refresh_history(
    history: &[domain::types::HistoryEntry],
    list: &gtk::Box,
    search: &str,
    show_timestamps: bool,
) {
    for child in list.children() {
        list.remove(&child);
    }
    let query = search.to_lowercase();
    let filtered: Vec<_> = history
        .iter()
        .rev()
        .filter(|e| {
            query.is_empty()
                || e.expression.to_lowercase().contains(&query)
                || e.result_text.to_lowercase().contains(&query)
        })
        .collect();

    if filtered.is_empty() {
        let msg = if query.is_empty() { "No calculations yet" } else { "No matching results" };
        let empty = gtk::Label::new(Some(msg));
        empty.style_context().add_class("panel-empty");
        list.pack_start(&empty, false, false, 0);
    } else {
        for entry in filtered {
            let item = gtk::Box::new(gtk::Orientation::Vertical, 2);
            item.style_context().add_class("panel-item");
            item.set_margin_bottom(2);

            if show_timestamps && entry.timestamp > 0 {
                let ts_lbl = gtk::Label::new(Some(&format_timestamp(entry.timestamp)));
                ts_lbl.style_context().add_class("panel-item-label");
                ts_lbl.set_xalign(0.0);
                item.pack_start(&ts_lbl, false, false, 0);
            }

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
    list.show_all();
}

fn refresh_memory(
    memory_slots: &[domain::types::MemorySlot],
    has_memory: bool,
    list: &gtk::Box,
) {
    for child in list.children() {
        list.remove(&child);
    }
    if has_memory {
        let item = gtk::Box::new(gtk::Orientation::Vertical, 2);
        item.style_context().add_class("panel-item");
        item.set_margin_bottom(2);
        let lbl = gtk::Label::new(Some("Quick Memory (M+/M-)"));
        lbl.style_context().add_class("panel-item-label");
        lbl.set_xalign(0.0);
        item.pack_start(&lbl, false, false, 0);
        list.pack_start(&item, false, false, 0);
    }

    if memory_slots.is_empty() && !has_memory {
        let empty = gtk::Label::new(Some(
            "No stored values\n\nPress S to store current value\nUse M+/M- in scientific mode",
        ));
        empty.style_context().add_class("panel-empty");
        list.pack_start(&empty, false, false, 0);
    } else {
        for slot in memory_slots {
            let item = gtk::Box::new(gtk::Orientation::Vertical, 2);
            item.style_context().add_class("panel-item");
            item.set_margin_bottom(2);

            let lbl = gtk::Label::new(Some(&slot.label));
            lbl.style_context().add_class("panel-item-label");
            lbl.set_xalign(0.0);

            let val = gtk::Label::new(Some(&domain::types::format_number_default(slot.value)));
            val.style_context().add_class("panel-item-result");
            val.set_xalign(1.0);

            item.pack_start(&lbl, false, false, 0);
            item.pack_start(&val, false, false, 0);
            list.pack_start(&item, false, false, 0);
        }
    }
    list.show_all();
}

fn refresh_pinned(pinned: &[domain::types::PinnedCalc], list: &gtk::Box) {
    for child in list.children() {
        list.remove(&child);
    }
    if pinned.is_empty() {
        let empty = gtk::Label::new(Some("No pinned results\n\nPress Ctrl+S to pin"));
        empty.style_context().add_class("panel-empty");
        list.pack_start(&empty, false, false, 0);
    } else {
        for pin in pinned {
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

            let val = gtk::Label::new(Some(&format!("= {}", domain::types::format_number_default(pin.result))));
            val.style_context().add_class("panel-item-result");
            val.set_xalign(1.0);

            item.pack_start(&lbl, false, false, 0);
            item.pack_start(&expr, false, false, 0);
            item.pack_start(&val, false, false, 0);
            list.pack_start(&item, false, false, 0);
        }
    }
    list.show_all();
}
