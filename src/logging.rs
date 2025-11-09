//! Structured logging for VRF operations and audit trails
//!
//! This module provides comprehensive logging capabilities for production VRF deployments,
//! including audit trails, performance monitoring, and debugging support.
//!
//! # Features
//!
//! - **Structured Logging**: JSON and text formats for easy parsing
//! - **Audit Trails**: Comprehensive operation tracking for compliance
//! - **Performance Monitoring**: Duration tracking for all operations
//! - **Security Logging**: Key usage tracking without exposing sensitive data
//! - **Debug Support**: Detailed diagnostics with `vrf-debug` feature
//!
//! # Usage
//!
//! ```rust
//! use cardano_vrf::{VrfLogger, LogLevel, VrfOperation};
//!
//! let logger = VrfLogger::new(LogLevel::Info);
//!
//! // Log a successful operation
//! logger.info(VrfOperation::Prove, "VRF proof generated successfully".to_string());
//!
//! // Log an error
//! logger.error(VrfOperation::Verify, "Proof verification failed".to_string());
//!
//! // Log with key identifier (doesn't expose key material)
//! logger.info(
//!     VrfOperation::KeyGeneration,
//!     format!("Generated key for validator-001")
//! );
//! ```
//!
//! # Security Considerations
//!
//! The logging system is designed with security in mind:
//! - Never logs private key material
//! - Public keys and key IDs are logged only when necessary
//! - Timing information helps detect side-channel attacks
//! - All log entries include timestamps for audit trails
//!
//! # Output Formats
//!
//! ## Text Format (Human-Readable)
//!
//! ```text
//! [2025-11-09 10:30:45] INFO PROVE VRF proof generated successfully
//! [2025-11-09 10:30:46] ERROR VERIFY Proof verification failed
//! ```
//!
//! ## JSON Format (Machine-Readable)
//!
//! ```json
//! {
//!   "timestamp": "2025-11-09T10:30:45Z",
//!   "level": "INFO",
//!   "operation": "PROVE",
//!   "message": "VRF proof generated successfully",
//!   "success": true
//! }
//! ```

use std::fmt;

/// Log level for VRF operations
///
/// Defines the severity levels for log messages, allowing filtering
/// of log output based on importance and detail level.
///
/// # Order
///
/// Log levels are ordered from most verbose (Debug) to least verbose (Error):
/// `Debug < Info < Warning < Error`
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::LogLevel;
///
/// let level = LogLevel::Info;
/// assert!(level > LogLevel::Debug);
/// assert!(level < LogLevel::Error);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Debug-level logging for detailed diagnostics
    ///
    /// Use for development and troubleshooting. Includes detailed
    /// information about internal operations and state.
    Debug,

    /// Informational messages about normal operations
    ///
    /// Standard operational messages indicating successful completion
    /// of routine tasks. Suitable for production monitoring.
    Info,

    /// Warning messages about potential issues
    ///
    /// Non-critical issues that don't prevent operation but may
    /// require attention or indicate suboptimal conditions.
    Warning,

    /// Error messages for operation failures
    ///
    /// Critical failures that prevent successful operation completion.
    /// Requires immediate attention in production environments.
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

/// VRF operation types for logging and categorization
///
/// Categorizes different types of VRF operations for structured logging,
/// metrics collection, and audit trails. Each operation type represents
/// a distinct cryptographic or key management activity.
///
/// # Usage
///
/// Operation types are used with logging and metrics to track:
/// - Operation frequency and patterns
/// - Performance characteristics per operation
/// - Security-relevant events
/// - Failure modes and error rates
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::{VrfOperation, VrfLogger, LogLevel};
///
/// let logger = VrfLogger::new(LogLevel::Info);
///
/// // Log different operation types
/// logger.info(VrfOperation::Prove, "Generated proof".to_string());
/// logger.info(VrfOperation::Verify, "Verified proof".to_string());
/// logger.info(VrfOperation::KeyGeneration, "Created new key".to_string());
/// ```
#[derive(Debug, Clone, Copy)]
pub enum VrfOperation {
    /// VRF proof generation operation
    ///
    /// Logged when generating a VRF proof from a secret key and message.
    /// Performance-critical operation in block production scenarios.
    Prove,

    /// VRF proof verification operation
    ///
    /// Logged when verifying a VRF proof against a public key and message.
    /// Common operation during blockchain validation.
    Verify,

    /// VRF keypair generation operation
    ///
    /// Logged when creating new VRF keypairs. Security-sensitive operation
    /// that should be audited in production environments.
    KeyGeneration,

    /// VRF key retrieval from storage operation
    ///
    /// Logged when loading existing keys from storage or HSM.
    /// Useful for tracking key access patterns.
    KeyRetrieval,

