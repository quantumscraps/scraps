use crate::drivers::ns16550a::NS16550A;
use crate::mutex::Mutex;

pub static UART: Mutex<NS16550A> = Mutex::new(NS16550A::new(0x1000_0000));

// Dumped dtb with `-M virt,dumpdtb=virt.out` to check timebase_freq
// which is 10,000,000
// Linux also uses HZ which is default to 1000
// 1,000,000,000 / (timebase_freq * HZ) < 0
// ~= 0.25
// therefore use TICKS_PER_NANO = 2.5
// (this seems pretty correct based on testing)
pub const TICKS_PER_NANO: u64 = 3; // 10 / 4 ~= 3, should probably add floats though
pub const NANOS_PER_TICK: u64 = 1;
pub const HAS_RDTIME: bool = false;
