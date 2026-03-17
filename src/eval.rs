#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryFunc {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Ln,
    Log10,
    Log2,
    Sqrt,
    Cbrt,
    Abs,
    Exp,
    TenPow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PostfixOp {
    Square,
    Cube,
    Reciprocal,
    Factorial,
    Percent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Constant(&'static str, f64),
    BinaryOp(BinaryOp),
    UnaryFunc(UnaryFunc),
    PostfixOp(PostfixOp),
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AngleMode {
    Radians,
    Degrees,
}

impl BinaryOp {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "−",
            Self::Multiply => "×",
            Self::Divide => "÷",
            Self::Power => "^",
            Self::Modulo => " mod ",
        }
    }
    fn precedence(self) -> u8 {
        match self {
            Self::Add | Self::Subtract => 1,
            Self::Multiply | Self::Divide | Self::Modulo => 2,
            Self::Power => 3,
        }
    }
    fn is_right_assoc(self) -> bool {
        matches!(self, Self::Power)
    }
}

impl UnaryFunc {
    pub fn name(self) -> &'static str {
        match self {
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "sin⁻¹",
            Self::Acos => "cos⁻¹",
            Self::Atan => "tan⁻¹",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Ln => "ln",
            Self::Log10 => "log",
            Self::Log2 => "log₂",
            Self::Sqrt => "√",
            Self::Cbrt => "³√",
            Self::Abs => "abs",
            Self::Exp => "eˣ",
            Self::TenPow => "10ˣ",
        }
    }
}

impl PostfixOp {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Square => "²",
            Self::Cube => "³",
            Self::Reciprocal => "⁻¹",
            Self::Factorial => "!",
            Self::Percent => "%",
        }
    }
}

pub fn token_display(token: &Token) -> String {
    match token {
        Token::Number(n) => format_number(*n),
        Token::Constant(name, _) => name.to_string(),
        Token::BinaryOp(op) => op.symbol().to_string(),
        Token::UnaryFunc(f) => format!("{}(", f.name()),
        Token::PostfixOp(p) => p.symbol().to_string(),
        Token::LeftParen => "(".to_string(),
        Token::RightParen => ")".to_string(),
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

// --- Shunting-yard evaluator ---

enum ShuntOp {
    Binary(BinaryOp),
    Func(UnaryFunc),
    LeftParen,
}

pub fn evaluate(tokens: &[Token], angle_mode: AngleMode) -> Result<f64, String> {
    if tokens.is_empty() {
        return Ok(0.0);
    }
    let mut output: Vec<f64> = Vec::new();
    let mut ops: Vec<ShuntOp> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) | Token::Constant(_, n) => output.push(*n),

            Token::BinaryOp(op) => {
                while let Some(top) = ops.last() {
                    let pop = match top {
                        ShuntOp::LeftParen => false,
                        ShuntOp::Func(_) => true,
                        ShuntOp::Binary(top_op) => {
                            if op.is_right_assoc() {
                                top_op.precedence() > op.precedence()
                            } else {
                                top_op.precedence() >= op.precedence()
                            }
                        }
                    };
                    if pop {
                        let popped = ops.pop().unwrap();
                        apply_shunt(&mut output, &popped, angle_mode)?;
                    } else {
                        break;
                    }
                }
                ops.push(ShuntOp::Binary(*op));
            }

            Token::UnaryFunc(f) => ops.push(ShuntOp::Func(*f)),
            Token::LeftParen => ops.push(ShuntOp::LeftParen),

            Token::RightParen => {
                while let Some(top) = ops.last() {
                    if matches!(top, ShuntOp::LeftParen) {
                        break;
                    }
                    let popped = ops.pop().unwrap();
                    apply_shunt(&mut output, &popped, angle_mode)?;
                }
                if matches!(ops.last(), Some(ShuntOp::LeftParen)) {
                    ops.pop();
                }
                if matches!(ops.last(), Some(ShuntOp::Func(_))) {
                    let popped = ops.pop().unwrap();
                    apply_shunt(&mut output, &popped, angle_mode)?;
                }
            }

            Token::PostfixOp(p) => {
                let val = output.pop().ok_or("Missing operand")?;
                output.push(apply_postfix(*p, val)?);
            }
        }
    }

    while let Some(op) = ops.pop() {
        if matches!(op, ShuntOp::LeftParen) {
            continue;
        }
        apply_shunt(&mut output, &op, angle_mode)?;
    }

    output.pop().ok_or_else(|| "Empty expression".to_string())
}

