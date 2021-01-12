/// See [std::print].
#[macro_export]
macro_rules! print {
	($($args:tt)+) => ({
			use core::fmt::Write;
			if let Some(ref mut stdout) = *$crate::STDOUT.lock() {
				let _ = write!(stdout, $($args)+);
			}
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

/// See [std::print].
#[macro_export]
macro_rules! print2 {
	($stdout:expr, $($args:tt)+) => ({
		use core::fmt::Write;
		if let Some(ref mut stdout) = $stdout {
			let _ = write!(stdout, $($args)+);
		}
	});
}

/// See [std::println].
#[macro_export]
macro_rules! println2 {
	($stdout:expr) => ({
		$crate::print2!($stdout, "\r\n")
	});
	($stdout:expr, $fmt:expr) => ({
		$crate::print2!($stdout, concat!($fmt, "\r\n"))
	});
	($stdout:expr, $fmt:expr, $($args:tt)+) => ({
		$crate::print2!($stdout, concat!($fmt, "\r\n"), $($args)+)
	});
}

/// Similar to [std::println] but prints with a timestamp.
#[macro_export]
macro_rules! printk2 {
	($stdout:expr) => {
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println2!($stdout, "[{:>5}.{:03}{:03}]", timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000)
	};
	($stdout:expr, $fmt:expr) => ({
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println2!($stdout, concat!("[{:>5}.{:03}{:03}] ", $fmt), timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000)
	});
	($stdout:expr, $fmt:expr, $($args:tt)+) => ({
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println2!($stdout, concat!("[{:>5}.{:03}{:03}] ", $fmt), timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000, $($args)+)
	});
}
