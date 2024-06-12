use anyhow::Context;
use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::debug_info;

#[derive(Debug)]
pub struct ParamContext {
    pub debug: bool,
    pub nvidia_smi_lgc_value: Option<String>,
}

pub const PARAM_NVIDIA_INIT_DEBUG: &str = "nvidia.init.debug";

pub type ParamHandler = fn(&str, &mut ParamContext) -> Result<()>;

pub fn process_kernel_params(
    handlers: HashMap<&str, ParamHandler>,
    context: &mut ParamContext,
    cmdline: Option<&str>,
) -> Result<()> {
    let content = match cmdline {
        Some(custom) => custom.to_string(),
        None => {
            let mut file = File::open("/proc/cmdline").context("Failed to open /proc/cmdline")?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .context("Failed to read /proc/cmdline")?;
            content
        }
    };
    // Split the content into key-value pairs
    for param in content.split_whitespace() {
        if let Some((key, value)) = param.split_once('=') {
            if let Some(handler) = handlers.get(key) {
                handler(value, context)?;
            }
        }
    }

    Ok(())
}

pub fn nvidia_init_debug(value: &str, context: &mut ParamContext) -> Result<()> {
    context.debug = value == "1";
    debug_info!(context, "debug={}", value);
    Ok(())
}

#[allow(dead_code)]
pub fn nvidia_smi_lgc(value: &str, context: &mut ParamContext) -> Result<()> {
    debug_info!(context, "nvidia-smi lgc {}", value);
    context.nvidia_smi_lgc_value = Some(value.to_string());
    Ok(())
}

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn test_nvidia_init_debug() {
        let mut context = ParamContext {
            debug: false,
            nvidia_smi_lgc_value: None,
        };
        nvidia_init_debug("1", &mut context).unwrap();
        assert_eq!(context.debug, true);
    }

    #[test]
    fn test_process_kernel_params_debug_1() {
        let mut handlers: HashMap<&str, ParamHandler> = HashMap::new();
        handlers.insert(PARAM_NVIDIA_INIT_DEBUG, nvidia_init_debug);
        let mut context = ParamContext {
            debug: false,
            nvidia_smi_lgc_value: None,
        };

        process_kernel_params(
            handlers,
            &mut context,
            Some(
                format!(
                    "nvidia.smi.lgc=1500 {}=1 nvidia.smi.lgc=1500",
                    PARAM_NVIDIA_INIT_DEBUG
                )
                .as_str(),
            ),
        )
        .unwrap();
        assert_eq!(context.debug, true);
    }
    #[test]
    fn test_process_kernel_params_debug_0() {
        let mut handlers: HashMap<&str, ParamHandler> = HashMap::new();
        handlers.insert(PARAM_NVIDIA_INIT_DEBUG, nvidia_init_debug);
        let mut context = ParamContext {
            debug: false,
            nvidia_smi_lgc_value: None,
        };

        process_kernel_params(
            handlers,
            &mut context,
            Some(
                format!(
                    "nvidia.smi.lgc=1500 {}=0 nvidia.smi.lgc=1500",
                    PARAM_NVIDIA_INIT_DEBUG
                )
                .as_str(),
            ),
        )
        .unwrap();
        assert_eq!(context.debug, false);
    }
    #[test]
    fn test_process_kernel_params_debug_none() {
        let mut handlers: HashMap<&str, ParamHandler> = HashMap::new();
        handlers.insert(PARAM_NVIDIA_INIT_DEBUG, nvidia_init_debug);
        let mut context = ParamContext {
            debug: false,
            nvidia_smi_lgc_value: None,
        };

        process_kernel_params(
            handlers,
            &mut context,
            Some(format!("nvidia.smi.lgc=1500 {}= ", PARAM_NVIDIA_INIT_DEBUG).as_str()),
        )
        .unwrap();
        assert_eq!(context.debug, false);
    }
}
