use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub expression: String,
    pub result_text: String,
    pub result: f64,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default)]
    pub session: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySlot {
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedCalc {
    pub label: String,
    pub expression: String,
    pub result: f64,
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

    pub fn precedence(self, standard: bool) -> u8 {
        if !standard {
            return 1;
        }
        match self {
            Self::Add | Self::Subtract => 1,
            Self::Multiply | Self::Divide | Self::Modulo => 2,
            Self::Power => 3,
        }
    }

    pub fn is_right_assoc(self) -> bool {
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
        Token::Number(n) => format_number_default(*n),
        Token::Constant(name, _) => name.to_string(),
        Token::BinaryOp(op) => op.symbol().to_string(),
        Token::UnaryFunc(f) => format!("{}(", f.name()),
        Token::PostfixOp(p) => p.symbol().to_string(),
        Token::LeftParen => "(".to_string(),
        Token::RightParen => ")".to_string(),
    }
}

pub fn format_number_default(val: f64) -> String {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConvertCategory {
    Length,
    Weight,
    Temperature,
    Speed,
    Volume,
}
