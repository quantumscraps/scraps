#[macro_export]
macro_rules! print {
	($($args:tt)+) => ({
			use core::fmt::Write;
			let _ = write!(crate::bsp::UART, $($args)+);
	});
}

#[macro_export]
macro_rules! println {
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

#[macro_export]
macro_rules! printk {
	() => {
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		println!("[{:>5}.{:03}{:03}]", timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000)
	};
	($fmt:expr) => ({
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		println!(concat!("[{:>5}.{:03}{:03}] ", $fmt), timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000)
	});
	($fmt:expr, $($args:tt)+) => ({
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		println!(concat!("[{:>5}.{:03}{:03}] ", $fmt), timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000, $($args)+)
	});
}
