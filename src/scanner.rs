use anyhow::{bail, Result};
use procmod_core::Process;
use procmod_scan::Pattern;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    I32,
    F32,
    F64,
    I16,
    I64,
    Bytes,
}

impl ValueType {
    pub fn size(&self) -> usize {
        match self {
            ValueType::I16 => 2,
            ValueType::I32 => 4,
            ValueType::F32 => 4,
            ValueType::F64 => 8,
            ValueType::I64 => 8,
            ValueType::Bytes => 0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ValueType::I16 => "i16",
            ValueType::I32 => "i32",
            ValueType::F32 => "f32",
            ValueType::F64 => "f64",
            ValueType::I64 => "i64",
            ValueType::Bytes => "bytes",
        }
    }

    pub fn format_value(&self, bytes: &[u8]) -> String {
        match self {
            ValueType::I16 if bytes.len() >= 2 => {
                format!("{}", i16::from_le_bytes(bytes[..2].try_into().unwrap()))
            }
            ValueType::I32 if bytes.len() >= 4 => {
                format!("{}", i32::from_le_bytes(bytes[..4].try_into().unwrap()))
            }
            ValueType::F32 if bytes.len() >= 4 => {
                let v = f32::from_le_bytes(bytes[..4].try_into().unwrap());
                format!("{v:.2}")
            }
            ValueType::F64 if bytes.len() >= 8 => {
                let v = f64::from_le_bytes(bytes[..8].try_into().unwrap());
                format!("{v:.4}")
            }
            ValueType::I64 if bytes.len() >= 8 => {
                format!("{}", i64::from_le_bytes(bytes[..8].try_into().unwrap()))
            }
            _ => hex_string(bytes),
        }
    }
}

fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub struct Candidate {
    pub address: usize,
    pub current_bytes: Vec<u8>,
}

pub struct ScanSession {
    pub candidates: Vec<Candidate>,
    pub value_type: ValueType,
    pub byte_pattern: Vec<u8>,
    pub history: Vec<String>,
}

impl ScanSession {
    pub fn format_value(&self, candidate: &Candidate) -> String {
        self.value_type.format_value(&candidate.current_bytes)
    }

    pub fn refresh(&mut self, process: &Process) {
        let size = self.read_size();
        for c in &mut self.candidates {
            if let Ok(data) = process.read_bytes(c.address, size) {
                c.current_bytes = data;
            }
        }
    }

    fn read_size(&self) -> usize {
        let type_size = self.value_type.size();
        if type_size > 0 {
            type_size
        } else {
            self.byte_pattern.len()
        }
    }
}

pub struct ParsedValue {
    pub bytes: Vec<u8>,
    pub value_type: ValueType,
    pub display: String,
}

pub fn parse_value(input: &str) -> Result<ParsedValue> {
    if let Ok(v) = input.parse::<f64>() {
        if input.contains('.') {
            let f = v as f32;
            return Ok(ParsedValue {
                bytes: f.to_le_bytes().to_vec(),
                value_type: ValueType::F32,
                display: format!("{f} (f32)"),
            });
        }

        let i = v as i64;
        if i >= i16::MIN as i64 && i <= i16::MAX as i64 && i.unsigned_abs() <= u16::MAX as u64 {
            return Ok(ParsedValue {
                bytes: (i as i32).to_le_bytes().to_vec(),
                value_type: ValueType::I32,
                display: format!("{i} (i32)"),
            });
        }
        if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
            return Ok(ParsedValue {
                bytes: (i as i32).to_le_bytes().to_vec(),
                value_type: ValueType::I32,
                display: format!("{i} (i32)"),
            });
        }
        return Ok(ParsedValue {
            bytes: i.to_le_bytes().to_vec(),
            value_type: ValueType::I64,
            display: format!("{i} (i64)"),
        });
    }

    let hex_bytes = parse_hex_bytes(input)?;
    Ok(ParsedValue {
        display: format!("bytes [{}]", hex_string(&hex_bytes)),
        value_type: ValueType::Bytes,
        bytes: hex_bytes,
    })
}

