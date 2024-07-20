use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, SecondsFormat, Utc};

/// A quantitative measurement.
pub struct Metric {
    /// The name of the metric.
    pub(crate) name: String,
    /// The value of the metric.
    pub(crate) value: u64,
    /// The timestamp associated with the metric.
    pub(crate) time: DateTime<Utc>,
    /// The dimensions associated with the metric.
    pub(crate) dimensions: HashMap<String, String>,
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
