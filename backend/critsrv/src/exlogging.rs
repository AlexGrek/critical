use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use chrono::{Utc};
use log;

// Global static instance of the logger
static GLOBAL_LOGGER: OnceCell<Arc<AsyncLogger>> = OnceCell::const_new();

#[derive(Debug, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub log_file_path: String,
}

#[derive(Debug)]
struct AsyncLogger {
    file_writer: Arc<Mutex<tokio::fs::File>>,
    file_path: String,
}

impl AsyncLogger {
    async fn new(config: LoggerConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_file_path)
            .await?;

        Ok(AsyncLogger {
            file_writer: Arc::new(Mutex::new(file)),
            file_path: config.log_file_path,
        })
    }

    async fn write_log(&self, level: LogLevel, message: &str, user: Option<&str>) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        
        let log_entry = match user {
            Some(u) => format!("[{}] [{}] [User: {}] {}\n", timestamp, level.as_str(), u, message),
            None => format!("[{}] [{}] {}\n", timestamp, level.as_str(), message),
        };

        // Attempt to write to file
        if let Ok(mut file) = self.file_writer.try_lock() {
            if let Err(e) = file.write_all(log_entry.as_bytes()).await {
                eprintln!("Failed to write to log file: {}", e);
            }
            if let Err(e) = file.flush().await {
                eprintln!("Failed to flush log file: {}", e);
            }
        } else {
            // If we can't acquire the lock immediately, spawn a task to try later
            let file_writer = Arc::clone(&self.file_writer);
            tokio::spawn(async move {
                let mut file = file_writer.lock().await;
                if let Err(e) = file.write_all(log_entry.as_bytes()).await {
                    eprintln!("Failed to write to log file: {}", e);
                }
                if let Err(e) = file.flush().await {
                    eprintln!("Failed to flush log file: {}", e);
                }
            });
        }
    }
}

/// Initialize the global logger with configuration
pub async fn configure_log_event(config: LoggerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let event_text = format!("Logger initialized with config: {:?}", &config);
    let logger = AsyncLogger::new(config).await?;
    GLOBAL_LOGGER.set(Arc::new(logger))
        .map_err(|_| "Logger already initialized")?;
    log_event(LogLevel::Info, &event_text, Some("root"));
    Ok(())
}

/// Log an event globally without awaiting the result
pub fn log_event(level: LogLevel, message: impl AsRef<str>, user: Option<impl AsRef<str>>) {
    // First, log using the standard log crate
    let msg = message.as_ref();
    match level {
        LogLevel::Error => log::error!("{}", msg),
        LogLevel::Warn => log::warn!("{}", msg),
        LogLevel::Info => log::info!("{}", msg),
        LogLevel::Debug => log::debug!("{}", msg),
        LogLevel::Trace => log::trace!("{}", msg),
    }

    // Then write to file asynchronously without blocking
    if let Some(logger) = GLOBAL_LOGGER.get() {
        let logger = Arc::clone(logger);
        let message = message.as_ref().to_string();
        let user_str = user.map(|u| u.as_ref().to_string());
        
        tokio::spawn(async move {
            logger.write_log(level, &message, user_str.as_deref()).await;
        });
    } else {
        log::error!("Logger not initialized. Call configure_log_event() first.");
    }
}

// Convenience macros for easier usage (optional)
#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        log_event(LogLevel::Error, $msg, None::<&str>)
    };
    ($msg:expr, $user:expr) => {
        log_event(LogLevel::Error, $msg, Some($user))
    };
}

