use alloc::prelude::v1::*;
use fdt_rs::index::{DevTreeIndex, DevTreeIndexItem};
use fdt_rs::prelude::*;
use ns16550a::NS16550A;

use crate::{driver_interfaces::Console, lookup_dtb_entry, STDOUT};

#[cfg(feature = "bsp_riscvirt")]
pub mod ns16550a;

#[cfg(feature = "bsp_raspi64")]
pub mod pl011;

/// Detects the stdout path from DTB and initializes it.
pub fn detect_stdout(dtb: &DevTreeIndex) {
    // Cannot print any errors since stdout isn't initialized yet!
    let stdout_path = match lookup_dtb_entry(dtb, "/chosen/stdout-path") {
        Some(path) => match path {
            DevTreeIndexItem::Prop(p) => match p.str() {
                Ok(s) => s,
                Err(_) => return,
            },
            DevTreeIndexItem::Node(_) => return,
        },
        None => return,
    };
    let device = match lookup_dtb_entry(dtb, stdout_path) {
        Some(dev) => match dev {
            DevTreeIndexItem::Prop(_) => return,
            DevTreeIndexItem::Node(n) => n,
        },
        None => return,
    };
    let compatible = match device
        .props()
        .filter(|prop| prop.name() == Ok("compatible"))
        .next()
        .map(|prop| prop.str())
    {
        Some(r) => match r {
            Ok(c) => c,
            Err(_) => return,
        },
        None => return,
    };
    *STDOUT.lock() = match compatible {
        "ns16550a" => NS16550A::from_dtb(&device).map(|x| Box::new(x) as Box<_>),
        _ => None,
    };
}
