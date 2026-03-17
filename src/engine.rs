#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    None,
}

#[derive(Debug)]
pub struct Engine {
    value: f64,
    buffer: String,
    op: Operation,
    error: bool,
    fresh_result: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            buffer: String::new(),
            op: Operation::None,
            error: false,
            fresh_result: false,
        }
    }

    pub fn display_text(&self) -> String {
        if self.error {
            return "Error".to_string();
        }
        if !self.buffer.is_empty() {
            self.buffer.clone()
        } else {
            format_number(self.value)
        }
    }

    pub fn active_op(&self) -> Operation {
        self.op
    }

    pub fn clear(&mut self) {
        self.value = 0.0;
        self.buffer.clear();
        self.op = Operation::None;
        self.error = false;
        self.fresh_result = false;
    }

    pub fn input_digit(&mut self, digit: char) {
        if self.error {
            return;
        }
        if self.fresh_result {
            self.buffer.clear();
            self.fresh_result = false;
        }
        // Prevent leading zeros ("00", "007") but allow "0." via input_decimal
        if digit == '0' && (self.buffer == "0" || self.buffer == "-0") {
            return;
        }
        if digit != '0' && self.buffer == "0" {
            self.buffer.clear();
        }
        if digit != '0' && self.buffer == "-0" {
            self.buffer = String::from("-");
        }
        self.buffer.push(digit);
    }

    pub fn input_decimal(&mut self) {
        if self.error {
            return;
        }
        if self.fresh_result {
            self.buffer = String::from("0");
            self.fresh_result = false;
        }
        if self.buffer.is_empty() {
            self.buffer.push('0');
        }
        if !self.buffer.contains('.') {
            self.buffer.push('.');
        }
    }

    pub fn set_operation(&mut self, op: Operation) {
        if self.error {
            return;
        }
        self.fresh_result = false;

        // Chain: if there's a pending op and new input, evaluate first
        if self.op != Operation::None && !self.buffer.is_empty() {
            self.evaluate();
            if self.error {
                return;
            }
        } else if !self.buffer.is_empty() {
            self.value = self.buffer.parse().unwrap_or(0.0);
            self.buffer.clear();
        }

        self.op = op;
    }

    pub fn calculate(&mut self) {
        if self.error {
            return;
        }
        self.evaluate();
    }

    pub fn backspace(&mut self) {
        if self.error || self.fresh_result {
            return;
        }
        self.buffer.pop();
    }

    pub fn toggle_sign(&mut self) {
        if self.error {
            return;
        }
        if !self.buffer.is_empty() {
            if self.buffer.starts_with('-') {
                self.buffer.remove(0);
            } else {
                self.buffer.insert(0, '-');
            }
        } else {
            self.value = -self.value;
        }
    }

    pub fn percent(&mut self) {
        if self.error {
            return;
        }
        if !self.buffer.is_empty() {
            if let Ok(val) = self.buffer.parse::<f64>() {
                self.buffer = format_number(val / 100.0);
            }
        } else {
            self.value /= 100.0;
        }
    }

    fn evaluate(&mut self) {
        let rhs = if self.buffer.is_empty() {
            0.0
        } else {
            self.buffer.parse().unwrap_or(0.0)
        };

        self.buffer.clear();

        self.value = match self.op {
            Operation::Add => self.value + rhs,
            Operation::Subtract => self.value - rhs,
            Operation::Multiply => self.value * rhs,
            Operation::Divide => {
                if rhs.abs() < f64::EPSILON {
                    self.error = true;
                    return;
                }
                self.value / rhs
            }
            Operation::None => rhs,
        };

        self.op = Operation::None;
        self.fresh_result = true;
    }
}

pub fn format_number(val: f64) -> String {
    if val.is_nan() || val.is_infinite() {
        return "Error".to_string();
    }
    if val == 0.0 {
        return "0".to_string();
    }
    if val.fract() == 0.0 && val.abs() < 1e15 {
        return format!("{}", val as i64);
    }
    if val.abs() >= 1e15 || val.abs() < 1e-4 {
        return format!("{:e}", val);
    }
    let s = format!("{:.10}", val);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_addition() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.set_operation(Operation::Add);
        e.input_digit('3');
        e.calculate();
        assert_eq!(e.display_text(), "8");
    }

    #[test]
    fn operation_chaining() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.set_operation(Operation::Add);
        e.input_digit('3');
        e.set_operation(Operation::Add);
        e.input_digit('2');
        e.calculate();
        assert_eq!(e.display_text(), "10");
    }

    #[test]
    fn division_by_zero() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.set_operation(Operation::Divide);
        e.input_digit('0');
        e.calculate();
        assert_eq!(e.display_text(), "Error");
    }

    #[test]
    fn no_leading_zeros() {
        let mut e = Engine::new();
        e.input_digit('0');
        e.input_digit('0');
        e.input_digit('7');
        assert_eq!(e.display_text(), "7");
    }

    #[test]
    fn decimal_input() {
        let mut e = Engine::new();
        e.input_decimal();
        e.input_digit('5');
        assert_eq!(e.display_text(), "0.5");
    }

    #[test]
    fn toggle_sign() {
        let mut e = Engine::new();
        e.input_digit('4');
        e.input_digit('2');
        e.toggle_sign();
        assert_eq!(e.display_text(), "-42");
        e.toggle_sign();
        assert_eq!(e.display_text(), "42");
    }

    #[test]
    fn percent() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.input_digit('0');
        e.percent();
        assert_eq!(e.display_text(), "0.5");
    }

    #[test]
    fn backspace() {
        let mut e = Engine::new();
        e.input_digit('1');
        e.input_digit('2');
        e.input_digit('3');
        e.backspace();
        assert_eq!(e.display_text(), "12");
    }

    #[test]
    fn fresh_result_then_digit() {
        let mut e = Engine::new();
        e.input_digit('5');
        e.set_operation(Operation::Add);
        e.input_digit('3');
        e.calculate(); 
        e.input_digit('2'); 
        assert_eq!(e.display_text(), "2");
    }

    #[test]
    fn format_integers() {
        assert_eq!(format_number(8.0), "8");
        assert_eq!(format_number(-42.0), "-42");
        assert_eq!(format_number(0.0), "0");
    }

    #[test]
    fn format_decimals() {
        assert_eq!(format_number(3.14), "3.14");
        assert_eq!(format_number(0.5), "0.5");
    }

    #[test]
    fn multiplication() {
        let mut e = Engine::new();
        e.input_digit('6');
        e.set_operation(Operation::Multiply);
        e.input_digit('7');
        e.calculate();
        assert_eq!(e.display_text(), "42");
    }

    #[test]
    fn subtraction() {
        let mut e = Engine::new();
        e.input_digit('9');
        e.set_operation(Operation::Subtract);
        e.input_digit('4');
        e.calculate();
        assert_eq!(e.display_text(), "5");
    }

    #[test]
    fn clear_resets_error() {
        let mut e = Engine::new();
        e.input_digit('1');
        e.set_operation(Operation::Divide);
        e.input_digit('0');
        e.calculate();
        assert_eq!(e.display_text(), "Error");
        e.clear();
        assert_eq!(e.display_text(), "0");
    }

    #[test]
    fn negative_zero_leading_digit() {
        let mut e = Engine::new();
        e.input_digit('0');
        e.toggle_sign(); 
        e.input_digit('5');
        assert_eq!(e.display_text(), "-5");
    }
}
