/// See [std::print].
#[macro_export]
macro_rules! print {
	($($args:tt)+) => ({
			use core::fmt::Write;
			let _ = write!(crate::bsp::UART.lock(), $($args)+);
	});
}

/// See [std::println].
#[macro_export]
macro_rules! println {
	() => ({
		$crate::print!("\r\n")
	});
	($fmt:expr) => ({
		$crate::print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		$crate::print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

/// Similar to [std::println] but prints with a timestamp.
#[macro_export]
macro_rules! printk {
	() => {
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println!("[{:>5}.{:03}{:03}]", timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000)
	};
	($fmt:expr) => ({
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println!(concat!("[{:>5}.{:03}{:03}] ", $fmt), timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000)
	});
	($fmt:expr, $($args:tt)+) => ({
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println!(concat!("[{:>5}.{:03}{:03}] ", $fmt), timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000, $($args)+)
	});
}