fn parse_hex_bytes(input: &str) -> Result<Vec<u8>> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut bytes = Vec::new();
    for token in tokens {
        let token = token.strip_prefix("0x").unwrap_or(token);
        if token.len() % 2 != 0 {
            bail!("invalid hex: {token}");
        }
        for i in (0..token.len()).step_by(2) {
            bytes.push(u8::from_str_radix(&token[i..i + 2], 16)?);
        }
    }
    if bytes.is_empty() {
        bail!("empty pattern");
    }
    Ok(bytes)
}

pub fn initial_scan(process: &Process, parsed: &ParsedValue) -> Result<ScanSession> {
    let pattern = Pattern::from_ida(&ida_from_bytes(&parsed.bytes))?;

    let regions = process
        .regions()
        .map_err(|e| anyhow::anyhow!("region query failed: {e}"))?;

    let read_size = if parsed.value_type.size() > 0 {
        parsed.value_type.size()
    } else {
        parsed.bytes.len()
    };

    let mut candidates = Vec::new();
    for region in &regions {
        if !region.protection.read {
            continue;
        }
        let data = match process.read_bytes(region.base, region.size) {
            Ok(d) => d,
            Err(_) => continue,
        };
        for offset in pattern.scan(&data) {
            let addr = region.base + offset;
            let current_bytes = if offset + read_size <= data.len() {
                data[offset..offset + read_size].to_vec()
            } else {
                process
                    .read_bytes(addr, read_size)
                    .unwrap_or_else(|_| vec![0; read_size])
            };
            candidates.push(Candidate {
                address: addr,
                current_bytes,
            });
        }
    }

    Ok(ScanSession {
        candidates,
        value_type: parsed.value_type,
        byte_pattern: parsed.bytes.clone(),
        history: vec![format!("scan {}", parsed.display)],
    })
}

pub fn narrow(process: &Process, session: &mut ScanSession, parsed: &ParsedValue) -> usize {
    let before = session.candidates.len();

    session.candidates.retain_mut(|c| {
        let size = if parsed.value_type.size() > 0 {
            parsed.value_type.size()
        } else {
            parsed.bytes.len()
        };
        match process.read_bytes(c.address, size) {
            Ok(data) => {
                c.current_bytes = data.clone();
                data.starts_with(&parsed.bytes)
            }
            Err(_) => false,
        }
    });

    let removed = before - session.candidates.len();
    session
        .history
        .push(format!("narrow {} (-{removed})", parsed.display));
    session.byte_pattern = parsed.bytes.clone();
    removed
}

fn ida_from_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_value_integer() {
        let v = parse_value("42").unwrap();
        assert_eq!(v.value_type, ValueType::I32);
        assert_eq!(v.bytes, 42i32.to_le_bytes().to_vec());
    }

    #[test]
    fn test_parse_value_float() {
        let v = parse_value("75.0").unwrap();
        assert_eq!(v.value_type, ValueType::F32);
        assert_eq!(v.bytes, 75.0f32.to_le_bytes().to_vec());
    }

    #[test]
    fn test_parse_value_negative() {
        let v = parse_value("-1").unwrap();
        assert_eq!(v.value_type, ValueType::I32);
        assert_eq!(v.bytes, (-1i32).to_le_bytes().to_vec());
    }

    #[test]
    fn test_parse_value_hex_bytes() {
        let v = parse_value("DE AD BE EF").unwrap();
        assert_eq!(v.value_type, ValueType::Bytes);
        assert_eq!(v.bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_format_value_f32() {
        let vt = ValueType::F32;
        assert_eq!(vt.format_value(&75.0f32.to_le_bytes()), "75.00");
    }

    #[test]
    fn test_format_value_i32() {
        let vt = ValueType::I32;
        assert_eq!(vt.format_value(&42i32.to_le_bytes()), "42");
    }
}