fn apply_shunt(output: &mut Vec<f64>, op: &ShuntOp, angle_mode: AngleMode) -> Result<(), String> {
    match op {
        ShuntOp::Binary(bin_op) => {
            let b = output.pop().ok_or("Missing operand")?;
            let a = output.pop().ok_or("Missing operand")?;
            output.push(apply_binary(*bin_op, a, b)?);
        }
        ShuntOp::Func(func) => {
            let a = output.pop().ok_or("Missing operand")?;
            output.push(apply_unary(*func, a, angle_mode)?);
        }
        ShuntOp::LeftParen => {}
    }
    Ok(())
}

fn apply_binary(op: BinaryOp, a: f64, b: f64) -> Result<f64, String> {
    match op {
        BinaryOp::Add => Ok(a + b),
        BinaryOp::Subtract => Ok(a - b),
        BinaryOp::Multiply => Ok(a * b),
        BinaryOp::Divide => {
            if b.abs() < f64::EPSILON {
                Err("Division by zero".to_string())
            } else {
                Ok(a / b)
            }
        }
        BinaryOp::Power => Ok(a.powf(b)),
        BinaryOp::Modulo => {
            if b.abs() < f64::EPSILON {
                Err("Division by zero".to_string())
            } else {
                Ok(a % b)
            }
        }
    }
}

pub fn apply_unary(func: UnaryFunc, a: f64, angle_mode: AngleMode) -> Result<f64, String> {
    let to_rad = |v: f64| match angle_mode {
        AngleMode::Radians => v,
        AngleMode::Degrees => v * std::f64::consts::PI / 180.0,
    };
    let from_rad = |v: f64| match angle_mode {
        AngleMode::Radians => v,
        AngleMode::Degrees => v * 180.0 / std::f64::consts::PI,
    };

    match func {
        UnaryFunc::Sin => Ok(to_rad(a).sin()),
        UnaryFunc::Cos => Ok(to_rad(a).cos()),
        UnaryFunc::Tan => Ok(to_rad(a).tan()),
        UnaryFunc::Asin => Ok(from_rad(a.asin())),
        UnaryFunc::Acos => Ok(from_rad(a.acos())),
        UnaryFunc::Atan => Ok(from_rad(a.atan())),
        UnaryFunc::Sinh => Ok(a.sinh()),
        UnaryFunc::Cosh => Ok(a.cosh()),
        UnaryFunc::Tanh => Ok(a.tanh()),
        UnaryFunc::Ln => {
            if a <= 0.0 {
                Err("Domain error".into())
            } else {
                Ok(a.ln())
            }
        }
        UnaryFunc::Log10 => {
            if a <= 0.0 {
                Err("Domain error".into())
            } else {
                Ok(a.log10())
            }
        }
        UnaryFunc::Log2 => {
            if a <= 0.0 {
                Err("Domain error".into())
            } else {
                Ok(a.log2())
            }
        }
        UnaryFunc::Sqrt => {
            if a < 0.0 {
                Err("Domain error".into())
            } else {
                Ok(a.sqrt())
            }
        }
        UnaryFunc::Cbrt => Ok(a.cbrt()),
        UnaryFunc::Abs => Ok(a.abs()),
        UnaryFunc::Exp => Ok(a.exp()),
        UnaryFunc::TenPow => Ok(10.0_f64.powf(a)),
    }
}

