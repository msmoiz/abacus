mod buf_reporter;
mod metric;
mod reporter;

use std::thread::{self};
use std::time::Duration;

use buf_reporter::BufReporter;
use reporter::set_reporter;

fn main() {
    let _guard = set_reporter(BufReporter::new());

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
