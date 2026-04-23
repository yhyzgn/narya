use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
}

pub struct RuleStore {
    pub unassigned: Vec<AppInfo>,
    pub direct: Vec<AppInfo>,
    pub proxy: Vec<AppInfo>,
}

impl RuleStore {
    pub fn new() -> Self {
        Self {
            unassigned: Vec::new(),
            direct: Vec::new(),
            proxy: Vec::new(),
        }
    }

    pub fn assign_to_direct(&mut self, app_id: &str) {
        if let Some(index) = self.unassigned.iter().position(|a| a.id == app_id) {
            let app = self.unassigned.remove(index);
            self.direct.push(app);
        } else if let Some(index) = self.proxy.iter().position(|a| a.id == app_id) {
            let app = self.proxy.remove(index);
            self.direct.push(app);
        }
    }

    pub fn assign_to_proxy(&mut self, app_id: &str) {
        if let Some(index) = self.unassigned.iter().position(|a| a.id == app_id) {
            let app = self.unassigned.remove(index);
            self.proxy.push(app);
        } else if let Some(index) = self.direct.iter().position(|a| a.id == app_id) {
            let app = self.direct.remove(index);
            self.proxy.push(app);
        }
    }

    pub fn unassign(&mut self, app_id: &str) {
        if let Some(index) = self.direct.iter().position(|a| a.id == app_id) {
            let app = self.direct.remove(index);
            self.unassigned.push(app);
        } else if let Some(index) = self.proxy.iter().position(|a| a.id == app_id) {
            let app = self.proxy.remove(index);
            self.unassigned.push(app);
        }
    }
}

pub type SharedRuleStore = Arc<RwLock<RuleStore>>;