pub fn apply_postfix(op: PostfixOp, val: f64) -> Result<f64, String> {
    match op {
        PostfixOp::Square => Ok(val * val),
        PostfixOp::Cube => Ok(val * val * val),
        PostfixOp::Reciprocal => {
            if val.abs() < f64::EPSILON {
                Err("Division by zero".into())
            } else {
                Ok(1.0 / val)
            }
        }
        PostfixOp::Factorial => {
            if val < 0.0 || val != val.floor() {
                return Err("Factorial requires non-negative integer".into());
            }
            if val > 170.0 {
                return Err("Overflow".into());
            }
            let n = val as u64;
            let mut result = 1.0_f64;
            for i in 2..=n {
                result *= i as f64;
            }
            Ok(result)
        }
        PostfixOp::Percent => Ok(val / 100.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn eval(tokens: &[Token]) -> f64 {
        evaluate(tokens, AngleMode::Degrees).unwrap()
    }

    #[test]
    fn basic_ops() {
        assert_eq!(eval(&[Token::Number(2.0), Token::BinaryOp(BinaryOp::Add), Token::Number(3.0)]), 5.0);
        assert_eq!(eval(&[Token::Number(10.0), Token::BinaryOp(BinaryOp::Subtract), Token::Number(4.0)]), 6.0);
        assert_eq!(eval(&[Token::Number(3.0), Token::BinaryOp(BinaryOp::Multiply), Token::Number(7.0)]), 21.0);
        assert_eq!(eval(&[Token::Number(15.0), Token::BinaryOp(BinaryOp::Divide), Token::Number(3.0)]), 5.0);
    }

    #[test]
    fn operator_precedence() {
        // 2 + 3 × 4 = 14
        let tokens = vec![
            Token::Number(2.0),
            Token::BinaryOp(BinaryOp::Add),
            Token::Number(3.0),
            Token::BinaryOp(BinaryOp::Multiply),
            Token::Number(4.0),
        ];
        assert_eq!(eval(&tokens), 14.0);
    }

    #[test]
    fn parentheses() {
        // (2 + 3) × 4 = 20
        let tokens = vec![
            Token::LeftParen,
            Token::Number(2.0),
            Token::BinaryOp(BinaryOp::Add),
            Token::Number(3.0),
            Token::RightParen,
            Token::BinaryOp(BinaryOp::Multiply),
            Token::Number(4.0),
        ];
        assert_eq!(eval(&tokens), 20.0);
    }

    #[test]
    fn sin_degrees() {
        // sin(30) = 0.5
        let tokens = vec![
            Token::UnaryFunc(UnaryFunc::Sin),
            Token::LeftParen,
            Token::Number(30.0),
            Token::RightParen,
        ];
        let result = eval(&tokens);
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn postfix_ops() {
        // 5² = 25
        let tokens = vec![Token::Number(5.0), Token::PostfixOp(PostfixOp::Square)];
        assert_eq!(eval(&tokens), 25.0);

        // 5! = 120
        let tokens = vec![Token::Number(5.0), Token::PostfixOp(PostfixOp::Factorial)];
        assert_eq!(eval(&tokens), 120.0);
    }

    #[test]
    fn power_right_assoc() {
        // 2^3^2 = 2^(3^2) = 2^9 = 512
        let tokens = vec![
            Token::Number(2.0),
            Token::BinaryOp(BinaryOp::Power),
            Token::Number(3.0),
            Token::BinaryOp(BinaryOp::Power),
            Token::Number(2.0),
        ];
        assert_eq!(eval(&tokens), 512.0);
    }

    #[test]
    fn constants() {
        let tokens = vec![Token::Constant("π", PI)];
        assert!((eval(&tokens) - PI).abs() < 1e-10);
    }

    #[test]
    fn complex_expression() {
        // 2 + sin(30) × 5 = 2 + 0.5 × 5 = 4.5
        let tokens = vec![
            Token::Number(2.0),
            Token::BinaryOp(BinaryOp::Add),
            Token::UnaryFunc(UnaryFunc::Sin),
            Token::LeftParen,
            Token::Number(30.0),
            Token::RightParen,
            Token::BinaryOp(BinaryOp::Multiply),
            Token::Number(5.0),
        ];
        let result = eval(&tokens);
        assert!((result - 4.5).abs() < 1e-10);
    }

    #[test]
    fn division_by_zero() {
        let tokens = vec![Token::Number(1.0), Token::BinaryOp(BinaryOp::Divide), Token::Number(0.0)];
        assert!(evaluate(&tokens, AngleMode::Degrees).is_err());
    }

    #[test]
    fn format_numbers() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(8.0), "8");
        assert_eq!(format_number(-42.0), "-42");
        assert_eq!(format_number(3.14), "3.14");
        assert_eq!(format_number(0.5), "0.5");
    }
}
