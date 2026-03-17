use super::types::ConvertCategory;

impl ConvertCategory {
    pub const ALL: &'static [ConvertCategory] = &[
        Self::Length, Self::Weight, Self::Temperature, Self::Speed, Self::Volume,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Length => "Length",
            Self::Weight => "Weight",
            Self::Temperature => "Temp",
            Self::Speed => "Speed",
            Self::Volume => "Volume",
        }
    }

    pub fn units(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Length => &[
                ("km", "Kilometer"), ("m", "Meter"), ("cm", "Centimeter"), ("mm", "Millimeter"),
                ("mi", "Mile"), ("yd", "Yard"), ("ft", "Foot"), ("in", "Inch"),
            ],
            Self::Weight => &[
                ("kg", "Kilogram"), ("g", "Gram"), ("mg", "Milligram"), ("lb", "Pound"),
                ("oz", "Ounce"), ("st", "Stone"), ("t", "Metric Ton"),
            ],
            Self::Temperature => &[("C", "Celsius"), ("F", "Fahrenheit"), ("K", "Kelvin")],
            Self::Speed => &[
                ("km/h", "km/h"), ("m/s", "m/s"), ("mph", "mph"), ("kn", "Knots"), ("ft/s", "ft/s"),
            ],
            Self::Volume => &[
                ("L", "Liter"), ("mL", "Milliliter"), ("gal", "Gallon"), ("qt", "Quart"),
                ("pt", "Pint"), ("cup", "Cup"), ("fl oz", "Fl Ounce"),
            ],
        }
    }
}

pub fn convert(cat: ConvertCategory, from: &str, to: &str, value: f64) -> f64 {
    if from == to { return value; }
    match cat {
        ConvertCategory::Temperature => convert_temp(from, to, value),
        _ => {
            let from_f = unit_factor(cat, from);
            let to_f = unit_factor(cat, to);
            value * from_f / to_f
        }
    }
}

fn convert_temp(from: &str, to: &str, value: f64) -> f64 {
    let celsius = match from {
        "C" => value,
        "F" => (value - 32.0) * 5.0 / 9.0,
        "K" => value - 273.15,
        _ => value,
    };
    match to {
        "C" => celsius,
        "F" => celsius * 9.0 / 5.0 + 32.0,
        "K" => celsius + 273.15,
        _ => celsius,
    }
}

fn unit_factor(cat: ConvertCategory, unit: &str) -> f64 {
    match cat {
        ConvertCategory::Length => match unit {
            "km" => 1000.0, "m" => 1.0, "cm" => 0.01, "mm" => 0.001,
            "mi" => 1609.344, "yd" => 0.9144, "ft" => 0.3048, "in" => 0.0254,
            _ => 1.0,
        },
        ConvertCategory::Weight => match unit {
            "kg" => 1.0, "g" => 0.001, "mg" => 0.000001, "lb" => 0.453592,
            "oz" => 0.0283495, "st" => 6.35029, "t" => 1000.0,
            _ => 1.0,
        },
        ConvertCategory::Speed => match unit {
            "km/h" => 1.0 / 3.6, "m/s" => 1.0, "mph" => 0.44704, "kn" => 0.514444, "ft/s" => 0.3048,
            _ => 1.0,
        },
        ConvertCategory::Volume => match unit {
            "L" => 1.0, "mL" => 0.001, "gal" => 3.78541, "qt" => 0.946353,
            "pt" => 0.473176, "cup" => 0.236588, "fl oz" => 0.0295735,
            _ => 1.0,
        },
        ConvertCategory::Temperature => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_conversion() {
        let result = convert(ConvertCategory::Length, "km", "m", 1.0);
        assert!((result - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn miles_to_km() {
        let result = convert(ConvertCategory::Length, "mi", "km", 1.0);
        assert!((result - 1.609344).abs() < 1e-4);
    }

    #[test]
    fn temp_c_to_f() {
        let result = convert(ConvertCategory::Temperature, "C", "F", 100.0);
        assert!((result - 212.0).abs() < 1e-6);
    }

    #[test]
    fn temp_f_to_c() {
        let result = convert(ConvertCategory::Temperature, "F", "C", 32.0);
        assert!(result.abs() < 1e-6);
    }

    #[test]
    fn temp_c_to_k() {
        let result = convert(ConvertCategory::Temperature, "C", "K", 0.0);
        assert!((result - 273.15).abs() < 1e-6);
    }

    #[test]
    fn identity_conversion() {
        assert_eq!(convert(ConvertCategory::Length, "m", "m", 42.0), 42.0);
    }

    #[test]
    fn weight_lb_to_kg() {
        let result = convert(ConvertCategory::Weight, "lb", "kg", 1.0);
        assert!((result - 0.453592).abs() < 1e-4);
    }

    #[test]
    fn speed_mph_to_kmh() {
        let result = convert(ConvertCategory::Speed, "mph", "km/h", 60.0);
        assert!((result - 96.5604).abs() < 0.1);
    }

    #[test]
    fn volume_gal_to_l() {
        let result = convert(ConvertCategory::Volume, "gal", "L", 1.0);
        assert!((result - 3.78541).abs() < 1e-3);
    }

    #[test]
    fn all_categories_have_units() {
        for cat in ConvertCategory::ALL {
            assert!(!cat.units().is_empty());
            assert!(!cat.name().is_empty());
        }
    }
}
