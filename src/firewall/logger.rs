use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tokio::sync::mpsc;

use crate::firewall::events::{FirewallEvent, Verdict};

/// Async JSONL file logger — receives events via channel, writes to disk
pub struct EventLogger {
    access_path: String,
    blocked_path: String,
}

impl EventLogger {
    pub fn new(access_path: &str, blocked_path: &str) -> Self {
        // Ensure directories exist
        for path in [access_path, blocked_path] {
            if let Some(parent) = Path::new(path).parent() {
                let _ = fs::create_dir_all(parent);
            }
        }
        Self {
            access_path: access_path.to_string(),
            blocked_path: blocked_path.to_string(),
        }
    }

    /// Spawn the logger task. Returns a sender for events.
    pub fn spawn(self) -> mpsc::UnboundedSender<FirewallEvent> {
        let (tx, mut rx) = mpsc::unbounded_channel::<FirewallEvent>();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                self.write_event(&event);
            }
        });

        tx
    }

    fn write_event(&self, event: &FirewallEvent) {
        if let Ok(json) = serde_json::to_string(event) {
            // Always write to access log
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.access_path)
            {
                let _ = writeln!(file, "{}", json);
            }

            // Write blocked/rate-limited to separate log
            match event.verdict {
                Verdict::Block | Verdict::RateLimited => {
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&self.blocked_path)
                    {
                        let _ = writeln!(file, "{}", json);
                    }
                }
                _ => {}
            }
        }
    }
}
