use std::fmt::Display;
use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, SecondsFormat, Utc};

fn main() {
    for _ in 0..6 {
        metric(
            "requests",
            1,
            HashMap::from([("user".into(), "bob".into())]),
        );
    }
}

/// An interface for reporting metrics.
struct Reporter {
    /// Buffered metrics.
    buf: Vec<Metric>,
    /// The current position in the buffer. Metrics buffered after this position
    /// have not been reported yet.
    pos: usize,
}

impl Reporter {
    /// Creates a new Reporter.
    const fn new() -> Self {
        Self {
            buf: vec![],
            pos: 0,
        }
    }

    /// Records a metric.
    fn record(&mut self, metric: Metric) {
        println!("record: {metric}");
        self.buf.push(metric);

        const FLUSH_THRESHOLD: usize = 3;
        if self.buf.len() - self.pos == FLUSH_THRESHOLD {
            self.flush();
        }
    }

    /// Flushes buffered metrics.
    fn flush(&mut self) {
        for metric in &self.buf[self.pos..] {
            println!("report: {metric}");
        }
        self.pos = self.buf.len();
    }
}

/// A quantitative measurement.
struct Metric {
    /// The name of the metric.
    name: String,
    /// The value of the metric.
    value: u64,
    /// The timestamp associated with the metric.
    time: DateTime<Utc>,
    /// The dimensions associated with the metric.
    dimensions: HashMap<String, String>,
}

impl Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;

        write!(f, "{{ ")?;
        let mut i = 0;
        for (key, value) in &self.dimensions {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{key}: {value}")?;
            i += 1;
        }
        write!(f, " }}")?;

        write!(
            f,
            " {} @ {}",
            self.value,
            self.time.to_rfc3339_opts(SecondsFormat::Secs, false)
        )
    }
}

fn metric(name: &str, value: u64, dimensions: HashMap<String, String>) {
    static mut REPORTER: Reporter = Reporter::new();

    let time = {
        let dt = SystemTime::now();
        let dt: DateTime<Utc> = dt.into();
        dt
    };

    let metric = Metric {
        name: name.into(),
        value,
        time,
        dimensions,
    };

    unsafe {
        REPORTER.record(metric);
    }
}
