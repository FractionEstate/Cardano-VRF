//! Production monitoring and metrics for VRF operations
//!
//! This module provides Prometheus-compatible metrics collection for tracking
//! VRF operation performance, success rates, and error conditions. Metrics are
//! designed for integration with modern observability stacks including Prometheus,
//! Grafana, and cloud monitoring services.
//!
//! # Supported Metrics
//!
//! ## Prove Operations
//! - `vrf_prove_total`: Total proof generation attempts
//! - `vrf_prove_success`: Successful proof generations
//! - `vrf_prove_failure`: Failed proof generations
//! - `vrf_prove_duration_us`: Average proof generation time (microseconds)
//!
//! ## Verify Operations
//! - `vrf_verify_total`: Total verification attempts
//! - `vrf_verify_success`: Successful verifications
//! - `vrf_verify_failure`: Failed verifications
//! - `vrf_verify_duration_us`: Average verification time (microseconds)
//!
//! ## HSM Operations
//! - `vrf_hsm_operations`: Total HSM interactions
//! - `vrf_hsm_errors`: HSM operation failures
//!
//! # Performance
//!
//! All metrics use lock-free atomic operations for minimal overhead. The metrics
//! collector is thread-safe and can be safely shared across multiple threads
//! using `Arc<VrfMetrics>` or by cloning (which uses internal `Arc`s).
//!
//! # Usage
//!
//! ```rust
//! use cardano_vrf::VrfMetrics;
//! use std::time::Instant;
//!
//! let metrics = VrfMetrics::new();
//!
//! // Track a prove operation
//! let start = Instant::now();
//! // ... perform VRF proof generation ...
//! let success = true; // or false on error
//! metrics.record_prove(start.elapsed(), success);
//!
//! // Export metrics in Prometheus format
//! println!("{}", metrics.prometheus_export());
//! ```
//!
//! # Integration with Prometheus
//!
//! The `prometheus_export()` method returns metrics in the standard Prometheus
//! text exposition format. Serve this via an HTTP endpoint for scraping:
//!
//! ```rust,ignore
//! use actix_web::{web, App, HttpServer, Responder};
//! use cardano_vrf::VrfMetrics;
//! use std::sync::Arc;
//!
//! async fn metrics_handler(metrics: web::Data<Arc<VrfMetrics>>) -> impl Responder {
//!     metrics.prometheus_export()
//! }
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let metrics = Arc::new(VrfMetrics::new());
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(web::Data::new(metrics.clone()))
//!             .route("/metrics", web::get().to(metrics_handler))
//!     })
//!     .bind("127.0.0.1:9090")?
//!     .run()
//!     .await
//! }
//! ```
//!
//! # Security Considerations
//!
//! - Metrics may reveal operation patterns and timing information
//! - Ensure metrics endpoints are not publicly accessible
//! - Consider rate limiting and authentication for metrics endpoints
//! - Monitor for unusual patterns that may indicate attacks

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Thread-safe metrics collector for VRF operations
///
/// Collects performance and reliability metrics for VRF proof generation,
/// verification, and HSM operations. All operations use lock-free atomics
/// for minimal performance impact.
///
/// # Thread Safety
///
/// This type is `Clone` and all clones share the same underlying metrics.
/// It can be safely shared across threads either by cloning or wrapping in `Arc`.
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::VrfMetrics;
/// use std::time::{Duration, Instant};
///
/// let metrics = VrfMetrics::new();
///
/// // Record successful proof generation (1.2ms)
/// metrics.record_prove(Duration::from_micros(1200), true);
///
/// // Record failed verification (800μs)
/// metrics.record_verify(Duration::from_micros(800), false);
///
/// // Get current statistics
/// let stats = metrics.summary();
/// println!("Prove success rate: {:.2}%", stats.prove_success_rate());
/// ```
#[derive(Clone)]
pub struct VrfMetrics {
    prove_total: Arc<AtomicU64>,
    prove_success: Arc<AtomicU64>,
    prove_failure: Arc<AtomicU64>,
    prove_duration_us: Arc<AtomicU64>,

    verify_total: Arc<AtomicU64>,
    verify_success: Arc<AtomicU64>,
    verify_failure: Arc<AtomicU64>,
    verify_duration_us: Arc<AtomicU64>,

