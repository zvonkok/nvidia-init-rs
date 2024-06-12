use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn query_cpu_vendor() -> Result<String> {
    let cpu_vendor_file = "/proc/cpuinfo";
    let file = File::open(cpu_vendor_file).context("Failed to open /proc/cpuinfo")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.context("Failed to read line from /proc/cpuinfo")?;
        if line.contains("AuthenticAMD") {
            return Ok("amd".to_string());
        } else if line.contains("GenuineIntel") {
            return Ok("intel".to_string());
        } else if line.contains("CPU implementer") && line.contains("0x41") {
            return Ok("arm".to_string());
        }
    }

    Err(anyhow::anyhow!("CPU vendor not found"))
}

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn test_query_cpu_vendor() {
        let vendor = query_cpu_vendor().unwrap();
        assert!(vendor == "amd" || vendor == "intel" || vendor == "arm");
    }
}
