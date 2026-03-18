use gtk::prelude::*;
use gtk::{
    Button, ComboBoxText, DrawingArea, Entry, Grid, Label, Notebook, Orientation, PolicyType,
    Revealer, RevealerTransitionType, ScrolledWindow, Stack, StackTransitionType, TextView,
    Window, WindowType,
};

use crate::domain::types::*;
use crate::services::config::Config;
use crate::services::theme::Theme;
use crate::ui::navigation::NavButton;

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

pub struct CalculatorUI {
    pub window: Window,
    pub expr_label: Label,
    pub result_label: Label,
    pub preview_label: Label,
    pub sci_grid: Grid,
    pub nav_buttons: Vec<NavButton>,
    pub action_buttons: Vec<(Button, ButtonAction)>,
    pub tab_bar: gtk::Box,
    pub tab_add_btn: Button,
    pub menu_popover: gtk::Popover,
    pub menu_basic_btn: Button,
    pub menu_sci_btn: Button,
    pub menu_help_btn: Button,
    pub menu_notes_btn: Button,
    pub menu_converter_btn: Button,
    pub menu_tools_btn: Button,
    pub menu_theme_btns: Vec<(Button, usize)>,
    pub panel_revealer: Revealer,
    pub panel_history_btn: Button,
    pub panel_memory_btn: Button,
    pub panel_pinned_btn: Button,
    pub history_search_entry: Entry,
    pub history_export_json_btn: Button,
    pub history_export_csv_btn: Button,
    pub history_clear_btn: Button,
    pub history_list: gtk::Box,
    pub memory_list: gtk::Box,
    pub pinned_list: gtk::Box,
    pub panel_stack: Stack,
    pub mode_panel_revealer: Revealer,
    pub mode_panel_stack: Stack,
    pub conv_value_entry: Entry,
    pub conv_from_combo: ComboBoxText,
    pub conv_to_combo: ComboBoxText,
    pub conv_result_label: Label,
    pub conv_cat_btns: Vec<Button>,
    pub conv_swap_btn: Button,
    pub conv_back_btn: Button,
    pub tip_amount_entry: Entry,
    pub tip_pct_btns: Vec<(Button, f64)>,
    pub tip_custom_entry: Entry,
    pub tip_result_label: Label,
    pub discount_price_entry: Entry,
    pub discount_pct_entry: Entry,
    pub discount_result_label: Label,
    pub tax_amount_entry: Entry,
    pub tax_rate_entry: Entry,
    pub tax_result_label: Label,
    pub tools_back_btn: Button,
    pub notes_textview: TextView,
    pub notes_result_label: Label,
    pub notes_back_btn: Button,
    pub angle_btn: Option<Button>,
}

