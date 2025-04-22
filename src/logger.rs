use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

pub struct Logger {
    log_file: Mutex<Option<std::fs::File>>,
}

impl Logger {
    pub fn new(log_file_path: &str) -> Self {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .ok();

        Logger {
            log_file: Mutex::new(log_file),
        }
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    fn log(&self, level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_message = format!("[{}] [{}] {}\n", timestamp, level, message);

        if let Some(file) = self.log_file.lock().unwrap().as_mut() {
            let _ = file.write_all(log_message.as_bytes());
        } else {
            // Fallback to stdout if file logging fails
            println!("{}", log_message);
        }
    }
}
