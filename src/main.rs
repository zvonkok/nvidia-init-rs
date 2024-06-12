use nix::unistd::{fork, ForkResult};
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

mod check_supported;
mod container_toolkit;
mod cpu_vendor;
mod get_devices;
mod macros;
mod mount;
mod proc_cmdline;
mod query_cc_mode;
mod udev;

#[macro_use]
extern crate log;
extern crate kernlog;

use container_toolkit::nvidia_container_toolkit;
use proc_cmdline::PARAM_NVIDIA_INIT_DEBUG;

fn main() {
    let mut early_handlers: HashMap<&str, proc_cmdline::ParamHandler> = HashMap::new();
    early_handlers.insert(PARAM_NVIDIA_INIT_DEBUG, proc_cmdline::nvidia_init_debug);

    let mut context = proc_cmdline::ParamContext {
        debug: false,
        nvidia_smi_lgc_value: None,
    };

    mount::mount_setup();
    kernlog::init().unwrap();

    proc_cmdline::process_kernel_params(early_handlers, &mut context, None).unwrap();

    let cpu_vendor = cpu_vendor::query_cpu_vendor().unwrap();
    debug_info!(context, "cpu_vendor: {}", cpu_vendor);

    let (gpu_bdfs, gpu_devids) = get_devices::get_gpu_devices(Path::new("/sys/bus/pci")).unwrap();

    if !gpu_bdfs.is_empty() {
        let _cc_mode = query_cc_mode::query_gpu_cc_mode(&gpu_bdfs).unwrap();
        let _supported =
            check_supported::check_gpu_supported(&gpu_devids, Path::new("/supported-gpu.devids"))
                .unwrap();
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child: _ }) => {
            let mut result = Command::new("/sbin/init");
            result.exec();
        }
        Ok(ForkResult::Child) => {
            if context.debug {
                info!("starting udev");
            }
            loop {
                udev::udev::udev();
                let (gpu_bdfs, gpu_devids) =
                    get_devices::get_gpu_devices(Path::new("/sys/bus/pci")).unwrap();
                let cc_mode = query_cc_mode::query_gpu_cc_mode(&gpu_bdfs).unwrap();

                let supported = check_supported::check_gpu_supported(
                    &gpu_devids,
                    Path::new("/supported-gpu.devids"),
                )
                .unwrap();

                if !supported {
                    panic!("Unsupported GPU detected")
                }

                debug_info!(context, "GPU CC mode: {}", cc_mode);

                nvidia_container_toolkit(&context).unwrap();

                //  nvidia_smi(&context).unwrap();
            }
        }
        Err(e) => {
            panic!("Fork failed: {}", e);
        }
    }
}
