use fdt_rs::base::DevTree;
use fdt_rs::prelude::*;

use crate::{
    driver_interfaces::{Console, UartConsole},
    util::{lookup_dtb_entry_node, HeaplessErrorExt, HeaplessResult},
    STDOUT,
};

use self::ns16550a::NS16550A;

#[cfg(feature = "bsp_riscvirt")]
pub mod ns16550a;

#[cfg(feature = "bsp_raspi64")]
pub mod pl011;

/// Detects the stdout path from DTB and initializes it.
pub fn detect_stdout(dtb: &DevTree) -> HeaplessResult<()> {
    // Cannot print any errors since stdout isn't initialized yet!
    // lol I give up on lookup_dtb_entry, just do it here
    let stdout_path = 'get_stdoutpath: {
        for node in dtb.nodes().iterator() {
            let node = node?;
            if node.name()? == "chosen" {
                // get prop!
                for prop in node.props().iterator() {
                    let prop = prop?;
                    if prop.name()? == "stdout-path" {
                        // this is it
                        break 'get_stdoutpath prop.str()?;
                    }
                }
            }
        }
        Err("Failed to find /chosen/stdout-path")?
    };
    let device = lookup_dtb_entry_node(dtb, stdout_path).context("Couldn't lookup stdout node")?;
    *STDOUT.lock() =
        Some(UartConsole::from_dtb(&device).context("Couldn't create console from DTB")?);
    Ok(())
}

/// Gets a known good UART.
pub fn known_good_uart() -> UartConsole {
    #[cfg(target_arch = "riscv64")]
    let mut uart = UartConsole::NS16550A(unsafe { NS16550A::new(0x1000_0000) });
    #[cfg(target_arch = "aarch64")]
    let mut uart = UartConsole::PL011(unsafe { PL011::new(0x4000_0000) });
    uart.init();
    uart
}
