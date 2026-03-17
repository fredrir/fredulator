pub struct FormatSettings {
    pub decimal_precision: u32,
    pub thousands_separator: String,
    pub scientific_notation: String,
    pub rounding_mode: String,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            decimal_precision: 10,
            thousands_separator: String::new(),
            scientific_notation: "auto".into(),
            rounding_mode: "half_up".into(),
        }
    }
}

impl From<&crate::services::config::FormatConfig> for FormatSettings {
    fn from(cfg: &crate::services::config::FormatConfig) -> Self {
        Self {
            decimal_precision: cfg.decimal_precision,
            thousands_separator: cfg.thousands_separator.clone(),
            scientific_notation: cfg.scientific_notation.clone(),
            rounding_mode: cfg.rounding_mode.clone(),
        }
    }
}

pub fn format_number(val: f64, settings: &FormatSettings) -> String {
    if val.is_nan() || val.is_infinite() {
        return "Error".to_string();
    }
    if val == 0.0 {
        return "0".to_string();
    }

    let use_sci = match settings.scientific_notation.as_str() {
        "always" => true,
        "never" => false,
        _ => val.abs() >= 1e15 || val.abs() < 1e-4,
    };

    if use_sci {
        return format!("{:e}", val);
    }

    if val.fract() == 0.0 && val.abs() < 1e15 {
        let s = format!("{}", val as i64);
        return add_thousands_sep(&s, &settings.thousands_separator);
    }

    let precision = settings.decimal_precision.min(20) as usize;
    let s = if settings.rounding_mode == "truncate" {
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
            add_thousands_sep(int_part, &settings.thousands_separator),
            dec_part
        )
    } else {
        add_thousands_sep(&s, &settings.thousands_separator)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_settings() -> FormatSettings {
        FormatSettings::default()
    }

    #[test]
    fn zero() {
        assert_eq!(format_number(0.0, &default_settings()), "0");
    }

    #[test]
    fn integer() {
        assert_eq!(format_number(42.0, &default_settings()), "42");
    }

    #[test]
    fn negative_integer() {
        assert_eq!(format_number(-7.0, &default_settings()), "-7");
    }

    #[test]
    fn simple_decimal() {
        assert_eq!(format_number(3.15, &default_settings()), "3.15");
    }

    #[test]
    fn trailing_zeros_trimmed() {
        assert_eq!(format_number(2.50, &default_settings()), "2.5");
    }

    #[test]
    fn nan_returns_error() {
        assert_eq!(format_number(f64::NAN, &default_settings()), "Error");
    }

    #[test]
    fn infinity_returns_error() {
        assert_eq!(format_number(f64::INFINITY, &default_settings()), "Error");
    }

    #[test]
    fn scientific_auto() {
        let s = &default_settings();
        let result = format_number(1e16, s);
        assert!(result.contains('e'));
        let result = format_number(1e-5, s);
        assert!(result.contains('e'));
    }

    #[test]
    fn scientific_always() {
        let s = FormatSettings {
            scientific_notation: "always".into(),
            ..default_settings()
        };
        let result = format_number(42.0, &s);
        assert!(result.contains('e'));
    }

    #[test]
    fn scientific_never() {
        let s = FormatSettings {
            scientific_notation: "never".into(),
            ..default_settings()
        };
        let result = format_number(123456789.0, &s);
        assert!(!result.contains('e'));
    }

    #[test]
    fn thousands_separator_comma() {
        let s = FormatSettings {
            thousands_separator: ",".into(),
            ..default_settings()
        };
        assert_eq!(format_number(1234567.0, &s), "1,234,567");
    }

    #[test]
    fn thousands_separator_with_decimal() {
        let s = FormatSettings {
            thousands_separator: ",".into(),
            ..default_settings()
        };
        assert_eq!(format_number(1234.5, &s), "1,234.5");
    }

    #[test]
    fn thousands_separator_negative() {
        let s = FormatSettings {
            thousands_separator: ",".into(),
            ..default_settings()
        };
        assert_eq!(format_number(-1234567.0, &s), "-1,234,567");
    }

    #[test]
    fn truncate_rounding() {
        let s = FormatSettings {
            rounding_mode: "truncate".into(),
            decimal_precision: 2,
            ..default_settings()
        };
        assert_eq!(format_number(3.149, &s), "3.14");
    }

    #[test]
    fn half_up_rounding() {
        let s = FormatSettings {
            rounding_mode: "half_up".into(),
            decimal_precision: 2,
            ..default_settings()
        };
        assert_eq!(format_number(3.145, &s), "3.15");
    }

    #[test]
    fn precision_zero() {
        let s = FormatSettings {
            decimal_precision: 0,
            ..default_settings()
        };
        assert_eq!(format_number(3.7, &s), "4");
    }

    #[test]
    fn combined_thousands_and_truncate() {
        let s = FormatSettings {
            thousands_separator: " ".into(),
            rounding_mode: "truncate".into(),
            decimal_precision: 3,
            ..default_settings()
        };
        assert_eq!(format_number(12345.6789, &s), "12 345.678");
    }
}
