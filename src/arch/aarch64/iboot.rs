#[repr(C)]
pub struct iBootVideo {
    pub baseAddr: u64,
    pub display: u64,
    pub rowBytes: u64,
    pub width: u64,
    pub height: u64,
    pub depth: u64
}

#[repr(C)]
pub struct iBootArgs {
    pub revision: u16,
    pub version: u16,
    pub virtBase: u64,
    pub physBase: u64,
    pub memSize: u64,
    pub topOfKernelData: u64,
    pub framebuffer: iBootVideo,
    pub machineType: u32,
    pub deviceTreeP: usize,
    pub deviceTreeLength: u32,
    pub cmdline: [u8; 256],
    pub bootFlags: u64,
    pub memSizeActual: u64
}