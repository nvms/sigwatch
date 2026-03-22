use anyhow::Result;
use procmod_core::Process;
use procmod_scan::Pattern;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub pattern: String,
    pub addresses: Vec<usize>,
}

pub fn scan_ida(process: &Process, signature: &str) -> Result<ScanResult> {
    let pattern = Pattern::from_ida(signature)?;
    let addresses = scan_readable_regions(process, &pattern)?;
    Ok(ScanResult {
        pattern: signature.to_string(),
        addresses,
    })
}

pub fn scan_code(process: &Process, bytes: &[u8], mask: &str) -> Result<ScanResult> {
    let pattern = Pattern::from_code(bytes, mask)?;
    let addresses = scan_readable_regions(process, &pattern)?;
    Ok(ScanResult {
        pattern: format!("code[{mask}]"),
        addresses,
    })
}

fn scan_readable_regions(process: &Process, pattern: &Pattern) -> Result<Vec<usize>> {
    let regions = process
        .regions()
        .map_err(|e| anyhow::anyhow!("region query failed: {e}"))?;

    let mut results = Vec::new();
    for region in &regions {
        if !region.protection.read {
            continue;
        }

        let data = match process.read_bytes(region.base, region.size) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for offset in pattern.scan(&data) {
            results.push(region.base + offset);
        }
    }
    Ok(results)
}
