/// Utilities for writing messages to the Janus log.
extern crate chrono;
extern crate colored;

pub use super::ffi::janus_log_level as JANUS_LOG_LEVEL;
use self::chrono::{DateTime, Local};
use self::colored::{Color, Colorize};
use super::ffi;
use std::ffi::CString;
use std::fmt::Write;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A Janus log level. Lower is more severe.
pub enum LogLevel {
    Fatal = 1,
    Err = 2,
    Warn = 3,
    Info = 4,
    Verb = 5,
    Huge = 6,
    Dbg = 7,
}

impl LogLevel {
    /// The color associated with each log level's label (if colors are enabled.)
    fn color(self) -> Option<Color> {
        match self {
            LogLevel::Fatal => Some(Color::Magenta),
            LogLevel::Err => Some(Color::Red),
            LogLevel::Warn => Some(Color::Yellow),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogParameters {
    pub log_timestamps: bool,
    pub log_colors: bool,
    pub clock: fn() -> DateTime<Local>,
}

impl Default for LogParameters {
    fn default() -> Self {
        unsafe {
            Self {
                log_timestamps: ffi::janus_log_timestamps == 1,
                log_colors: ffi::janus_log_colors == 1,
                clock: Local::now,
            }
        }
    }
}

/// Writes a message at the given log level to the Janus log, using the provided parameters to control
/// how the log message is formatted.
pub fn log(level: LogLevel, message: fmt::Arguments, params: LogParameters) {
    unsafe {
        let output = CString::new(print_log(level, message, params)).expect("Null character in log message :(");
        ffi::janus_vprintf(output.as_ptr())
    }
}

/// Prints a message at the given log level into an owned string, using the provided parameters to control
/// how the log message is formatted.
pub fn print_log(level: LogLevel, message: fmt::Arguments, params: LogParameters) -> String {
    let mut output = String::with_capacity(150); // reasonably conservative size for typical messages
    if params.log_timestamps {
        write!(output, "{} ", (params.clock)().format("[%a %b %e %T %Y]")).unwrap();
    }
    if level <= LogLevel::Warn {
        let prefix = format!("[{:?}] ", level).to_uppercase();
        let prefix = match level.color() {
            Some(c) if params.log_colors => format!("{}", prefix.color(c)),
            _ => prefix,
        };
        write!(output, "{}", prefix).unwrap();
    }
    output.write_fmt(message).expect("Error constructing log message!");
    output.push('\n');
    output
}

#[macro_export]
macro_rules! janus_log_enabled {
    ($lvl:expr) => (($lvl as i32) <= unsafe { $crate::debug::JANUS_LOG_LEVEL })
}

#[macro_export]
macro_rules! janus_log {
    ($lvl:expr, $($arg:tt)+) => ({
        let lvl = $lvl;
        if janus_log_enabled!(lvl) {
            $crate::debug::log(lvl, format_args!($($arg)+), $crate::debug::LogParameters::default())
        }
    })
}

#[macro_export]
macro_rules! janus_fatal {
    ($($arg:tt)+) => (janus_log!($crate::debug::LogLevel::Fatal, $($arg)+))
}

#[macro_export]
macro_rules! janus_err {
    ($($arg:tt)+) => (janus_log!($crate::debug::LogLevel::Err, $($arg)+))
}

#[macro_export]
macro_rules! janus_warn {
    ($($arg:tt)+) => (janus_log!($crate::debug::LogLevel::Warn, $($arg)+))
}

#[macro_export]
macro_rules! janus_info {
    ($($arg:tt)+) => (janus_log!($crate::debug::LogLevel::Info, $($arg)+))
}

#[macro_export]
macro_rules! janus_verb {
    ($($arg:tt)+) => (janus_log!($crate::debug::LogLevel::Verb, $($arg)+))
}

#[macro_export]
macro_rules! janus_huge {
    ($($arg:tt)+) => (janus_log!($crate::debug::LogLevel::Huge, $($arg)+))
}

#[macro_export]
macro_rules! janus_dbg {
    ($($arg:tt)+) => (janus_log!($crate::debug::LogLevel::Dbg, $($arg)+))
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::chrono::TimeZone;

    fn fixed_clock(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> DateTime<Local> {
        Local.ymd(year, month, day).and_hms(hour, min, sec)
    }

    #[test]
    fn log_format_correctness() {
        assert_eq!(
            "[Tue Oct 10 01:37:46 2017] [WARN] Test message.\n",
            print_log(
                LogLevel::Warn,
                format_args!("{}", "Test message."),
                LogParameters {
                    log_timestamps: true,
                    ..default_log_parameters()
                }
            )
        )
    }

    #[test]
    fn log_colored_output() {
        assert_eq!(
            "\u{1b}[35m[FATAL] \u{1b}[0mCrash!\n",
            print_log(
                LogLevel::Fatal,
                format_args!("{}", "Crash!"),
                LogParameters {
                    log_colors: true,
                    ..default_log_parameters()
                }
            )
        );

        assert_eq!(
            "\u{1b}[31m[ERR] \u{1b}[0mAn error occurred!\n",
            print_log(
                LogLevel::Err,
                format_args!("{}", "An error occurred!"),
                LogParameters {
                    log_colors: true,
                    ..default_log_parameters()
                }
            )
        );

        assert_eq!(
            "\u{1b}[33m[WARN] \u{1b}[0mAttention!\n",
            print_log(
                LogLevel::Warn,
                format_args!("{}", "Attention!"),
                LogParameters {
                    log_colors: true,
                    ..default_log_parameters()
                }
            )
        );

        assert_eq!(
            "Just a message.\n",
            print_log(
                LogLevel::Info,
                format_args!("{}", "Just a message."),
                LogParameters {
                    log_colors: true,
                    ..default_log_parameters()
                }
            )
        );
    }

    fn default_log_parameters() -> LogParameters {
        LogParameters {
            log_timestamps: false,
            log_colors: false,
            clock: || fixed_clock(2017, 10, 10, 1, 37, 46),
        }
    }
}