    /// Hardware Security Module (HSM) operation
    ///
    /// Logged for HSM-specific operations including key management,
    /// signing, and device communication.
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

/// Structured log entry containing operation details and metadata
///
/// Represents a single log event with comprehensive metadata including
/// timestamp, severity level, operation type, and optional contextual
/// information like key identifiers and operation duration.
///
/// # Fields
///
/// - `timestamp`: When the event occurred (UTC)
/// - `level`: Severity level of the log entry
/// - `operation`: Type of VRF operation being logged
/// - `message`: Human-readable description of the event
/// - `key_id`: Optional identifier for the key involved (if applicable)
/// - `duration_us`: Optional operation duration in microseconds
/// - `success`: Optional boolean indicating operation outcome
///
/// # Serialization
///
/// Log entries can be serialized to multiple formats:
/// - Text format: Human-readable for console output
/// - JSON format: Machine-readable for log aggregation systems
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::{LogEntry, LogLevel, VrfOperation};
///
/// let entry = LogEntry::new(
///     LogLevel::Info,
///     VrfOperation::Prove,
///     "Generated VRF proof".to_string()
/// );
///
/// // Add contextual information
/// let entry = entry
///     .with_key_id("validator-001".to_string())
///     .with_success(true);
///
/// // Export as JSON for log aggregation
/// println!("{}", entry.to_json());
/// ```
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
    /// Creates a new log entry with the current timestamp
    ///
    /// Initializes a log entry with the specified severity level, operation type,
    /// and message. The timestamp is automatically set to the current system time.
    /// Optional metadata can be added using builder methods.
    ///
    /// # Arguments
    ///
    /// * `level` - Log severity level (Debug, Info, Warning, Error)
    /// * `operation` - VRF operation being logged
    /// * `message` - Descriptive message about the event
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{LogEntry, LogLevel, VrfOperation};
    ///
    /// let entry = LogEntry::new(
    ///     LogLevel::Info,
    ///     VrfOperation::Prove,
    ///     "Generated VRF proof successfully".to_string()
    /// );
    /// ```
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

    /// Adds a key identifier to the log entry
    ///
    /// Associates a key ID with this log entry for tracking operations
    /// on specific VRF keys. Useful for auditing and debugging key-specific
    /// issues in multi-key deployments.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier for the VRF key involved in this operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{LogEntry, LogLevel, VrfOperation};
    ///
    /// let entry = LogEntry::new(
    ///     LogLevel::Info,
    ///     VrfOperation::Prove,
    ///     "Proof generated".to_string()
    /// ).with_key_id("validator-key-01".to_string());
    /// ```
    pub fn with_key_id(mut self, key_id: String) -> Self {
        self.key_id = Some(key_id);
        self
    }

