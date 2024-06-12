use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn check_gpu_supported(
    gpu_device_ids: &Vec<String>,
    supported_gpu_devids: &Path,
) -> Result<bool> {
    if !supported_gpu_devids.exists() {
        println!(
            "nvidia: {} file not found, skipping check",
            supported_gpu_devids.display()
        );
        return Ok(false);
    }

    let file = File::open(supported_gpu_devids)
        .context(format!("Failed to open {:?}", supported_gpu_devids))?;
    let reader = BufReader::new(file);

    let supported_ids: Vec<String> = reader
        .lines()
        .map(|line| line.expect("Could not read line"))
        .collect();

    for devid in gpu_device_ids {
        if !supported_ids.contains(devid) {
            println!("GPU {} is not supported, returning", devid);
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_check_gpu_supported() {
        let gpu_device_ids = &vec!["0x2330".to_string()];
        let supported_gpu_devids = Path::new("/supported-gpu.devids");
        let result = check_gpu_supported(gpu_device_ids, supported_gpu_devids).unwrap();
        assert_eq!(result, true);
    }
}
