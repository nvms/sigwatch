use crate::watch::{WatchFormat, Watchpoint};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub format: WatchFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
}

impl LayoutDef {
    pub fn to_watchpoints(&self, base_address: usize) -> Vec<Watchpoint> {
        self.fields
            .iter()
            .map(|f| {
                Watchpoint::new(
                    format!("{}.{}", self.name, f.name),
                    base_address + f.offset,
                    f.size,
                    f.format,
                )
            })
            .collect()
    }
}

pub fn load_layout(path: &str) -> Result<LayoutDef> {
    let content = std::fs::read_to_string(path)?;
    let layout: LayoutDef = serde_json::from_str(&content)?;
    if layout.fields.is_empty() {
        bail!("layout has no fields");
    }
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_layout() -> LayoutDef {
        LayoutDef {
            name: "Player".to_string(),
            fields: vec![
                FieldDef {
                    name: "health".to_string(),
                    offset: 0x100,
                    size: 4,
                    format: WatchFormat::Float,
                },
                FieldDef {
                    name: "position".to_string(),
                    offset: 0x200,
                    size: 12,
                    format: WatchFormat::Vector3,
                },
            ],
        }
    }

    #[test]
    fn test_to_watchpoints() {
        let layout = sample_layout();
        let watches = layout.to_watchpoints(0x1000);

        assert_eq!(watches.len(), 2);
        assert_eq!(watches[0].label, "Player.health");
        assert_eq!(watches[0].address, 0x1100);
        assert_eq!(watches[0].size, 4);
        assert_eq!(watches[1].label, "Player.position");
        assert_eq!(watches[1].address, 0x1200);
        assert_eq!(watches[1].size, 12);
    }

    #[test]
    fn test_layout_serialization() {
        let layout = sample_layout();
        let json = serde_json::to_string(&layout).unwrap();
        let deserialized: LayoutDef = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Player");
        assert_eq!(deserialized.fields.len(), 2);
    }
}
