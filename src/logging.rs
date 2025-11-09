//! Structured logging for VRF operations
//!
//! Provides audit trail and debugging capabilities

use std::fmt;

/// Log level for VRF operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Debug-level logging for detailed diagnostics
    Debug,
    /// Informational messages about normal operations
    Info,
    /// Warning messages about potential issues
    Warning,
    /// Error messages for operation failures
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// VRF operation types for logging
#[derive(Debug, Clone, Copy)]
pub enum VrfOperation {
    /// VRF proof generation
    Prove,
    /// VRF proof verification
    Verify,
    /// VRF keypair generation
    KeyGeneration,
    /// VRF key retrieval from storage
    KeyRetrieval,
    /// HSM-related operations
    HsmOperation,
}

impl fmt::Display for VrfOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VrfOperation::Prove => write!(f, "PROVE"),
            VrfOperation::Verify => write!(f, "VERIFY"),
            VrfOperation::KeyGeneration => write!(f, "KEYGEN"),
            VrfOperation::KeyRetrieval => write!(f, "KEYGET"),
            VrfOperation::HsmOperation => write!(f, "HSM"),
        }
    }
}

/// Structured log entry
#[derive(Debug)]
pub struct LogEntry {
    timestamp: std::time::SystemTime,
    level: LogLevel,
    operation: VrfOperation,
    message: String,
    key_id: Option<String>,
    duration_us: Option<u64>,
    success: Option<bool>,
}

impl LogEntry {
    /// Creates a new log entry
    ///
    /// # Arguments
    ///
    /// * `level` - Log severity level
    /// * `operation` - VRF operation being logged
    /// * `message` - Descriptive message
    pub fn new(level: LogLevel, operation: VrfOperation, message: String) -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            level,
            operation,
            message,
            key_id: None,
            duration_us: None,
            success: None,
        }
    }

    /// Adds key identifier to log entry
    pub fn with_key_id(mut self, key_id: String) -> Self {
        self.key_id = Some(key_id);
        self
    }

    /// Adds operation duration to log entry
    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration_us = Some(duration.as_micros() as u64);
        self
    }

    /// Adds success/failure status to log entry
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }

    /// Format as JSON for structured logging
    pub fn to_json(&self) -> String {
        let timestamp = self.timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut json = format!(
            "{{\"timestamp\":{},\"level\":\"{}\",\"operation\":\"{}\",\"message\":\"{}\"",
            timestamp, self.level, self.operation, self.message
        );

        if let Some(ref key_id) = self.key_id {
            json.push_str(&format!(",\"key_id\":\"{}\"", key_id));
        }

        if let Some(duration) = self.duration_us {
            json.push_str(&format!(",\"duration_us\":{}", duration));
        }

        if let Some(success) = self.success {
            json.push_str(&format!(",\"success\":{}", success));
        }

        json.push('}');
        json
    }

    /// Format as human-readable text
    pub fn to_text(&self) -> String {
        let timestamp = self.timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut text = format!(
            "[{}] {} {} - {}",
            timestamp, self.level, self.operation, self.message
        );

        if let Some(ref key_id) = self.key_id {
            text.push_str(&format!(" | key={}", key_id));
        }

        if let Some(duration) = self.duration_us {
            text.push_str(&format!(" | duration={}μs", duration));
        }

        if let Some(success) = self.success {
            text.push_str(&format!(" | success={}", success));
        }

        text
    }
}

/// Logger for VRF operations
pub struct VrfLogger {
    min_level: LogLevel,
}

impl VrfLogger {
    /// Creates a new logger with specified minimum level
    ///
    /// # Arguments
    ///
    /// * `min_level` - Minimum log level to output
    pub fn new(min_level: LogLevel) -> Self {
        Self { min_level }
    }

    /// Logs an entry if it meets the minimum level threshold
    pub fn log(&self, entry: LogEntry) {
        if entry.level >= self.min_level {
            // In production, this would go to a proper logging framework
            // (e.g., tracing, slog, log4rs)
            eprintln!("{}", entry.to_json());
        }
    }

    /// Log a debug-level message
    ///
    /// Debug messages are typically used for detailed diagnostic information
    /// useful during development and troubleshooting.
    ///
    /// # Arguments
    ///
    /// * `operation` - The VRF operation being performed
    /// * `message` - Descriptive message
    pub fn debug(&self, operation: VrfOperation, message: String) {
        self.log(LogEntry::new(LogLevel::Debug, operation, message));
    }

    /// Log an info-level message
    ///
    /// Info messages record normal operational events such as successful
    /// proof generation or verification.
    ///
    /// # Arguments
    ///
    /// * `operation` - The VRF operation being performed
    /// * `message` - Descriptive message
    pub fn info(&self, operation: VrfOperation, message: String) {
        self.log(LogEntry::new(LogLevel::Info, operation, message));
    }

    /// Log a warning-level message
    ///
    /// Warnings indicate potential issues that don't prevent operation
    /// but may require attention (e.g., degraded performance, retries).
    ///
    /// # Arguments
    ///
    /// * `operation` - The VRF operation being performed
    /// * `message` - Descriptive message
    pub fn warning(&self, operation: VrfOperation, message: String) {
        self.log(LogEntry::new(LogLevel::Warning, operation, message));
    }

    /// Log an error-level message
    ///
    /// Errors indicate operation failures that require immediate attention.
    /// These should be monitored and alerted on in production systems.
    ///
    /// # Arguments
    ///
    /// * `operation` - The VRF operation that failed
    /// * `message` - Error description
    pub fn error(&self, operation: VrfOperation, message: String) {
        self.log(LogEntry::new(LogLevel::Error, operation, message));
    }
}

impl Default for VrfLogger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_log_entry_json() {
        let entry = LogEntry::new(
            LogLevel::Info,
            VrfOperation::Prove,
            "Test message".to_string(),
        )
        .with_key_id("test_key".to_string())
        .with_duration(Duration::from_micros(1234))
        .with_success(true);

        let json = entry.to_json();
        assert!(json.contains("\"level\":\"INFO\""));
        assert!(json.contains("\"operation\":\"PROVE\""));
        assert!(json.contains("\"message\":\"Test message\""));
        assert!(json.contains("\"key_id\":\"test_key\""));
        assert!(json.contains("\"duration_us\":1234"));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_log_entry_text() {
        let entry = LogEntry::new(
            LogLevel::Error,
            VrfOperation::Verify,
            "Verification failed".to_string(),
        )
        .with_success(false);

        let text = entry.to_text();
        assert!(text.contains("ERROR"));
        assert!(text.contains("VERIFY"));
        assert!(text.contains("Verification failed"));
        assert!(text.contains("success=false"));
    }
}
