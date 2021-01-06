/// Core-Local Interrupt Controller (CLINT)
pub struct CLINT {
    base_address: usize,
}

impl CLINT {
    /// # Safety
    /// Only safe if the base address is valid.
    pub const unsafe fn new(base_address: usize) -> Self {
        Self {
            base_address: base_address,
        }
    }

    pub const fn uninit() -> Self {
        Self { base_address: 0 }
    }

    /// Checks that this CLINT is initialized.
    fn assert_init(&self) {
        assert_ne!(self.base_address, 0, "CLINT is uninit!");
    }

    /// Gets the address of the mtime register.
    pub fn mtime_address(&self) -> *const u64 {
        self.assert_init();
        unsafe { (self.base_address as *const u8).add(0xbff8).cast::<u64>() }
    }

    /// Reads the mtime register.
    pub fn mtime(&self) -> u64 {
        self.assert_init();
        unsafe { self.mtime_address().read_volatile() }
    }

    /// Gets the address of the mtimecmp register.
    pub fn mtimecmp_address(&self) -> *mut u64 {
        self.assert_init();
        unsafe { (self.base_address as *mut u8).add(0x4000).cast::<u64>() }
    }

    /// Reads the mtimecmp register.
    pub fn mtimecmp(&self) -> u64 {
        self.assert_init();
        unsafe { self.mtimecmp_address().read_volatile() }
    }

    /// Writes to the mtimecmp register.
    pub fn set_mtimecmp(&mut self, value: u64) {
        self.assert_init();
        unsafe { self.mtimecmp_address().write_volatile(value) }
    }
}
