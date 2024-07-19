use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, Utc};

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
        dt.to_rfc3339()
    };

    let mut message = format!("emitting metric ({timestamp}): {name} = {value}");

    let dimensions = dimensions
        .iter()
        .map(|(key, val)| format!("{key}: {val}"))
        .collect::<Vec<_>>()
        .join(", ");

    message.push_str(&format!(" {{ {dimensions} }}"));

    println!("{message}");
}
