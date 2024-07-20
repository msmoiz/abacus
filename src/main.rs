use std::cell::OnceCell;
use std::fmt::{Debug, Display};
use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, SecondsFormat, Utc};

static mut REPORTER: OnceCell<Box<dyn Reporter + 'static>> = OnceCell::new();

macro_rules! metric {
    ( $name: expr, $value: expr $(, $dim_key:expr => $dim_value:expr)* ) => {
        let mut dimensions = HashMap::new();
        $(
            dimensions.insert($dim_key.into(), $dim_value.into());
        )*
        metric($name, $value, dimensions)
    };
}

fn main() {
    let _guard = init_reporter(BufReporter::new());

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

/// Describes the interface for an object that reports metrics.
trait Reporter: Debug {
    /// Reports a metric.
    fn report(&self, metric: Metric);

    /// Closes the reporter.
    ///
    /// Does nothing by default.
    fn close(&mut self) {}
}

/// Initializes the global reporter.
///
/// Returns a guard that will flush the reporter when dropped.
fn init_reporter<R>(reporter: R) -> ReporterGuard
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

/// A reporter that reports metrics with buffering.
#[derive(Debug)]
struct BufReporter {
    /// Handle for the reporting thread.
    handle: Option<JoinHandle<()>>,
    /// Interface to send metrics to the reporting thread.
    sender: Sender<Message>,
}

impl BufReporter {
    /// Creates a new Reporter.
    fn new() -> Self {
        let (handle, sender) = Self::report();
        Self {
            handle: Some(handle),
            sender,
        }
    }

    /// Reports metrics in the background.
    ///
    /// Starts a separate thread that reports metrics. Metrics can be passed to
    /// the thread using the channel handle returned.
    fn report() -> (JoinHandle<()>, Sender<Message>) {
        let (sender, receiver) = channel();
        let user_sender = sender.clone();
        let timer_sender = sender.clone();

        let handle = thread::spawn(move || {
            let mut buf = vec![];
            let mut pos = 0;

            loop {
                let Ok(message) = receiver.recv() else { break };

                match message {
                    Message::Metric(metric) => {
                        println!("buffering: {metric}");
                        buf.push(metric);
                        const FLUSH_THRESHOLD: usize = 3;
                        if buf.len() - pos == FLUSH_THRESHOLD {
                            println!("flushing metrics ({}..{})", pos, buf.len() - 1);
                            pos = buf.len();
                        }
                    }
                    Message::Flush => {
                        println!("flushing metrics ({}..{})", pos, buf.len() - 1);
                        pos = buf.len();
                    }
                    Message::Close => {
                        if pos < buf.len() {
                            println!("flushing metrics ({}..{})", pos, buf.len() - 1);
                        }
                        break;
                    }
                }
            }
        });

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(3));
            timer_sender.send(Message::Flush).unwrap();
        });

        (handle, user_sender)
    }
}

impl Reporter for BufReporter {
    fn report(&self, metric: Metric) {
        self.sender.send(Message::Metric(metric)).unwrap();
    }

    fn close(&mut self) {
        self.sender.send(Message::Close).unwrap();
        self.handle.take().map(|t| t.join());
    }
}

/// An RAII guard for the global reporter.
///
/// Flushes the reporter when dropped. This is useful for reporting metrics that
/// have been buffered but not flushed on program end or during a panic.
struct ReporterGuard;

impl Drop for ReporterGuard {
    fn drop(&mut self) {
        unsafe {
            REPORTER.get_mut().map(|r| r.close());
        }
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

/// Message to send to the reporting thread.
enum Message {
    /// Report a metric.
    Metric(Metric),
    /// Flush metrics.
    Flush,
    /// Close the thread.
    Close,
}

fn metric(name: &str, value: u64, dimensions: HashMap<String, String>) {
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
