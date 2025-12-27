use chrono::Local;

#[macro_export]
macro_rules! logger {
    ($level:expr, $($arg:tt)*) => {{
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{}][{:?}] {}", ts, $level, format!($($arg)*));
    }};
}

#[allow(dead_code)]
pub fn _init() {
    let _ = Local::now();
}


#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error
}