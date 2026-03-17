use super::eval;
use super::types::*;

#[derive(Debug, Clone)]
struct Snapshot {
    tokens: Vec<Token>,
    buffer: String,
    result: Option<f64>,
    last_value: f64,
    error: Option<String>,
    open_parens: usize,
    user_calculated: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct EvalSettings {
    pub angle_mode: AngleMode,
    pub standard_precedence: bool,
    pub auto_evaluate: bool,
    pub max_history: usize,
}

impl Default for EvalSettings {
    fn default() -> Self {
        Self {
            angle_mode: AngleMode::Degrees,
            standard_precedence: true,
            auto_evaluate: true,
            max_history: 200,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    tokens: Vec<Token>,
    buffer: String,
    result: Option<f64>,
    last_value: f64,
    memory: f64,
    angle_mode: AngleMode,
    error: Option<String>,
    open_parens: usize,
    user_calculated: bool,
    undo_stack: Vec<Snapshot>,
    pub history: Vec<HistoryEntry>,
    pub memory_slots: Vec<MemorySlot>,
    pub pinned: Vec<PinnedCalc>,
    pub note: String,
    settings: EvalSettings,
}

impl Engine {
    pub fn new(settings: EvalSettings) -> Self {
        Self {
            tokens: Vec::new(),
            buffer: String::new(),
            result: None,
            last_value: 0.0,
            memory: 0.0,
            angle_mode: settings.angle_mode,
            error: None,
            open_parens: 0,
            user_calculated: false,
            undo_stack: Vec::new(),
            history: Vec::new(),
            memory_slots: Vec::new(),
            pinned: Vec::new(),
            note: String::new(),
            settings,
        }
    }

    fn save_snapshot(&mut self) {
        self.undo_stack.push(Snapshot {
            tokens: self.tokens.clone(),
            buffer: self.buffer.clone(),
            result: self.result,
            last_value: self.last_value,
            error: self.error.clone(),
            open_parens: self.open_parens,
            user_calculated: self.user_calculated,
        });
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) {
        if let Some(snap) = self.undo_stack.pop() {
            self.tokens = snap.tokens;
            self.buffer = snap.buffer;
            self.result = snap.result;
            self.last_value = snap.last_value;
            self.error = snap.error;
            self.open_parens = snap.open_parens;
            self.user_calculated = snap.user_calculated;
        }
    }

    pub fn auto_eval(&self) -> Option<String> {
        if !self.settings.auto_evaluate {
            return None;
        }
        if self.error.is_some() || self.user_calculated {
            return None;
        }
        let mut tokens = self.tokens.clone();
        if !self.buffer.is_empty() {
            if let Ok(val) = self.buffer.parse::<f64>() {
                tokens.push(Token::Number(val));
            }
        }
        if tokens.is_empty() {
            return None;
        }
        for _ in 0..self.open_parens {
            tokens.push(Token::RightParen);
        }
        match eval::evaluate(&tokens, self.angle_mode, self.settings.standard_precedence) {
            Ok(val) => {
                let text = format_number_default(val);
                let current = self.main_display_text();
                if text != current { Some(text) } else { None }
            }
            Err(_) => None,
        }
    }

    pub fn main_display_text(&self) -> String {
        if let Some(ref err) = self.error {
            return err.clone();
        }
        if self.user_calculated {
            if let Some(result) = self.result {
                return format_number_default(result);
            }
        }
        let mut s = String::new();
        for token in &self.tokens {
            s.push_str(&token_display(token));
        }
        s.push_str(&self.buffer);
        if s.is_empty() {
            return format_number_default(self.last_value);
        }
        s
    }

    pub fn secondary_display_text(&self) -> String {
        if self.user_calculated && self.result.is_some() {
            let mut s = String::new();
            for token in &self.tokens {
                s.push_str(&token_display(token));
            }
            s.push('=');
            return s;
        }
        String::new()
    }

    pub fn show_secondary(&self) -> bool {
        self.user_calculated && self.result.is_some()
    }

    pub fn expression_text(&self) -> String {
        let mut s = String::new();
        for token in &self.tokens {
            s.push_str(&token_display(token));
        }
        s.push_str(&self.buffer);
        if self.user_calculated && self.result.is_some() {
            s.push('=');
        }
        s
    }

    pub fn angle_mode(&self) -> AngleMode {
        self.angle_mode
    }

    pub fn has_memory(&self) -> bool {
        self.memory != 0.0
    }

    pub fn current_value(&self) -> f64 {
        if let Some(r) = self.result {
            r
        } else if let Ok(v) = self.buffer.parse::<f64>() {
            v
        } else {
            self.last_value
        }
    }

    pub fn input_digit(&mut self, digit: char) {
        if self.error.is_some() { return; }
        self.save_snapshot();
        self.start_fresh_if_needed();
        if self.buffer.is_empty()
            && matches!(self.tokens.last(), Some(Token::Constant(..) | Token::RightParen | Token::PostfixOp(_)))
        {
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        }
        if digit == '0' && (self.buffer == "0" || self.buffer == "-0") { return; }
        if digit != '0' && self.buffer == "0" { self.buffer.clear(); }
        if digit != '0' && self.buffer == "-0" { self.buffer = "-".to_string(); }
        self.buffer.push(digit);
    }

    pub fn input_decimal(&mut self) {
        if self.error.is_some() { return; }
        self.save_snapshot();
        self.start_fresh_if_needed();
        if self.buffer.is_empty() { self.buffer.push('0'); }
        if !self.buffer.contains('.') { self.buffer.push('.'); }
    }

    pub fn input_binary_op(&mut self, op: BinaryOp) {
        if self.error.is_some() { return; }
        self.save_snapshot();

        if op == BinaryOp::Subtract && self.buffer.is_empty() && self.result.is_none() {
            let needs_unary = self.tokens.is_empty()
                || matches!(self.tokens.last(), Some(Token::BinaryOp(_) | Token::LeftParen | Token::UnaryFunc(_)));
            if needs_unary {
                self.buffer = "-".to_string();
                return;
            }
        }

        if self.tokens.is_empty() && self.buffer.is_empty() && self.result.is_none() { return; }

        if let Some(result) = self.result.take() {
            self.tokens.clear();
            self.tokens.push(Token::Number(result));
            self.user_calculated = false;
        } else {
            self.finalize_buffer();
        }

        if matches!(self.tokens.last(), Some(Token::BinaryOp(_))) {
            self.tokens.pop();
        }
        self.tokens.push(Token::BinaryOp(op));
    }

    pub fn input_unary_func(&mut self, func: UnaryFunc) {
        if self.error.is_some() { return; }
        self.save_snapshot();

        if let Some(result) = self.result.take() {
            self.tokens.clear();
            self.user_calculated = false;
            self.error = None;
            match eval::apply_unary(func, result, self.angle_mode) {
                Ok(val) => {
                    self.tokens.push(Token::UnaryFunc(func));
                    self.tokens.push(Token::LeftParen);
                    self.tokens.push(Token::Number(result));
                    self.tokens.push(Token::RightParen);
                    self.result = Some(val);
                    self.last_value = val;
                }
                Err(msg) => self.error = Some(msg),
            }
            return;
        }

        self.start_fresh_if_needed();
        if !self.buffer.is_empty() {
            self.finalize_buffer();
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        } else if matches!(self.tokens.last(), Some(Token::Number(_) | Token::Constant(..) | Token::RightParen | Token::PostfixOp(_))) {
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        }
        self.tokens.push(Token::UnaryFunc(func));
        self.tokens.push(Token::LeftParen);
        self.open_parens += 1;
    }

    pub fn input_postfix_op(&mut self, op: PostfixOp) {
        if self.error.is_some() { return; }
        self.save_snapshot();

        if let Some(result) = self.result.take() {
            self.tokens.clear();
            self.user_calculated = false;
            self.error = None;
            match eval::apply_postfix(op, result) {
                Ok(val) => {
                    self.tokens.push(Token::Number(result));
                    self.tokens.push(Token::PostfixOp(op));
                    self.result = Some(val);
                    self.last_value = val;
                }
                Err(msg) => self.error = Some(msg),
            }
            return;
        }

        self.finalize_buffer();
        self.tokens.push(Token::PostfixOp(op));
        match eval::evaluate(&self.tokens, self.angle_mode, self.settings.standard_precedence) {
            Ok(val) => {
                self.result = Some(val);
                self.last_value = val;
            }
            Err(msg) => self.error = Some(msg),
        }
    }

    pub fn input_constant(&mut self, value: f64, name: &'static str) {
        if self.error.is_some() { return; }
        self.save_snapshot();
        self.start_fresh_if_needed();
        if !self.buffer.is_empty() {
            self.finalize_buffer();
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        } else if matches!(self.tokens.last(), Some(Token::Number(_) | Token::Constant(..) | Token::RightParen | Token::PostfixOp(_))) {
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        }
        self.tokens.push(Token::Constant(name, value));
        self.last_value = value;
    }

    pub fn input_left_paren(&mut self) {
        if self.error.is_some() { return; }
        self.save_snapshot();
        self.start_fresh_if_needed();
        if !self.buffer.is_empty() {
            self.finalize_buffer();
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        } else if matches!(self.tokens.last(), Some(Token::Number(_) | Token::Constant(..) | Token::RightParen | Token::PostfixOp(_))) {
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        }
        self.tokens.push(Token::LeftParen);
        self.open_parens += 1;
    }

    pub fn input_right_paren(&mut self) {
        if self.error.is_some() || self.open_parens == 0 { return; }
        self.save_snapshot();
        self.finalize_buffer();
        self.tokens.push(Token::RightParen);
        self.open_parens -= 1;
    }

    pub fn input_ee(&mut self) {
        if self.error.is_some() { return; }
        self.save_snapshot();
        self.finalize_buffer();
        self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        self.tokens.push(Token::Number(10.0));
        self.tokens.push(Token::BinaryOp(BinaryOp::Power));
    }

    pub fn calculate(&mut self, timestamp: u64, session: u64) {
        if self.error.is_some() { return; }
        self.save_snapshot();
        self.finalize_buffer();
        for _ in 0..self.open_parens {
            self.tokens.push(Token::RightParen);
        }
        self.open_parens = 0;

        match eval::evaluate(&self.tokens, self.angle_mode, self.settings.standard_precedence) {
            Ok(val) => {
                let mut expr_str = String::new();
                for token in &self.tokens {
                    expr_str.push_str(&token_display(token));
                }
                self.history.push(HistoryEntry {
                    expression: expr_str,
                    result_text: format_number_default(val),
                    result: val,
                    timestamp,
                    session,
                });
                let max = self.settings.max_history;
                if self.history.len() > max {
                    self.history.remove(0);
                }
                self.result = Some(val);
                self.last_value = val;
                self.error = None;
                self.user_calculated = true;
            }
            Err(msg) => {
                self.error = Some(msg);
                self.result = None;
            }
        }
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
        self.buffer.clear();
        self.result = None;
        self.last_value = 0.0;
        self.error = None;
        self.open_parens = 0;
        self.user_calculated = false;
    }

    pub fn backspace(&mut self) {
        if self.result.is_some() || self.error.is_some() { return; }
        self.save_snapshot();
        if !self.buffer.is_empty() {
            self.buffer.pop();
        } else if let Some(token) = self.tokens.pop() {
            match token {
                Token::RightParen => { self.open_parens += 1; }
                Token::LeftParen => {
                    self.open_parens = self.open_parens.saturating_sub(1);
                    if matches!(self.tokens.last(), Some(Token::UnaryFunc(_))) {
                        self.tokens.pop();
                    }
                }
                _ => {}
            }
        }
    }

    pub fn toggle_sign(&mut self) {
        if self.error.is_some() { return; }
        self.save_snapshot();
        if !self.buffer.is_empty() {
            if self.buffer.starts_with('-') {
                self.buffer.remove(0);
            } else {
                self.buffer.insert(0, '-');
            }
        }
    }

    pub fn memory_clear(&mut self) { self.memory = 0.0; }
    pub fn memory_recall(&mut self) {
        self.start_fresh_if_needed();
        self.buffer = format_number_default(self.memory);
        self.last_value = self.memory;
    }
    pub fn memory_add(&mut self) {
        if let Some(r) = self.result { self.memory += r; }
        else if let Ok(v) = self.buffer.parse::<f64>() { self.memory += v; }
    }
    pub fn memory_subtract(&mut self) {
        if let Some(r) = self.result { self.memory -= r; }
        else if let Ok(v) = self.buffer.parse::<f64>() { self.memory -= v; }
    }

    pub fn memory_store(&mut self, label: String) {
        let val = self.current_value();
        self.memory_slots.push(MemorySlot { label, value: val });
    }
    pub fn pin_result(&mut self, label: String) {
        let val = self.current_value();
        let expr = self.expression_text();
        self.pinned.push(PinnedCalc { label, expression: expr, result: val });
    }
    pub fn clear_history(&mut self) { self.history.clear(); }

    pub fn toggle_angle_mode(&mut self) {
        self.angle_mode = match self.angle_mode {
            AngleMode::Radians => AngleMode::Degrees,
            AngleMode::Degrees => AngleMode::Radians,
        };
    }

    fn finalize_buffer(&mut self) {
        if !self.buffer.is_empty() {
            if let Ok(val) = self.buffer.parse::<f64>() {
                self.tokens.push(Token::Number(val));
                self.last_value = val;
            }
            self.buffer.clear();
        }
    }

    fn start_fresh_if_needed(&mut self) {
        if self.result.is_some() || self.error.is_some() {
            self.tokens.clear();
            self.result = None;
            self.error = None;
            self.user_calculated = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> Engine {
        Engine::new(EvalSettings::default())
    }

    #[test]
    fn expression_display() {
        let mut e = engine();
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        assert_eq!(e.expression_text(), "2+3");
    }

    #[test]
    fn calculate_shows_equals() {
        let mut e = engine();
        e.input_digit('5');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.calculate(0, 0);
        assert_eq!(e.expression_text(), "5+3=");
        assert_eq!(e.main_display_text(), "8");
    }

    #[test]
    fn chain_from_result() {
        let mut e = engine();
        e.input_digit('5');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.calculate(0, 0);
        e.input_binary_op(BinaryOp::Multiply);
        e.input_digit('2');
        e.calculate(0, 0);
        assert_eq!(e.main_display_text(), "16");
    }

    #[test]
    fn precedence_in_engine() {
        let mut e = engine();
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.input_binary_op(BinaryOp::Multiply);
        e.input_digit('4');
        e.calculate(0, 0);
        assert_eq!(e.main_display_text(), "14");
    }

    #[test]
    fn no_precedence_in_engine() {
        let mut e = Engine::new(EvalSettings {
            standard_precedence: false,
            ..EvalSettings::default()
        });
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.input_binary_op(BinaryOp::Multiply);
        e.input_digit('4');
        e.calculate(0, 0);
        assert_eq!(e.main_display_text(), "20");
    }

    #[test]
    fn unary_func_on_result() {
        let mut e = engine();
        e.input_digit('3');
        e.input_digit('0');
        e.calculate(0, 0);
        e.input_unary_func(UnaryFunc::Sin);
        assert!((e.current_value() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn postfix_auto_evaluates() {
        let mut e = engine();
        e.input_digit('5');
        e.input_postfix_op(PostfixOp::Square);
        assert_eq!(e.current_value(), 25.0);
    }

    #[test]
    fn constant_display() {
        let mut e = engine();
        e.input_constant(std::f64::consts::PI, "\u{03c0}");
        assert!(e.expression_text().contains('\u{03c0}'));
    }

    #[test]
    fn unary_minus() {
        let mut e = engine();
        e.input_binary_op(BinaryOp::Subtract);
        e.input_digit('5');
        e.calculate(0, 0);
        assert_eq!(e.main_display_text(), "-5");
    }

    #[test]
    fn memory_operations() {
        let mut e = engine();
        e.input_digit('4');
        e.input_digit('2');
        e.calculate(0, 0);
        e.memory_add();
        assert!(e.has_memory());
        e.clear();
        e.memory_recall();
        assert_eq!(e.main_display_text(), "42");
        e.memory_clear();
        assert!(!e.has_memory());
    }

    #[test]
    fn division_by_zero_error() {
        let mut e = engine();
        e.input_digit('5');
        e.input_binary_op(BinaryOp::Divide);
        e.input_digit('0');
        e.calculate(0, 0);
        assert_eq!(e.main_display_text(), "Division by zero");
        e.clear();
        assert_eq!(e.main_display_text(), "0");
    }

    #[test]
    fn parentheses() {
        let mut e = engine();
        e.input_left_paren();
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.input_right_paren();
        e.input_binary_op(BinaryOp::Multiply);
        e.input_digit('4');
        e.calculate(0, 0);
        assert_eq!(e.main_display_text(), "20");
    }

    #[test]
    fn undo_works() {
        let mut e = engine();
        e.input_digit('5');
        e.input_digit('3');
        e.undo();
        assert_eq!(e.main_display_text(), "5");
    }

    #[test]
    fn history_recorded() {
        let mut e = engine();
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.calculate(100, 1);
        assert_eq!(e.history.len(), 1);
        assert_eq!(e.history[0].result, 5.0);
        assert_eq!(e.history[0].timestamp, 100);
        assert_eq!(e.history[0].session, 1);
    }

    #[test]
    fn history_limit() {
        let mut e = Engine::new(EvalSettings { max_history: 3, ..EvalSettings::default() });
        for i in 0..5 {
            e.input_digit(char::from_digit(i % 10, 10).unwrap());
            e.calculate(0, 0);
        }
        assert_eq!(e.history.len(), 3);
    }

    #[test]
    fn auto_eval_preview() {
        let mut e = engine();
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        assert_eq!(e.auto_eval(), Some("5".to_string()));
    }

    #[test]
    fn auto_eval_disabled() {
        let mut e = Engine::new(EvalSettings { auto_evaluate: false, ..EvalSettings::default() });
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        assert_eq!(e.auto_eval(), None);
    }

    #[test]
    fn memory_slots() {
        let mut e = engine();
        e.input_digit('4');
        e.input_digit('2');
        e.calculate(0, 0);
        e.memory_store("test".to_string());
        assert_eq!(e.memory_slots.len(), 1);
        assert_eq!(e.memory_slots[0].value, 42.0);
    }

    #[test]
    fn pinned_calcs() {
        let mut e = engine();
        e.input_digit('1');
        e.input_digit('0');
        e.calculate(0, 0);
        e.pin_result("budget".to_string());
        assert_eq!(e.pinned.len(), 1);
        assert_eq!(e.pinned[0].label, "budget");
    }

    #[test]
    fn backspace_behavior() {
        let mut e = engine();
        e.input_digit('1');
        e.input_digit('2');
        e.input_digit('3');
        e.backspace();
        assert_eq!(e.main_display_text(), "12");
        e.backspace();
        assert_eq!(e.main_display_text(), "1");
    }

    #[test]
    fn toggle_sign() {
        let mut e = engine();
        e.input_digit('5');
        e.toggle_sign();
        assert_eq!(e.main_display_text(), "-5");
        e.toggle_sign();
        assert_eq!(e.main_display_text(), "5");
    }

    #[test]
    fn error_blocks_input() {
        let mut e = engine();
        e.input_digit('5');
        e.input_binary_op(BinaryOp::Divide);
        e.input_digit('0');
        e.calculate(0, 0);
        e.input_digit('3');
        assert_eq!(e.main_display_text(), "Division by zero");
    }

    #[test]
    fn ee_input() {
        let mut e = engine();
        e.input_digit('5');
        e.input_ee();
        e.input_digit('3');
        e.calculate(0, 0);
        assert_eq!(e.main_display_text(), "5000");
    }
}
