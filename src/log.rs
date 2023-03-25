// Make the compiler less whiny
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(dead_code)]

/// Usage:
///
/// Set an optional log file:
///     log::set_file("foo.log");
///
/// Set a log level, everything equal to or below it will be logged:
///     log::set_level(log::Level::Warning);
///
/// Log things:
///     log::info!("Look ma, I'm a log statement!");
///     log::debug!("debug message: {}", 42));


// Advantages of this approach:
//  - Doesn't use lazy_static or any other external crates (can just drop this file is another project).
//  - Doesn't require complicated instance management code.
//  - Smol


use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

// This is our 'singleton', which contains
// all the logger state wrapped in a mutex.
pub static LOGGER: Mutex<Logger> = Mutex::new(Logger {
    level: Level::Info,
    log_file: None,
});

#[derive(PartialOrd, PartialEq)]
pub enum Level {
    Error,
    Warning,
    Notice,
    Info,
    Debug,
    Trace,
    None,
}

pub struct Logger {
    level: Level,
    log_file: Option<File>,
}

impl Logger {
    pub fn log(&self, level: Level, msg: &str) {
        if self.level >= level {
            if let Some(mut f) = self.log_file.as_ref() {
                f.write_all(msg.as_bytes()).unwrap();
                f.write_all(b"\n").unwrap();
            } else {
                println!("{}", msg);
            }
        }
    }
}

pub fn set_level(level: Level) {
    LOGGER.lock().unwrap().level = level;
}

pub fn set_file(path: &str) {
    LOGGER.lock().unwrap().log_file = Some(File::create(path).unwrap());
}

// These macros are essentially a thin wrapper around Logger.log().
// They acquire a mutable reference to the Logger by locking the associated mutex
// and then call log() after passing the args to the format macro (to obtain a &str).

// The below was originally condensed (along with the enum) into a single macro
// using a bit of nesting magic, but I decided to leave the expanded versions in
// place to make it easier to understand.

macro_rules! none {
    ($($args:tt)*) => { log::LOGGER.lock().unwrap().log(log::Level::None, &format!($($args)*)); }
}
macro_rules! error {
    ($($args:tt)*) => { log::LOGGER.lock().unwrap().log(log::Level::Error, &format!($($args)*)); }
}
macro_rules! warning {
    ($($args:tt)*) => { log::LOGGER.lock().unwrap().log(log::Level::Warning, &format!($($args)*)); }
}
macro_rules! notice {
    ($($args:tt)*) => { log::LOGGER.lock().unwrap().log(log::Level::Notice, &format!($($args)*)); }
}
macro_rules! info {
    ($($args:tt)*) => { log::LOGGER.lock().unwrap().log(log::Level::Info, &format!($($args)*)); }
}
macro_rules! debug {
    ($($args:tt)*) => { log::LOGGER.lock().unwrap().log(log::Level::Debug, &format!($($args)*)); }
}
macro_rules! trace {
    ($($args:tt)*) => { log::LOGGER.lock().unwrap().log(log::Level::Trace, &format!($($args)*)); }
}

pub (crate) use none;
pub (crate) use error;
pub (crate) use warning;
pub (crate) use notice;
pub (crate) use info;
pub (crate) use debug;
pub (crate) use trace;
