use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ProxyNode {
    pub name: String,
    pub protocol: String,
    pub delay: Option<u64>,
}

#[derive(Clone, Default)]
pub struct ProfileStore {
    pub url: String,
    pub nodes: Vec<ProxyNode>,
    pub is_loading: bool,
    pub last_error: Option<String>,
}

impl ProfileStore {
    pub fn new(url: String) -> Self {
        Self {
            url,
            nodes: Vec::new(),
            is_loading: false,
            last_error: None,
        }
    }
}

pub type SharedProfileStore = Arc<RwLock<ProfileStore>>;
