use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Copy, Default, Debug)]
pub struct TrafficData {
    pub up: f32,
    pub down: f32,
}

pub struct TrafficStore {
    history: VecDeque<TrafficData>,
    max_samples: usize,
}

impl TrafficStore {
    pub fn new(max_samples: usize) -> Self {
        let mut history = VecDeque::with_capacity(max_samples);
        for _ in 0..max_samples {
            history.push_back(TrafficData::default());
        }
        Self {
            history,
            max_samples,
        }
    }

    pub fn push(&mut self, data: TrafficData) {
        if self.history.len() >= self.max_samples {
            self.history.pop_front();
        }
        self.history.push_back(data);
    }

    pub fn get_history(&self) -> Vec<TrafficData> {
        self.history.iter().cloned().collect()
    }

    pub fn last(&self) -> TrafficData {
        self.history.back().cloned().unwrap_or_default()
    }
}

pub type SharedTrafficStore = Arc<RwLock<TrafficStore>>;