    /// Adds operation duration to the log entry
    ///
    /// Records how long the operation took to complete, stored in microseconds.
    /// Essential for performance monitoring and identifying bottlenecks in
    /// production VRF operations.
    ///
    /// # Arguments
    ///
    /// * `duration` - Time taken to complete the operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{LogEntry, LogLevel, VrfOperation};
    /// use std::time::{Duration, Instant};
    ///
    /// let start = Instant::now();
    /// // ... perform VRF operation ...
    /// let elapsed = start.elapsed();
    ///
    /// let entry = LogEntry::new(
    ///     LogLevel::Info,
    ///     VrfOperation::Prove,
    ///     "Proof completed".to_string()
    /// ).with_duration(elapsed);
    /// ```
    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration_us = Some(duration.as_micros() as u64);
        self
    }

    /// Adds success/failure status to the log entry
    ///
    /// Marks the operation as successful or failed. Critical for monitoring
    /// error rates and identifying problematic operations in production.
    ///
    /// # Arguments
    ///
    /// * `success` - `true` if operation completed successfully, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{LogEntry, LogLevel, VrfOperation};
    ///
    /// // Successful operation
    /// let success_entry = LogEntry::new(
    ///     LogLevel::Info,
    ///     VrfOperation::Verify,
    ///     "Proof verified".to_string()
    /// ).with_success(true);
    ///
    /// // Failed operation
    /// let failure_entry = LogEntry::new(
    ///     LogLevel::Error,
    ///     VrfOperation::Verify,
    ///     "Invalid proof".to_string()
    /// ).with_success(false);
    /// ```
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }

    /// Formats the log entry as JSON for structured logging
    ///
    /// Serializes the log entry to JSON format, suitable for ingestion by
    /// log aggregation systems like ELK, Splunk, or CloudWatch. All fields
    /// are included with appropriate types (numbers, strings, booleans).
    ///
    /// # Format
    ///
    /// ```json
    /// {
    ///   "timestamp": 1704067200,
    ///   "level": "INFO",
    ///   "operation": "PROVE",
    ///   "message": "Generated proof",
    ///   "key_id": "validator-01",      // optional
    ///   "duration_us": 1234,           // optional
    ///   "success": true                // optional
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{LogEntry, LogLevel, VrfOperation};
    ///
    /// let entry = LogEntry::new(
    ///     LogLevel::Info,
    ///     VrfOperation::Prove,
    ///     "Generated proof".to_string()
    /// ).with_key_id("key-01".to_string())
    ///  .with_success(true);
    ///
    /// // Send to log aggregator
    /// println!("{}", entry.to_json());
    /// ```
    pub fn to_json(&self) -> String {
        let timestamp = self
            .timestamp
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

    /// Formats the log entry as human-readable text
    ///
    /// Generates a single-line text representation suitable for console output
    /// or traditional log files. Includes timestamp, level, operation, message,
    /// and optional metadata in a concise format.
    ///
    /// # Format
    ///
    /// ```text
    /// [timestamp] LEVEL OPERATION - message | key=key_id | duration=123μs | success=true
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{LogEntry, LogLevel, VrfOperation};
    /// use std::time::Duration;
    ///
    /// let entry = LogEntry::new(
    ///     LogLevel::Info,
    ///     VrfOperation::Verify,
    ///     "Proof verified successfully".to_string()
    /// ).with_key_id("validator-01".to_string())
    ///  .with_duration(Duration::from_micros(850))
    ///  .with_success(true);
    ///
    /// // Output: [1704067200] INFO VERIFY - Proof verified successfully | key=validator-01 | duration=850μs | success=true
    /// println!("{}", entry.to_text());
    /// ```
    pub fn to_text(&self) -> String {
        let timestamp = self
            .timestamp
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

/// Lightweight logger for VRF operations with configurable verbosity
///
/// Provides structured logging capabilities for VRF operations with
/// support for multiple severity levels and filtering. In production,
/// integrate with established logging frameworks like `tracing`, `slog`,
/// or `log4rs` for advanced features like log rotation and forwarding.
///
/// # Filtering
///
/// Only log entries at or above the configured minimum level are emitted.
/// This allows runtime control of logging verbosity without code changes.
///
/// # Output
///
/// By default, logs are emitted to stderr in JSON format. This can be
/// redirected to files or piped to log aggregation systems.
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::{VrfLogger, LogLevel, VrfOperation};
///
/// // Create logger with Info level (filters out Debug)
/// let logger = VrfLogger::new(LogLevel::Info);
///
/// // This will be logged
/// logger.info(VrfOperation::Prove, "Proof generated".to_string());
///
/// // This will be filtered out
/// logger.debug(VrfOperation::Prove, "Internal state: ...".to_string());
/// ```
///
/// # Production Integration
///
/// ```rust,ignore
/// use cardano_vrf::{VrfLogger, LogLevel};
///
/// // Set level from environment variable
/// let level = std::env::var("LOG_LEVEL")
///     .unwrap_or_else(|_| "INFO".to_string());
/// let log_level = match level.as_str() {
///     "DEBUG" => LogLevel::Debug,
///     "WARNING" => LogLevel::Warning,
///     "ERROR" => LogLevel::Error,
///     _ => LogLevel::Info,
/// };
///
/// let logger = VrfLogger::new(log_level);
/// ```
pub struct VrfLogger {
    min_level: LogLevel,
}

impl VrfLogger {
    /// Creates a new logger with the specified minimum log level
    ///
    /// Only log entries at or above this level will be emitted. This allows
    /// runtime configuration of logging verbosity without recompilation.
    ///
    /// # Arguments
    ///
    /// * `min_level` - Minimum severity level to output (Debug < Info < Warning < Error)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{VrfLogger, LogLevel};
    ///
    /// // Production logger (Info and above)
    /// let prod_logger = VrfLogger::new(LogLevel::Info);
    ///
    /// // Development logger (all levels including Debug)
    /// let dev_logger = VrfLogger::new(LogLevel::Debug);
    ///
    /// // Error-only logger for critical systems
    /// let critical_logger = VrfLogger::new(LogLevel::Error);
    /// ```
    pub fn new(min_level: LogLevel) -> Self {
        Self { min_level }
    }

    /// Logs an entry if it meets the minimum level threshold
    ///
    /// Filters log entries based on the configured minimum level. Entries
    /// below the threshold are silently discarded. This method is typically
    /// called by the convenience methods (`debug`, `info`, etc.) rather than
    /// directly.
    ///
    /// # Output Format
    ///
    /// Logs are emitted to stderr in JSON format for easy parsing by log
    /// aggregation tools. In production, redirect to a logging framework.
    ///
    /// # Arguments
    ///
    /// * `entry` - The log entry to potentially emit
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{VrfLogger, LogEntry, LogLevel, VrfOperation};
    ///
    /// let logger = VrfLogger::new(LogLevel::Info);
    ///
    /// let entry = LogEntry::new(
    ///     LogLevel::Info,
    ///     VrfOperation::Prove,
    ///     "Custom log entry".to_string()
    /// ).with_key_id("key-123".to_string());
    ///
    /// logger.log(entry);
    /// ```
    pub fn log(&self, entry: LogEntry) {
        if entry.level >= self.min_level {
            // In production, this would go to a proper logging framework
            // (e.g., tracing, slog, log4rs)
            eprintln!("{}", entry.to_json());
        }
    }

    /// Logs a debug-level message
    ///
    /// Debug messages provide detailed diagnostic information useful during
    /// development and troubleshooting. These are typically disabled in
    /// production environments to reduce log volume.
    ///
    /// Use for: Internal state dumps, detailed execution flow, development aids
    ///
    /// # Arguments
    ///
    /// * `operation` - The VRF operation being performed
    /// * `message` - Descriptive debug message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{VrfLogger, LogLevel, VrfOperation};
    ///
    /// let logger = VrfLogger::new(LogLevel::Debug);
    /// logger.debug(
    ///     VrfOperation::Prove,
    ///     "Processing message hash: 0x1234...".to_string()
    /// );
    /// ```
    pub fn debug(&self, operation: VrfOperation, message: String) {
        self.log(LogEntry::new(LogLevel::Debug, operation, message));
    }

    /// Logs an info-level message
    ///
    /// Info messages record normal operational events that indicate the system
    /// is functioning correctly. This is the default logging level for production.
    ///
    /// Use for: Successful operations, normal state transitions, audit trails
    ///
    /// # Arguments
    ///
    /// * `operation` - The VRF operation being performed
    /// * `message` - Descriptive informational message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{VrfLogger, LogLevel, VrfOperation};
    ///
    /// let logger = VrfLogger::new(LogLevel::Info);
    /// logger.info(
    ///     VrfOperation::Prove,
    ///     "VRF proof generated successfully".to_string()
    /// );
    /// logger.info(
    ///     VrfOperation::Verify,
    ///     "Proof verification succeeded".to_string()
    /// );
    /// ```
    pub fn info(&self, operation: VrfOperation, message: String) {
        self.log(LogEntry::new(LogLevel::Info, operation, message));
    }

    /// Logs a warning-level message
    ///
    /// Warnings indicate potential issues that don't prevent operation but may
    /// require attention or could lead to problems if not addressed. The system
    /// continues to function but operation may be degraded.
    ///
    /// Use for: Retries, degraded performance, unusual but handled conditions,
    /// deprecated feature usage
    ///
    /// # Arguments
    ///
    /// * `operation` - The VRF operation being performed
    /// * `message` - Description of the warning condition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{VrfLogger, LogLevel, VrfOperation};
    ///
    /// let logger = VrfLogger::new(LogLevel::Info);
    /// logger.warning(
    ///     VrfOperation::HsmOperation,
    ///     "HSM connection slow, operation took 5s".to_string()
    /// );
    /// logger.warning(
    ///     VrfOperation::KeyRetrieval,
    ///     "Key cache miss, loading from HSM".to_string()
    /// );
    /// ```
    pub fn warning(&self, operation: VrfOperation, message: String) {
        self.log(LogEntry::new(LogLevel::Warning, operation, message));
    }

    /// Logs an error-level message
    ///
    /// Errors indicate operation failures that prevent the requested action from
    /// completing successfully. These require immediate attention and should trigger
    /// alerts in production monitoring systems.
    ///
    /// Use for: Failed operations, invalid inputs, cryptographic failures, HSM errors,
    /// unrecoverable conditions
    ///
    /// # Arguments
    ///
    /// * `operation` - The VRF operation that failed
    /// * `message` - Detailed error description
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::{VrfLogger, LogLevel, VrfOperation};
    ///
    /// let logger = VrfLogger::new(LogLevel::Info);
    /// logger.error(
    ///     VrfOperation::Verify,
    ///     "Invalid VRF proof: verification failed".to_string()
    /// );
    /// logger.error(
    ///     VrfOperation::HsmOperation,
    ///     "HSM connection failed: timeout after 30s".to_string()
    /// );
    /// ```
    pub fn error(&self, operation: VrfOperation, message: String) {
        self.log(LogEntry::new(LogLevel::Error, operation, message));
    }
}

impl Default for VrfLogger {
    /// Creates a logger with default configuration (Info level)
    ///
    /// The default logger emits Info, Warning, and Error messages while
    /// filtering out Debug messages. This is suitable for production use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::VrfLogger;
    ///
    /// let logger = VrfLogger::default();
    /// // Equivalent to: VrfLogger::new(LogLevel::Info)
    /// ```
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
