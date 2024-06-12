use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn get_gpu_devices(base_path: &Path) -> Result<(Vec<String>, Vec<String>)> {
    let mut gpu_bdfs = Vec::new();
    let mut gpu_device_ids = Vec::new();

    // Iterate over PCI devices in the provided base path
    for entry in
        fs::read_dir(base_path.join("devices")).context("Failed to read devices directory")?
    {
        let entry = entry.context("Failed to read entry in devices directory")?;
        let device_dir = entry.path();

        // Read the vendor ID
        let vendor_path = device_dir.join("vendor");
        let vendor = fs::read_to_string(&vendor_path)
            .unwrap_or_default()
            .trim()
            .to_string();

        // Read the class ID
        let class_path = device_dir.join("class");
        let class = fs::read_to_string(&class_path)
            .unwrap_or_default()
            .trim()
            .to_string();

        // Check if the device is an NVIDIA GPU (vendor ID 0x10de) and has the correct class ID
        if vendor == "0x10de" && (class == "0x030000" || class == "0x030200") {
            // Extract the BDF (bus, device, function) using the directory name
            if let Some(bdf) = device_dir.file_name().and_then(|bdf| bdf.to_str()) {
                gpu_bdfs.push(bdf.to_string());
            }

            // Read the device ID
            let device_path = device_dir.join("device");
            let device_id = fs::read_to_string(&device_path)
                .unwrap_or_default()
                .trim()
                .to_string();
            gpu_device_ids.push(device_id);
        }
    }

    Ok((gpu_bdfs, gpu_device_ids))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, write};
    use tempfile::tempdir;

    #[test]
    fn test_get_gpu_devices() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path();

        // Create mock PCI devices
        let device_1_path = base_path.join("devices/0000:01:00.0");
        let device_2_path = base_path.join("devices/0000:02:00.0");
        let non_gpu_device_path = base_path.join("devices/0000:03:00.0");

        // Create directories for devices
        create_dir_all(&device_1_path)?;
        create_dir_all(&device_2_path)?;
        create_dir_all(&non_gpu_device_path)?;

        // Create mock files for device 1 (NVIDIA GPU)
        write(device_1_path.join("vendor"), "0x10de")?;
        write(device_1_path.join("class"), "0x030000")?;
        write(device_1_path.join("device"), "1234")?;

        // Create mock files for device 2 (NVIDIA GPU)
        write(device_2_path.join("vendor"), "0x10de")?;
        write(device_2_path.join("class"), "0x030200")?;
        write(device_2_path.join("device"), "5678")?;

        // Create mock files for non-GPU device
        write(non_gpu_device_path.join("vendor"), "0x1234")?;
        write(non_gpu_device_path.join("class"), "0x567800")?;
        write(non_gpu_device_path.join("device"), "abcd")?;

        // Run the function with the mock PCI space
        let (gpu_bdfs, gpu_device_ids) = get_gpu_devices(base_path)?;

        // Check the results
        assert_eq!(gpu_bdfs.len(), 2);
        assert!(gpu_bdfs.contains(&"0000:01:00.0".to_string()));
        assert!(gpu_bdfs.contains(&"0000:02:00.0".to_string()));

        assert_eq!(gpu_device_ids.len(), 2);
        assert!(gpu_device_ids.contains(&"1234".to_string()));
        assert!(gpu_device_ids.contains(&"5678".to_string()));

        Ok(())
    }

    #[test]
    fn test_get_gpu_devices_baremetal() {
        let (bdfs, devids) = get_gpu_devices(Path::new("/sys/bus/pci")).unwrap();

        println!("BDFs: {:?}", bdfs);
        println!("DEVIDSs: {:?}", devids);
    }
}