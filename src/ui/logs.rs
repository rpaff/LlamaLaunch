use std::io::{BufRead, BufReader};
use std::process::Child;
use std::sync::mpsc;
use std::thread;

/// Stream for reading stdout/stderr from a running child process.
/// Reads with line buffering and splits on \n, \r\n, or bare \r — this
/// guarantees that every newline-terminated message from llama.cpp arrives
/// in the log panel the instant it is written to the pipe.
pub struct ModelLogStream {
    receiver: mpsc::Receiver<String>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl ModelLogStream {
    /// Start threads to read stdout and stderr from the child process in parallel.
    /// Uses a bounded channel so backpressure prevents unbounded memory growth
    /// during long-running servers with heavy output.
    pub fn new(child: &mut Child) -> Self {
        // Buffer of 8192 lines — prevents log drops during idle periods between
        // model generations when the llama.cpp server writes heartbeat/metrics to
        // stdout/stderr but the UI hasn't polled for a while.
        const CHANNEL_CAPACITY: usize = 8192;
        let (sender, receiver) = mpsc::sync_channel(CHANNEL_CAPACITY);

        let stdout = child.stdout.take().expect("stdout should be piped");
        let stderr = child.stderr.take().expect("stderr should be piped");

        // Two threads — one for each stream — read concurrently.
        // A dedicated reader handles \r, \n, and \r\n so progress-bars
        // (which use bare \r) are delivered as individual log lines rather
        // than silently swallowed or merged into the next line.
        let stdout_sender = sender.clone();
        let stdout_handle = thread::spawn(move || {
            read_lines(stdout, stdout_sender);
        });

        let stderr_handle = thread::spawn(move || {
            read_lines(stderr, sender);
        });

        let join_handle = thread::spawn(move || {
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
        });

        ModelLogStream {
            receiver,
            join_handle: Some(join_handle),
        }
    }

    /// Poll up to `max_lines` new log entries from the channel.
    pub fn poll(&mut self, max_lines: usize) -> Vec<String> {
        let mut logs = Vec::new();
        while logs.len() < max_lines {
            match self.receiver.try_recv() {
                Ok(line) => logs.push(line),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
        logs
    }

    /// Stop reading and wait for the threads to finish.
    pub fn finish(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Read bytes from a stream, splitting on any combination of \r / \n,
/// and send each non-empty line through the channel immediately.
fn read_lines<T: std::io::Read>(reader: T, sender: mpsc::SyncSender<String>) {
    let buf_reader = BufReader::with_capacity(128 * 1024, reader);

    for result in buf_reader.split(b'\n') {
        match result {
            Ok(raw) => {
                // Split on \r as well (handles \r\n and bare \r from progress bars).
                let mut remaining: &[u8] = &raw[..];
                while !remaining.is_empty() {
                    if let Some(pos) = remaining.iter().position(|&b| b == b'\r') {
                        let chunk = &remaining[..pos];
                        if !is_blank(chunk) {
                            let text = String::from_utf8_lossy(chunk).into_owned();
                            // On full channel, drop to avoid blocking the reader.
                            let _ = sender.try_send(text);
                        }
                        remaining = &remaining[pos + 1..]; // skip \r
                    } else {
                        // No more \r in this segment — emit the rest.
                        if !is_blank(remaining) {
                            let text = String::from_utf8_lossy(remaining).into_owned();
                            let _ = sender.try_send(text);
                        }
                        break;
                    }
                }
            }
            Err(_) => break, // stream ended
        }
    }
}

/// Return true if the byte slice contains only whitespace / control chars.
fn is_blank(buf: &[u8]) -> bool {
    buf.iter().all(|b| b.is_ascii_whitespace())
}
