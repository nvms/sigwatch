use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Hex,
    Decimal,
    Float,
    HealthBar,
    Vector3,
    Boolean,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Hex => write!(f, "hex"),
            Format::Decimal => write!(f, "dec"),
            Format::Float => write!(f, "float"),
            Format::HealthBar => write!(f, "health"),
            Format::Vector3 => write!(f, "vec3"),
            Format::Boolean => write!(f, "bool"),
        }
    }
}

pub fn format_bytes(bytes: &[u8], fmt: Format) -> String {
    match fmt {
        Format::Hex => format_hex(bytes),
        Format::Decimal => format_decimal(bytes),
        Format::Float => format_float(bytes),
        Format::HealthBar => format_health_bar(bytes),
        Format::Vector3 => format_vector3(bytes),
        Format::Boolean => format_boolean(bytes),
    }
}

fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_decimal(bytes: &[u8]) -> String {
    match bytes.len() {
        1 => format!("{}", bytes[0]),
        2 => format!("{}", i16::from_le_bytes(bytes.try_into().unwrap_or([0; 2]))),
        4 => format!("{}", i32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]))),
        8 => format!("{}", i64::from_le_bytes(bytes.try_into().unwrap_or([0; 8]))),
        _ => format_hex(bytes),
    }
}

fn format_float(bytes: &[u8]) -> String {
    match bytes.len() {
        4 => {
            let v = f32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]));
            format!("{v:.4}")
        }
        8 => {
            let v = f64::from_le_bytes(bytes.try_into().unwrap_or([0; 8]));
            format!("{v:.4}")
        }
        _ => format_hex(bytes),
    }
}

fn format_health_bar(bytes: &[u8]) -> String {
    if bytes.len() < 8 {
        return format_hex(bytes);
    }
    let current = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let max = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if max <= 0.0 {
        return format!("{current:.0} / ???");
    }
    let ratio = (current / max).clamp(0.0, 1.0);
    let bar_width = 20;
    let filled = (ratio * bar_width as f32) as usize;
    let empty = bar_width - filled;
    format!(
        "{current:.0}/{max:.0} [{}{}]",
        "#".repeat(filled),
        "-".repeat(empty)
    )
}

fn format_vector3(bytes: &[u8]) -> String {
    if bytes.len() < 12 {
        return format_hex(bytes);
    }
    let x = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let y = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let z = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
    format!("({x:.2}, {y:.2}, {z:.2})")
}

fn format_boolean(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "??".to_string();
    }
    if bytes[0] == 0 { "false" } else { "true" }.to_string()
}

#[allow(dead_code)]
pub fn parse_format(s: &str) -> Option<Format> {
    match s {
        "hex" => Some(Format::Hex),
        "dec" | "decimal" => Some(Format::Decimal),
        "float" | "f32" | "f64" => Some(Format::Float),
        "health" | "healthbar" => Some(Format::HealthBar),
        "vec3" | "vector3" => Some(Format::Vector3),
        "bool" | "boolean" => Some(Format::Boolean),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hex() {
        let bytes = [0xDE, 0xAD, 0xBE, 0xEF];
        assert_eq!(format_bytes(&bytes, Format::Hex), "DE AD BE EF");
    }

    #[test]
    fn test_format_decimal_i32() {
        let bytes = 42i32.to_le_bytes();
        assert_eq!(format_bytes(&bytes, Format::Decimal), "42");
    }

    #[test]
    fn test_format_float_f32() {
        let bytes = 3.14f32.to_le_bytes();
        assert_eq!(format_bytes(&bytes, Format::Float), "3.1400");
    }

    #[test]
    fn test_format_health_bar() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&75.0f32.to_le_bytes());
        bytes.extend_from_slice(&100.0f32.to_le_bytes());
        let result = format_bytes(&bytes, Format::HealthBar);
        assert!(result.starts_with("75/100"));
        assert!(result.contains('['));
    }

    #[test]
    fn test_format_vector3() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1.5f32.to_le_bytes());
        bytes.extend_from_slice(&2.5f32.to_le_bytes());
        bytes.extend_from_slice(&3.5f32.to_le_bytes());
        assert_eq!(format_bytes(&bytes, Format::Vector3), "(1.50, 2.50, 3.50)");
    }

    #[test]
    fn test_format_boolean() {
        assert_eq!(format_bytes(&[0], Format::Boolean), "false");
        assert_eq!(format_bytes(&[1], Format::Boolean), "true");
        assert_eq!(format_bytes(&[255], Format::Boolean), "true");
    }

    #[test]
    fn test_parse_format() {
        assert_eq!(parse_format("hex"), Some(Format::Hex));
        assert_eq!(parse_format("dec"), Some(Format::Decimal));
        assert_eq!(parse_format("float"), Some(Format::Float));
        assert_eq!(parse_format("health"), Some(Format::HealthBar));
        assert_eq!(parse_format("vec3"), Some(Format::Vector3));
        assert_eq!(parse_format("bool"), Some(Format::Boolean));
        assert_eq!(parse_format("garbage"), None);
    }
}
