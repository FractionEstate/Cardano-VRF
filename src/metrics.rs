//! Production monitoring and metrics
//!
//! Provides Prometheus-compatible metrics for VRF operations

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Metrics collector for VRF operations
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
    /// Creates a new metrics collector
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

    /// Record a prove operation
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

    /// Record a verify operation
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

    /// Record an HSM operation
    pub fn record_hsm_operation(&self, success: bool) {
        self.hsm_operations.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.hsm_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get Prometheus-formatted metrics
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

    /// Get JSON-formatted metrics
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

/// Timer for measuring operation duration
pub struct MetricsTimer {
    start: Instant,
}

impl MetricsTimer {
    /// Starts a new timer
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Gets elapsed time since timer creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Default for MetricsTimer {
    fn default() -> Self {
        Self::new()
    }
}
