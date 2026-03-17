#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Length,
    Weight,
    Temperature,
    Speed,
    Volume,
}

impl Category {
    pub const ALL: &'static [Category] = &[
        Self::Length,
        Self::Weight,
        Self::Temperature,
        Self::Speed,
        Self::Volume,
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
                ("km", "Kilometer"),
                ("m", "Meter"),
                ("cm", "Centimeter"),
                ("mm", "Millimeter"),
                ("mi", "Mile"),
                ("yd", "Yard"),
                ("ft", "Foot"),
                ("in", "Inch"),
            ],
            Self::Weight => &[
                ("kg", "Kilogram"),
                ("g", "Gram"),
                ("mg", "Milligram"),
                ("lb", "Pound"),
                ("oz", "Ounce"),
                ("st", "Stone"),
                ("t", "Metric Ton"),
            ],
            Self::Temperature => &[
                ("C", "Celsius"),
                ("F", "Fahrenheit"),
                ("K", "Kelvin"),
            ],
            Self::Speed => &[
                ("km/h", "km/h"),
                ("m/s", "m/s"),
                ("mph", "mph"),
                ("kn", "Knots"),
                ("ft/s", "ft/s"),
            ],
            Self::Volume => &[
                ("L", "Liter"),
                ("mL", "Milliliter"),
                ("gal", "Gallon"),
                ("qt", "Quart"),
                ("pt", "Pint"),
                ("cup", "Cup"),
                ("fl oz", "Fl Ounce"),
            ],
        }
    }
}

pub fn convert(cat: Category, from: &str, to: &str, value: f64) -> f64 {
    if from == to {
        return value;
    }
    match cat {
        Category::Temperature => convert_temp(from, to, value),
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

fn unit_factor(cat: Category, unit: &str) -> f64 {
    match cat {
        Category::Length => match unit {
            "km" => 1000.0,
            "m" => 1.0,
            "cm" => 0.01,
            "mm" => 0.001,
            "mi" => 1609.344,
            "yd" => 0.9144,
            "ft" => 0.3048,
            "in" => 0.0254,
            _ => 1.0,
        },
        Category::Weight => match unit {
            "kg" => 1.0,
            "g" => 0.001,
            "mg" => 0.000001,
            "lb" => 0.453592,
            "oz" => 0.0283495,
            "st" => 6.35029,
            "t" => 1000.0,
            _ => 1.0,
        },
        Category::Speed => match unit {
            "km/h" => 1.0 / 3.6,
            "m/s" => 1.0,
            "mph" => 0.44704,
            "kn" => 0.514444,
            "ft/s" => 0.3048,
            _ => 1.0,
        },
        Category::Volume => match unit {
            "L" => 1.0,
            "mL" => 0.001,
            "gal" => 3.78541,
            "qt" => 0.946353,
            "pt" => 0.473176,
            "cup" => 0.236588,
            "fl oz" => 0.0295735,
            _ => 1.0,
        },
        Category::Temperature => 1.0,
    }
}