    hsm_operations: Arc<AtomicU64>,
    hsm_errors: Arc<AtomicU64>,
}

impl Default for VrfMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl VrfMetrics {
    /// Creates a new metrics collector with all counters initialized to zero
    ///
    /// All metrics start at zero and increment as operations are recorded.
    /// The collector is immediately ready for use and thread-safe.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::VrfMetrics;
    ///
    /// let metrics = VrfMetrics::new();
    /// // Ready to track operations
    /// ```
    pub fn new() -> Self {
        Self {
            prove_total: Arc::new(AtomicU64::new(0)),
            prove_success: Arc::new(AtomicU64::new(0)),
            prove_failure: Arc::new(AtomicU64::new(0)),
            prove_duration_us: Arc::new(AtomicU64::new(0)),

            verify_total: Arc::new(AtomicU64::new(0)),
            verify_success: Arc::new(AtomicU64::new(0)),
            verify_failure: Arc::new(AtomicU64::new(0)),
            verify_duration_us: Arc::new(AtomicU64::new(0)),

            hsm_operations: Arc::new(AtomicU64::new(0)),
            hsm_errors: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Records a VRF proof generation operation
    ///
    /// Updates prove-specific metrics including total attempts, success/failure counts,
    /// and cumulative duration. Call this immediately after each proof generation
    /// attempt regardless of outcome.
    ///
    /// # Arguments
    ///
    /// * `duration` - Time taken to generate the proof
    /// * `success` - `true` if proof generation succeeded, `false` if it failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::VrfMetrics;
    /// use std::time::Instant;
    ///
    /// let metrics = VrfMetrics::new();
    ///
    /// let start = Instant::now();
    /// // ... VRF proof generation code ...
    /// let result = Ok(()); // or Err(...) on failure
    ///
    /// metrics.record_prove(start.elapsed(), result.is_ok());
    /// ```
    pub fn record_prove(&self, duration: std::time::Duration, success: bool) {
        self.prove_total.fetch_add(1, Ordering::Relaxed);
        self.prove_duration_us
            .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);

        if success {
            self.prove_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.prove_failure.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Records a VRF proof verification operation
    ///
    /// Updates verify-specific metrics including total attempts, success/failure counts,
    /// and cumulative duration. Call this immediately after each verification attempt.
    ///
    /// # Arguments
    ///
    /// * `duration` - Time taken to verify the proof
    /// * `success` - `true` if verification succeeded, `false` if proof was invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::VrfMetrics;
    /// use std::time::Instant;
    ///
    /// let metrics = VrfMetrics::new();
    ///
    /// let start = Instant::now();
    /// // ... VRF proof verification code ...
    /// let is_valid = true; // or false if proof invalid
    ///
    /// metrics.record_verify(start.elapsed(), is_valid);
    /// ```
    pub fn record_verify(&self, duration: std::time::Duration, success: bool) {
        self.verify_total.fetch_add(1, Ordering::Relaxed);
        self.verify_duration_us
            .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);

        if success {
            self.verify_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.verify_failure.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Records a Hardware Security Module (HSM) operation
    ///
    /// Tracks HSM interactions including key operations, signing requests, and
    /// device communication. Essential for monitoring HSM health and performance.
    ///
    /// # Arguments
    ///
    /// * `success` - `true` if HSM operation succeeded, `false` on error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::VrfMetrics;
    ///
    /// let metrics = VrfMetrics::new();
    ///
    /// // Successful HSM key retrieval
    /// metrics.record_hsm_operation(true);
    ///
    /// // Failed HSM connection
    /// metrics.record_hsm_operation(false);
    /// ```
    pub fn record_hsm_operation(&self, success: bool) {
        self.hsm_operations.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.hsm_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Exports metrics in Prometheus text exposition format
    ///
    /// Generates a complete Prometheus-compatible metrics export containing
    /// all VRF operation counters and gauges. This output can be served via
    /// an HTTP endpoint for Prometheus scraping.
    ///
    /// # Format
    ///
    /// Returns metrics with HELP and TYPE annotations following the
    /// [Prometheus exposition format](https://prometheus.io/docs/instrumenting/exposition_formats/).
    ///
    /// # Metrics Included
    ///
    /// - **Counters**: prove_total, prove_success, prove_failure, verify_total,
    ///   verify_success, verify_failure, hsm_operations, hsm_errors
    /// - **Gauges**: prove_duration_microseconds_avg, verify_duration_microseconds_avg
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::VrfMetrics;
    /// use std::time::Duration;
    ///
    /// let metrics = VrfMetrics::new();
    /// metrics.record_prove(Duration::from_micros(1200), true);
    /// metrics.record_verify(Duration::from_micros(800), true);
    ///
    /// // Export for Prometheus
    /// let export = metrics.prometheus_format();
    /// println!("{}", export);
    /// // Output includes:
    /// // # HELP vrf_prove_total Total VRF prove operations
    /// // # TYPE vrf_prove_total counter
    /// // vrf_prove_total 1
    /// // ...
    /// ```
    pub fn prometheus_format(&self) -> String {
        let prove_total = self.prove_total.load(Ordering::Relaxed);
        let prove_success = self.prove_success.load(Ordering::Relaxed);
        let prove_failure = self.prove_failure.load(Ordering::Relaxed);
        let prove_duration = self.prove_duration_us.load(Ordering::Relaxed);

        let verify_total = self.verify_total.load(Ordering::Relaxed);
        let verify_success = self.verify_success.load(Ordering::Relaxed);
        let verify_failure = self.verify_failure.load(Ordering::Relaxed);
        let verify_duration = self.verify_duration_us.load(Ordering::Relaxed);

        let hsm_ops = self.hsm_operations.load(Ordering::Relaxed);
        let hsm_errors = self.hsm_errors.load(Ordering::Relaxed);

        let avg_prove_us = if prove_total > 0 {
            prove_duration / prove_total
        } else {
            0
        };
        let avg_verify_us = if verify_total > 0 {
            verify_duration / verify_total
        } else {
            0
        };

        format!(
            "# HELP vrf_prove_total Total VRF prove operations\n\
             # TYPE vrf_prove_total counter\n\
             vrf_prove_total {}\n\
             \n\
             # HELP vrf_prove_success Successful VRF prove operations\n\
             # TYPE vrf_prove_success counter\n\
             vrf_prove_success {}\n\
             \n\
             # HELP vrf_prove_failure Failed VRF prove operations\n\
             # TYPE vrf_prove_failure counter\n\
             vrf_prove_failure {}\n\
             \n\
             # HELP vrf_prove_duration_microseconds_avg Average VRF prove duration\n\
             # TYPE vrf_prove_duration_microseconds_avg gauge\n\
             vrf_prove_duration_microseconds_avg {}\n\
             \n\
             # HELP vrf_verify_total Total VRF verify operations\n\
             # TYPE vrf_verify_total counter\n\
             vrf_verify_total {}\n\
             \n\
             # HELP vrf_verify_success Successful VRF verify operations\n\
             # TYPE vrf_verify_success counter\n\
             vrf_verify_success {}\n\
             \n\
             # HELP vrf_verify_failure Failed VRF verify operations\n\
             # TYPE vrf_verify_failure counter\n\
             vrf_verify_failure {}\n\
             \n\
             # HELP vrf_verify_duration_microseconds_avg Average VRF verify duration\n\
             # TYPE vrf_verify_duration_microseconds_avg gauge\n\
             vrf_verify_duration_microseconds_avg {}\n\
             \n\
             # HELP vrf_hsm_operations Total HSM operations\n\
             # TYPE vrf_hsm_operations counter\n\
             vrf_hsm_operations {}\n\
             \n\
             # HELP vrf_hsm_errors HSM operation errors\n\
             # TYPE vrf_hsm_errors counter\n\
             vrf_hsm_errors {}\n",
            prove_total,
            prove_success,
            prove_failure,
            avg_prove_us,
            verify_total,
            verify_success,
            verify_failure,
            avg_verify_us,
            hsm_ops,
            hsm_errors
        )
    }

    /// Exports metrics in compact JSON format
    ///
    /// Generates a structured JSON representation of all metrics, suitable for
    /// programmatic consumption by monitoring dashboards or custom analytics tools.
    ///
    /// # Format
    ///
    /// Returns a compact JSON object with nested structures for prove, verify,
    /// and HSM metrics. Includes both absolute counters and computed averages.
    ///
    /// # Structure
    ///
    /// ```json
    /// {
    ///   "prove": {
    ///     "total": 100,
    ///     "success": 98,
    ///     "failure": 2,
    ///     "avg_duration_us": 1200
    ///   },
    ///   "verify": {
    ///     "total": 500,
    ///     "success": 495,
    ///     "failure": 5,
    ///     "avg_duration_us": 800
    ///   },
    ///   "hsm": {
    ///     "operations": 10,
    ///     "errors": 1
    ///   }
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::VrfMetrics;
    /// use std::time::Duration;
    ///
    /// let metrics = VrfMetrics::new();
    /// metrics.record_prove(Duration::from_micros(1200), true);
    ///
    /// // Export as JSON for custom dashboard
    /// let json = metrics.json_format();
    /// println!("{}", json);
    /// ```
    pub fn json_format(&self) -> String {
        let prove_total = self.prove_total.load(Ordering::Relaxed);
        let prove_success = self.prove_success.load(Ordering::Relaxed);
        let prove_failure = self.prove_failure.load(Ordering::Relaxed);
        let prove_duration = self.prove_duration_us.load(Ordering::Relaxed);

        let verify_total = self.verify_total.load(Ordering::Relaxed);
        let verify_success = self.verify_success.load(Ordering::Relaxed);
        let verify_failure = self.verify_failure.load(Ordering::Relaxed);
        let verify_duration = self.verify_duration_us.load(Ordering::Relaxed);

        let hsm_ops = self.hsm_operations.load(Ordering::Relaxed);
        let hsm_errors = self.hsm_errors.load(Ordering::Relaxed);

        let avg_prove_us = if prove_total > 0 {
            prove_duration / prove_total
        } else {
            0
        };
        let avg_verify_us = if verify_total > 0 {
            verify_duration / verify_total
        } else {
            0
        };

        format!(
            "{{\
                \"prove\": {{\
                    \"total\": {},\
                    \"success\": {},\
                    \"failure\": {},\
                    \"avg_duration_us\": {}\
                }},\
                \"verify\": {{\
                    \"total\": {},\
                    \"success\": {},\
                    \"failure\": {},\
                    \"avg_duration_us\": {}\
                }},\
                \"hsm\": {{\
                    \"operations\": {},\
                    \"errors\": {}\
                }}\
            }}",
            prove_total,
            prove_success,
            prove_failure,
            avg_prove_us,
            verify_total,
            verify_success,
            verify_failure,
            avg_verify_us,
            hsm_ops,
            hsm_errors
        )
    }
}

/// Convenience timer for measuring VRF operation durations
///
/// Automatically tracks elapsed time from creation. Use this to simplify
/// timing measurements when recording metrics for VRF operations.
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::{VrfMetrics, MetricsTimer};
///
/// let metrics = VrfMetrics::new();
/// let timer = MetricsTimer::new();
///
/// // ... perform VRF proof generation ...
/// let success = true;
///
/// // Record with automatic duration measurement
/// metrics.record_prove(timer.elapsed(), success);
/// ```
///
/// # Alternative Usage
///
/// For more control, use `std::time::Instant` directly:
///
/// ```rust
/// use cardano_vrf::VrfMetrics;
/// use std::time::Instant;
///
/// let metrics = VrfMetrics::new();
/// let start = Instant::now();
///
/// // ... operation ...
///
/// metrics.record_prove(start.elapsed(), true);
/// ```
pub struct MetricsTimer {
    start: Instant,
}

impl MetricsTimer {
    /// Creates and starts a new metrics timer
    ///
    /// The timer begins immediately upon creation. Use `elapsed()` to get
    /// the duration at any point after creation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::MetricsTimer;
    ///
    /// let timer = MetricsTimer::new();
    /// // ... perform operation ...
    /// let duration = timer.elapsed();
    /// println!("Operation took: {:?}", duration);
    /// ```
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Returns elapsed time since timer creation
    ///
    /// Can be called multiple times to get cumulative duration. The timer
    /// continues running and does not reset after calling this method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::MetricsTimer;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let timer = MetricsTimer::new();
    ///
    /// thread::sleep(Duration::from_millis(10));
    /// let checkpoint1 = timer.elapsed();
    ///
    /// thread::sleep(Duration::from_millis(10));
    /// let checkpoint2 = timer.elapsed();
    ///
    /// assert!(checkpoint2 > checkpoint1);
    /// ```
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Default for MetricsTimer {
    fn default() -> Self {
        Self::new()
    }
}
