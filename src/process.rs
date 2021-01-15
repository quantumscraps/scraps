use crate::arch::{Fregs, Regs};

/// Represents a scheduled process
pub struct Process {
    regs: Regs,
    fregs: Fregs,
    pid: u64,
    // Currently only allow 8 disjoint mappings...
    // virt_base, size (bytes), phys_base
    pages: [(usize, usize, usize); 8],
}