#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        log_event(LogLevel::Info, $msg, None::<&str>)
    };
    ($msg:expr, $user:expr) => {
        log_event(LogLevel::Info, $msg, Some($user))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_logger_initialization_and_usage() {
        // Initialize logger
        let config = LoggerConfig {
            log_file_path: "test.log".to_string(),
        };
        
        configure_log_event(config).await.unwrap();

        // Test logging
        log_event(LogLevel::Info, "Test message", Some("test_user"));
        log_event(LogLevel::Error, "Error message", None::<&str>);
        log_event(LogLevel::Warn, "Warning message", Some("warn_user"));
        
        // Give some time for async operations to complete
        sleep(Duration::from_millis(200)).await;
        
        // Test macros
        log_info!("Info via macro");
        log_error!("Error via macro", "macro_user");
        
        sleep(Duration::from_millis(200)).await;
        
        // Test reading latest logs
        match get_latest_logs(3).await {
            Ok(logs) => {
                println!("Latest 3 logs:");
                for (i, log) in logs.iter().enumerate() {
                    println!("{}: {}", i + 1, log);
                }
            }
            Err(e) => eprintln!("Failed to read logs: {}", e),
        }
        
        // Test filtered logs
        match get_latest_logs_filtered(10, Some(LogLevel::Warn)).await {
            Ok(logs) => {
                println!("Latest logs (Warn and above):");
                for log in logs {
                    println!("{}", log);
                }
            }
            Err(e) => eprintln!("Failed to read filtered logs: {}", e),
        }
    }
}

/// Read the n latest log statements from the log file
/// Returns lines in chronological order (oldest first among the n latest)
pub async fn get_latest_logs(n: usize) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let logger = GLOBAL_LOGGER.get()
        .ok_or("Logger not initialized. Call configure_log_event() first.")?;
    
    let file_path = &logger.file_path;
    
    if n == 0 {
        return Ok(Vec::new());
    }
    
    let mut file = tokio::fs::File::open(file_path).await?;
    let file_size = file.metadata().await?.len();
    
    if file_size == 0 {
        return Ok(Vec::new());
    }
    
    let mut lines = Vec::new();
    let mut buffer = Vec::new();
    let chunk_size = 8192; // 8KB chunks
    let mut position = file_size;
    let mut current_line = Vec::new();
    
    // Read file backwards in chunks
    while position > 0 && lines.len() < n {
        let read_size = std::cmp::min(chunk_size, position);
        position -= read_size;
        
        // Read chunk
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        file.seek(std::io::SeekFrom::Start(position)).await?;
        buffer.resize(read_size as usize, 0);
        file.read_exact(&mut buffer).await?;
        
        // Process chunk backwards
        for &byte in buffer.iter().rev() {
            if byte == b'\n' {
                if !current_line.is_empty() {
                    // Reverse the line since we built it backwards
                    current_line.reverse();
                    if let Ok(line) = String::from_utf8(current_line.clone()) {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            lines.push(trimmed.to_string());
                            if lines.len() >= n {
                                break;
                            }
                        }
                    }
                    current_line.clear();
                }
            } else {
                current_line.push(byte);
            }
        }
        
        if lines.len() >= n {
            break;
        }
    }
    
    // Handle the last line if we reached the beginning of file
    if position == 0 && !current_line.is_empty() && lines.len() < n {
        current_line.reverse();
        if let Ok(line) = String::from_utf8(current_line) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }
    }
    
    // Reverse to get chronological order (oldest first among the n latest)
    lines.reverse();
    
    Ok(lines)
}

/// Get latest logs with filtering by log level
pub async fn get_latest_logs_filtered(
    n: usize, 
    min_level: Option<LogLevel>
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let all_lines = get_latest_logs(n * 2).await?; // Get more lines to account for filtering
    
    let mut filtered_lines = Vec::new();
    
    for line in all_lines {
        if let Some(ref level) = min_level {
            // Simple level filtering based on log format
            let should_include = match level {
                LogLevel::Error => line.contains("[ERROR]"),
                LogLevel::Warn => line.contains("[ERROR]") || line.contains("[WARN]"),
                LogLevel::Info => line.contains("[ERROR]") || line.contains("[WARN]") || line.contains("[INFO]"),
                LogLevel::Debug => line.contains("[ERROR]") || line.contains("[WARN]") || line.contains("[INFO]") || line.contains("[DEBUG]"),
                LogLevel::Trace => true, // Include all levels
            };
            
            if should_include {
                filtered_lines.push(line);
                if filtered_lines.len() >= n {
                    break;
                }
            }
        } else {
            filtered_lines.push(line);
            if filtered_lines.len() >= n {
                break;
            }
        }
    }
    
    Ok(filtered_lines)
}
