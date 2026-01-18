
#[macro_export]
macro_rules! logger {
    ($level:expr, $($arg:tt)*) => {{
        let ts = chrono::Local::now().format("%H:%M:%S");

        let (color, level_str) = match $level {
            LogLevel::Info  => ("\x1b[34m", "INFO"),
            LogLevel::Warn  => ("\x1b[33m", "WARN"),
            LogLevel::Error => ("\x1b[31m", "ERROR"),
        };
        let reset = "\x1b[0m";

        println!("{}[{}][{}] {}{}", color, level_str, ts, format!($($arg)*), reset);
    }};
}



#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error
}