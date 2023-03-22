//! macro definitions

/// Print a formatted log message to `stderr` to display in server logs
///
/// Performs formatting to match other common SQL error logs, roughly:
///
/// ```text
/// 2022-10-15 13:12:54+00:00 [Warning] Udf: this is the message
/// ```
///
/// ```
/// # #[cfg(not(miri))] // need to skip Miri because. it can't cross FFI
/// # fn test() {
///
/// use udf::udf_log;
///
/// // Prints "2022-10-08 05:27:30+00:00 [Error] UDF: this is an error"
/// // This matches the default entrypoint log format
/// udf_log!(Error: "this is an error");
///
/// udf_log!(Warning: "this is a warning");
///
/// udf_log!(Note: "this is info: value {}", 10 + 10);
///
/// udf_log!(Debug: "this is a debug message");
///
/// udf_log!("i print without the '[Level] UDF:' formatting");
///
/// # }
/// # #[cfg(not(miri))]
/// # test();
/// ```
#[macro_export]
macro_rules! udf_log {
    (Critical: $($msg:tt)*) => {{
        let formatted = format!("[Critical] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    (Error: $($msg:tt)*) => {{
        let formatted = format!("[Error] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    (Warning: $($msg:tt)*) => {{
        let formatted = format!("[Warning] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    (Note: $($msg:tt)*) => {{
        let formatted = format!("[Note] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    (Debug: $($msg:tt)*) => {{
        let formatted = format!("[Debug] UDF: {}", format!($($msg)*));
        udf_log!(formatted);
    }};
    ($msg:tt) => {
        eprintln!(
            "{} {}",
            $crate::chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%:z"),
            $msg
        );
    };
}

/// Log a call to a function, with optional printing of state
///
/// Log calls are only printed with feature "logging-debug". Call state is only
/// printed with feature "logging-debug-calls".
macro_rules! log_call {
    // Log for entering a function
    (enter: $name:literal, $type:ty, $($state:expr),*) => {
        log_call!(@common: "ENTER", "receive", $name, $type, $($state),*)
    };

    // Logging for exiting a function
    (exit: $name:literal, $type:ty, $($state:expr),*) => {
        log_call!(@common: "EXIT", "return", $name, $type, $($state),*)
    };

    // Internal macro, common enter/exit printing
    (@common:
        $enter_or_exit:literal,
        $receive_or_return:literal,
        $fn_name:literal,
        $type:ty,
        $($state:expr),*
    ) => {{
        #[cfg(feature = "logging-debug")]
        $crate::udf_log!(
            Debug: "{} {} for '{}'",
            $enter_or_exit, $fn_name, std::any::type_name::<$type>()
        );

        cfg_if::cfg_if! {
            if #[cfg(feature = "logging-debug-calls")] {
                $crate::udf_log!(Debug: "data {} state at {}", $receive_or_return, $fn_name);
                // For each specified item, print the expression and its value
                $(
                    $crate::udf_log!(
                        Debug: "[{}]: {} = {:#?}",
                        $receive_or_return, std::stringify!($state), $state
                    );
                )*
            }
        }
    }}
}
