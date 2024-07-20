use std::{cell::OnceCell, collections::HashMap, fmt::Debug, time::SystemTime};

use chrono::{DateTime, Utc};

use crate::metric::Metric;

static mut REPORTER: OnceCell<Box<dyn Reporter + 'static>> = OnceCell::new();

/// Describes the interface for an object that reports metrics.
pub trait Reporter: Debug {
    /// Reports a metric.
    fn report(&self, metric: Metric);

    /// Closes the reporter.
    ///
    /// Does nothing by default.
    fn close(&mut self) {}
}

/// An RAII guard for the global reporter.
///
/// Flushes the reporter when dropped. This is useful for reporting metrics that
/// have been buffered but not flushed on program end or during a panic.
pub struct ReporterGuard;

impl Drop for ReporterGuard {
    fn drop(&mut self) {
        unsafe {
            REPORTER.get_mut().map(|r| r.close());
        }
    }
}

/// Sets the global reporter.
///
/// Returns a guard that will flush the reporter when dropped. It is an error to
/// set the reporter more than once during the lifetime of a program.
pub fn set_reporter<R>(reporter: R) -> ReporterGuard
where
    R: Reporter + 'static,
{
    unsafe {
        REPORTER
            .set(Box::new(reporter))
            .expect("reporter should only be set once");
    }

    ReporterGuard
}

/// Reports a metric using the global reporter.
pub fn metric(name: &str, value: u64, dimensions: HashMap<String, String>) {
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
        REPORTER.get().map(|r| r.report(metric));
    }
}

/// Reports a metric using the global reporter.
///
/// Name and value must be specified. You may also specify optional dimensions
/// as key-value pairs. Name and dimension key-value fields may be populated by
/// any type that implements Into<String>.
///
/// # Example
///
/// ```
/// metric!("requests", 1);
/// metric!("requests", 2, "user" => "bob");
/// ```
#[macro_export]
macro_rules! metric {
    ( $name: expr, $value: expr $(, $dim_key:expr => $dim_value:expr)* ) => {
        let mut dimensions = std::collections::HashMap::new();
        $(
            dimensions.insert($dim_key.into(), $dim_value.into());
        )*
        crate::reporter::metric($name, $value, dimensions)
    };
}
