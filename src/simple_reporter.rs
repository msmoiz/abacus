use crate::{metric::Metric, reporter::Reporter};

/// A reporter that reports metrics without buffering.
#[derive(Debug)]
pub struct SimpleReporter;

impl SimpleReporter {
    /// Creates a new Reporter.
    pub fn new() -> Self {
        SimpleReporter
    }
}

impl Reporter for SimpleReporter {
    fn report(&self, metric: Metric) {
        println!("recording: {metric}");
    }
}
