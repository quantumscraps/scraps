/// See [std::print].
#[macro_export]
macro_rules! print {
	($($args:tt)+) => ({
			use core::fmt::Write;
			#[allow(unused_unsafe)]
			unsafe {
				let _ = write!(crate::bsp::UART.lock(), $($args)+);
			}
	});
}

/// See [std::panic_print].
/// # Safety
/// Safe only to call once.
#[macro_export]
macro_rules! panic_print {
	($($args:tt)+) => ({
			use core::fmt::Write;
			#[allow(unused_unsafe)]
			// Safety: !! UNSAFE !!
			unsafe {
				crate::bsp::UART.force_unlock();
				let _ = write!(crate::bsp::UART.lock(), $($args)+);
			}
	});
}
/// See [std::println].
/// # Safety
/// Safe only to call once.
#[macro_export]
macro_rules! panic_println {
	() => ({
		$crate::panic_print!("\r\n")
	});
	($fmt:expr) => ({
		$crate::panic_print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		$crate::panic_print!(concat!($fmt, "\r\n"), $($args)+)
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
	($uart:expr, $($args:tt)+) => ({
			use core::fmt::Write;
			#[allow(unused_unsafe)]
			unsafe {
				let _ = write!($uart, $($args)+);
			}
	});
}

/// See [std::println].
#[macro_export]
macro_rules! println2 {
	($uart:expr) => ({
		$crate::print2!($uart, "\r\n")
	});
	($uart:expr, $fmt:expr) => ({
		$crate::print2!($uart, concat!($fmt, "\r\n"))
	});
	($uart:expr, $fmt:expr, $($args:tt)+) => ({
		$crate::print2!($uart, concat!($fmt, "\r\n"), $($args)+)
	});
}

/// Similar to [std::println] but prints with a timestamp.
#[macro_export]
macro_rules! printk2 {
	($uart:expr) => {
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println2!($uart, "[{:>5}.{:03}{:03}]", timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000)
	};
	($uart:expr, $fmt:expr) => ({
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println2!($uart, concat!("[{:>5}.{:03}{:03}] ", $fmt), timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000)
	});
	($uart:expr, $fmt:expr, $($args:tt)+) => ({
		use crate::time::TimeCounter;
		let timestamp = crate::time::time_counter().uptime();
		let timestamp_us = timestamp.subsec_micros();
		$crate::println2!($uart, concat!("[{:>5}.{:03}{:03}] ", $fmt), timestamp.as_secs(), timestamp_us / 1000, timestamp_us % 1000, $($args)+)
	});
}
