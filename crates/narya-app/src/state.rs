use crate::ipc::IpcClient;
use gpui::*;
use narya_core;
use narya_ipc::{IpcNotification, IpcRequest};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

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
    pub selected_subscription_id: Option<String>,
}

pub struct AppState {
    pub nodes: Vec<narya_core::Node>,
    pub subscriptions: Vec<narya_core::Subscription>,
    pub active_node_id: Option<String>,
    pub kernel_running: bool,
    pub filter_text: String,

    // Subscription specific state
    pub selected_subscription_id: Option<String>,
    pub active_subscription_tab: SubscriptionTab,
    pub subscription_filter_text: String,

    pub subscription_list_state: ListState,

    pub log_lines: Vec<LogMessage>,
    pub kernels: Vec<narya_ipc::KernelInfo>,
}

impl AppState {
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
            selected_subscription_id: self.selected_subscription_id.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&persisted) {
            let _ = std::fs::write(Self::config_path(), json);
        }
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

    pub fn set_subscription_filter_text(&mut self, text: String, cx: &mut Context<Self>) {
        self.subscription_filter_text = text;
        cx.notify();
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

    pub fn set_proxy_running(model: Entity<Self>, cx: &mut App, next_state: bool) {
        let active_node = model
            .read(cx)
            .active_node_id
            .as_ref()
            .and_then(|id| model.read(cx).nodes.iter().find(|n| n.id == *id))
            .cloned();

        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            let model = model.clone();
            async move {
                let Some(mut client) = IpcClient::connect_default().await.ok() else {
                    return;
                };

                let method = if next_state {
                    "StartKernel"
                } else {
                    "StopKernel"
                };
                let params = if next_state {
                    let Some(node) = active_node else {
                        return;
                    };
                    serde_json::to_value(node).unwrap_or(serde_json::json!(null))
                } else {
                    serde_json::json!(null)
                };

                let req_kernel = IpcRequest {
                    id: 2,
                    method: method.to_string(),
                    params,
                };

                match client.send_request(req_kernel).await {
                    Ok(response) if response.error.is_none() => {}
                    Ok(response) => {
                        eprintln!(
                            "{} failed: {}",
                            method,
                            response
                                .error
                                .unwrap_or_else(|| "unknown daemon error".to_string())
                        );
                        return;
                    }
                    Err(error) => {
                        eprintln!("{} transport failed: {}", method, error);
                        return;
                    }
                }

                let req_proxy = IpcRequest {
                    id: 1,
                    method: "SetSystemProxy".to_string(),
                    params: serde_json::json!(next_state),
                };

                match client.send_request(req_proxy).await {
                    Ok(response) if response.error.is_none() => {
                        let _ = model.update(&mut cx, |state, cx| {
                            state.kernel_running = next_state;
                            state.save();
                            cx.notify();
                        });
                    }
                    Ok(response) => {
                        eprintln!(
                            "SetSystemProxy failed: {}",
                            response
                                .error
                                .unwrap_or_else(|| "unknown daemon error".to_string())
                        );
                    }
                    Err(error) => {
                        eprintln!("SetSystemProxy transport failed: {}", error);
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
                                    let _ = model.update(&mut cx_inner, |state, cx| {
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
                        // Fallback to simulation if daemon is offline
                        cx_inner
                            .background_executor()
                            .timer(Duration::from_secs(1))
                            .await;
                        let _ = model.update(&mut cx_inner, |state, cx| {
                            if let Some(active_id) = &state.active_node_id {
                                if let Some(node) =
                                    state.nodes.iter_mut().find(|n| n.id == *active_id)
                                {
                                    node.download_speed = (node.download_speed
                                        + (rand::random::<f32>() - 0.5) * 2.0)
                                        .max(0.0);
                                    node.upload_speed = (node.upload_speed
                                        + (rand::random::<f32>() - 0.5) * 1.0)
                                        .max(0.0);
                                }
                            }
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
                    let _ = model.update(&mut cx, |state, cx| {
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

                    if let Ok(content) = fetch_result {
                        if let Ok((format, new_nodes)) =
                            narya_subscription::parse_subscription(&content)
                        {
                            let _ = model.update(&mut cx, |state, cx| {
                                if let Some(sub) =
                                    state.subscriptions.iter_mut().find(|s| s.id == sub_id)
                                {
                                    sub.status = "更新成功".to_string();
                                    sub.update_time = "刚刚".to_string();
                                    sub.node_count = new_nodes.len() as u32;
                                    sub.format = Some(format.as_str().to_string());
                                }

                                // Replace nodes for this demo. In a real app, we'd tag them and keep others.
                                state.nodes = new_nodes;
                                if let Some(first_node) = state.nodes.first() {
                                    state.active_node_id = Some(first_node.id.clone());
                                }

                                state.save();
                                cx.notify();
                            });
                            return;
                        }
                    }

                    // If failed
                    let _ = model.update(&mut cx, |state, cx| {
                        if let Some(sub) = state.subscriptions.iter_mut().find(|s| s.id == sub_id) {
                            sub.status = "更新失败".to_string();
                            cx.notify();
                        }
                    });
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
                        id: 3,
                        method: "GetKernelStatus".to_string(),
                        params: serde_json::json!(null),
                    };
                    if let Ok(res) = client.send_request(request).await {
                        if let Some(result) = res.result {
                            if let Ok(kernels) =
                                serde_json::from_value::<Vec<narya_ipc::KernelInfo>>(result)
                            {
                                let _ = model.update(&mut cx, |state, cx| {
                                    state.kernels = kernels;
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

    pub fn install_kernel(_model: Entity<Self>, _cx: &mut App, _kernel_name: String) {
        eprintln!("Kernel installation is not implemented yet; refusing to report fake success");
    }

    pub fn test_all_latency(model: Entity<Self>, cx: &mut App) {
        // Collect IDs first to avoid borrow checker issues with model.read(cx) and model.update(cx)
        let ids: Vec<String> = model.read(cx).nodes.iter().map(|n| n.id.clone()).collect();
        let weak_model = model.downgrade();

        for id in ids {
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
                    // Simulate network delay
                    let delay = 500 + rand::random::<u64>() % 2000;
                    cx.background_executor()
                        .timer(Duration::from_millis(delay))
                        .await;
                    let new_latency = Some(20 + rand::random::<u32>() % 200);

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

    pub fn init_or_mock() -> Self {
        let persisted = Self::load_persisted();
        if !persisted.subscriptions.is_empty() {
            return Self {
                active_node_id: persisted.active_node_id,
                nodes: persisted.nodes,
                subscriptions: persisted.subscriptions,
                kernel_running: persisted.kernel_running,
                filter_text: String::new(),
                selected_subscription_id: persisted.selected_subscription_id,
                active_subscription_tab: SubscriptionTab::Overview,
                subscription_filter_text: String::new(),
                subscription_list_state: ListState::new(0, ListAlignment::Top, px(100.0)),
                log_lines: Vec::new(),
                kernels: Vec::new(),
            };
        }

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
                    encryption: "none".to_string(),
                    udp: true,
                    tls: true,
                    skip_cert_verify: true,
                    transport: "udp".to_string(),
                    last_test: "1 min ago".to_string(),
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
                    encryption: "auto".to_string(),
                    udp: true,
                    tls: true,
                    skip_cert_verify: false,
                    transport: "tcp".to_string(),
                    last_test: "5 mins ago".to_string(),
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
                    encryption: "none".to_string(),
                    udp: true,
                    tls: true,
                    skip_cert_verify: false,
                    transport: "grpc".to_string(),
                    last_test: "10 mins ago".to_string(),
                },
            },
        ];

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

        let state = Self {
            active_node_id: Some("hk-01".to_string()),
            nodes,
            subscriptions,
            kernel_running: false,
            filter_text: String::new(),
            selected_subscription_id: Some("sub-1".to_string()),
            active_subscription_tab: SubscriptionTab::Overview,
            subscription_filter_text: String::new(),
            subscription_list_state: ListState::new(5, ListAlignment::Top, px(100.0)),
            log_lines: Vec::new(),
            kernels: vec![
                narya_ipc::KernelInfo {
                    name: "sing-box".to_string(),
                    installed: false,
                    version: None,
                    running: false,
                },
                narya_ipc::KernelInfo {
                    name: "mihomo".to_string(),
                    installed: false,
                    version: None,
                    running: false,
                },
                narya_ipc::KernelInfo {
                    name: "xray-core".to_string(),
                    installed: false,
                    version: None,
                    running: false,
                },
            ],
        };
        state.save();
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_mock() {
        let state = AppState::init_or_mock();
        assert!(!state.nodes.is_empty());
        assert!(!state.subscriptions.is_empty());
        assert!(state.active_node_id.is_some());
    }
}
