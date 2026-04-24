use std::sync::Arc;
use parking_lot::RwLock;

pub struct LogStore {
    pub logs: Vec<String>,
}

impl LogStore {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn add_logs(&mut self, logs: String) {
        if logs.is_empty() {
            return;
        }
        for line in logs.lines() {
            if !line.is_empty() {
                self.logs.push(line.to_string());
            }
        }
        // Limit to last 1000 lines
        if self.logs.len() > 1000 {
            let start = self.logs.len() - 1000;
            self.logs = self.logs[start..].to_vec();
        }
    }
}

pub type SharedLogStore = Arc<RwLock<LogStore>>;
