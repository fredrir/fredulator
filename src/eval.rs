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
            Self::Subtract => "\u{2212}",
            Self::Multiply => "\u{00d7}",
            Self::Divide => "\u{00f7}",
            Self::Power => "^",
            Self::Modulo => " mod ",
        }
    }
    fn precedence(self) -> u8 {
        if !crate::config::get().behavior.operator_precedence {
            return 1;
        }
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
            Self::Asin => "sin\u{207b}\u{00b9}",
            Self::Acos => "cos\u{207b}\u{00b9}",
            Self::Atan => "tan\u{207b}\u{00b9}",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Ln => "ln",
            Self::Log10 => "log",
            Self::Log2 => "log\u{2082}",
            Self::Sqrt => "\u{221a}",
            Self::Cbrt => "\u{00b3}\u{221a}",
            Self::Abs => "abs",
            Self::Exp => "e\u{02e3}",
            Self::TenPow => "10\u{02e3}",
        }
    }
}

impl PostfixOp {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Square => "\u{00b2}",
            Self::Cube => "\u{00b3}",
            Self::Reciprocal => "\u{207b}\u{00b9}",
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
    let cfg = &crate::config::get().format;

    if val.is_nan() || val.is_infinite() {
        return "Error".to_string();
    }
    if val == 0.0 {
        return "0".to_string();
    }

    let use_sci = match cfg.scientific_notation.as_str() {
        "always" => true,
        "never" => false,
        _ => val.abs() >= 1e15 || val.abs() < 1e-4,
    };

    if use_sci {
        return format!("{:e}", val);
    }

    if val.fract() == 0.0 && val.abs() < 1e15 {
        let s = format!("{}", val as i64);
        return add_thousands_sep(&s, &cfg.thousands_separator);
    }

    let precision = cfg.decimal_precision.min(20) as usize;
    let s = if cfg.rounding_mode == "truncate" {
        let factor = 10f64.powi(precision as i32);
        let truncated = (val * factor).trunc() / factor;
        format!("{:.prec$}", truncated, prec = precision)
    } else {
        format!("{:.prec$}", val, prec = precision)
    };
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();

    if let Some(dot_pos) = s.find('.') {
        let (int_part, dec_part) = s.split_at(dot_pos);
        format!(
            "{}{}",
            add_thousands_sep(int_part, &cfg.thousands_separator),
            dec_part
        )
    } else {
        add_thousands_sep(&s, &cfg.thousands_separator)
    }
}

fn add_thousands_sep(s: &str, sep: &str) -> String {
    if sep.is_empty() {
        return s.to_string();
    }
    let negative = s.starts_with('-');
    let digits = if negative { &s[1..] } else { s };
    let mut result = String::new();
    let len = digits.len();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push_str(sep);
        }
        result.push(ch);
    }
    if negative {
        format!("-{}", result)
    } else {
        result
    }
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

// --- Text expression parser for math notes ---

