use std::{
    sync::mpsc::{channel, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{metric::Metric, reporter::Reporter};

/// Message to send to the reporting thread.
enum Message {
    /// Report a metric.
    Metric(Metric),
    /// Flush metrics.
    Flush,
    /// Close the thread.
    Close,
}

/// A reporter that reports metrics with buffering.
#[derive(Debug)]
pub struct BufReporter {
    /// Handle for the reporting thread.
    handle: Option<JoinHandle<()>>,
    /// Interface to send metrics to the reporting thread.
    sender: Sender<Message>,
}

impl BufReporter {
    /// Creates a new Reporter.
    pub fn new() -> Self {
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
