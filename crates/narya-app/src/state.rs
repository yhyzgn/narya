use crate::ipc::IpcClient;
use gpui::*;
use narya_core;
use narya_ipc::{IpcNotification, IpcRequest, PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

fn default_rules() -> Vec<narya_rules::Rule> {
    vec![
        narya_rules::Rule {
            id: "private-network".into(),
            priority: 10,
            conditions: vec![narya_rules::Condition::IpCidr {
                network: "192.168.0.0".parse::<IpAddr>().expect("valid default CIDR"),
                prefix: 16,
            }],
            action: narya_rules::Action::Direct,
        },
        narya_rules::Rule {
            id: "local-domains".into(),
            priority: 20,
            conditions: vec![narya_rules::Condition::DomainSuffix("lan".into())],
            action: narya_rules::Action::Direct,
        },
        narya_rules::Rule {
            id: "ai-services".into(),
            priority: 100,
            conditions: vec![narya_rules::Condition::DomainSuffix("openai.com".into())],
            action: narya_rules::Action::Proxy("proxy".into()),
        },
        narya_rules::Rule {
            id: "default-block".into(),
            priority: 1000,
            conditions: vec![narya_rules::Condition::Any],
            action: narya_rules::Action::Block,
        },
    ]
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionTab {
    Overview,
    Nodes,
    Rules,
    Conversion,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub time: String,
    pub level: String,
    pub module: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PersistedState {
    pub nodes: Vec<narya_core::Node>,
    pub subscriptions: Vec<narya_core::Subscription>,
    pub active_node_id: Option<String>,
    pub kernel_running: bool,
    #[serde(default)]
    pub active_kernel: String,
    pub selected_subscription_id: Option<String>,
    #[serde(default)]
    pub rules: Vec<narya_rules::Rule>,
    #[serde(default)]
    pub groups: Vec<narya_rules::RoutingGroup>,
    #[serde(default)]
    pub rule_sets: Vec<narya_rules::RuleSetSource>,
    #[serde(default)]
    pub setting_autostart: bool,
    #[serde(default)]
    pub setting_start_minimized: bool,
    #[serde(default = "default_true")]
    pub setting_close_to_tray: bool,
    #[serde(default = "default_true")]
    pub setting_restore_proxy: bool,
    #[serde(default = "default_true")]
    pub setting_auto_update: bool,
    #[serde(default)]
    pub appearance_mode: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelCatalogOption {
    pub kernel: String,
    pub version: String,
    pub platform: String,
    pub architecture: String,
    pub source: String,
    pub sha256: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoutingBundle {
    version: u32,
    rules: Vec<narya_rules::Rule>,
    groups: Vec<narya_rules::RoutingGroup>,
    rule_sets: Vec<narya_rules::RuleSetSource>,
}

fn parse_condition(kind: &str, value: &str) -> Result<narya_rules::Condition, String> {
    let value = value.trim();
    match kind {
        "domain" if !value.is_empty() => Ok(narya_rules::Condition::Domain(value.into())),
        "domain_suffix" if !value.is_empty() => {
            Ok(narya_rules::Condition::DomainSuffix(value.into()))
        }
        "process" if !value.is_empty() => Ok(narya_rules::Condition::Process(value.into())),
        "rule_set" if !value.is_empty() => Ok(narya_rules::Condition::RuleSet(value.into())),
        "ip_cidr" => {
            let (network, prefix) = value
                .rsplit_once('/')
                .ok_or_else(|| "CIDR 条件格式应为地址/前缀，例如 10.0.0.0/8".to_string())?;
            let network = network
                .parse::<IpAddr>()
                .map_err(|_| "CIDR 地址无效".to_string())?;
            let prefix = prefix
                .parse::<u8>()
                .map_err(|_| "CIDR 前缀无效".to_string())?;
            Ok(narya_rules::Condition::IpCidr { network, prefix })
        }
        "port" => Ok(narya_rules::Condition::Port(
            value
                .parse::<u16>()
                .map_err(|_| "端口必须是 1-65535 的整数".to_string())?,
        )),
        "any" => Ok(narya_rules::Condition::Any),
        _ => Err("条件值不能为空，或条件类型不支持".into()),
    }
}

fn parse_rule_set_format(value: &str) -> narya_rules::RuleSetFormat {
    match value {
        "domain" => narya_rules::RuleSetFormat::Domain,
        "ip_cidr" => narya_rules::RuleSetFormat::IpCidr,
        "classical" => narya_rules::RuleSetFormat::Classical,
        _ => narya_rules::RuleSetFormat::SingBoxBinary,
    }
}

fn write_routing_bundle(path: &str, bundle: &RoutingBundle) -> Result<(), String> {
    let path = std::path::Path::new(path);
    if !path.is_absolute() {
        return Err("导入导出路径必须是绝对路径".into());
    }
    let json = serde_json::to_vec_pretty(bundle).map_err(|error| error.to_string())?;
    let temp = path.with_extension("narya.tmp");
    std::fs::write(&temp, json).map_err(|error| error.to_string())?;
    std::fs::rename(&temp, path).map_err(|error| error.to_string())
}

fn read_routing_bundle(path: &str) -> Result<RoutingBundle, String> {
    let path = std::path::Path::new(path);
    if !path.is_absolute() {
        return Err("导入导出路径必须是绝对路径".into());
    }
    let json = std::fs::read(path).map_err(|error| error.to_string())?;
    let bundle: RoutingBundle = serde_json::from_slice(&json).map_err(|error| error.to_string())?;
    if bundle.version != 1 {
        return Err(format!("不支持的规则配置版本 {}", bundle.version));
    }
    narya_rules::RuleSet::compile(bundle.rules.clone())
        .map_err(|error| format!("规则校验失败：{error}"))?;
    let mut group_ids = std::collections::HashSet::new();
    for group in &bundle.groups {
        group
            .validate()
            .map_err(|error| format!("分流组校验失败：{error}"))?;
        if !group_ids.insert(group.id.as_str()) {
            return Err(format!("重复的分流组 ID：{}", group.id));
        }
    }
    let mut rule_set_ids = std::collections::HashSet::new();
    for source in &bundle.rule_sets {
        source
            .validate()
            .map_err(|error| format!("规则集校验失败：{error}"))?;
        if !rule_set_ids.insert(source.id.as_str()) {
            return Err(format!("重复的规则集 ID：{}", source.id));
        }
    }
    for rule in &bundle.rules {
        for condition in &rule.conditions {
            if let narya_rules::Condition::RuleSet(id) = condition {
                if !rule_set_ids.contains(id.as_str()) {
                    return Err(format!("规则 {} 引用了未知规则集 {}", rule.id, id));
                }
                if bundle
                    .rule_sets
                    .iter()
                    .find(|source| source.id == *id)
                    .is_some_and(|source| !source.enabled)
                {
                    return Err(format!("规则 {} 引用了已禁用规则集 {}", rule.id, id));
                }
            }
        }
        if let narya_rules::Action::Proxy(target) = &rule.action {
            if !group_ids.contains(target.as_str()) {
                return Err(format!("规则 {} 引用了未知分流组 {}", rule.id, target));
            }
        }
    }
    Ok(bundle)
}

fn subscription_refresh_failure_summary(stage: &str, error: &str) -> String {
    let mut message = error.trim().replace('\n', " ");
    const MAX_ERROR_LEN: usize = 180;
    if message.chars().count() > MAX_ERROR_LEN {
        message = message.chars().take(MAX_ERROR_LEN).collect::<String>();
        message.push_str("...");
    }
    if message.is_empty() {
        format!("{stage}：未知错误")
    } else {
        format!("{stage}：{message}")
    }
}

fn mark_subscription_refresh_failed(
    subscriptions: &mut [narya_core::Subscription],
    sub_id: &str,
    summary: &str,
) {
    if let Some(sub) = subscriptions.iter_mut().find(|s| s.id == sub_id) {
        sub.status = summary.to_string();
    }
}

pub struct AppState {
    pub nodes: Vec<narya_core::Node>,
    pub subscriptions: Vec<narya_core::Subscription>,
    pub active_node_id: Option<String>,
    pub kernel_running: bool,
    pub filter_text: String,
    pub rule_filter_text: String,
    pub rule_action_filter: String,

    // Subscription specific state
    pub selected_subscription_id: Option<String>,
    pub active_subscription_tab: SubscriptionTab,
    pub subscription_filter_text: String,
    pub subscription_draft_name: String,
    pub subscription_draft_url: String,
    pub subscription_error: Option<String>,

    pub subscription_list_state: ListState,

    pub log_lines: Vec<LogMessage>,
    pub kernels: Vec<narya_ipc::KernelInfo>,
    pub rules: Vec<narya_rules::Rule>,
    pub groups: Vec<narya_rules::RoutingGroup>,
    pub rule_sets: Vec<narya_rules::RuleSetSource>,
    pub routing_mode: narya_platform::ProxyMode,
    pub routing_configured: Option<String>,
    pub routing_active: narya_platform::ProxyMode,
    pub kernel_healthy: bool,
    pub active_kernel: String,
    pub kernel_artifact_kernel: String,
    pub kernel_artifact_version: String,
    pub kernel_artifact_source: String,
    pub kernel_artifact_sha256: String,
    pub kernel_artifact_signature: String,
    pub kernel_artifact_public_key: String,
    pub kernel_operation: Option<String>,
    pub kernel_error: Option<String>,
    pub kernel_catalog_source: String,
    pub kernel_catalog_trusted_key: String,
    pub kernel_catalog_status: Option<String>,
    pub kernel_catalog_entries: Vec<KernelCatalogOption>,
    pub rule_set_draft_id: String,
    pub rule_set_draft_source: String,
    pub rule_set_draft_version: String,
    pub rule_set_draft_sha256: String,
    pub rule_set_draft_format: String,
    pub rule_set_draft_signature: String,
    pub rule_set_draft_public_key: String,
    pub rule_set_error: Option<String>,
    pub group_error: Option<String>,
    pub rule_editor_error: Option<String>,
    pub rule_io_path: String,
    pub rule_io_status: Option<String>,
    pub settings_category: usize,
    pub setting_autostart: bool,
    pub setting_start_minimized: bool,
    pub setting_close_to_tray: bool,
    pub setting_restore_proxy: bool,
    pub setting_auto_update: bool,
    pub appearance_mode: usize,
}

impl AppState {
    pub fn set_settings_category(&mut self, index: usize, cx: &mut Context<Self>) {
        self.settings_category = index;
        cx.notify();
    }

    pub fn set_setting_value(&mut self, key: &'static str, value: bool, cx: &mut Context<Self>) {
        match key {
            "autostart" => self.setting_autostart = value,
            "start_minimized" => self.setting_start_minimized = value,
            "close_to_tray" => self.setting_close_to_tray = value,
            "restore_proxy" => self.setting_restore_proxy = value,
            "auto_update" => self.setting_auto_update = value,
            _ => return,
        }
        self.save();
        cx.notify();
    }

    pub fn set_appearance_mode(&mut self, mode: usize, cx: &mut Context<Self>) {
        self.appearance_mode = mode.min(2);
        self.save();
        cx.notify();
    }

    pub fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("narya");
        std::fs::create_dir_all(&path).ok();
        path.push("state.json");
        path
    }

    pub fn save(&self) {
        let persisted = PersistedState {
            nodes: self.nodes.clone(),
            subscriptions: self.subscriptions.clone(),
            active_node_id: self.active_node_id.clone(),
            kernel_running: self.kernel_running,
            active_kernel: self.active_kernel.clone(),
            selected_subscription_id: self.selected_subscription_id.clone(),
            rules: self.rules.clone(),
            groups: self.groups.clone(),
            rule_sets: self.rule_sets.clone(),
            setting_autostart: self.setting_autostart,
            setting_start_minimized: self.setting_start_minimized,
            setting_close_to_tray: self.setting_close_to_tray,
            setting_restore_proxy: self.setting_restore_proxy,
            setting_auto_update: self.setting_auto_update,
            appearance_mode: self.appearance_mode,
        };
        if let Err(error) = Self::save_persisted(&persisted) {
            eprintln!("failed to persist Narya state: {error}");
        }
    }

    fn save_persisted(persisted: &PersistedState) -> std::io::Result<()> {
        let path = Self::config_path();
        let temp = path.with_extension("json.tmp");
        let json = serde_json::to_vec_pretty(persisted).map_err(std::io::Error::other)?;
        std::fs::write(&temp, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o600))?;
        }
        std::fs::rename(temp, path)
    }

    pub fn load_persisted() -> PersistedState {
        std::fs::read_to_string(Self::config_path())
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }

    pub fn set_filter_text(&mut self, text: String, cx: &mut Context<Self>) {
        self.filter_text = text;
        cx.notify();
    }

    pub fn set_rule_filter_text(&mut self, text: String, cx: &mut Context<Self>) {
        self.rule_filter_text = text;
        cx.notify();
    }

    pub fn set_rule_action_filter(&mut self, filter: String, cx: &mut Context<Self>) {
        self.rule_action_filter = filter;
        cx.notify();
    }

    pub fn set_rule_action(model: Entity<Self>, cx: &mut App, rule_id: String, target: String) {
        model.update(cx, |state, cx| {
            if let Some(rule) = state.rules.iter_mut().find(|rule| rule.id == rule_id) {
                rule.action = match target.as_str() {
                    "direct" => narya_rules::Action::Direct,
                    "block" => narya_rules::Action::Block,
                    value if value.starts_with("dns:") => {
                        narya_rules::Action::Dns(value.trim_start_matches("dns:").into())
                    }
                    value if !value.trim().is_empty() => narya_rules::Action::Proxy(value.into()),
                    _ => rule.action.clone(),
                };
                state.save();
                cx.notify();
            }
        });
    }

    pub fn set_rule_condition(
        model: Entity<Self>,
        cx: &mut App,
        rule_id: String,
        index: usize,
        kind: String,
        value: String,
    ) {
        model.update(cx, |state, cx| {
            let Some(rule) = state.rules.iter_mut().find(|rule| rule.id == rule_id) else {
                return;
            };
            let condition = match parse_condition(&kind, &value) {
                Ok(condition) => condition,
                Err(error) => {
                    state.rule_editor_error = Some(error);
                    cx.notify();
                    return;
                }
            };
            if let Some(existing) = rule.conditions.get_mut(index) {
                *existing = condition;
            } else {
                state.rule_editor_error = Some("条件索引已失效，请刷新规则页".into());
                cx.notify();
                return;
            }
            if let Err(error) = narya_rules::RuleSet::compile(state.rules.clone()) {
                state.rule_editor_error = Some(format!("规则校验失败：{error}"));
                cx.notify();
                return;
            }
            state.rule_editor_error = None;
            state.save();
            cx.notify();
        });
    }

    pub fn add_rule_condition(model: Entity<Self>, cx: &mut App, rule_id: String) {
        model.update(cx, |state, cx| {
            if let Some(rule) = state.rules.iter_mut().find(|rule| rule.id == rule_id) {
                rule.conditions
                    .push(narya_rules::Condition::DomainSuffix("example.com".into()));
                state.rule_editor_error = None;
                state.save();
                cx.notify();
            }
        });
    }

    pub fn remove_rule_condition(model: Entity<Self>, cx: &mut App, rule_id: String, index: usize) {
        model.update(cx, |state, cx| {
            let Some(rule) = state.rules.iter_mut().find(|rule| rule.id == rule_id) else {
                return;
            };
            if rule.conditions.len() <= 1 {
                state.rule_editor_error = Some("规则至少需要一个条件".into());
                cx.notify();
                return;
            }
            if index < rule.conditions.len() {
                rule.conditions.remove(index);
                state.rule_editor_error = None;
                state.save();
                cx.notify();
            }
        });
    }

    pub fn set_rule_priority(model: Entity<Self>, cx: &mut App, rule_id: String, priority: i32) {
        model.update(cx, |state, cx| {
            if let Some(rule) = state.rules.iter_mut().find(|rule| rule.id == rule_id) {
                rule.priority = priority;
                state.rules.sort_by_key(|rule| rule.priority);
                state.save();
                cx.notify();
            }
        });
    }

    pub fn set_routing_mode(&mut self, mode: narya_platform::ProxyMode, cx: &mut Context<Self>) {
        if self.kernel_running {
            return;
        }
        self.routing_mode = mode;
        self.save();
        cx.notify();
    }

    pub fn set_kernel_artifact_kernel(&mut self, kernel: String, cx: &mut Context<Self>) {
        self.kernel_artifact_kernel = kernel;
        self.save();
        self.kernel_error = None;
        cx.notify();
    }

    pub fn select_kernel_and_start(model: Entity<Self>, cx: &mut App, kernel: String) {
        let installed = model
            .read(cx)
            .kernels
            .iter()
            .any(|item| item.name == kernel && item.installed);
        if !installed {
            model.update(cx, |state, cx| {
                state.kernel_error = Some(format!("内核 {kernel} 尚未安装，不能切换"));
                cx.notify();
            });
            return;
        }
        if !model.read(cx).kernel_running {
            model.update(cx, |state, cx| {
                state.active_kernel = kernel;
                state.kernel_error = None;
                state.save();
                cx.notify();
            });
            Self::set_proxy_running(model, cx, true);
            return;
        }
        Self::switch_kernel(model, cx, kernel);
    }

    pub fn switch_kernel(model: Entity<Self>, cx: &mut App, kernel: String) {
        let (active_node, routing_mode, rules, groups, rule_sets) = {
            let state = model.read(cx);
            (
                state
                    .active_node_id
                    .as_ref()
                    .and_then(|id| state.nodes.iter().find(|node| node.id == *id))
                    .cloned(),
                state.routing_mode,
                state.rules.clone(),
                state.groups.clone(),
                state.rule_sets.clone(),
            )
        };
        let Some(node) = active_node else {
            model.update(cx, |state, cx| {
                state.kernel_error = Some("没有活动节点，无法热切换内核".into());
                cx.notify();
            });
            return;
        };
        model.update(cx, |state, cx| {
            state.kernel_operation = Some(format!("正在热切换到 {kernel}…"));
            state.kernel_error = None;
            cx.notify();
        });
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let model = model.clone();
            async move {
                let result = async {
                    let mut client = IpcClient::connect_default().await?;
                    let response = client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 7,
                            method: "SwitchKernel".into(),
                            params: serde_json::json!({
                                "kernel": kernel,
                                "node": node,
                                "routing": routing_plan(routing_mode),
                                "rules": rules,
                                "groups": groups,
                                "rule_sets": rule_sets,
                            }),
                        })
                        .await?;
                    if let Some(error) = response.error {
                        anyhow::bail!(error);
                    }
                    let response = client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 8,
                            method: "SetRoutingMode".into(),
                            params: serde_json::json!({ "mode": routing_mode.as_str() }),
                        })
                        .await?;
                    if let Some(error) = response.error {
                        anyhow::bail!(error);
                    }
                    Ok::<(), anyhow::Error>(())
                }
                .await;
                model.update(&mut cx, |state, cx| {
                    match result {
                        Ok(()) => {
                            state.active_kernel = kernel.clone();
                            state.kernel_running = true;
                            state.kernel_healthy = true;
                            state.kernel_operation = Some(format!("已切换到 {kernel}"));
                            state.save();
                        }
                        Err(error) => {
                            state.kernel_operation = None;
                            state.kernel_error = Some(format!("内核热切换失败：{error}"));
                        }
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn set_kernel_artifact_version(&mut self, version: String, cx: &mut Context<Self>) {
        self.kernel_artifact_version = version;
        self.kernel_error = None;
        cx.notify();
    }

    pub fn set_kernel_artifact_source(&mut self, source: String, cx: &mut Context<Self>) {
        self.kernel_artifact_source = source;
        self.kernel_error = None;
        cx.notify();
    }

    pub fn set_kernel_artifact_sha256(&mut self, sha256: String, cx: &mut Context<Self>) {
        self.kernel_artifact_sha256 = sha256;
        self.kernel_error = None;
        cx.notify();
    }

    pub fn set_kernel_artifact_signature(&mut self, signature: String, cx: &mut Context<Self>) {
        self.kernel_artifact_signature = signature;
        self.kernel_error = None;
        cx.notify();
    }

    pub fn set_kernel_artifact_public_key(&mut self, public_key: String, cx: &mut Context<Self>) {
        self.kernel_artifact_public_key = public_key;
        self.kernel_error = None;
        cx.notify();
    }

    pub fn set_kernel_catalog_source(&mut self, source: String, cx: &mut Context<Self>) {
        self.kernel_catalog_source = source;
        self.kernel_catalog_status = None;
        cx.notify();
    }

    pub fn set_kernel_catalog_trusted_key(&mut self, key: String, cx: &mut Context<Self>) {
        self.kernel_catalog_trusted_key = key;
        self.kernel_catalog_status = None;
        cx.notify();
    }

    pub fn refresh_kernel_catalog(model: Entity<Self>, cx: &mut App) {
        let (source, trusted_key) = {
            let state = model.read(cx);
            (
                state.kernel_catalog_source.trim().to_string(),
                state.kernel_catalog_trusted_key.trim().to_string(),
            )
        };
        if source.is_empty() || trusted_key.is_empty() {
            model.update(cx, |state, cx| {
                state.kernel_catalog_status = Some("清单来源和信任根公钥均为必填项".into());
                cx.notify();
            });
            return;
        }
        model.update(cx, |state, cx| {
            state.kernel_catalog_status = Some("正在验证内核发布清单…".into());
            cx.notify();
        });
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let result = async {
                    let mut client = IpcClient::connect_default().await?;
                    let response = client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 61,
                            method: "RefreshKernelCatalog".into(),
                            params: serde_json::json!({
                                "source": source,
                                "trusted_key": trusted_key,
                            }),
                        })
                        .await?;
                    if let Some(error) = response.error {
                        anyhow::bail!(error);
                    }
                    let result = response.result.unwrap_or_default();
                    let entries: Vec<KernelCatalogOption> =
                        serde_json::from_value(result.get("entries").cloned().unwrap_or_default())?;
                    let digest = result
                        .get("digest")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("unknown")
                        .to_string();
                    Ok::<(Vec<KernelCatalogOption>, String), anyhow::Error>((entries, digest))
                }
                .await;
                model.update(&mut cx, |state, cx| {
                    match result {
                        Ok((entries, digest)) => {
                            state.kernel_catalog_entries = entries;
                            state.kernel_catalog_status = Some(format!("清单已验证 · {digest}"));
                        }
                        Err(error) => {
                            state.kernel_catalog_status = Some(format!("清单验证失败：{error}"));
                            state.kernel_catalog_entries.clear();
                        }
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn set_rule_set_draft_id(&mut self, value: String, cx: &mut Context<Self>) {
        self.rule_set_draft_id = value;
        self.rule_set_error = None;
        cx.notify();
    }

    pub fn set_rule_set_draft_source(&mut self, value: String, cx: &mut Context<Self>) {
        self.rule_set_draft_source = value;
        self.rule_set_error = None;
        cx.notify();
    }

    pub fn set_rule_set_draft_version(&mut self, value: String, cx: &mut Context<Self>) {
        self.rule_set_draft_version = value;
        self.rule_set_error = None;
        cx.notify();
    }

    pub fn set_rule_set_draft_sha256(&mut self, value: String, cx: &mut Context<Self>) {
        self.rule_set_draft_sha256 = value;
        self.rule_set_error = None;
        cx.notify();
    }

    pub fn set_rule_set_draft_format(&mut self, value: String, cx: &mut Context<Self>) {
        self.rule_set_draft_format = value;
        self.rule_set_error = None;
        cx.notify();
    }

    pub fn set_rule_set_draft_signature(&mut self, value: String, cx: &mut Context<Self>) {
        self.rule_set_draft_signature = value;
        self.rule_set_error = None;
        cx.notify();
    }

    pub fn set_rule_set_draft_public_key(&mut self, value: String, cx: &mut Context<Self>) {
        self.rule_set_draft_public_key = value;
        self.rule_set_error = None;
        cx.notify();
    }

    pub fn add_rule_set(model: Entity<Self>, cx: &mut App) {
        let source = {
            let state = model.read(cx);
            narya_rules::RuleSetSource {
                id: state.rule_set_draft_id.trim().to_string(),
                source: state.rule_set_draft_source.trim().to_string(),
                version: state.rule_set_draft_version.trim().to_string(),
                sha256: state.rule_set_draft_sha256.trim().to_string(),
                format: parse_rule_set_format(&state.rule_set_draft_format),
                enabled: true,
                signature: state.rule_set_draft_signature.trim().to_string(),
                public_key: state.rule_set_draft_public_key.trim().to_string(),
            }
        };
        let remote = source.source.starts_with("https://");
        model.update(cx, |state, cx| {
            if let Err(error) = source.validate() {
                state.rule_set_error = Some(format!("规则集校验失败：{error}"));
                cx.notify();
                return;
            }
            if !remote
                && !source.source.starts_with("file://")
                && !std::path::Path::new(&source.source).is_absolute()
            {
                state.rule_set_error = Some("规则集必须使用绝对本地路径或 file:// 路径".into());
                cx.notify();
                return;
            }
            if state.rule_sets.iter().any(|item| item.id == source.id) {
                state.rule_set_error = Some(format!("规则集 ID 已存在：{}", source.id));
                cx.notify();
                return;
            }
            if remote {
                state.rule_set_error = Some(format!("正在验证并缓存 {}…", source.id));
                cx.notify();
                return;
            }
            state.rule_sets.push(source.clone());
            state
                .rule_sets
                .sort_by(|left, right| left.id.cmp(&right.id));
            state.rule_set_draft_id.clear();
            state.rule_set_draft_source.clear();
            state.rule_set_draft_version.clear();
            state.rule_set_draft_sha256.clear();
            state.rule_set_draft_format.clear();
            state.rule_set_draft_signature.clear();
            state.rule_set_draft_public_key.clear();
            state.rule_set_error = None;
            state.save();
            cx.notify();
        });
        if remote {
            let model = model.clone();
            cx.spawn(move |cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    let result = async {
                        let mut client = IpcClient::connect_default().await?;
                        let response = client
                            .send_request(IpcRequest {
                                version: PROTOCOL_VERSION,
                                id: 51,
                                method: "FetchRuleSet".into(),
                                params: serde_json::to_value(&source)?,
                            })
                            .await?;
                        if let Some(error) = response.error {
                            anyhow::bail!(error);
                        }
                        Ok::<(), anyhow::Error>(())
                    }
                    .await;
                    model.update(&mut cx, |state, cx| {
                        match result {
                            Ok(()) => {
                                if !state.rule_sets.iter().any(|item| item.id == source.id) {
                                    state.rule_sets.push(source.clone());
                                    state
                                        .rule_sets
                                        .sort_by(|left, right| left.id.cmp(&right.id));
                                    state.rule_set_draft_id.clear();
                                    state.rule_set_draft_source.clear();
                                    state.rule_set_draft_version.clear();
                                    state.rule_set_draft_sha256.clear();
                                    state.rule_set_draft_format.clear();
                                    state.rule_set_draft_signature.clear();
                                    state.rule_set_draft_public_key.clear();
                                    state.save();
                                }
                                state.rule_set_error = Some(format!("已验证并缓存 {}", source.id));
                            }
                            Err(error) => {
                                state.rule_set_error = Some(format!("规则集更新失败：{error}"))
                            }
                        }
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }

    pub fn remove_rule_set(model: Entity<Self>, cx: &mut App, rule_set_id: String) {
        model.update(cx, |state, cx| {
            let referenced = state.rules.iter().any(|rule| {
                rule.conditions.iter().any(|condition| {
                    matches!(condition, narya_rules::Condition::RuleSet(id) if id == &rule_set_id)
                })
            });
            if referenced {
                state.rule_set_error = Some(format!(
                    "规则集 {} 仍被规则引用，请先移除规则条件",
                    rule_set_id
                ));
                cx.notify();
                return;
            }
            state.rule_sets.retain(|item| item.id != rule_set_id);
            state.rule_set_error = None;
            state.save();
            cx.notify();
        });
    }

    pub fn set_rule_set_enabled(
        model: Entity<Self>,
        cx: &mut App,
        rule_set_id: String,
        enabled: bool,
    ) {
        model.update(cx, |state, cx| {
            if !enabled
                && state.rules.iter().any(|rule| {
                    rule.conditions.iter().any(
                        |condition| matches!(condition, narya_rules::Condition::RuleSet(id) if id == &rule_set_id),
                    )
                })
            {
                state.rule_set_error = Some(format!(
                    "规则集 {} 仍被规则引用，不能禁用；请先修改这些规则",
                    rule_set_id
                ));
                cx.notify();
                return;
            }
            if let Some(source) = state.rule_sets.iter_mut().find(|source| source.id == rule_set_id)
            {
                source.enabled = enabled;
                state.rule_set_error = None;
                state.save();
                cx.notify();
            }
        });
    }

    pub fn install_kernel(model: Entity<Self>, cx: &mut App) {
        let (
            kernel,
            version,
            source,
            sha256,
            signature,
            public_key,
            catalog_platform,
            catalog_architecture,
            upgrade,
        ) = {
            let state = model.read(cx);
            let kernel = state.kernel_artifact_kernel.trim().to_string();
            let version = state.kernel_artifact_version.trim().to_string();
            let source = state.kernel_artifact_source.trim().to_string();
            let catalog = state.kernel_catalog_entries.iter().find(|entry| {
                entry.kernel == kernel && entry.version == version && entry.source == source
            });
            let upgrade = state
                .kernels
                .iter()
                .any(|installed| installed.name == kernel && installed.installed);
            (
                kernel,
                version,
                source,
                state.kernel_artifact_sha256.trim().to_string(),
                state.kernel_artifact_signature.trim().to_string(),
                state.kernel_artifact_public_key.trim().to_string(),
                catalog
                    .map(|entry| entry.platform.clone())
                    .unwrap_or_default(),
                catalog
                    .map(|entry| entry.architecture.clone())
                    .unwrap_or_default(),
                upgrade,
            )
        };
        if kernel.is_empty() || version.is_empty() || source.is_empty() || sha256.is_empty() {
            model.update(cx, |state, cx| {
                state.kernel_error = Some("内核、版本、来源和 SHA-256 均为必填项".into());
                cx.notify();
            });
            return;
        }
        model.update(cx, |state, cx| {
            state.kernel_operation = Some(if upgrade {
                "正在升级内核…".into()
            } else {
                "正在安装内核…".into()
            });
            state.kernel_error = None;
            cx.notify();
        });

        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let model = model.clone();
            async move {
                let result = async {
                    let mut client = IpcClient::connect_default().await?;
                    let method = if upgrade {
                        "UpgradeKernel"
                    } else {
                        "InstallKernel"
                    };
                    let response = client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 20,
                            method: method.into(),
                            params: serde_json::json!({
                                "kernel": kernel,
                                "version": version,
                                "source": source,
                                "sha256": sha256,
                                "signature": signature,
                                "public_key": public_key,
                                "catalog_version": version,
                                "catalog_platform": catalog_platform,
                                "catalog_architecture": catalog_architecture,
                            }),
                        })
                        .await?;
                    if let Some(error) = response.error {
                        anyhow::bail!(error);
                    }
                    let result = response.result.unwrap_or_default();
                    let installed_version = result
                        .get("version")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("unknown")
                        .to_string();
                    Ok::<(String, String), anyhow::Error>((kernel, installed_version))
                }
                .await;

                match result {
                    Ok((kernel, version)) => {
                        model.update(&mut cx, |state, cx| {
                            if let Some(info) =
                                state.kernels.iter_mut().find(|info| info.name == kernel)
                            {
                                info.installed = true;
                                info.version = Some(version.clone());
                                info.state = "installed".into();
                                info.running = false;
                                info.healthy = false;
                                info.failure = None;
                            }
                            state.kernel_operation = Some(format!("已完成：{kernel} {version}"));
                            state.save();
                            cx.notify();
                        });
                    }
                    Err(error) => {
                        model.update(&mut cx, |state, cx| {
                            state.kernel_operation = None;
                            state.kernel_error = Some(error.to_string());
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();
    }

    pub fn install_kernel_named(model: Entity<Self>, cx: &mut App, kernel: String) {
        let upgrade = model
            .read(cx)
            .kernels
            .iter()
            .any(|info| info.name == kernel && info.installed);
        model.update(cx, |state, cx| {
            state.kernel_operation = Some(if upgrade {
                format!("正在从官方目录升级 {kernel}…")
            } else {
                format!("正在从官方目录安装 {kernel}…")
            });
            state.kernel_error = None;
            cx.notify();
        });
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let model = model.clone();
            async move {
                let result = async {
                    let mut client = IpcClient::connect_default().await?;
                    let method = if upgrade {
                        "UpgradeOfficialKernel"
                    } else {
                        "InstallOfficialKernel"
                    };
                    let response = client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 20,
                            method: method.into(),
                            params: serde_json::json!({"kernel": kernel}),
                        })
                        .await?;
                    if let Some(error) = response.error {
                        anyhow::bail!(error);
                    }
                    let result = response.result.unwrap_or_default();
                    let installed_version = result
                        .get("version")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("unknown")
                        .to_string();
                    Ok::<(String, String), anyhow::Error>((kernel, installed_version))
                }
                .await;
                match result {
                    Ok((kernel, version)) => {
                        model.update(&mut cx, |state, cx| {
                            if let Some(info) =
                                state.kernels.iter_mut().find(|info| info.name == kernel)
                            {
                                info.installed = true;
                                info.version = Some(version.clone());
                                info.state = "installed".into();
                                info.running = false;
                                info.healthy = false;
                                info.failure = None;
                            }
                            state.kernel_operation = Some(format!("已完成：{kernel} {version}"));
                            state.save();
                            cx.notify();
                        });
                    }
                    Err(error) => {
                        model.update(&mut cx, |state, cx| {
                            state.kernel_operation = None;
                            state.kernel_error = Some(error.to_string());
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();
    }

    pub fn uninstall_kernel(model: Entity<Self>, cx: &mut App, kernel: String) {
        let installed = model
            .read(cx)
            .kernels
            .iter()
            .any(|info| info.name == kernel && info.installed);
        if !installed {
            model.update(cx, |state, cx| {
                state.kernel_error = Some(format!("内核 {kernel} 当前未安装"));
                cx.notify();
            });
            return;
        }
        model.update(cx, |state, cx| {
            state.kernel_operation = Some(format!("正在卸载 {kernel}…"));
            state.kernel_error = None;
            cx.notify();
        });
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let model = model.clone();
            async move {
                let result = async {
                    let mut client = IpcClient::connect_default().await?;
                    let response = client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 21,
                            method: "UninstallKernel".into(),
                            params: serde_json::json!({"kernel": kernel}),
                        })
                        .await?;
                    if let Some(error) = response.error {
                        anyhow::bail!(error);
                    }
                    Ok::<(), anyhow::Error>(())
                }
                .await;
                match result {
                    Ok(()) => {
                        model.update(&mut cx, |state, cx| {
                            if let Some(info) =
                                state.kernels.iter_mut().find(|info| info.name == kernel)
                            {
                                info.installed = false;
                                info.version = None;
                                info.running = false;
                                info.healthy = false;
                                info.state = "not_installed".into();
                                info.failure = None;
                            }
                            state.kernel_operation = Some(format!("已卸载 {kernel}"));
                            state.save();
                            cx.notify();
                        });
                    }
                    Err(error) => model.update(&mut cx, |state, cx| {
                        state.kernel_operation = None;
                        state.kernel_error = Some(format!("卸载失败：{error}"));
                        cx.notify();
                    }),
                }
            }
        })
        .detach();
    }

    pub fn set_subscription_filter_text(&mut self, text: String, cx: &mut Context<Self>) {
        self.subscription_filter_text = text;
        cx.notify();
    }

    pub fn set_subscription_draft_name(&mut self, value: String, cx: &mut Context<Self>) {
        self.subscription_draft_name = value;
        self.subscription_error = None;
        cx.notify();
    }

    pub fn set_subscription_draft_url(&mut self, value: String, cx: &mut Context<Self>) {
        self.subscription_draft_url = value;
        self.subscription_error = None;
        cx.notify();
    }

    pub fn add_subscription(model: Entity<Self>, cx: &mut App) {
        let (name, url) = {
            let state = model.read(cx);
            (
                state.subscription_draft_name.trim().to_string(),
                state.subscription_draft_url.trim().to_string(),
            )
        };
        if name.is_empty() || !url.starts_with("https://") {
            model.update(cx, |state, cx| {
                state.subscription_error = Some("订阅名称必填，地址必须使用 HTTPS".into());
                cx.notify();
            });
            return;
        }
        if model
            .read(cx)
            .subscriptions
            .iter()
            .any(|item| item.url == url)
        {
            model.update(cx, |state, cx| {
                state.subscription_error = Some("该订阅地址已存在".into());
                cx.notify();
            });
            return;
        }
        let id = format!("sub-{}", chrono::Utc::now().timestamp_micros());
        model.update(cx, |state, cx| {
            state.subscriptions.push(narya_core::Subscription {
                id: id.clone(),
                name,
                url,
                icon: "globe".into(),
                node_count: 0,
                used_nodes: 0,
                update_time: "尚未更新".into(),
                traffic_used: 0.0,
                traffic_total: 0.0,
                expiration: "未知".into(),
                status: "等待更新".into(),
                format: None,
            });
            state.selected_subscription_id = Some(id.clone());
            state.subscription_draft_name.clear();
            state.subscription_draft_url.clear();
            state.subscription_error = None;
            state.save();
            cx.notify();
        });
        Self::refresh_subscription(model, cx, id);
    }

    pub fn select_subscription(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_subscription_id = Some(id);
        self.save();
        cx.notify();
    }

    pub fn set_subscription_tab(&mut self, tab: SubscriptionTab, cx: &mut Context<Self>) {
        self.active_subscription_tab = tab;
        cx.notify();
    }

    pub fn remove_rule(model: Entity<Self>, cx: &mut App, rule_id: String) {
        model.update(cx, |state, cx| {
            state.rules.retain(|rule| rule.id != rule_id);
            state.save();
            cx.notify();
        });
    }

    pub fn add_rule(model: Entity<Self>, cx: &mut App) {
        model.update(cx, |state, cx| {
            let next = state
                .rules
                .iter()
                .map(|rule| rule.priority)
                .max()
                .unwrap_or(0)
                + 10;
            let id = format!("custom-{}", state.rules.len() + 1);
            state.rules.push(narya_rules::Rule {
                id,
                priority: next,
                conditions: vec![narya_rules::Condition::DomainSuffix("example.com".into())],
                action: narya_rules::Action::Proxy("proxy".into()),
            });
            state.rules.sort_by_key(|rule| rule.priority);
            state.save();
            cx.notify();
        });
    }

    pub fn add_group(model: Entity<Self>, cx: &mut App) {
        model.update(cx, |state, cx| {
            let id = format!("proxy-{}", state.groups.len() + 1);
            state.groups.push(narya_rules::RoutingGroup {
                id,
                strategy: narya_rules::GroupStrategy::Select,
                members: vec!["proxy-node".into()],
                url: None,
                interval_secs: None,
            });
            state.group_error = None;
            state.save();
            cx.notify();
        });
    }

    pub fn remove_group(model: Entity<Self>, cx: &mut App, group_id: String) {
        if group_id == "proxy" {
            return;
        }
        model.update(cx, |state, cx| {
            let referenced = state.rules.iter().any(|rule| {
                matches!(&rule.action, narya_rules::Action::Proxy(target) if target == &group_id)
            });
            if referenced {
                state.group_error = Some(format!(
                    "分流组 {} 仍被规则引用，请先修改这些规则的动作",
                    group_id
                ));
                cx.notify();
                return;
            }
            state.groups.retain(|group| group.id != group_id);
            state.group_error = None;
            state.save();
            cx.notify();
        });
    }

    pub fn set_group_strategy(model: Entity<Self>, cx: &mut App, group_id: String, index: usize) {
        model.update(cx, |state, cx| {
            let strategy = match index {
                1 => narya_rules::GroupStrategy::UrlTest,
                2 => narya_rules::GroupStrategy::Fallback,
                3 => narya_rules::GroupStrategy::LoadBalance,
                _ => narya_rules::GroupStrategy::Select,
            };
            if let Some(group) = state.groups.iter_mut().find(|group| group.id == group_id) {
                group.strategy = strategy;
                if matches!(
                    strategy,
                    narya_rules::GroupStrategy::UrlTest | narya_rules::GroupStrategy::Fallback
                ) && group.url.is_none()
                {
                    group.url = Some("https://www.gstatic.com/generate_204".into());
                }
                if let Err(error) = group.validate() {
                    state.group_error = Some(format!("分流组校验失败：{error}"));
                } else {
                    state.group_error = None;
                    state.save();
                }
                cx.notify();
            }
        });
    }

    pub fn set_group_members(model: Entity<Self>, cx: &mut App, group_id: String, value: String) {
        model.update(cx, |state, cx| {
            if let Some(group) = state.groups.iter_mut().find(|group| group.id == group_id) {
                group.members = value
                    .split(',')
                    .map(str::trim)
                    .filter(|member| !member.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
                if let Err(error) = group.validate() {
                    state.group_error = Some(format!("分流组校验失败：{error}"));
                } else {
                    state.group_error = None;
                    state.save();
                }
                cx.notify();
            }
        });
    }

    pub fn set_group_url(model: Entity<Self>, cx: &mut App, group_id: String, value: String) {
        model.update(cx, |state, cx| {
            if let Some(group) = state.groups.iter_mut().find(|group| group.id == group_id) {
                group.url = (!value.trim().is_empty()).then(|| value.trim().to_string());
                if let Err(error) = group.validate() {
                    state.group_error = Some(format!("分流组校验失败：{error}"));
                } else {
                    state.group_error = None;
                    state.save();
                }
                cx.notify();
            }
        });
    }

    pub fn set_group_interval(model: Entity<Self>, cx: &mut App, group_id: String, value: String) {
        model.update(cx, |state, cx| {
            if let Some(group) = state.groups.iter_mut().find(|group| group.id == group_id) {
                group.interval_secs = if value.trim().is_empty() {
                    None
                } else {
                    match value.trim().parse::<u64>() {
                        Ok(interval) if interval > 0 => Some(interval),
                        _ => {
                            state.group_error = Some("URL 测试间隔必须是正整数秒".into());
                            cx.notify();
                            return;
                        }
                    }
                };
                state.group_error = None;
                state.save();
                cx.notify();
            }
        });
    }

    pub fn set_rule_io_path(&mut self, value: String, cx: &mut Context<Self>) {
        self.rule_io_path = value;
        self.rule_io_status = None;
        cx.notify();
    }

    pub fn export_rules(model: Entity<Self>, cx: &mut App) {
        let (path, bundle) = {
            let state = model.read(cx);
            (
                state.rule_io_path.trim().to_string(),
                RoutingBundle {
                    version: 1,
                    rules: state.rules.clone(),
                    groups: state.groups.clone(),
                    rule_sets: state.rule_sets.clone(),
                },
            )
        };
        let result = write_routing_bundle(&path, &bundle);
        model.update(cx, |state, cx| {
            state.rule_io_status = Some(match result {
                Ok(()) => format!("已导出到 {}", path),
                Err(error) => format!("导出失败：{error}"),
            });
            cx.notify();
        });
    }

    pub fn import_rules(model: Entity<Self>, cx: &mut App) {
        let path = model.read(cx).rule_io_path.trim().to_string();
        let result = read_routing_bundle(&path);
        model.update(cx, |state, cx| {
            match result {
                Ok(bundle) => {
                    state.rules = bundle.rules;
                    state.groups = bundle.groups;
                    state.rule_sets = bundle.rule_sets;
                    state.rule_io_status = Some(format!("已导入 {}", path));
                    state.rule_editor_error = None;
                    state.group_error = None;
                    state.save();
                }
                Err(error) => state.rule_io_status = Some(format!("导入失败：{error}")),
            }
            cx.notify();
        });
    }

    pub fn handle_notification(&mut self, notif: IpcNotification, cx: &mut Context<Self>) {
        match notif {
            IpcNotification::TrafficUpdate { down, up } => {
                if let Some(active_id) = &self.active_node_id {
                    if let Some(node) = self.nodes.iter_mut().find(|n| n.id == *active_id) {
                        node.download_speed = down;
                        node.upload_speed = up;
                    }
                }
            }
            IpcNotification::StatusUpdate { running } => {
                self.kernel_running = running;
            }
            IpcNotification::LogLine { level, message } => {
                let time = chrono::Local::now().format("%H:%M:%S").to_string();
                self.log_lines.push(LogMessage {
                    time,
                    level,
                    module: "core".to_string(), // extracted from message ideally, simplified here
                    content: message,
                });
                if self.log_lines.len() > 1000 {
                    self.log_lines.remove(0); // keep it bounded
                }
            }
            IpcNotification::KernelStatusUpdate { kernels } => {
                self.kernels = kernels;
            }
        }
        cx.notify();
    }

    pub fn select_node(&mut self, node_id: String, cx: &mut Context<Self>) {
        self.active_node_id = Some(node_id);
        self.save();
        cx.notify();
    }

    pub fn connect_node(model: Entity<Self>, cx: &mut App, node_id: String) {
        model.update(cx, |state, cx| {
            state.active_node_id = Some(node_id);
            state.save();
            cx.notify();
        });
        Self::set_proxy_running(model, cx, true);
    }

    pub fn toggle_proxy(model: Entity<Self>, cx: &mut App) {
        let next_state = !model.read(cx).kernel_running;
        Self::set_proxy_running(model, cx, next_state);
    }

    pub fn switch_routing_mode(model: Entity<Self>, cx: &mut App, mode: narya_platform::ProxyMode) {
        let (active_node, kernel, rules, groups, rule_sets) = {
            let state = model.read(cx);
            (
                state
                    .active_node_id
                    .as_ref()
                    .and_then(|id| state.nodes.iter().find(|node| node.id == *id))
                    .cloned(),
                state.active_kernel.clone(),
                state.rules.clone(),
                state.groups.clone(),
                state.rule_sets.clone(),
            )
        };
        let Some(node) = active_node else {
            model.update(cx, |state, cx| {
                state.kernel_error = Some("没有活动节点，无法切换路由模式".into());
                cx.notify();
            });
            return;
        };
        model.update(cx, |state, cx| {
            state.kernel_operation = Some(format!("正在切换到 {}…", mode.as_str()));
            state.kernel_error = None;
            cx.notify();
        });
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let model = model.clone();
            async move {
                let result = async {
                    let mut client = IpcClient::connect_default().await?;
                    let response = client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 9,
                            method: "SwitchKernel".into(),
                            params: serde_json::json!({
                                "kernel": kernel,
                                "node": node,
                                "routing": routing_plan(mode),
                                "rules": rules,
                                "groups": groups,
                                "rule_sets": rule_sets,
                            }),
                        })
                        .await?;
                    if let Some(error) = response.error {
                        anyhow::bail!(error);
                    }
                    let response = client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 10,
                            method: "SetRoutingMode".into(),
                            params: serde_json::json!({ "mode": mode.as_str() }),
                        })
                        .await?;
                    if let Some(error) = response.error {
                        anyhow::bail!(error);
                    }
                    Ok::<(), anyhow::Error>(())
                }
                .await;
                model.update(&mut cx, |state, cx| {
                    match result {
                        Ok(()) => {
                            state.routing_mode = mode;
                            state.routing_active = mode;
                            state.kernel_running = true;
                            state.kernel_healthy = true;
                            state.kernel_operation = Some(format!("已切换到 {}", mode.as_str()));
                            state.save();
                        }
                        Err(error) => {
                            state.kernel_operation = None;
                            state.kernel_error = Some(format!("路由模式切换失败：{error}"));
                        }
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn set_proxy_running(model: Entity<Self>, cx: &mut App, next_state: bool) {
        let (active_node, routing_mode, rules, groups, rule_sets, active_kernel) = {
            let state = model.read(cx);
            (
                state
                    .active_node_id
                    .as_ref()
                    .and_then(|id| state.nodes.iter().find(|n| n.id == *id))
                    .cloned(),
                state.routing_mode,
                state.rules.clone(),
                state.groups.clone(),
                state.rule_sets.clone(),
                state.active_kernel.clone(),
            )
        };

        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let model = model.clone();
            async move {
                let mut client = match IpcClient::connect_default().await {
                    Ok(client) => client,
                    Err(error) => {
                        model.update(&mut cx, |state, cx| {
                            state.kernel_operation = None;
                            state.kernel_error = Some(format!("无法连接 daemon：{error}"));
                            cx.notify();
                        });
                        return;
                    }
                };

                let method = if next_state {
                    "StartKernel"
                } else {
                    "StopKernel"
                };
                let params = if next_state {
                    let Some(node) = active_node else {
                        model.update(&mut cx, |state, cx| {
                            state.kernel_operation = None;
                            state.kernel_error = Some("没有活动节点，无法启动内核".into());
                            cx.notify();
                        });
                        return;
                    };
                    serde_json::json!({
                        "kernel": active_kernel,
                        "node": node,
                        "routing": routing_plan(routing_mode),
                        "rules": rules,
                        "groups": groups,
                        "rule_sets": rule_sets,
                    })
                } else {
                    serde_json::json!(null)
                };

                let req_kernel = IpcRequest {
                    version: PROTOCOL_VERSION,
                    id: 2,
                    method: method.to_string(),
                    params,
                };

                match client.send_request(req_kernel).await {
                    Ok(response) if response.error.is_none() => {}
                    Ok(response) => {
                        let error = response
                            .error
                            .unwrap_or_else(|| "unknown daemon error".to_string());
                        model.update(&mut cx, |state, cx| {
                            state.kernel_operation = None;
                            state.kernel_error = Some(format!("{method} 失败：{error}"));
                            cx.notify();
                        });
                        return;
                    }
                    Err(error) => {
                        model.update(&mut cx, |state, cx| {
                            state.kernel_operation = None;
                            state.kernel_error = Some(format!("{method} 传输失败：{error}"));
                            cx.notify();
                        });
                        return;
                    }
                }

                let route_response = if next_state {
                    client
                        .send_request(IpcRequest {
                            version: PROTOCOL_VERSION,
                            id: 1,
                            method: "SetRoutingMode".to_string(),
                            params: serde_json::json!({ "mode": routing_mode.as_str() }),
                        })
                        .await
                } else {
                    Ok(narya_ipc::IpcResponse {
                        version: PROTOCOL_VERSION,
                        id: 1,
                        result: Some(serde_json::json!(true)),
                        error: None,
                    })
                };

                match route_response {
                    Ok(response) if response.error.is_none() => {
                        model.update(&mut cx, |state, cx| {
                            state.kernel_running = next_state;
                            state.routing_active = if next_state {
                                routing_mode
                            } else {
                                narya_platform::ProxyMode::Disabled
                            };
                            state.kernel_healthy = next_state;
                            state.save();
                            cx.notify();
                        });
                    }
                    Ok(response) => {
                        let error = response
                            .error
                            .unwrap_or_else(|| "unknown daemon error".to_string());
                        model.update(&mut cx, |state, cx| {
                            state.kernel_operation = None;
                            state.kernel_error = Some(format!("SetRoutingMode 失败：{error}"));
                            cx.notify();
                        });
                        if next_state {
                            let _ = client
                                .send_request(IpcRequest {
                                    version: PROTOCOL_VERSION,
                                    id: 4,
                                    method: "StopKernel".to_string(),
                                    params: serde_json::json!(null),
                                })
                                .await;
                        }
                    }
                    Err(error) => {
                        model.update(&mut cx, |state, cx| {
                            state.kernel_operation = None;
                            state.kernel_error = Some(format!("SetRoutingMode 传输失败：{error}"));
                            cx.notify();
                        });
                        if next_state {
                            let _ = client
                                .send_request(IpcRequest {
                                    version: PROTOCOL_VERSION,
                                    id: 4,
                                    method: "StopKernel".to_string(),
                                    params: serde_json::json!(null),
                                })
                                .await;
                        }
                    }
                }
            }
        })
        .detach();
    }

    pub fn start_traffic_monitor(model: Entity<Self>, cx: &mut App) {
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx_inner = cx.clone();
            let model = model.clone();
            async move {
                loop {
                    // Try to connect to daemon
                    if let Ok(mut client) = IpcClient::connect_default().await {
                        println!("Connected to daemon IPC");
                        loop {
                            match client.next_notification().await {
                                Ok(notif) => {
                                    model.update(&mut cx_inner, |state, cx| {
                                        state.handle_notification(notif, cx);
                                    });
                                }
                                Err(e) => {
                                    eprintln!("IPC notification loop error: {}", e);
                                    break;
                                }
                            }
                        }
                    } else {
                        // A disconnected daemon is an unknown state, never a simulated connection.
                        cx_inner
                            .background_executor()
                            .timer(Duration::from_secs(1))
                            .await;
                        model.update(&mut cx_inner, |state, cx| {
                            state.kernel_running = false;
                            state.kernel_healthy = false;
                            state.routing_active = narya_platform::ProxyMode::Disabled;
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();
    }

    pub fn refresh_subscription(model: Entity<Self>, cx: &mut App, sub_id: String) {
        let url = model
            .read(cx)
            .subscriptions
            .iter()
            .find(|s| s.id == sub_id)
            .map(|s| s.url.clone());

        if let Some(url) = url {
            cx.spawn(move |cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                let model = model.clone();
                async move {
                    // Set status to "更新中..." (Updating...)
                    model.update(&mut cx, |state, cx| {
                        if let Some(sub) = state.subscriptions.iter_mut().find(|s| s.id == sub_id) {
                            sub.status = "更新中...".to_string();
                            cx.notify();
                        }
                    });

                    // Execute the network request inside a Tokio runtime since reqwest requires it
                    let fetch_result = cx
                        .background_executor()
                        .spawn(async move {
                            let rt = tokio::runtime::Runtime::new()
                                .expect("Failed to create Tokio runtime");
                            rt.block_on(async {
                                narya_subscription::fetch_remote_subscription(&url).await
                            })
                        })
                        .await;

                    let result = match fetch_result {
                        Ok(content) => {
                            narya_subscription::parse_subscription(&content).map_err(|error| {
                                subscription_refresh_failure_summary(
                                    "订阅解析失败",
                                    &error.to_string(),
                                )
                            })
                        }
                        Err(error) => Err(subscription_refresh_failure_summary(
                            "订阅下载失败",
                            &error.to_string(),
                        )),
                    };

                    match result {
                        Ok((format, mut new_nodes)) => {
                            for node in &mut new_nodes {
                                node.tag = Some(sub_id.clone());
                            }
                            model.update(&mut cx, |state, cx| {
                                if let Some(sub) =
                                    state.subscriptions.iter_mut().find(|s| s.id == sub_id)
                                {
                                    sub.status = "更新成功".to_string();
                                    sub.update_time = "刚刚".to_string();
                                    sub.node_count = new_nodes.len() as u32;
                                    sub.format = Some(format.as_str().to_string());
                                }

                                state
                                    .nodes
                                    .retain(|node| node.tag.as_deref() != Some(sub_id.as_str()));
                                state.nodes.extend(new_nodes);
                                if let Some(first_node) = state.nodes.first() {
                                    state.active_node_id = Some(first_node.id.clone());
                                }

                                state.save();
                                cx.notify();
                            });
                        }
                        Err(summary) => model.update(&mut cx, |state, cx| {
                            mark_subscription_refresh_failed(
                                &mut state.subscriptions,
                                &sub_id,
                                &summary,
                            );
                            state.subscription_error = Some(summary);
                            cx.notify();
                        }),
                    }
                }
            })
            .detach();
        }
    }

    pub fn fetch_kernel_status(model: Entity<Self>, cx: &mut App) {
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let model = model.clone();
            async move {
                if let Ok(mut client) = IpcClient::connect_default().await {
                    let request = IpcRequest {
                        version: PROTOCOL_VERSION,
                        id: 3,
                        method: "GetKernelStatus".to_string(),
                        params: serde_json::json!(null),
                    };
                    if let Ok(res) = client.send_request(request).await {
                        if let Some(result) = res.result {
                            if let Ok(kernels) =
                                serde_json::from_value::<Vec<narya_ipc::KernelInfo>>(result)
                            {
                                model.update(&mut cx, |state, cx| {
                                    state.kernels = kernels;
                                    state.kernel_healthy = state
                                        .kernels
                                        .iter()
                                        .any(|kernel| kernel.running && kernel.healthy);
                                    cx.notify();
                                });
                            }
                        }
                    }
                }
            }
        })
        .detach();
    }

    pub fn fetch_routing_status(model: Entity<Self>, cx: &mut App) {
        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let Ok(mut client) = IpcClient::connect_default().await else {
                    return;
                };
                let request = IpcRequest {
                    version: PROTOCOL_VERSION,
                    id: 5,
                    method: "GetRoutingStatus".to_string(),
                    params: serde_json::json!(null),
                };
                let Ok(response) = client.send_request(request).await else {
                    return;
                };
                let Some(result) = response.result else {
                    return;
                };
                let configured: Option<narya_platform::ProxyMode> = result
                    .get("configured_mode")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|mode| mode.parse().ok());
                let active: narya_platform::ProxyMode = result
                    .get("active_mode")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|mode| mode.parse().ok())
                    .unwrap_or(narya_platform::ProxyMode::Disabled);
                let healthy = result
                    .get("kernel_healthy")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                model.update(&mut cx, |state, cx| {
                    state.routing_configured = configured.map(|mode| mode.as_str().to_string());
                    state.routing_active = active;
                    state.kernel_healthy = healthy;
                    state.kernel_running = healthy && active != narya_platform::ProxyMode::Disabled;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn test_all_latency(model: Entity<Self>, cx: &mut App) {
        let nodes = model
            .read(cx)
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node.details.address.clone()))
            .collect::<Vec<_>>();
        let weak_model = model.downgrade();

        for (id, address) in nodes {
            let weak_model = weak_model.clone();

            // Clear current latency to show loading state
            model.update(cx, |state, cx| {
                if let Some(node) = state.nodes.iter_mut().find(|n| n.id == id) {
                    node.latency = None;
                }
                cx.notify();
            });

            cx.spawn(move |cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                let id = id.clone();
                async move {
                    let new_latency = crate::ipc::measure_tcp_latency(address).await.ok();

                    let _ = weak_model.update(&mut cx, |state, cx| {
                        if let Some(node) = state.nodes.iter_mut().find(|n| n.id == id) {
                            node.latency = new_latency;
                        }
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }

    pub fn load_or_default() -> Self {
        let persisted = Self::load_persisted();
        if !persisted.subscriptions.is_empty() {
            let active_kernel = if persisted.active_kernel.is_empty() {
                "sing-box".to_string()
            } else {
                persisted.active_kernel.clone()
            };
            return Self {
                active_node_id: persisted.active_node_id,
                nodes: persisted.nodes,
                subscriptions: persisted.subscriptions,
                // A persisted flag cannot prove that the daemon and kernel are healthy.
                // The live IPC status probe is the only source of truth after startup.
                kernel_running: false,
                filter_text: String::new(),
                rule_filter_text: String::new(),
                rule_action_filter: "all".to_string(),
                selected_subscription_id: persisted.selected_subscription_id,
                active_subscription_tab: SubscriptionTab::Overview,
                subscription_filter_text: String::new(),
                subscription_draft_name: String::new(),
                subscription_draft_url: String::new(),
                subscription_error: None,
                subscription_list_state: ListState::new(0, ListAlignment::Top, px(100.0)),
                log_lines: Vec::new(),
                kernels: Vec::new(),
                rules: if persisted.rules.is_empty() {
                    default_rules()
                } else {
                    persisted.rules
                },
                groups: if persisted.groups.is_empty() {
                    vec![narya_rules::RoutingGroup::default_proxy()]
                } else {
                    persisted.groups
                },
                rule_sets: persisted.rule_sets,
                routing_mode: narya_platform::ProxyMode::SystemProxy,
                routing_configured: None,
                routing_active: narya_platform::ProxyMode::Disabled,
                kernel_healthy: false,
                active_kernel: active_kernel.clone(),
                kernel_artifact_kernel: active_kernel,
                kernel_artifact_version: String::new(),
                kernel_artifact_source: String::new(),
                kernel_artifact_sha256: String::new(),
                kernel_artifact_signature: String::new(),
                kernel_artifact_public_key: String::new(),
                kernel_operation: None,
                kernel_error: None,
                kernel_catalog_source: String::new(),
                kernel_catalog_trusted_key: String::new(),
                kernel_catalog_status: None,
                kernel_catalog_entries: Vec::new(),
                rule_set_draft_id: String::new(),
                rule_set_draft_source: String::new(),
                rule_set_draft_version: String::new(),
                rule_set_draft_sha256: String::new(),
                rule_set_draft_format: String::new(),
                rule_set_draft_signature: String::new(),
                rule_set_draft_public_key: String::new(),
                rule_set_error: None,
                group_error: None,
                rule_editor_error: None,
                rule_io_path: String::new(),
                rule_io_status: None,
                settings_category: 0,
                setting_autostart: persisted.setting_autostart,
                setting_start_minimized: persisted.setting_start_minimized,
                setting_close_to_tray: persisted.setting_close_to_tray,
                setting_restore_proxy: persisted.setting_restore_proxy,
                setting_auto_update: persisted.setting_auto_update,
                appearance_mode: persisted.appearance_mode.min(2),
            };
        }

        #[cfg(feature = "demo-data")]
        let nodes = vec![
            narya_core::Node {
                id: "hk-01".to_string(),
                name: "香港 HK 01".to_string(),
                country_code: "HK".to_string(),
                protocol: "Shadowsocks".to_string(),
                tag: Some("推荐".to_string()),
                latency: Some(48),
                usage_pct: 23,
                download_speed: 12.45,
                upload_speed: 3.26,
                details: narya_core::NodeDetails {
                    address: "hkg01.narya.net:443".to_string(),
                    encryption: "2022-blake3-aes-128-gcm:demo-password".to_string(),
                    udp: true,
                    tls: false,
                    skip_cert_verify: false,
                    transport: "tcp".to_string(),
                    last_test: "刚刚".to_string(),
                    options: narya_core::ProtocolOptions::default(),
                },
            },
            narya_core::Node {
                id: "sg-01".to_string(),
                name: "新加坡 SG 01".to_string(),
                country_code: "SG".to_string(),
                protocol: "Hysteria2".to_string(),
                tag: None,
                latency: Some(55),
                usage_pct: 20,
                download_speed: 11.2,
                upload_speed: 4.1,
                details: narya_core::NodeDetails {
                    address: "sgp01.narya.net:443".to_string(),
                    encryption: "password:demo-hysteria2-password".to_string(),
                    udp: true,
                    tls: true,
                    skip_cert_verify: true,
                    transport: "udp".to_string(),
                    last_test: "1 min ago".to_string(),
                    options: narya_core::ProtocolOptions::default(),
                },
            },
            narya_core::Node {
                id: "jp-01".to_string(),
                name: "日本 JP 01".to_string(),
                country_code: "JP".to_string(),
                protocol: "Vmess".to_string(),
                tag: None,
                latency: Some(62),
                usage_pct: 18,
                download_speed: 9.8,
                upload_speed: 3.2,
                details: narya_core::NodeDetails {
                    address: "tyo01.narya.net:443".to_string(),
                    encryption: "uuid:00000000-0000-0000-0000-000000000003".to_string(),
                    udp: true,
                    tls: true,
                    skip_cert_verify: false,
                    transport: "tcp".to_string(),
                    last_test: "5 mins ago".to_string(),
                    options: narya_core::ProtocolOptions::default(),
                },
            },
            narya_core::Node {
                id: "us-01".to_string(),
                name: "美国 US 01".to_string(),
                country_code: "US".to_string(),
                protocol: "VLESS Reality".to_string(),
                tag: None,
                latency: Some(110),
                usage_pct: 32,
                download_speed: 8.7,
                upload_speed: 2.9,
                details: narya_core::NodeDetails {
                    address: "lax01.narya.net:443".to_string(),
                    encryption: "uuid:00000000-0000-0000-0000-000000000004".to_string(),
                    udp: true,
                    tls: true,
                    skip_cert_verify: false,
                    transport: "grpc".to_string(),
                    last_test: "10 mins ago".to_string(),
                    options: narya_core::ProtocolOptions::default(),
                },
            },
        ];

        #[cfg(not(feature = "demo-data"))]
        let nodes: Vec<narya_core::Node> = Vec::new();

        #[cfg(feature = "demo-data")]
        let subscriptions = vec![
            narya_core::Subscription {
                id: "sub-0".to_string(),
                name: "Multi-Proxy Fetcher".to_string(),
                url: "https://raw.githubusercontent.com/4n0nymou3/multi-proxy-config-fetcher/refs/heads/main/configs/proxy_configs.txt".to_string(),
                icon: "globe".to_string(),
                node_count: 450,
                used_nodes: 12,
                update_time: "3 小时前".to_string(),
                traffic_used: 12.5,
                traffic_total: 5000.0,
                expiration: "2099-12-31".to_string(),
                status: "有效".to_string(),
                format: Some("V2Ray Base64".to_string()),
            },
            narya_core::Subscription {
                id: "sub-1".to_string(),
                name: "Ermaozi Clash".to_string(),
                url: "https://raw.githubusercontent.com/ermaozi/get_subscribe/main/subscribe/clash.yml".to_string(),
                icon: "plane".to_string(),
                node_count: 128,
                used_nodes: 45,
                update_time: "刚刚".to_string(),
                traffic_used: 156.4,
                traffic_total: 1024.0,
                expiration: "永久有效".to_string(),
                status: "当前使用".to_string(),
                format: Some("Clash YAML".to_string()),
            },
            narya_core::Subscription {
                id: "sub-2".to_string(),
                name: "ChromeGo Merge".to_string(),
                url: "https://raw.githubusercontent.com/Misaka-blog/chromego_merge/refs/heads/main/sub/merged_proxies_new.yaml".to_string(),
                icon: "chrome".to_string(),
                node_count: 860,
                used_nodes: 8,
                update_time: "1 天前".to_string(),
                traffic_used: 45.2,
                traffic_total: 10000.0,
                expiration: "永久有效".to_string(),
                status: "有效".to_string(),
                format: Some("Clash YAML".to_string()),
            },
            narya_core::Subscription {
                id: "sub-3".to_string(),
                name: "Cmlius Auto Worker".to_string(),
                url: "https://sub.cmliussss.workers.dev/auto".to_string(),
                icon: "zap".to_string(),
                node_count: 230,
                used_nodes: 15,
                update_time: "12 小时前".to_string(),
                traffic_used: 88.0,
                traffic_total: 2048.0,
                expiration: "2027-01-01".to_string(),
                status: "有效".to_string(),
                format: Some("Sing-box JSON".to_string()),
            },
            narya_core::Subscription {
                id: "sub-4".to_string(),
                name: "Mahsa Free Config".to_string(),
                url: "https://raw.githubusercontent.com/mahsanet/MahsaFreeConfig/refs/heads/main/mci/sub_1.txt".to_string(),
                icon: "shield".to_string(),
                node_count: 56,
                used_nodes: 2,
                update_time: "2 小时前".to_string(),
                traffic_used: 5.1,
                traffic_total: 100.0,
                expiration: "2026-12-31".to_string(),
                status: "有效".to_string(),
                format: Some("V2Ray Base64".to_string()),
            },
        ];

        #[cfg(not(feature = "demo-data"))]
        let subscriptions: Vec<narya_core::Subscription> = Vec::new();
        let active_node_id = nodes.first().map(|node: &narya_core::Node| node.id.clone());
        let selected_subscription_id = subscriptions.first().map(|item| item.id.clone());
        let subscription_count = subscriptions.len();

        let state = Self {
            active_node_id,
            nodes,
            subscriptions,
            kernel_running: false,
            filter_text: String::new(),
            rule_filter_text: String::new(),
            rule_action_filter: "all".to_string(),
            selected_subscription_id,
            active_subscription_tab: SubscriptionTab::Overview,
            subscription_filter_text: String::new(),
            subscription_draft_name: String::new(),
            subscription_draft_url: String::new(),
            subscription_error: None,
            subscription_list_state: ListState::new(
                subscription_count,
                ListAlignment::Top,
                px(100.0),
            ),
            log_lines: Vec::new(),
            kernels: vec![
                narya_ipc::KernelInfo {
                    name: "sing-box".to_string(),
                    installed: false,
                    version: None,
                    running: false,
                    healthy: false,
                    state: "not_installed".to_string(),
                    failure: None,
                },
                narya_ipc::KernelInfo {
                    name: "mihomo".to_string(),
                    installed: false,
                    version: None,
                    running: false,
                    healthy: false,
                    state: "not_installed".to_string(),
                    failure: None,
                },
                narya_ipc::KernelInfo {
                    name: "xray-core".to_string(),
                    installed: false,
                    version: None,
                    running: false,
                    healthy: false,
                    state: "not_installed".to_string(),
                    failure: None,
                },
                narya_ipc::KernelInfo {
                    name: "v2ray-core".to_string(),
                    installed: false,
                    version: None,
                    running: false,
                    healthy: false,
                    state: "not_installed".to_string(),
                    failure: None,
                },
            ],
            rules: default_rules(),
            groups: vec![narya_rules::RoutingGroup::default_proxy()],
            rule_sets: Vec::new(),
            routing_mode: narya_platform::ProxyMode::SystemProxy,
            routing_configured: None,
            routing_active: narya_platform::ProxyMode::Disabled,
            kernel_healthy: false,
            active_kernel: "sing-box".into(),
            kernel_artifact_kernel: "sing-box".into(),
            kernel_artifact_version: String::new(),
            kernel_artifact_source: String::new(),
            kernel_artifact_sha256: String::new(),
            kernel_artifact_signature: String::new(),
            kernel_artifact_public_key: String::new(),
            kernel_operation: None,
            kernel_error: None,
            kernel_catalog_source: String::new(),
            kernel_catalog_trusted_key: String::new(),
            kernel_catalog_status: None,
            kernel_catalog_entries: Vec::new(),
            rule_set_draft_id: String::new(),
            rule_set_draft_source: String::new(),
            rule_set_draft_version: String::new(),
            rule_set_draft_sha256: String::new(),
            rule_set_draft_format: String::new(),
            rule_set_draft_signature: String::new(),
            rule_set_draft_public_key: String::new(),
            rule_set_error: None,
            group_error: None,
            rule_editor_error: None,
            rule_io_path: String::new(),
            rule_io_status: None,
            settings_category: 0,
            setting_autostart: false,
            setting_start_minimized: false,
            setting_close_to_tray: true,
            setting_restore_proxy: true,
            setting_auto_update: true,
            appearance_mode: 0,
        };
        state.save();
        state
    }
}

fn routing_plan(mode: narya_platform::ProxyMode) -> narya_platform::RoutingPlan {
    narya_platform::RoutingPlan {
        mode,
        system_proxy: narya_platform::SystemProxyPlan {
            http_host: "127.0.0.1".into(),
            http_port: 2080,
            socks_host: "127.0.0.1".into(),
            socks_port: 1080,
            bypass_domains: vec!["localhost".into(), "127.0.0.1".into(), "::1".into()],
        },
        tun: (mode == narya_platform::ProxyMode::Tun).then(|| narya_platform::TunPlan {
            interface_name: "narya0".into(),
            address: "172.19.0.1/30".into(),
            auto_route: true,
            strict_route: true,
            hijack_dns: true,
            excluded_routes: vec!["127.0.0.0/8".into(), "224.0.0.0/4".into()],
        }),
        dns: narya_platform::DnsPlan {
            resolver: vec!["https://1.1.1.1/dns-query".into()],
            direct: vec!["local".into()],
            proxy: vec!["https://1.1.1.1/dns-query".into()],
            hijack: mode == narya_platform::ProxyMode::Tun,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_contains_no_fabricated_runtime_data() {
        let state = AppState::load_or_default();
        assert!(state.nodes.is_empty());
        assert!(state.subscriptions.is_empty());
        assert!(state.active_node_id.is_none());
    }
}
