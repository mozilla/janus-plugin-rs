/// Utilities for writing messages to the Janus log.
extern crate chrono;
extern crate colored;

use self::chrono::{DateTime, Local};
use self::colored::{Color, Colorize};
use super::ffi;
use std::ffi::CString;
use std::fmt::Write;

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
    fn color(&self) -> Option<Color> {
        match *self {
            LogLevel::Fatal => Some(Color::Magenta),
            LogLevel::Err => Some(Color::Red),
            LogLevel::Warn => Some(Color::Yellow),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogParameters {
    pub max_log_level: i32,
    pub log_timestamps: bool,
    pub log_colors: bool,
    pub clock: fn() -> DateTime<Local>,
}

impl Default for LogParameters {
    fn default() -> Self {
        unsafe {
            Self {
                max_log_level: ffi::janus_log_level,
                log_timestamps: ffi::janus_log_timestamps == 1,
                log_colors: ffi::janus_log_colors == 1,
                clock: Local::now,
            }
        }
    }
}

/// Writes a message at the given log level to the Janus log, using the provided parameters to control
/// how the log message is formatted.
pub fn write_log(level: LogLevel, message: &str, params: LogParameters) {
    if level as i32 <= params.max_log_level {
        let output = CString::new(print_log(level, message, params)).expect("Null character in log message :(");
        unsafe { ffi::janus_vprintf(output.as_ptr()) }
    }
}

/// Prints a message at the given log level into an owned string, using the provided parameters to control
/// how the log message is formatted.
pub fn print_log(level: LogLevel, message: &str, params: LogParameters) -> String {
    let mut output = String::with_capacity(message.len() + 40);
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
    write!(output, "{}\n", message).unwrap();
    output
}

/// Writes a message at the given log level to the Janus log.
pub fn log(level: LogLevel, message: &str) {
    write_log(level, message, LogParameters::default());
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
                "Test message.",
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
                "Crash!",
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
                "An error occurred!",
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
                "Attention!",
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
                "Just a message.",
                LogParameters {
                    log_colors: true,
                    ..default_log_parameters()
                }
            )
        );
    }

    fn default_log_parameters() -> LogParameters {
        LogParameters {
            max_log_level: 6,
            log_timestamps: false,
            log_colors: false,
            clock: || fixed_clock(2017, 10, 10, 1, 37, 46),
        }
    }
}
