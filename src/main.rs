use std::fmt::Write;
use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, SecondsFormat, Utc};

fn main() {
    metric(
        "requests",
        1,
        HashMap::from([("user".into(), "bob".into())]),
    );
}

fn metric(name: &str, value: u64, dimensions: HashMap<String, String>) {
    let timestamp = {
        let dt = SystemTime::now();
        let dt: DateTime<Utc> = dt.into();
        dt.to_rfc3339_opts(SecondsFormat::Secs, false)
    };

    let mut message = format!("{name}");

    let dimensions = dimensions
        .iter()
        .map(|(key, val)| format!("{key}: {val}"))
        .collect::<Vec<_>>()
        .join(", ");

    write!(&mut message, "{{ {dimensions} }}").unwrap();

    write!(&mut message, " {value} @ {timestamp}").unwrap();

    println!("{message}");
}
