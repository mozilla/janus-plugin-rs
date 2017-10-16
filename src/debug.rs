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

struct LogParameters {
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

fn print_log(level: LogLevel, message: &str, params: LogParameters) {
    if level as i32 <= params.max_log_level {
        let output = format_log(level, message, params);
        let output_cstr = CString::new(output).expect("Log messages must be valid C strings.");
        unsafe { ffi::janus_vprintf(output_cstr.as_ptr()) }
    }
}

fn format_log(level: LogLevel, message: &str, params: LogParameters) -> String {
    let mut output = String::with_capacity(message.len() + 40);
    if params.log_timestamps {
        write!(output, "{} ", (params.clock)().format("[%a %b %e %T %Y]")).unwrap();
    }
    if level <= LogLevel::Warn {
        let name = format!("[{:?}] ", level).to_uppercase();
        match (params.log_colors, level.color()) {
            (true, Some(c)) => output.push_str(&name.color(c)),
            _ => output.push_str(&name),
        }
    }
    output.push_str(message);
    output.push('\n');
    output
}

/// Writes a message at the given log level to the Janus log.
pub fn log(level: LogLevel, message: &str) {
    print_log(level, message, LogParameters::default());
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
            format_log(
                LogLevel::Warn,
                "Test message.",
                LogParameters {
                    max_log_level: 6,
                    log_timestamps: true,
                    log_colors: false,
                    clock: || fixed_clock(2017, 10, 10, 1, 37, 46),
                }
            )
        )
    }
}