pub fn build(config: &Config) -> CalculatorUI {
    let wcfg = &config.window;
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Fredulator");
    window.set_default_size(wcfg.default_width, wcfg.default_height);
    window.set_resizable(true);
    window.style_context().add_class("main-window");

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

    let outer_tab_bar = gtk::Box::new(Orientation::Horizontal, 4);
    outer_tab_bar.style_context().add_class("tab-bar");

    let tab_add_btn = Button::with_label("+");
    tab_add_btn.style_context().add_class("tab-add");
    tab_add_btn.set_can_focus(false);

    let menu_btn = Button::with_label("\u{2261}");
    menu_btn.style_context().add_class("menu-button");
    menu_btn.set_can_focus(false);

    let tab_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    tab_scroll.set_policy(PolicyType::Automatic, PolicyType::Never);
    tab_scroll.set_hexpand(true);

    let tab_bar = gtk::Box::new(Orientation::Horizontal, 4);
    tab_bar.style_context().add_class("tab-buttons-inner");
    tab_scroll.add(&tab_bar);

    outer_tab_bar.pack_start(&tab_scroll, true, true, 0);
    outer_tab_bar.pack_end(&menu_btn, false, false, 0);
    outer_tab_bar.pack_end(&tab_add_btn, false, false, 0);

    let menu_popover = gtk::Popover::new(Some(&menu_btn));
    let menu_box = gtk::Box::new(Orientation::Vertical, 2);
    menu_box.set_margin_top(8);
    menu_box.set_margin_bottom(8);
    menu_box.set_margin_start(8);
    menu_box.set_margin_end(8);

    let mode_selector = gtk::Box::new(Orientation::Horizontal, 2);
    mode_selector.style_context().add_class("mode-selector");
    let menu_basic_btn = Button::with_label("Basic");
    menu_basic_btn.style_context().add_class("active");
    menu_basic_btn.set_hexpand(true);
    let menu_sci_btn = Button::with_label("Scientific");
    menu_sci_btn.set_hexpand(true);
    mode_selector.pack_start(&menu_basic_btn, true, true, 0);
    mode_selector.pack_start(&menu_sci_btn, true, true, 0);
    menu_box.pack_start(&mode_selector, false, false, 0);

    let sep0 = gtk::Separator::new(Orientation::Horizontal);
    menu_box.pack_start(&sep0, false, false, 4);

    let menu_notes_btn = Button::with_label("\u{270e} Math Notes     [Ctrl+n]");
    menu_notes_btn.style_context().add_class("menu-item");
    menu_notes_btn.set_halign(gtk::Align::Fill);
    let menu_converter_btn = Button::with_label("\u{21c4} Converter      [Ctrl+e]");
    menu_converter_btn.style_context().add_class("menu-item");
    menu_converter_btn.set_halign(gtk::Align::Fill);
    let menu_tools_btn = Button::with_label("% Quick Tools   [Ctrl+r]");
    menu_tools_btn.style_context().add_class("menu-item");
    menu_tools_btn.set_halign(gtk::Align::Fill);

    menu_box.pack_start(&menu_notes_btn, false, false, 0);
    menu_box.pack_start(&menu_converter_btn, false, false, 0);
    menu_box.pack_start(&menu_tools_btn, false, false, 0);

    let sep = gtk::Separator::new(Orientation::Horizontal);
    menu_box.pack_start(&sep, false, false, 4);

    let theme_header = Label::new(Some("THEMES"));
    theme_header.style_context().add_class("menu-header");
    theme_header.set_xalign(0.0);
    menu_box.pack_start(&theme_header, false, false, 0);

    let mut menu_theme_btns = Vec::new();
    for (i, theme) in Theme::ALL.iter().enumerate() {
        let row_box = gtk::Box::new(Orientation::Horizontal, 6);
        row_box.set_margin_start(4);

        let accent = theme.accent_color();
        let dot = DrawingArea::new();
        dot.set_size_request(12, 12);
        dot.style_context().add_class("theme-dot");
        let r = u8::from_str_radix(&accent[1..3], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&accent[3..5], 16).unwrap_or(0) as f64 / 255.0;
        let b_val = u8::from_str_radix(&accent[5..7], 16).unwrap_or(0) as f64 / 255.0;
        dot.connect_draw(move |_, cr| {
            cr.set_source_rgb(r, g, b_val);
            cr.arc(6.0, 6.0, 6.0, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();
            gtk::Inhibit(true)
        });

        let lbl = Label::new(Some(theme.name()));
        lbl.set_xalign(0.0);

        row_box.pack_start(&dot, false, false, 0);
        row_box.pack_start(&lbl, true, true, 0);

        let btn = Button::new();
        btn.style_context().add_class("menu-item");
        btn.set_halign(gtk::Align::Fill);
        btn.add(&row_box);

        menu_box.pack_start(&btn, false, false, 0);
        menu_theme_btns.push((btn, i));
    }

    let sep2 = gtk::Separator::new(Orientation::Horizontal);
    menu_box.pack_start(&sep2, false, false, 4);

    let shortcuts_header = Label::new(Some("PANELS"));
    shortcuts_header.style_context().add_class("menu-header");
    shortcuts_header.set_xalign(0.0);
    menu_box.pack_start(&shortcuts_header, false, false, 0);

    let info_labels = [
        "History         [Ctrl+h]",
        "Memory          [Ctrl+m]",
        "Pinned          [Ctrl+p]",
        "Pin result      [Ctrl+s]",
        "Undo            [u/Ctrl+z]",
    ];
    for info in &info_labels {
        let l = Label::new(Some(info));
        l.style_context().add_class("menu-item");
        l.set_xalign(0.0);
        menu_box.pack_start(&l, false, false, 0);
    }

    let sep3 = gtk::Separator::new(Orientation::Horizontal);
    menu_box.pack_start(&sep3, false, false, 4);

    let menu_help_btn = Button::with_label("? Shortcuts");
    menu_help_btn.style_context().add_class("menu-item");
    menu_help_btn.set_halign(gtk::Align::Fill);
    menu_box.pack_start(&menu_help_btn, false, false, 0);

    menu_box.show_all();
    menu_popover.add(&menu_box);

    {
        let popover = menu_popover.clone();
        menu_btn.connect_clicked(move |_| {
            popover.popup();
        });
    }

    let expr_label = Label::new(Some(" "));
    expr_label.style_context().add_class("expression-label");
    expr_label.set_xalign(1.0);
    expr_label.set_hexpand(true);
    expr_label.set_selectable(false);
    expr_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    expr_label.set_opacity(0.0);

    let result_label = Label::new(Some("0"));
    result_label.style_context().add_class("result-label");
    result_label.set_xalign(1.0);
    result_label.set_hexpand(true);
    result_label.set_vexpand(true);
    result_label.set_selectable(false);
    result_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let preview_label = Label::new(Some(" "));
    preview_label.style_context().add_class("preview-label");
    preview_label.set_xalign(1.0);
    preview_label.set_hexpand(true);
    preview_label.set_selectable(false);
    preview_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    preview_label.set_opacity(0.0);

    let display_box = gtk::Box::new(Orientation::Vertical, 0);
    display_box.style_context().add_class("display-area");
    display_box.pack_start(&expr_label, false, false, 0);
    display_box.pack_start(&result_label, true, true, 0);
    display_box.pack_start(&preview_label, false, false, 0);

    let layout_cfg = &config.layout;
    let spacing = layout_cfg.button_spacing as i32;

    let sci_grid = Grid::new();
    sci_grid.style_context().add_class("sci-grid");
    sci_grid.set_row_spacing(spacing as u32);
    sci_grid.set_column_spacing(spacing as u32);
    sci_grid.set_column_homogeneous(true);
    sci_grid.set_row_homogeneous(true);

    let sci_btns: Vec<(&str, &str, ButtonAction, usize, usize)> = vec![
        ("MC", "memory-button", ButtonAction::MemoryClear, 0, 0),
        ("MR", "memory-button", ButtonAction::MemoryRecall, 1, 0),
        ("M+", "memory-button", ButtonAction::MemoryAdd, 2, 0),
        ("M\u{2212}", "memory-button", ButtonAction::MemorySubtract, 0, 1),
        ("(", "paren-button", ButtonAction::LeftParen, 1, 1),
        (")", "paren-button", ButtonAction::RightParen, 2, 1),
        ("Deg", "toggle-button", ButtonAction::ToggleAngleMode, 0, 2),
        ("x\u{00b2}", "power-button", ButtonAction::PostfixOp(PostfixOp::Square), 1, 2),
        ("x\u{00b3}", "power-button", ButtonAction::PostfixOp(PostfixOp::Cube), 2, 2),
        ("x\u{02b8}", "power-button", ButtonAction::BinaryOp(BinaryOp::Power), 0, 3),
        ("\u{215f}x", "power-button", ButtonAction::PostfixOp(PostfixOp::Reciprocal), 1, 3),
        ("\u{221a}", "power-button", ButtonAction::UnaryFunc(UnaryFunc::Sqrt), 2, 3),
        ("\u{00b3}\u{221a}", "power-button", ButtonAction::UnaryFunc(UnaryFunc::Cbrt), 0, 4),
        ("sin", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Sin), 1, 4),
        ("cos", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Cos), 2, 4),
        ("tan", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Tan), 0, 5),
        ("ln", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Ln), 1, 5),
        ("log", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Log10), 2, 5),
        ("n!", "function-button", ButtonAction::PostfixOp(PostfixOp::Factorial), 0, 6),
        ("\u{03c0}", "constant-button", ButtonAction::Constant(std::f64::consts::PI, "\u{03c0}"), 1, 6),
        ("e", "constant-button", ButtonAction::Constant(std::f64::consts::E, "e"), 2, 6),
        ("EE", "function-button", ButtonAction::EE, 0, 7),
        ("sin\u{207b}\u{00b9}", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Asin), 1, 7),
        ("cos\u{207b}\u{00b9}", "function-button", ButtonAction::UnaryFunc(UnaryFunc::Acos), 2, 7),
    ];

    let mut angle_btn_ref = None;
    for (label, class, action, col, row) in sci_btns {
        let b = mk(
            label, class, action, col, row, true,
            &mut action_buttons, &mut nav_buttons,
        );
        sci_grid.attach(&b, col as i32, row as i32, 1, 1);
        if matches!(action, ButtonAction::ToggleAngleMode) {
            angle_btn_ref = Some(b);
        }
    }

    let main_grid = Grid::new();
    main_grid.style_context().add_class("calc-grid");
    main_grid.set_row_spacing(spacing as u32);
    main_grid.set_column_spacing(spacing as u32);
    main_grid.set_column_homogeneous(true);
    main_grid.set_row_homogeneous(true);

    let b = mk("AC", "clear-button", ButtonAction::Clear, 0, 0, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 0, 0, 1, 1);
    let b = mk("+/\u{2212}", "util-button", ButtonAction::ToggleSign, 1, 0, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 1, 0, 1, 1);
    let b = mk("%", "util-button", ButtonAction::PostfixOp(PostfixOp::Percent), 2, 0, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 2, 0, 1, 1);
    let b = mk("\u{00f7}", "op-button", ButtonAction::BinaryOp(BinaryOp::Divide), 3, 0, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 0, 1, 1);

    for (i, d) in ['7', '8', '9'].iter().enumerate() {
        let b = mk(&d.to_string(), "digit-button", ButtonAction::Digit(*d), i, 1, false, &mut action_buttons, &mut nav_buttons);
        main_grid.attach(&b, i as i32, 1, 1, 1);
    }
    let b = mk("\u{00d7}", "op-button", ButtonAction::BinaryOp(BinaryOp::Multiply), 3, 1, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 1, 1, 1);

    for (i, d) in ['4', '5', '6'].iter().enumerate() {
        let b = mk(&d.to_string(), "digit-button", ButtonAction::Digit(*d), i, 2, false, &mut action_buttons, &mut nav_buttons);
        main_grid.attach(&b, i as i32, 2, 1, 1);
    }
    let b = mk("\u{2212}", "op-button", ButtonAction::BinaryOp(BinaryOp::Subtract), 3, 2, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 2, 1, 1);

    for (i, d) in ['1', '2', '3'].iter().enumerate() {
        let b = mk(&d.to_string(), "digit-button", ButtonAction::Digit(*d), i, 3, false, &mut action_buttons, &mut nav_buttons);
        main_grid.attach(&b, i as i32, 3, 1, 1);
    }
    let b = mk("+", "op-button", ButtonAction::BinaryOp(BinaryOp::Add), 3, 3, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 3, 1, 1);

    let d0 = mk("0", "digit-button", ButtonAction::Digit('0'), 1, 4, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&d0, 0, 4, 2, 1);
    // Extra nav entry at col=0 so 'j' from the '1' button (col=0) reaches '0'
    nav_buttons.push(NavButton { button: d0.clone(), col: 0, row: 4, scientific: false });
    let b = mk(".", "digit-button", ButtonAction::Decimal, 2, 4, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 2, 4, 1, 1);
    let b = mk("=", "equals-button", ButtonAction::Equals, 3, 4, false, &mut action_buttons, &mut nav_buttons);
    main_grid.attach(&b, 3, 4, 1, 1);

    let panel_revealer = Revealer::new();
    panel_revealer.set_transition_type(RevealerTransitionType::SlideRight);
    panel_revealer.set_transition_duration(200);

    let panel_container = gtk::Box::new(Orientation::Vertical, 4);
    panel_container.style_context().add_class("panel-container");
    panel_container.set_size_request(200, -1);

    let panel_tabs = gtk::Box::new(Orientation::Horizontal, 2);
    panel_tabs.set_margin_top(4);
    panel_tabs.set_margin_start(4);
    panel_tabs.set_margin_end(4);

    let panel_history_btn = Button::with_label("History");
    panel_history_btn.style_context().add_class("panel-tab");
    panel_history_btn.style_context().add_class("active");
    panel_history_btn.set_hexpand(true);
    panel_history_btn.set_can_focus(false);

    let panel_memory_btn = Button::with_label("Memory");
    panel_memory_btn.style_context().add_class("panel-tab");
    panel_memory_btn.set_hexpand(true);
    panel_memory_btn.set_can_focus(false);

    let panel_pinned_btn = Button::with_label("Pinned");
    panel_pinned_btn.style_context().add_class("panel-tab");
    panel_pinned_btn.set_hexpand(true);
    panel_pinned_btn.set_can_focus(false);

    panel_tabs.pack_start(&panel_history_btn, true, true, 0);
    panel_tabs.pack_start(&panel_memory_btn, true, true, 0);
    panel_tabs.pack_start(&panel_pinned_btn, true, true, 0);

    panel_container.pack_start(&panel_tabs, false, false, 0);

    let panel_stack = Stack::new();
    panel_stack.set_transition_type(StackTransitionType::Crossfade);

    let history_panel = gtk::Box::new(Orientation::Vertical, 2);
    let history_search_entry = Entry::new();
    history_search_entry.set_placeholder_text(Some("Search history..."));
    history_search_entry.style_context().add_class("panel-search");
    history_search_entry.set_margin_start(4);
    history_search_entry.set_margin_end(4);
    history_search_entry.set_margin_top(4);
    history_panel.pack_start(&history_search_entry, false, false, 0);

    let history_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    let history_list = gtk::Box::new(Orientation::Vertical, 4);
    history_list.set_margin_start(4);
    history_list.set_margin_end(4);
    let history_empty = Label::new(Some("No calculations yet"));
    history_empty.style_context().add_class("panel-empty");
    history_list.pack_start(&history_empty, false, false, 0);
    history_scroll.add(&history_list);
    history_panel.pack_start(&history_scroll, true, true, 0);

    let history_toolbar = gtk::Box::new(Orientation::Horizontal, 2);
    history_toolbar.set_margin_start(4);
    history_toolbar.set_margin_end(4);
    history_toolbar.set_margin_bottom(4);
    let history_export_json_btn = Button::with_label("JSON");
    history_export_json_btn.style_context().add_class("panel-tab");
    history_export_json_btn.set_can_focus(false);
    let history_export_csv_btn = Button::with_label("CSV");
    history_export_csv_btn.style_context().add_class("panel-tab");
    history_export_csv_btn.set_can_focus(false);
    let history_clear_btn = Button::with_label("Clear");
    history_clear_btn.style_context().add_class("panel-tab");
    history_clear_btn.set_can_focus(false);
    history_toolbar.pack_start(&history_export_json_btn, true, true, 0);
    history_toolbar.pack_start(&history_export_csv_btn, true, true, 0);
    history_toolbar.pack_end(&history_clear_btn, true, true, 0);
    history_panel.pack_start(&history_toolbar, false, false, 0);

    panel_stack.add_named(&history_panel, "history");

    let memory_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    let memory_list = gtk::Box::new(Orientation::Vertical, 4);
    memory_list.set_margin_start(4);
    memory_list.set_margin_end(4);
    let memory_empty = Label::new(Some("No stored values"));
    memory_empty.style_context().add_class("panel-empty");
    memory_list.pack_start(&memory_empty, false, false, 0);
    memory_scroll.add(&memory_list);
    panel_stack.add_named(&memory_scroll, "memory");

    let pinned_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    let pinned_list = gtk::Box::new(Orientation::Vertical, 4);
    pinned_list.set_margin_start(4);
    pinned_list.set_margin_end(4);
    let pinned_empty = Label::new(Some("No pinned results"));
    pinned_empty.style_context().add_class("panel-empty");
    pinned_list.pack_start(&pinned_empty, false, false, 0);
    pinned_scroll.add(&pinned_list);
    panel_stack.add_named(&pinned_scroll, "pinned");

    panel_container.pack_start(&panel_stack, true, true, 0);
    panel_revealer.add(&panel_container);

    let mode_stack = Stack::new();
    mode_stack.set_transition_type(StackTransitionType::SlideLeftRight);
    mode_stack.set_transition_duration(200);

    let calc_view = gtk::Box::new(Orientation::Vertical, 0);
    calc_view.pack_start(&display_box, false, false, 0);

    let grid_box = gtk::Box::new(Orientation::Horizontal, 6);
    grid_box.pack_start(&sci_grid, true, true, 0);
    grid_box.pack_start(&main_grid, true, true, 0);
    calc_view.pack_start(&grid_box, true, true, 0);
    mode_stack.add_named(&calc_view, "calculator");

    let conv_view = gtk::Box::new(Orientation::Vertical, 8);
    conv_view.style_context().add_class("converter-panel");
    conv_view.set_margin_top(8);
    conv_view.set_margin_start(12);
    conv_view.set_margin_end(12);

    let conv_header_box = gtk::Box::new(Orientation::Horizontal, 8);
    let conv_back_btn = Button::with_label("\u{2190} Back");
    conv_back_btn.style_context().add_class("back-button");
    conv_back_btn.set_can_focus(false);
    let conv_header = Label::new(Some("Unit Converter"));
    conv_header.style_context().add_class("mode-header");
    conv_header_box.pack_start(&conv_back_btn, false, false, 0);
    conv_header_box.pack_start(&conv_header, false, false, 8);
    conv_view.pack_start(&conv_header_box, false, false, 0);

    let conv_cat_box = gtk::Box::new(Orientation::Horizontal, 4);
    let mut conv_cat_btns = Vec::new();
    for cat in ConvertCategory::ALL {
        let btn = Button::with_label(cat.name());
        btn.style_context().add_class("converter-cat-btn");
        btn.set_hexpand(true);
        btn.set_can_focus(false);
        conv_cat_box.pack_start(&btn, true, true, 0);
        conv_cat_btns.push(btn);
    }
    if let Some(first) = conv_cat_btns.first() {
        first.style_context().add_class("active");
    }
    conv_view.pack_start(&conv_cat_box, false, false, 0);

    let from_label = Label::new(Some("From:"));
    from_label.set_xalign(0.0);
    conv_view.pack_start(&from_label, false, false, 0);

    let conv_from_box = gtk::Box::new(Orientation::Horizontal, 8);
    let conv_value_entry = Entry::new();
    conv_value_entry.set_text("1");
    conv_value_entry.set_hexpand(true);
    let conv_from_combo = ComboBoxText::new();
    conv_from_box.pack_start(&conv_value_entry, true, true, 0);
    conv_from_box.pack_start(&conv_from_combo, false, false, 0);
    conv_view.pack_start(&conv_from_box, false, false, 0);

    let conv_swap_btn = Button::with_label("\u{21c5} Swap");
    conv_swap_btn.style_context().add_class("converter-swap");
    conv_swap_btn.set_halign(gtk::Align::Center);
    conv_swap_btn.set_can_focus(false);
    conv_view.pack_start(&conv_swap_btn, false, false, 0);

    let to_label = Label::new(Some("To:"));
    to_label.set_xalign(0.0);
    conv_view.pack_start(&to_label, false, false, 0);

    let conv_to_box = gtk::Box::new(Orientation::Horizontal, 8);
    let conv_result_label = Label::new(Some("1"));
    conv_result_label.style_context().add_class("converter-result");
    conv_result_label.set_xalign(1.0);
    conv_result_label.set_hexpand(true);
    let conv_to_combo = ComboBoxText::new();
    conv_to_box.pack_start(&conv_result_label, true, true, 0);
    conv_to_box.pack_start(&conv_to_combo, false, false, 0);
    conv_view.pack_start(&conv_to_box, false, false, 0);

    for (abbr, _name) in ConvertCategory::Length.units() {
        conv_from_combo.append_text(abbr);
        conv_to_combo.append_text(abbr);
    }
    conv_from_combo.set_active(Some(0));
    conv_to_combo.set_active(Some(1));

    let tools_view = gtk::Box::new(Orientation::Vertical, 8);
    tools_view.style_context().add_class("tools-panel");
    tools_view.set_margin_top(8);
    tools_view.set_margin_start(12);
    tools_view.set_margin_end(12);

    let tools_header_box = gtk::Box::new(Orientation::Horizontal, 8);
    let tools_back_btn = Button::with_label("\u{2190} Back");
    tools_back_btn.style_context().add_class("back-button");
    tools_back_btn.set_can_focus(false);
    let tools_header = Label::new(Some("Quick Tools"));
    tools_header.style_context().add_class("mode-header");
    tools_header_box.pack_start(&tools_back_btn, false, false, 0);
    tools_header_box.pack_start(&tools_header, false, false, 8);
    tools_view.pack_start(&tools_header_box, false, false, 0);

    let tools_notebook = Notebook::new();

    let tip_page = gtk::Box::new(Orientation::Vertical, 8);
    tip_page.set_margin_top(12);
    tip_page.set_margin_start(8);
    tip_page.set_margin_end(8);
    let tip_lbl = Label::new(Some("Bill amount:"));
    tip_lbl.set_xalign(0.0);
    tip_page.pack_start(&tip_lbl, false, false, 0);
    let tip_amount_entry = Entry::new();
    tip_amount_entry.set_placeholder_text(Some("0.00"));
    tip_page.pack_start(&tip_amount_entry, false, false, 0);

    let tip_pct_box = gtk::Box::new(Orientation::Horizontal, 4);
    let tip_pcts = [15.0, 18.0, 20.0, 25.0];
    let mut tip_pct_btns = Vec::new();
    for pct in &tip_pcts {
        let btn = Button::with_label(&format!("{}%", pct));
        btn.style_context().add_class("tools-pct-btn");
        btn.set_hexpand(true);
        tip_pct_box.pack_start(&btn, true, true, 0);
        tip_pct_btns.push((btn, *pct));
    }
    tip_page.pack_start(&tip_pct_box, false, false, 0);

    let custom_box = gtk::Box::new(Orientation::Horizontal, 4);
    let custom_lbl = Label::new(Some("Custom %:"));
    let tip_custom_entry = Entry::new();
    tip_custom_entry.set_placeholder_text(Some("20"));
    tip_custom_entry.set_hexpand(true);
    custom_box.pack_start(&custom_lbl, false, false, 0);
    custom_box.pack_start(&tip_custom_entry, true, true, 0);
    tip_page.pack_start(&custom_box, false, false, 0);

    let tip_result_label = Label::new(Some("Tip: 0  |  Total: 0"));
    tip_result_label.style_context().add_class("tools-result");
    tip_page.pack_start(&tip_result_label, false, false, 8);

    tools_notebook.append_page(&tip_page, Some(&Label::new(Some("Tip"))));

    let disc_page = gtk::Box::new(Orientation::Vertical, 8);
    disc_page.set_margin_top(12);
    disc_page.set_margin_start(8);
    disc_page.set_margin_end(8);
    let disc_lbl1 = Label::new(Some("Original price:"));
    disc_lbl1.set_xalign(0.0);
    disc_page.pack_start(&disc_lbl1, false, false, 0);
    let discount_price_entry = Entry::new();
    discount_price_entry.set_placeholder_text(Some("0.00"));
    disc_page.pack_start(&discount_price_entry, false, false, 0);
    let disc_lbl2 = Label::new(Some("Discount %:"));
    disc_lbl2.set_xalign(0.0);
    disc_page.pack_start(&disc_lbl2, false, false, 0);
    let discount_pct_entry = Entry::new();
    discount_pct_entry.set_placeholder_text(Some("10"));
    disc_page.pack_start(&discount_pct_entry, false, false, 0);
    let discount_result_label = Label::new(Some("Save: 0  |  Final: 0"));
    discount_result_label.style_context().add_class("tools-result");
    disc_page.pack_start(&discount_result_label, false, false, 8);

    tools_notebook.append_page(&disc_page, Some(&Label::new(Some("Discount"))));

    let tax_page = gtk::Box::new(Orientation::Vertical, 8);
    tax_page.set_margin_top(12);
    tax_page.set_margin_start(8);
    tax_page.set_margin_end(8);
    let tax_lbl1 = Label::new(Some("Amount:"));
    tax_lbl1.set_xalign(0.0);
    tax_page.pack_start(&tax_lbl1, false, false, 0);
    let tax_amount_entry = Entry::new();
    tax_amount_entry.set_placeholder_text(Some("0.00"));
    tax_page.pack_start(&tax_amount_entry, false, false, 0);
    let tax_lbl2 = Label::new(Some("Tax rate %:"));
    tax_lbl2.set_xalign(0.0);
    tax_page.pack_start(&tax_lbl2, false, false, 0);
    let tax_rate_entry = Entry::new();
    tax_rate_entry.set_placeholder_text(Some("25"));
    tax_page.pack_start(&tax_rate_entry, false, false, 0);
    let tax_result_label = Label::new(Some("Tax: 0  |  Total: 0"));
    tax_result_label.style_context().add_class("tools-result");
    tax_page.pack_start(&tax_result_label, false, false, 8);

    tools_notebook.append_page(&tax_page, Some(&Label::new(Some("Tax"))));

    tools_view.pack_start(&tools_notebook, true, true, 0);

    let notes_view = gtk::Box::new(Orientation::Vertical, 8);
    notes_view.style_context().add_class("notes-panel");
    notes_view.set_margin_top(8);
    notes_view.set_margin_start(12);
    notes_view.set_margin_end(12);

    let notes_header_box = gtk::Box::new(Orientation::Horizontal, 8);
    let notes_back_btn = Button::with_label("\u{2190} Back");
    notes_back_btn.style_context().add_class("back-button");
    notes_back_btn.set_can_focus(false);
    let notes_header = Label::new(Some("Math Notes"));
    notes_header.style_context().add_class("mode-header");
    let notes_hint = Label::new(Some("One expression per line. Auto-evaluates."));
    notes_hint.style_context().add_class("panel-item-label");
    notes_header_box.pack_start(&notes_back_btn, false, false, 0);
    notes_header_box.pack_start(&notes_header, false, false, 8);
    notes_view.pack_start(&notes_header_box, false, false, 0);
    notes_view.pack_start(&notes_hint, false, false, 0);

    let notes_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    notes_scroll.set_vexpand(true);
    let notes_textview = TextView::new();
    notes_textview.set_wrap_mode(gtk::WrapMode::Word);
    notes_scroll.add(&notes_textview);
    notes_view.pack_start(&notes_scroll, true, true, 0);

    let results_label = Label::new(Some("Results:"));
    results_label.set_xalign(0.0);
    results_label.style_context().add_class("panel-item-label");
    notes_view.pack_start(&results_label, false, false, 0);

    let notes_result_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    notes_result_scroll.set_min_content_height(100);
    let notes_result_label = Label::new(None);
    notes_result_label.style_context().add_class("notes-result");
    notes_result_label.set_xalign(0.0);
    notes_result_label.set_yalign(0.0);
    notes_result_label.set_selectable(true);
    notes_result_scroll.add(&notes_result_label);
    notes_view.pack_start(&notes_result_scroll, true, true, 0);

    let mode_panel_stack = Stack::new();
    mode_panel_stack.set_transition_type(StackTransitionType::Crossfade);
    mode_panel_stack.set_transition_duration(150);
    mode_panel_stack.add_named(&conv_view, "converter");
    mode_panel_stack.add_named(&tools_view, "tools");
    mode_panel_stack.add_named(&notes_view, "notes");

    let mode_panel_container = gtk::Box::new(Orientation::Vertical, 0);
    mode_panel_container.style_context().add_class("mode-panel-container");
    mode_panel_container.pack_start(&mode_panel_stack, true, true, 0);

    let mode_panel_revealer = Revealer::new();
    mode_panel_revealer.set_transition_type(RevealerTransitionType::SlideLeft);
    mode_panel_revealer.set_transition_duration(200);
    mode_panel_revealer.add(&mode_panel_container);
    mode_panel_revealer.set_reveal_child(false);

    let content_box = gtk::Box::new(Orientation::Horizontal, 0);
    content_box.pack_start(&panel_revealer, false, false, 0);
    content_box.pack_start(&mode_stack, true, true, 0);
    content_box.pack_start(&mode_panel_revealer, false, false, 0);

    let vbox = gtk::Box::new(Orientation::Vertical, 0);
    vbox.pack_start(&outer_tab_bar, false, false, 0);
    vbox.pack_start(&content_box, true, true, 0);

    window.add(&vbox);

    CalculatorUI {
        window,
        expr_label,
        result_label,
        preview_label,
        sci_grid,
        nav_buttons,
        action_buttons,
        tab_bar,
        tab_add_btn,
        menu_popover,
        menu_basic_btn,
        menu_sci_btn,
        menu_help_btn,
        menu_notes_btn,
        menu_converter_btn,
        menu_tools_btn,
        menu_theme_btns,
        panel_revealer,
        panel_history_btn,
        panel_memory_btn,
        panel_pinned_btn,
        history_search_entry,
        history_export_json_btn,
        history_export_csv_btn,
        history_clear_btn,
        history_list,
        memory_list,
        pinned_list,
        panel_stack,
        mode_panel_revealer,
        mode_panel_stack,
        conv_value_entry,
        conv_from_combo,
        conv_to_combo,
        conv_result_label,
        conv_cat_btns,
        conv_swap_btn,
        conv_back_btn,
        tip_amount_entry,
        tip_pct_btns,
        tip_custom_entry,
        tip_result_label,
        discount_price_entry,
        discount_pct_entry,
        discount_result_label,
        tax_amount_entry,
        tax_rate_entry,
        tax_result_label,
        tools_back_btn,
        notes_textview,
        notes_result_label,
        notes_back_btn,
        angle_btn: angle_btn_ref,
    }
}
