#![allow(dead_code)]

use modular_bitfield::prelude::*;

#[bitfield(bits = 64)]
struct Sv39PTE {
    valid: bool,
    #[bits = 3]
    permissions: XWRPermissions,
    user: bool,
    global: bool,
    accessed: bool,
    dirty: bool,
    ppn0: B9,
    ppn1: B9,
    ppn2: B26,
    page_offset: B12,
}

#[derive(BitfieldSpecifier)]
#[bits = 3]
enum XWRPermissions {
    Pointer = 0b000,
    ReadOnly = 0b001,
    WriteOnly = 0b010,
    ReadWrite = 0b011,
    ExecOnly = 0b100,
    ReadExec = 0b101,
    WriteExec = 0b110,
    ReadWriteExec = 0b111,
}

#[inline(never)]
pub const unsafe fn init() {}
