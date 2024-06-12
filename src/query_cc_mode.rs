use anyhow::{Context, Result};
use std::process::Command;

pub fn query_gpu_cc_mode(gpu_bdfs: &Vec<String>) -> Result<String> {
    let mut mode: Option<String> = None;

    for bdf in gpu_bdfs {
        let output = Command::new("/sbin/nvidia_gpu_tools")
            .args([
                "--mmio-access-type=sysfs",
                "--query-cc-mode",
                "--gpu-bdf",
                bdf,
            ])
            .output()
            .with_context(|| format!("Failed to execute nvidia_gpu_tools for BDF: {}", bdf))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        let current_mode = if output_str.contains("CC mode is on") {
            "on".to_string()
        } else {
            "off".to_string()
        };

        match &mode {
            Some(m) if m != &current_mode => {
                return Err(anyhow::anyhow!(
                    "Inconsistent CC mode detected: {} has mode '{}', expected '{}'",
                    bdf,
                    current_mode,
                    m
                ));
            }
            _ => mode = Some(current_mode),
        }
    }

    mode.ok_or_else(|| anyhow::anyhow!("No GPUs found"))
}
