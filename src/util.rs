pub unsafe fn sb(address: usize, value: u8) {
    let reg = address as *mut u8;
    reg.write_volatile(value);
}

pub unsafe fn lb(address: usize) -> u8 {
    let reg = address as *const u8;
    reg.read_volatile()
}

pub unsafe fn sw(address: usize, value: u16) {
    let reg = address as *mut u16;
    reg.write_volatile(value);
}

pub unsafe fn lw(address: usize) -> u16 {
    let reg = address as *const u16;
    reg.read_volatile()
}

pub unsafe fn sd(address: usize, value: u32) {
    let reg = address as *mut u32;
    reg.write_volatile(value);
}

pub unsafe fn ld(address: usize) -> u32 {
    let reg = address as *const u32;
    reg.read_volatile()
}

pub unsafe fn sq(address: usize, value: u64) {
    let reg = address as *mut u64;
    reg.write_volatile(value);
}

pub unsafe fn lq(address: usize) -> u64 {
    let reg = address as *const u64;
    reg.read_volatile()
}