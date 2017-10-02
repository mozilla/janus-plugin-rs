extern crate chrono;
extern crate colored;

use self::chrono::Local;
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

/// Writes a message at the given log level to the Janus log.
pub fn log(level: LogLevel, message: &str) {
    let max_log_level = unsafe { ffi::janus_log_level };
    if level as i32 <= max_log_level {
        let mut output = String::with_capacity(message.len() + 40);
        let are_timestamps_enabled = unsafe { ffi::janus_log_timestamps == 1 };
        let are_colors_enabled = unsafe { ffi::janus_log_colors == 1 };
        if are_timestamps_enabled {
            write!(output, "{} ", Local::now().format("[%a %b %e %T %Y]")).unwrap();
        }
        if level <= LogLevel::Warn {
            let name = format!("[{:?}] ", level).to_uppercase();
            match (are_colors_enabled, level.color()) {
                (true, Some(c)) => output.push_str(&name.color(c)),
                _ => output.push_str(&name),
            }
        }
        output.push_str(message);
        output.push('\n');

        let output_cstr = CString::new(output).expect("Log messages must be valid C strings.");
        unsafe { ffi::janus_vprintf(output_cstr.as_ptr()) }
    }
}
