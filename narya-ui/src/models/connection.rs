use api::tracker::ConnectionMeta;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct ConnectionStore {
    pub active_connections: Vec<ConnectionMeta>,
}

impl ConnectionStore {
    pub fn new() -> Self {
        Self {
            active_connections: Vec::new(),
        }
    }
}

pub type SharedConnectionStore = Arc<RwLock<ConnectionStore>>;