pub fn parse_expression(input: &str) -> Result<Vec<Token>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(vec![]);
    }

    let input = input
        .replace(" of ", " * ")
        .replace('\u{00d7}', "*")
        .replace('\u{00f7}', "/")
        .replace('\u{2212}', "-");

    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            ' ' | '\t' => {
                i += 1;
            }
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let val: f64 = num_str.parse().map_err(|_| "Invalid number".to_string())?;

                // Check for % immediately after number
                if i < chars.len() && chars[i] == '%' {
                    tokens.push(Token::Number(val));
                    tokens.push(Token::PostfixOp(PostfixOp::Percent));
                    i += 1;
                } else {
                    // Implicit multiply before ( or letter
                    if i < chars.len()
                        && (chars[i] == '(' || chars[i].is_alphabetic() || chars[i] == '\u{03c0}')
                    {
                        tokens.push(Token::Number(val));
                        tokens.push(Token::BinaryOp(BinaryOp::Multiply));
                    } else {
                        tokens.push(Token::Number(val));
                    }
                }
            }
            '+' => {
                tokens.push(Token::BinaryOp(BinaryOp::Add));
                i += 1;
            }
            '-' => {
                let is_unary = tokens.is_empty()
                    || matches!(
                        tokens.last(),
                        Some(Token::BinaryOp(_) | Token::LeftParen)
                    );
                if is_unary && i + 1 < chars.len() && (chars[i + 1].is_ascii_digit() || chars[i + 1] == '.') {
                    i += 1;
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    let val: f64 = num_str
                        .parse::<f64>()
                        .map(|v| -v)
                        .map_err(|_| "Invalid number".to_string())?;
                    tokens.push(Token::Number(val));
                } else {
                    tokens.push(Token::BinaryOp(BinaryOp::Subtract));
                    i += 1;
                }
            }
            '*' => {
                tokens.push(Token::BinaryOp(BinaryOp::Multiply));
                i += 1;
            }
            '/' => {
                tokens.push(Token::BinaryOp(BinaryOp::Divide));
                i += 1;
            }
            '^' => {
                tokens.push(Token::BinaryOp(BinaryOp::Power));
                i += 1;
            }
            '(' => {
                // Implicit multiply: )( or number(
                if matches!(
                    tokens.last(),
                    Some(Token::RightParen | Token::Number(_) | Token::Constant(..))
                ) {
                    tokens.push(Token::BinaryOp(BinaryOp::Multiply));
                }
                tokens.push(Token::LeftParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RightParen);
                i += 1;
            }
            '!' => {
                tokens.push(Token::PostfixOp(PostfixOp::Factorial));
                i += 1;
            }
            '\u{03c0}' => {
                if matches!(
                    tokens.last(),
                    Some(Token::Number(_) | Token::Constant(..) | Token::RightParen)
                ) {
                    tokens.push(Token::BinaryOp(BinaryOp::Multiply));
                }
                tokens.push(Token::Constant("\u{03c0}", std::f64::consts::PI));
                i += 1;
            }
            _ if ch.is_alphabetic() => {
                let start = i;
                while i < chars.len() && chars[i].is_alphabetic() {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let word_lower = word.to_lowercase();

                // Implicit multiply before functions/constants
                let need_mul = matches!(
                    tokens.last(),
                    Some(Token::Number(_) | Token::Constant(..) | Token::RightParen)
                );

                match word_lower.as_str() {
                    "pi" => {
                        if need_mul {
                            tokens.push(Token::BinaryOp(BinaryOp::Multiply));
                        }
                        tokens.push(Token::Constant("\u{03c0}", std::f64::consts::PI));
                    }
                    "e" if i >= chars.len() || chars[i] != '(' => {
                        if need_mul {
                            tokens.push(Token::BinaryOp(BinaryOp::Multiply));
                        }
                        tokens.push(Token::Constant("e", std::f64::consts::E));
                    }
                    _ => {
                        let func = match word_lower.as_str() {
                            "sin" => Some(UnaryFunc::Sin),
                            "cos" => Some(UnaryFunc::Cos),
                            "tan" => Some(UnaryFunc::Tan),
                            "asin" | "arcsin" => Some(UnaryFunc::Asin),
                            "acos" | "arccos" => Some(UnaryFunc::Acos),
                            "atan" | "arctan" => Some(UnaryFunc::Atan),
                            "sinh" => Some(UnaryFunc::Sinh),
                            "cosh" => Some(UnaryFunc::Cosh),
                            "tanh" => Some(UnaryFunc::Tanh),
                            "ln" => Some(UnaryFunc::Ln),
                            "log" => Some(UnaryFunc::Log10),
                            "sqrt" => Some(UnaryFunc::Sqrt),
                            "cbrt" => Some(UnaryFunc::Cbrt),
                            "abs" => Some(UnaryFunc::Abs),
                            "exp" => Some(UnaryFunc::Exp),
                            "mod" => None,
                            _ => None,
                        };
                        if let Some(f) = func {
                            if need_mul {
                                tokens.push(Token::BinaryOp(BinaryOp::Multiply));
                            }
                            tokens.push(Token::UnaryFunc(f));
                            if !(i < chars.len() && chars[i] == '(') {
                                tokens.push(Token::LeftParen);
                            }
                        } else if word_lower == "mod" {
                            tokens.push(Token::BinaryOp(BinaryOp::Modulo));
                        } else if let Some(result) = eval_plugin_function(&word_lower, &chars, &mut i) {
                            if need_mul {
                                tokens.push(Token::BinaryOp(BinaryOp::Multiply));
                            }
                            tokens.push(Token::Number(result));
                        }
                    }
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    // Auto-close unclosed parens
    let open = tokens.iter().filter(|t| matches!(t, Token::LeftParen)).count();
    let close = tokens
        .iter()
        .filter(|t| matches!(t, Token::RightParen))
        .count();
    for _ in 0..open.saturating_sub(close) {
        tokens.push(Token::RightParen);
    }

    Ok(tokens)
}

fn eval_plugin_function(name: &str, chars: &[char], i: &mut usize) -> Option<f64> {
    let plugins = &crate::config::get().plugins.functions;
    let expr_template = plugins.get(name)?;

    if *i < chars.len() && chars[*i] == '(' {
        *i += 1;
        let start = *i;
        let mut depth = 1;
        while *i < chars.len() {
            match chars[*i] {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            *i += 1;
        }
        let arg_str: String = chars[start..*i].iter().collect();
        if *i < chars.len() && chars[*i] == ')' {
            *i += 1;
        }

        let arg_tokens = parse_expression(&arg_str).ok()?;
        let arg_val = evaluate(&arg_tokens, AngleMode::Degrees).ok()?;
        let substituted = expr_template.replace("x", &format!("{}", arg_val));
        let sub_tokens = parse_expression(&substituted).ok()?;
        evaluate(&sub_tokens, AngleMode::Degrees).ok()
    } else {
        None
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
        assert_eq!(
            eval(&[
                Token::Number(2.0),
                Token::BinaryOp(BinaryOp::Add),
                Token::Number(3.0)
            ]),
            5.0
        );
        assert_eq!(
            eval(&[
                Token::Number(10.0),
                Token::BinaryOp(BinaryOp::Subtract),
                Token::Number(4.0)
            ]),
            6.0
        );
        assert_eq!(
            eval(&[
                Token::Number(3.0),
                Token::BinaryOp(BinaryOp::Multiply),
                Token::Number(7.0)
            ]),
            21.0
        );
        assert_eq!(
            eval(&[
                Token::Number(15.0),
                Token::BinaryOp(BinaryOp::Divide),
                Token::Number(3.0)
            ]),
            5.0
        );
    }

    #[test]
    fn operator_precedence() {
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
        let tokens = vec![Token::Number(5.0), Token::PostfixOp(PostfixOp::Square)];
        assert_eq!(eval(&tokens), 25.0);

        let tokens = vec![Token::Number(5.0), Token::PostfixOp(PostfixOp::Factorial)];
        assert_eq!(eval(&tokens), 120.0);
    }

    #[test]
    fn power_right_assoc() {
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
        let tokens = vec![Token::Constant("\u{03c0}", PI)];
        assert!((eval(&tokens) - PI).abs() < 1e-10);
    }

    #[test]
    fn complex_expression() {
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
        let tokens = vec![
            Token::Number(1.0),
            Token::BinaryOp(BinaryOp::Divide),
            Token::Number(0.0),
        ];
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

    #[test]
    fn parse_simple() {
        let tokens = parse_expression("2 + 3").unwrap();
        let result = evaluate(&tokens, AngleMode::Degrees).unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn parse_implicit_multiply() {
        let tokens = parse_expression("2(3+4)").unwrap();
        let result = evaluate(&tokens, AngleMode::Degrees).unwrap();
        assert_eq!(result, 14.0);
    }

    #[test]
    fn parse_percent_of() {
        let tokens = parse_expression("50% of 200").unwrap();
        let result = evaluate(&tokens, AngleMode::Degrees).unwrap();
        assert_eq!(result, 100.0);
    }

    #[test]
    fn parse_functions() {
        let tokens = parse_expression("sin(30)").unwrap();
        let result = evaluate(&tokens, AngleMode::Degrees).unwrap();
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn parse_pi() {
        let tokens = parse_expression("2pi").unwrap();
        let result = evaluate(&tokens, AngleMode::Degrees).unwrap();
        assert!((result - 2.0 * PI).abs() < 1e-10);
    }
}
