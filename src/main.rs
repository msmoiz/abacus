use metrics::reporter;
use std::thread::{self};
use std::time::Duration;

use metrics::metric;
use metrics::reporter::set_reporter;
use metrics::simple_reporter::SimpleReporter;

fn main() {
    let _guard = set_reporter(SimpleReporter::new());

    for _ in 0..8 {
        metric!("requests", 1, "user" => "alice");
    }

    for _ in 0..3 {
        thread::spawn(|| {
            let thread_id = format!("{:?}", thread::current().id());
            metric!("requests", 3, "thread" => thread_id);
        });
    }

    thread::sleep(Duration::from_secs(5));

    metric!("requests", 1, "user" => "bob", "id" => "12345");
}
