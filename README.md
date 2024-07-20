# Abacus

This is a simple implementation of a metric reporting interface. It provides the
user with a simple method for emitting metrics from any scope and any thread
while supporting batched uploads and periodic flushes in the background. It also
flushes remaining metrics on program end.

```rust
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
```
