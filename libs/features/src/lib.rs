use platform_types::Logger;

pub static mut GLOBAL_LOGGER: Logger = None;
pub static mut GLOBAL_ERROR_LOGGER: Logger = None;

fn logger_log(logger: Logger, s: &str) {
    if let Some(l) = logger {
        l(s);
    }
}

pub fn log(s: &str) {
    logger_log(unsafe { GLOBAL_LOGGER }, s)
}

pub fn invariant_violation(s: &str) {
    logger_log(unsafe { GLOBAL_ERROR_LOGGER }, s)
}

#[cfg(feature = "logging")]
#[macro_export]
macro_rules! log {
    ($e:expr) => {
        log(&format!(concat!(stringify!($e), ": {:#?}"), $e));
    };
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! log {
    ($($whatever:tt)*) => {};
}

#[cfg(feature = "invariant-checking")]
#[macro_export]
macro_rules! invariant_violation {
    () => ({
        invariant_violation(&format!("invariant was violated! {}:{}", file!(), line!()));
        panic!("invariant was violated!")
    });
    ($code:block, $($rest:tt)*) => {
        invariant_violation!($($rest)*)
    };
    ($msg:expr) => ({
        invariant_violation(&format!("{} {}:{}", $msg, file!(), line!()));
        panic!($msg)
    });
    ($msg:expr,) => (
        invariant_violation!($msg)
    );
    ($fmt:expr, $($arg:tt)+) => ({
        invariant_violation(&format!($fmt, $($arg)*));
        panic!($fmt, $($arg)*)
    });
}

#[cfg(not(feature = "invariant-checking"))]
#[macro_export]
macro_rules! invariant_violation {
    ($code:block, $($rest:tt)*) => {
        $code
    };
    ($($whatever:tt)*) => {};
}

#[cfg(feature = "invariant-checking")]
#[macro_export]
macro_rules! invariant_assert {
    ($($arg:tt)+) => ({
        assert!($($arg)*)
    });
}

#[cfg(not(feature = "invariant-checking"))]
#[macro_export]
macro_rules! invariant_assert {
    ($($whatever:tt)*) => {};
}

#[cfg(feature = "invariant-checking")]
#[macro_export]
macro_rules! invariant_assert_eq {
    ($($arg:tt)+) => ({
        assert_eq!($($arg)*)
    });
}

#[cfg(not(feature = "invariant-checking"))]
#[macro_export]
macro_rules! invariant_assert_eq {
    ($($whatever:tt)*) => {};
}

// This is only slightly nicer to use than using the body of the macro directly, but
// it's nice to have all the features stuff in one place as a form of documentation.
#[macro_export]
macro_rules! invariants_checked {
    () => {{
        cfg!(feature = "invariant-checking")
    }};
}

#[macro_export]
macro_rules! loops_allowed {
    () => {
        cfg!(feature = "loops-allowed")
    };
}
