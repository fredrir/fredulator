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
    Sqrt,
    Cbrt,
    Abs,
    Exp,
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
            Self::Sqrt => "\u{221a}",
            Self::Cbrt => "\u{00b3}\u{221a}",
            Self::Abs => "abs",
            Self::Exp => "e\u{02e3}",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_zero() {
        assert_eq!(format_number_default(0.0), "0");
    }

    #[test]
    fn format_negative_zero() {
        assert_eq!(format_number_default(-0.0), "0");
    }

    #[test]
    fn format_integer() {
        assert_eq!(format_number_default(42.0), "42");
    }

    #[test]
    fn format_negative_integer() {
        assert_eq!(format_number_default(-7.0), "-7");
    }

    #[test]
    fn format_large_integer() {
        assert_eq!(format_number_default(999_999_999_999_999.0), "999999999999999");
    }

    #[test]
    fn format_decimal() {
        assert_eq!(format_number_default(3.14), "3.14");
    }

    #[test]
    fn format_trailing_zeros_trimmed() {
        assert_eq!(format_number_default(2.50), "2.5");
    }

    #[test]
    fn format_nan() {
        assert_eq!(format_number_default(f64::NAN), "Error");
    }

    #[test]
    fn format_infinity() {
        assert_eq!(format_number_default(f64::INFINITY), "Error");
    }

    #[test]
    fn format_neg_infinity() {
        assert_eq!(format_number_default(f64::NEG_INFINITY), "Error");
    }

    #[test]
    fn format_scientific_large() {
        let s = format_number_default(1e16);
        assert!(s.contains('e'), "expected scientific notation, got {}", s);
    }

    #[test]
    fn format_scientific_small() {
        let s = format_number_default(1e-5);
        assert!(s.contains('e'), "expected scientific notation, got {}", s);
    }

    #[test]
    fn display_number_token() {
        assert_eq!(token_display(&Token::Number(42.0)), "42");
    }

    #[test]
    fn display_constant_token() {
        assert_eq!(token_display(&Token::Constant("π", std::f64::consts::PI)), "π");
    }

    #[test]
    fn display_binary_op_token() {
        assert_eq!(token_display(&Token::BinaryOp(BinaryOp::Add)), "+");
    }

    #[test]
    fn display_unary_func_token() {
        assert_eq!(token_display(&Token::UnaryFunc(UnaryFunc::Sin)), "sin(");
    }

    #[test]
    fn display_postfix_op_token() {
        assert_eq!(token_display(&Token::PostfixOp(PostfixOp::Factorial)), "!");
    }

    #[test]
    fn display_parens() {
        assert_eq!(token_display(&Token::LeftParen), "(");
        assert_eq!(token_display(&Token::RightParen), ")");
    }

    #[test]
    fn binary_op_symbols() {
        assert_eq!(BinaryOp::Add.symbol(), "+");
        assert_eq!(BinaryOp::Subtract.symbol(), "\u{2212}");
        assert_eq!(BinaryOp::Multiply.symbol(), "\u{00d7}");
        assert_eq!(BinaryOp::Divide.symbol(), "\u{00f7}");
        assert_eq!(BinaryOp::Power.symbol(), "^");
        assert_eq!(BinaryOp::Modulo.symbol(), " mod ");
    }

    #[test]
    fn precedence_standard() {
        assert_eq!(BinaryOp::Add.precedence(true), 1);
        assert_eq!(BinaryOp::Subtract.precedence(true), 1);
        assert_eq!(BinaryOp::Multiply.precedence(true), 2);
        assert_eq!(BinaryOp::Divide.precedence(true), 2);
        assert_eq!(BinaryOp::Modulo.precedence(true), 2);
        assert_eq!(BinaryOp::Power.precedence(true), 3);
    }

    #[test]
    fn precedence_non_standard_all_equal() {
        for op in [
            BinaryOp::Add,
            BinaryOp::Subtract,
            BinaryOp::Multiply,
            BinaryOp::Divide,
            BinaryOp::Power,
            BinaryOp::Modulo,
        ] {
            assert_eq!(op.precedence(false), 1, "{:?} should have precedence 1 when non-standard", op);
        }
    }

    #[test]
    fn only_power_is_right_assoc() {
        assert!(BinaryOp::Power.is_right_assoc());
        assert!(!BinaryOp::Add.is_right_assoc());
        assert!(!BinaryOp::Subtract.is_right_assoc());
        assert!(!BinaryOp::Multiply.is_right_assoc());
        assert!(!BinaryOp::Divide.is_right_assoc());
        assert!(!BinaryOp::Modulo.is_right_assoc());
    }

    #[test]
    fn unary_func_names() {
        assert_eq!(UnaryFunc::Sin.name(), "sin");
        assert_eq!(UnaryFunc::Cos.name(), "cos");
        assert_eq!(UnaryFunc::Tan.name(), "tan");
        assert_eq!(UnaryFunc::Ln.name(), "ln");
        assert_eq!(UnaryFunc::Log10.name(), "log");
        assert_eq!(UnaryFunc::Sqrt.name(), "\u{221a}");
        assert_eq!(UnaryFunc::Abs.name(), "abs");
    }

    #[test]
    fn inverse_trig_names_contain_superscript() {
        assert!(UnaryFunc::Asin.name().starts_with("sin"));
        assert!(UnaryFunc::Acos.name().starts_with("cos"));
        assert!(UnaryFunc::Atan.name().starts_with("tan"));
    }

    #[test]
    fn postfix_symbols() {
        assert_eq!(PostfixOp::Square.symbol(), "\u{00b2}");
        assert_eq!(PostfixOp::Cube.symbol(), "\u{00b3}");
        assert_eq!(PostfixOp::Factorial.symbol(), "!");
        assert_eq!(PostfixOp::Percent.symbol(), "%");
    }

    #[test]
    fn history_entry_roundtrip() {
        let entry = HistoryEntry {
            expression: "2+3".into(),
            result_text: "5".into(),
            result: 5.0,
            timestamp: 1000,
            session: 1,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.expression, "2+3");
        assert_eq!(back.result, 5.0);
        assert_eq!(back.timestamp, 1000);
    }

    #[test]
    fn history_entry_defaults_on_missing_fields() {
        let json = r#"{"expression":"1+1","result_text":"2","result":2.0}"#;
        let entry: HistoryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.timestamp, 0);
        assert_eq!(entry.session, 0);
    }

    #[test]
    fn memory_slot_roundtrip() {
        let slot = MemorySlot { label: "A".into(), value: 42.0 };
        let json = serde_json::to_string(&slot).unwrap();
        let back: MemorySlot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.label, "A");
        assert_eq!(back.value, 42.0);
    }

    #[test]
    fn pinned_calc_roundtrip() {
        let pin = PinnedCalc {
            label: "tax".into(),
            expression: "100*0.25".into(),
            result: 25.0,
        };
        let json = serde_json::to_string(&pin).unwrap();
        let back: PinnedCalc = serde_json::from_str(&json).unwrap();
        assert_eq!(back.label, "tax");
        assert_eq!(back.result, 25.0);
    }
}
