// Define the debug_info! macro
#[macro_export]
macro_rules! debug_info {
	($context:expr, $($arg:tt)*) => {
	    if $context.debug {
		info!($($arg)*);
	    }
	};
    }
