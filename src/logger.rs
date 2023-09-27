#![allow(dead_code)]
#![allow(unused_macros)]

use std::{
    env,
    fs::{File, OpenOptions},
    hash::{Hash, Hasher},
    io::Write,
    sync::{Arc, Mutex, OnceLock},
    time::SystemTime,
};

static INSTANCE: OnceLock<Arc<Mutex<Option<Logger>>>> = OnceLock::new();

const FILENAME: &str = "debug.log";

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
    Off,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Off => write!(f, "OFF"),
        }
    }
}

/// The logger struct. A singleton that can only be created once.
#[derive(Clone, Debug)]
pub struct Logger {
    file: Arc<Mutex<File>>,
    level: LogLevel,
    filename: String,
}

trait FormatTime {
    fn format(&self, format: &str) -> String;
}

/// Convert a SystemTime to a string using only std library functions.
impl FormatTime for SystemTime {
    fn format(&self, format: &str) -> String {
        let (secs, nanos) = match self.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs(), dur.subsec_nanos()),
            Err(_) => (0, 0),
        };

        let mut output = String::new();
        let mut chars = format.chars();

        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    let c = chars.next().unwrap_or('%');
                    match c {
                        'Y' => output.push_str(&format!("{:04}", secs / 31536000)),
                        'm' => output.push_str(&format!("{:02}", (secs % 31536000) / 2592000)),
                        'd' => output.push_str(&format!("{:02}", (secs % 2592000) / 86400)),
                        'H' => output.push_str(&format!("{:02}", (secs % 86400) / 3600)),
                        'M' => output.push_str(&format!("{:02}", (secs % 3600) / 60)),
                        'S' => output.push_str(&format!("{:02}", secs % 60)),
                        'f' => output.push_str(&format!("{:03}", nanos / 1_000_000)),
                        'Z' => output.push_str("UTC"),
                        _ => output.push(c),
                    }
                }
                _ => output.push(c),
            }
        }

        output
    }
}

/// Generate temp file name
///
/// Returns a string that looks like this:
/// `temp-8444741687653642537.log`
fn generate_temp_file_name() -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let now = SystemTime::now();
    let now_string = now.format("%Y-%m-%d %H:%M:%S%.3f %Z");
    now_string.hash(&mut hasher);
    let hash = hasher.finish();
    let prefix = "temp-";
    let suffix = ".log";
    // make sure it's exactly 32 characters long
    let len = 32 - prefix.len() - suffix.len();
    let hash = format!("{hash:0>len$}");

    format!("temp-{hash}.log")
}

fn get_file_and_filename() -> (Arc<Mutex<File>>, String) {
    let filename: String;
    let file: Arc<Mutex<File>>;
    if !cfg!(test) {
        filename = FILENAME.to_string();
        file = Arc::new(Mutex::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename.clone())
                .unwrap(),
        ));
    } else {
        // create a temp file using the std library
        let temp_dir = env::temp_dir();
        // append "logger" to the temp dir so it's like this:
        // /tmp/logger/temp-af44fa0-1f2c-4b5a-9c1f-7f8e9d0a1b2c.log
        let temp_dir = temp_dir.join("logger");
        // remove the temp dir if it already exists
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).unwrap();
        }
        std::fs::create_dir(&temp_dir).unwrap();
        let temp_file_name = generate_temp_file_name();
        let temp_file_path = temp_dir.join(temp_file_name);
        filename = temp_file_path.to_str().unwrap().to_string();

        file = Arc::new(Mutex::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(temp_file_path)
                .unwrap(),
        ));
    }

    (file, filename)
}

impl Logger {
    /// Create a new logger. This is a singleton, so it can only be called once.
    fn new() -> Self {
        let level = LogLevel::Info;
        let (file, filename) = get_file_and_filename();

        Self {
            file,
            level,
            filename,
        }
    }

    /// Set the log level. This will only log messages that are equal to or above the log level.
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// Log a message at the given level.
    pub fn log<W: Write>(&self, info: &LogInfo, writer: Option<&mut W>) {
        let now = SystemTime::now();
        let thread = info.thread.clone().unwrap_or_else(|| {
            let thread = std::thread::current();
            let name = thread.name();

            match name {
                Some(name) => name.to_string(),
                None => {
                    let id = thread.id();
                    // need to get the number from the output like "ThreadId(2)" -> "2"
                    let id = format!("{id:?}");
                    let id = id.split('(').collect::<Vec<&str>>()[1];
                    let id = id.split(')').collect::<Vec<&str>>()[0];
                    format!("unnamed-{}", id)
                }
            }
        });
        let location = format!("{}:{}", info.filepath, info.line_number);
        let level = info.level;
        let message = info.message.clone();
        let output = format!(
            "[{}] [{}] [{}] [{}] {}\n",
            now.format("%Y-%m-%d %H:%M:%S%.3f %Z"),
            level,
            thread,
            location,
            message
        );

        if let Some(writer) = writer {
            writer.write_all(output.as_bytes()).unwrap();
            return;
        }

        let mut file = self.file.lock().unwrap();
        file.write_all(output.as_bytes()).unwrap();
    }

    pub fn get_instance() -> Logger {
        // Check if the instance is already created.
        let current_global_instance =
            Arc::clone(INSTANCE.get_or_init(|| Arc::new(Mutex::new(None))));
        let mut current_global_instance_lock = current_global_instance.lock().unwrap();
        if current_global_instance_lock.is_none() {
            // If the instance is not created, create it.
            let logger = Logger::new();
            *current_global_instance_lock = Some(logger.clone());
            logger
        } else {
            // If the instance is already created, return it.
            current_global_instance_lock.clone().unwrap()
        }
    }
}

#[derive(Clone)]
pub struct LogInfo {
    pub level: LogLevel,
    pub message: String,
    pub filepath: &'static str,
    pub line_number: u32,
    pub thread: Option<String>,
}

#[macro_export]
macro_rules! log {
    ($level:expr, $message:expr) => {
        let message = $message.to_string();
        let logger = Logger::get_instance();
        let info = LogInfo {
            level: $level,
            message,
            filepath: file!(),
            line_number: line!(),
            thread: None,
        };
        let writer: Option<&mut Vec<u8>> = None;
        logger.log(&info, writer);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($message:expr) => {
        log!(LogLevel::Debug, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        log!(LogLevel::Debug, message);
    };
}

#[macro_export]
macro_rules! log_info {
    ($message:expr) => {
        log!(LogLevel::Info, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        log!(LogLevel::Info, message);
    };
}

#[macro_export]
macro_rules! log_warning {
    ($message:expr) => {
        log!(LogLevel::Warning, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        log!(LogLevel::Warning, message);
    };
}

#[macro_export]
macro_rules! log_error {
    ($message:expr) => {
        log!(LogLevel::Error, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        log!(LogLevel::Error, message);
    };
}

#[macro_export]
macro_rules! log_trace {
    ($message:expr) => {
        log!(LogLevel::Trace, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        log!(LogLevel::Trace, message);
    };
}

#[macro_export]
macro_rules! log_text {
    ($message:expr) => {
        log!(LogLevel::Off, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        log!(LogLevel::Off, message);
    };
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}
