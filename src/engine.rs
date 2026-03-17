use crate::eval::{
    apply_postfix, apply_unary, evaluate, format_number, token_display, AngleMode, BinaryOp,
    PostfixOp, Token, UnaryFunc,
};

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
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            buffer: String::new(),
            result: None,
            last_value: 0.0,
            memory: 0.0,
            angle_mode: AngleMode::Degrees,
            error: None,
            open_parens: 0,
            user_calculated: false,
        }
    }

    /// The main large display: shows expression during input, result after =.
    pub fn main_display_text(&self) -> String {
        if let Some(ref err) = self.error {
            return err.clone();
        }
        if self.user_calculated {
            if let Some(result) = self.result {
                return format_number(result);
            }
        }
        // During input: show expression being built
        let mut s = String::new();
        for token in &self.tokens {
            s.push_str(&token_display(token));
        }
        s.push_str(&self.buffer);
        if s.is_empty() {
            return format_number(self.last_value);
        }
        s
    }

    /// The secondary display: shows expression only after calculation.
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

    pub fn result_text(&self) -> String {
        if let Some(ref err) = self.error {
            return err.clone();
        }
        if let Some(result) = self.result {
            return format_number(result);
        }
        if !self.buffer.is_empty() {
            return self.buffer.clone();
        }
        format_number(self.last_value)
    }

    pub fn angle_mode(&self) -> AngleMode {
        self.angle_mode
    }

    pub fn has_memory(&self) -> bool {
        self.memory != 0.0
    }

    // --- Input methods ---

    pub fn input_digit(&mut self, digit: char) {
        if self.error.is_some() {
            return;
        }
        self.start_fresh_if_needed();

        // Implicit multiply: digit after constant/paren/postfix
        if self.buffer.is_empty() {
            if matches!(
                self.tokens.last(),
                Some(Token::Constant(..) | Token::RightParen | Token::PostfixOp(_))
            ) {
                self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
            }
        }

        if digit == '0' && (self.buffer == "0" || self.buffer == "-0") {
            return;
        }
        if digit != '0' && self.buffer == "0" {
            self.buffer.clear();
        }
        if digit != '0' && self.buffer == "-0" {
            self.buffer = "-".to_string();
        }
        self.buffer.push(digit);
    }

    pub fn input_decimal(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.start_fresh_if_needed();
        if self.buffer.is_empty() {
            self.buffer.push('0');
        }
        if !self.buffer.contains('.') {
            self.buffer.push('.');
        }
    }

    pub fn input_binary_op(&mut self, op: BinaryOp) {
        if self.error.is_some() {
            return;
        }

        // Unary minus
        if op == BinaryOp::Subtract && self.buffer.is_empty() && self.result.is_none() {
            let needs_unary = self.tokens.is_empty()
                || matches!(
                    self.tokens.last(),
                    Some(Token::BinaryOp(_) | Token::LeftParen | Token::UnaryFunc(_))
                );
            if needs_unary {
                self.buffer = "-".to_string();
                return;
            }
        }

        // Ignore op at start with no value (except minus handled above)
        if self.tokens.is_empty() && self.buffer.is_empty() && self.result.is_none() {
            return;
        }

        if let Some(result) = self.result.take() {
            self.tokens.clear();
            self.tokens.push(Token::Number(result));
            self.user_calculated = false;
        } else {
            self.finalize_buffer();
        }

        // Replace consecutive operators
        if matches!(self.tokens.last(), Some(Token::BinaryOp(_))) {
            self.tokens.pop();
        }
        self.tokens.push(Token::BinaryOp(op));
    }

    pub fn input_unary_func(&mut self, func: UnaryFunc) {
        if self.error.is_some() {
            return;
        }

        // Apply to result immediately
        if let Some(result) = self.result.take() {
            self.tokens.clear();
            self.user_calculated = false;
            self.error = None;
            match apply_unary(func, result, self.angle_mode) {
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

        // Implicit multiply: func after number/constant/paren
        if !self.buffer.is_empty() {
            self.finalize_buffer();
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        } else if matches!(
            self.tokens.last(),
            Some(Token::Number(_) | Token::Constant(..) | Token::RightParen | Token::PostfixOp(_))
        ) {
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        }

        self.tokens.push(Token::UnaryFunc(func));
        self.tokens.push(Token::LeftParen);
        self.open_parens += 1;
    }

    pub fn input_postfix_op(&mut self, op: PostfixOp) {
        if self.error.is_some() {
            return;
        }

        if let Some(result) = self.result.take() {
            self.tokens.clear();
            self.user_calculated = false;
            self.error = None;
            match apply_postfix(op, result) {
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

        // Auto-evaluate for immediate feedback
        match evaluate(&self.tokens, self.angle_mode) {
            Ok(val) => {
                self.result = Some(val);
                self.last_value = val;
            }
            Err(msg) => self.error = Some(msg),
        }
    }

    pub fn input_constant(&mut self, value: f64, name: &'static str) {
        if self.error.is_some() {
            return;
        }
        self.start_fresh_if_needed();

        if !self.buffer.is_empty() {
            self.finalize_buffer();
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        } else if matches!(
            self.tokens.last(),
            Some(Token::Number(_) | Token::Constant(..) | Token::RightParen | Token::PostfixOp(_))
        ) {
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        }

        self.tokens.push(Token::Constant(name, value));
        self.last_value = value;
    }

    pub fn input_left_paren(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.start_fresh_if_needed();

        if !self.buffer.is_empty() {
            self.finalize_buffer();
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        } else if matches!(
            self.tokens.last(),
            Some(Token::Number(_) | Token::Constant(..) | Token::RightParen | Token::PostfixOp(_))
        ) {
            self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        }

        self.tokens.push(Token::LeftParen);
        self.open_parens += 1;
    }

    pub fn input_right_paren(&mut self) {
        if self.error.is_some() || self.open_parens == 0 {
            return;
        }
        self.finalize_buffer();
        self.tokens.push(Token::RightParen);
        self.open_parens -= 1;
    }

    pub fn input_ee(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.finalize_buffer();
        self.tokens.push(Token::BinaryOp(BinaryOp::Multiply));
        self.tokens.push(Token::Number(10.0));
        self.tokens.push(Token::BinaryOp(BinaryOp::Power));
    }

    pub fn calculate(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.finalize_buffer();
        for _ in 0..self.open_parens {
            self.tokens.push(Token::RightParen);
        }
        self.open_parens = 0;

        match evaluate(&self.tokens, self.angle_mode) {
            Ok(val) => {
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

    pub fn clear_entry(&mut self) {
        self.buffer.clear();
    }

    pub fn backspace(&mut self) {
        if self.result.is_some() || self.error.is_some() {
            return;
        }
        if !self.buffer.is_empty() {
            self.buffer.pop();
        } else if let Some(token) = self.tokens.pop() {
            match token {
                Token::RightParen => {
                    self.open_parens += 1;
                }
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
        if self.error.is_some() {
            return;
        }
        if !self.buffer.is_empty() {
            if self.buffer.starts_with('-') {
                self.buffer.remove(0);
            } else {
                self.buffer.insert(0, '-');
            }
        }
    }

    // --- Memory ---

    pub fn memory_clear(&mut self) {
        self.memory = 0.0;
    }

    pub fn memory_recall(&mut self) {
        self.start_fresh_if_needed();
        self.buffer = format_number(self.memory);
        self.last_value = self.memory;
    }

    pub fn memory_add(&mut self) {
        if let Some(r) = self.result {
            self.memory += r;
        } else if let Ok(v) = self.buffer.parse::<f64>() {
            self.memory += v;
        }
    }

    pub fn memory_subtract(&mut self) {
        if let Some(r) = self.result {
            self.memory -= r;
        } else if let Ok(v) = self.buffer.parse::<f64>() {
            self.memory -= v;
        }
    }

    pub fn toggle_angle_mode(&mut self) {
        self.angle_mode = match self.angle_mode {
            AngleMode::Radians => AngleMode::Degrees,
            AngleMode::Degrees => AngleMode::Radians,
        };
    }

    // --- Internal ---

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

    #[test]
    fn expression_display() {
        let mut e = Engine::new();
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        assert_eq!(e.expression_text(), "2+3");
        assert_eq!(e.result_text(), "3");
    }

    #[test]
    fn calculate_shows_equals() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.calculate();
        assert_eq!(e.expression_text(), "5+3=");
        assert_eq!(e.result_text(), "8");
    }

    #[test]
    fn chain_from_result() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.calculate();
        e.input_binary_op(BinaryOp::Multiply);
        e.input_digit('2');
        e.calculate();
        assert_eq!(e.result_text(), "16");
    }

    #[test]
    fn precedence_in_engine() {
        let mut e = Engine::new();
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.input_binary_op(BinaryOp::Multiply);
        e.input_digit('4');
        e.calculate();
        assert_eq!(e.result_text(), "14");
    }

    #[test]
    fn unary_func_on_result() {
        let mut e = Engine::new();
        e.input_digit('3');
        e.input_digit('0');
        e.calculate();
        e.input_unary_func(UnaryFunc::Sin);
        let val: f64 = e.result_text().parse().unwrap();
        assert!((val - 0.5).abs() < 1e-10);
    }

    #[test]
    fn postfix_auto_evaluates() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.input_postfix_op(PostfixOp::Square);
        assert_eq!(e.result_text(), "25");
    }

    #[test]
    fn constant_display() {
        let mut e = Engine::new();
        e.input_constant(std::f64::consts::PI, "π");
        assert!(e.expression_text().contains('π'));
    }

    #[test]
    fn unary_minus() {
        let mut e = Engine::new();
        e.input_binary_op(BinaryOp::Subtract);
        e.input_digit('5');
        e.calculate();
        assert_eq!(e.result_text(), "-5");
    }

    #[test]
    fn memory_operations() {
        let mut e = Engine::new();
        e.input_digit('4');
        e.input_digit('2');
        e.calculate();
        e.memory_add();
        assert!(e.has_memory());
        e.clear();
        e.memory_recall();
        assert_eq!(e.result_text(), "42");
        e.memory_clear();
        assert!(!e.has_memory());
    }

    #[test]
    fn division_by_zero_error() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.input_binary_op(BinaryOp::Divide);
        e.input_digit('0');
        e.calculate();
        assert_eq!(e.result_text(), "Division by zero");
        e.clear();
        assert_eq!(e.result_text(), "0");
    }

    #[test]
    fn parentheses() {
        let mut e = Engine::new();
        e.input_left_paren();
        e.input_digit('2');
        e.input_binary_op(BinaryOp::Add);
        e.input_digit('3');
        e.input_right_paren();
        e.input_binary_op(BinaryOp::Multiply);
        e.input_digit('4');
        e.calculate();
        assert_eq!(e.result_text(), "20");
        assert_eq!(e.expression_text(), "(2+3)×4=");
    }
}
