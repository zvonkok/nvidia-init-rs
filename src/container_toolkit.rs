use anyhow::{anyhow, Context, Result};
use std::process::Command;

use crate::debug_info;
use crate::proc_cmdline::ParamContext;
use std::fs::File;
use std::io::Read;

pub fn nvidia_smi(context: &ParamContext) -> Result<()> {
    debug_info!(context, "nvidia-smi");

    let output = Command::new("/bin/nvidia-smi")
        .output()
        .context("failed to execute nvidia-smi")?;

    println!(
        "nvidia-smi {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

pub fn nvidia_container_toolkit(context: &ParamContext) -> Result<()> {
    debug_info!(
        context,
        "nvidia-ctk system create-device-nodes --control-devices --load-kernel-modules"
    );

    // Run the first nvidia-ctk command
    let output = Command::new("/bin/nvidia-ctk")
        .args([
            "-d",
            "system",
            "create-device-nodes",
            "--control-devices",
            "--load-kernel-modules",
        ])
        .output()
        .context("failed to execute /bin/nvidia-ctk system create-device-nodes")?;

    if !output.status.success() {
        return Err(anyhow!(
            "nvidia-ctk system create-device-nodes failed with status: {}\n error:{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    println!(
        "nvidia-ctk system {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    debug_info!(
        context,
        "nvidia-ctk cdi generate --output=/var/run/cdi/nvidia.yaml"
    );
    // Run the second nvidia-ctk command
    let output = Command::new("/bin/nvidia-ctk")
        .args(["-d", "cdi", "generate", "--output=/var/run/cdi/nvidia.yaml"])
        .output()
        .context("failed to execute /bin/nvidia-ctk cdi generate")?;

    if !output.status.success() {
        return Err(anyhow!(
            "nvidia-ctk cdi generate --output=/var/run/cdi/nvidia.yaml status: {}\n error:{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    println!(
        "nvidia-ctk cdi {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    load_and_print_file(context, "/var/run/cdi/nvidia.yaml")?;

    Ok(())
}

fn load_and_print_file(_context: &ParamContext, file_path: &str) -> Result<()> {
    let mut file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .with_context(|| format!("Failed to read file: {}", file_path))?;
    println!("{}", contents);

    Ok(())
}
