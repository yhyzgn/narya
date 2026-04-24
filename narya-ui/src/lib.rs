use gpui::prelude::*;
use gpui::*;
use gpui_platform::application;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

mod components;
mod ipc_client;
mod models;

use crate::components::proxy_list::ProxyList;
use crate::components::rule_panel::RulePanel;
use crate::components::traffic_chart::TrafficChart;
use crate::models::connection::{ConnectionStore, SharedConnectionStore};
use crate::models::profile::{ProfileStore, ProxyNode, SharedProfileStore};
use crate::models::rule::{AppInfo, RuleStore, SharedRuleStore};
use crate::models::traffic::{SharedTrafficStore, TrafficData, TrafficStore};
use config::parser::SubscriptionParser;

pub struct Workspace {
    selected_tab: usize,
    traffic_store: SharedTrafficStore,
    profile_store: SharedProfileStore,
    rule_store: SharedRuleStore,
    connection_store: SharedConnectionStore,
    start_time: std::time::Instant,
    focus_handle: FocusHandle,
}

impl EventEmitter<()> for Workspace {}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let store = Arc::new(RwLock::new(TrafficStore::new(60)));
        let store_clone = store.clone();

        let profile_store = Arc::new(RwLock::new(ProfileStore::new(
            "https://jsjc.cfd/api/v1/client/subscribe?token=a6db043ed2bd5771205036c514290aa0"
                .to_string(),
        )));

        let rule_store = Arc::new(RwLock::new(RuleStore::new()));
        let connection_store = Arc::new(RwLock::new(ConnectionStore::new()));
        let conn_store_clone = connection_store.clone();

        // 流量数据模拟轮询
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(1000));
                let data = TrafficData {
                    up: (rand::random::<f32>() * 50.0).abs(),
                    down: (rand::random::<f32>() * 200.0).abs(),
                };
                store_clone.write().push(data);
            }
        });

        // 活跃连接轮询 (IPC)
        utils::TOKIO_RUNTIME.spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(2000)).await;
                if let Ok(response) = ipc_client::send_command("get_connections").await {
                    if let Ok(conns) =
                        serde_json::from_str::<Vec<api::tracker::ConnectionMeta>>(&response)
                    {
                        conn_store_clone.write().active_connections = conns;
                    }
                }
            }
        });

        Self {
            selected_tab: 0,
            traffic_store: store,
            profile_store,
            rule_store,
            connection_store,
            start_time: std::time::Instant::now(),
            focus_handle: cx.focus_handle(),
        }
    }

    fn select_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_tab = index;
        if index == 3 {
            self.refresh_apps(window, cx);
        }
        cx.notify();
    }

    fn refresh_apps(&self, window: &mut Window, cx: &mut Context<Self>) {
        let rule_store = self.rule_store.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        utils::TOKIO_RUNTIME.spawn(async move {
            let response = ipc_client::send_command("get_apps").await;
            if let Ok(resp) = response {
                if let Ok(apps) = serde_json::from_str::<Vec<api::tracker::AppIdentity>>(&resp) {
                    let _ = tx.send(apps);
                }
            }
        });

        fn poll(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>, rx: std::sync::mpsc::Receiver<Vec<api::tracker::AppIdentity>>, rule_store: SharedRuleStore) {
            if let Ok(apps) = rx.try_recv() {
                let mut store = rule_store.write();
                let current_assigned: std::collections::HashSet<String> = store
                    .direct
                    .iter()
                    .map(|a| a.id.clone())
                    .chain(store.proxy.iter().map(|a| a.id.clone()))
                    .collect();

                store.unassigned = apps
                    .into_iter()
                    .filter(|app| !current_assigned.contains(&app.identifier))
                    .map(|app| AppInfo {
                        id: app.identifier,
                        name: app.name,
                        icon: app.icon_path,
                    })
                    .collect();
                cx.notify();
            } else {
                cx.on_next_frame(window, move |workspace, window, cx| {
                    poll(workspace, window, cx, rx, rule_store);
                });
            }
        }

        cx.on_next_frame(window, move |workspace, window, cx| {
            poll(workspace, window, cx, rx, rule_store);
        });
    }

    fn fetch_subscription(&self, window: &mut Window, cx: &mut Context<Self>) {
        {
            let mut store = self.profile_store.write();
            store.is_loading = true;
            store.last_error = None;
        }
        let profile_store = self.profile_store.clone();
        let url = profile_store.read().url.clone();
        let (tx, rx) = std::sync::mpsc::channel();

        utils::TOKIO_RUNTIME.spawn(async move {
            let result = SubscriptionParser::fetch_and_parse(&url).await;
            if let Ok(ref conf) = result {
                if let Ok(json) = serde_json::to_string(conf) {
                    let cmd = format!("update_config {}", json);
                    let _ = ipc_client::send_command(&cmd).await;
                }
            }
            let _ = tx.send(result);
        });

        fn poll(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>, rx: std::sync::mpsc::Receiver<anyhow::Result<config::model::NaryaConfig>>, profile_store: SharedProfileStore) {
            if let Ok(result) = rx.try_recv() {
                let mut p_store = profile_store.write();
                p_store.is_loading = false;
                match result {
                    Ok(conf) => {
                        let nodes: Vec<ProxyNode> = conf
                            .proxies
                            .iter()
                            .map(|p| ProxyNode {
                                name: p.name.clone(),
                                protocol: p.proxy_type.clone(),
                                delay: None,
                                server: p.server.clone(),
                                port: p.port,
                            })
                            .collect();
                        p_store.nodes = nodes;
                        drop(p_store);
                        // 触发测速
                        workspace.test_all_latencies(window, cx);
                    }
                    Err(e) => {
                        p_store.last_error = Some(e.to_string());
                    }
                }
                cx.notify();
            } else {
                cx.on_next_frame(window, move |workspace, window, cx| {
                    poll(workspace, window, cx, rx, profile_store);
                });
            }
        }

        cx.on_next_frame(window, move |workspace, window, cx| {
            poll(workspace, window, cx, rx, profile_store);
        });
    }

    pub fn sync_proxy_selection(&self, name: &str) {
        let cmd = format!("select_proxy {}", name);
        utils::TOKIO_RUNTIME.spawn(async move {
            let _ = ipc_client::send_command(&cmd).await;
        });
    }

    async fn tcp_ping(server: String, port: u16) -> Option<u64> {
        let start = std::time::Instant::now();
        let addr = format!("{}:{}", server, port);
        match tokio::time::timeout(
            Duration::from_secs(3),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => Some(start.elapsed().as_millis() as u64),
            _ => None,
        }
    }

    fn test_all_latencies(&self, window: &mut Window, cx: &mut Context<Self>) {
        let profile_store = self.profile_store.clone();

        let nodes: Vec<(String, String, u16)> = {
            profile_store
                .read()
                .nodes
                .iter()
                .map(|n| (n.name.clone(), n.server.clone(), n.port))
                .collect()
        };
        for (node_name, server, port) in nodes {
            let p_store = profile_store.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            utils::TOKIO_RUNTIME.spawn(async move {
                let delay = Self::tcp_ping(server, port).await;
                let _ = tx.send(delay);
            });

            fn poll(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>, rx: std::sync::mpsc::Receiver<Option<u64>>, p_store: SharedProfileStore, node_name: String) {
                if let Ok(delay) = rx.try_recv() {
                    let mut store = p_store.write();
                    if let Some(node) = store.nodes.iter_mut().find(|n| n.name == node_name) {
                        node.delay = delay;
                    }
                    cx.notify();
                } else {
                    cx.on_next_frame(window, move |workspace, window, cx| {
                        poll(workspace, window, cx, rx, p_store, node_name);
                    });
                }
            }

            cx.on_next_frame(window, move |workspace, window, cx| {
                poll(workspace, window, cx, rx, p_store, node_name);
            });
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let entity = cx.entity().clone();
        let selected_tab = self.selected_tab;

        if selected_tab == 3 {
            _window.focus(&self.focus_handle, cx);
            cx.on_next_frame(_window, |_, _, cx| cx.notify());
        }

        div()
            .size_full()
            .flex()
            .bg(rgb(0x141414))
            .track_focus(&self.focus_handle)
            .on_key_down(move |event, _, cx| {
                if selected_tab == 3 {
                    let key: &str = event.keystroke.key.as_ref();
                    entity.update(cx, |workspace, cx| {
                        let mut store = workspace.rule_store.write();
                        if key.len() == 1 {
                            store.search_query.push_str(&key.to_lowercase());
                            cx.notify();
                        } else if key == "backspace" {
                            store.search_query.pop();
                            cx.notify();
                        } else if key == "escape" {
                            store.search_query.clear();
                            cx.notify();
                        }
                    });
                }
            })
            .child(
                div()
                    .w_56()
                    .h_full()
                    .bg(rgb(0x1d1d1d))
                    .border_r_1()
                    .border_color(rgb(0x303030))
                    .flex()
                    .flex_col()
                    .p_2()
                    .child(
                        div()
                            .p_4()
                            .mb_4()
                            .text_xl()
                            .text_color(rgb(0x1677ff))
                            .child("NARYA"),
                    )
                    .child(self.render_tab(0, "Dashboard", &cx.entity().clone(), cx))
                    .child(self.render_tab(1, "Proxies", &cx.entity().clone(), cx))
                    .child(self.render_tab(2, "Profiles", &cx.entity().clone(), cx))
                    .child(self.render_tab(3, "Rules", &cx.entity().clone(), cx))
                    .child(self.render_tab(4, "Settings", &cx.entity().clone(), cx))
                    .child(div().flex_1())
                    .child(
                        div()
                            .p_4()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w_2().h_2().bg(rgb(0x52c41a)).rounded_full())
                            .child(div().text_xs().text_color(rgb(0x555555)).child("Core Running v0.1.0")),
                    ),
            )
            .child(
                div().flex_1().h_full().bg(rgb(0x141414)).p_6().child(
                    div()
                        .size_full()
                        .bg(rgb(0x1d1d1d))
                        .rounded_xl()
                        .border_1()
                        .border_color(rgb(0x303030))
                        .p_6()
                        .overflow_hidden()
                        .child(match self.selected_tab {
                            0 => self.render_dashboard(_window, cx).into_any_element(),
                            1 => self.render_proxies(cx).into_any_element(),
                            2 => self.render_profiles(cx).into_any_element(),
                            3 => RulePanel::render(&self.rule_store, cx).into_any_element(),
                            4 => div()
                                .text_color(rgb(0x888888))
                                .child("App Settings")
                                .into_any_element(),
                            _ => div().child("Under Construction").into_any_element(),
                        }),
                ),
            )
    }
}

impl Workspace {
    fn render_tab(
        &self,
        index: usize,
        label: &'static str,
        entity: &Entity<Self>,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.selected_tab == index;
        let entity = entity.clone();

        div()
            .id(index)
            .relative()
            .px_3()
            .py_2()
            .mx_2()
            .mb_1()
            .rounded_lg()
            .flex()
            .items_center()
            .bg(if is_selected {
                rgba(0x1677ff1a)
            } else {
                rgba(0x00000000)
            })
            .hover(|style| {
                if !is_selected {
                    style.bg(rgba(0xffffff0a))
                } else {
                    style
                }
            })
            .cursor_pointer()
            .on_click(move |_, window, cx| {
                entity.update(cx, |workspace, cx| {
                    workspace.select_tab(index, window, cx);
                });
            })
            .child(
                div()
                    .w_1()
                    .h_4()
                    .rounded_full()
                    .bg(if is_selected {
                        rgb(0x1677ff)
                    } else {
                        rgba(0x00000000)
                    })
            )
            .child(
                div()
                    .ml_3()
                    .text_sm()
                    .font_weight(if is_selected { FontWeight::MEDIUM } else { FontWeight::NORMAL })
                    .text_color(if is_selected {
                        rgb(0xffffff)
                    } else {
                        rgb(0x888888)
                    })
                    .child(label),
            )
    }

    fn render_dashboard(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.on_next_frame(window, |_, _, cx| cx.notify());
        let store = self.traffic_store.read();
        let current_speed = store.last();
        let uptime = self.start_time.elapsed().as_secs();

        let conn_store = self.connection_store.read();
        let conns = conn_store.active_connections.clone();
        drop(conn_store);

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .mb_6()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_2xl().text_color(rgb(0xffffff)).child("Overview"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x555555))
                                    .child(format!("Uptime: {}s", uptime)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_6()
                            .child(self.render_stat_card(
                                "Upload",
                                format!("{:.1}", current_speed.up),
                                "KB/s",
                                rgb(0x52c41a),
                            ))
                            .child(self.render_stat_card(
                                "Download",
                                format!("{:.1}", current_speed.down),
                                "KB/s",
                                rgb(0x1677ff),
                            )),
                    ),
            )
            .child(
                div()
                    .h_48()
                    .bg(rgb(0x141414))
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(0x303030))
                    .p_4()
                    .mb_6()
                    .child(TrafficChart::render(self.traffic_store.clone(), cx)),
            )
            .child(
                div()
                    .flex_1()
                    .bg(rgb(0x141414))
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(0x303030))
                    .p_4()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .h_full()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x888888))
                                    .mb_4()
                                    .child("Active Connections"),
                            )
                            .child(
                                div().id("conn-list").flex_1().overflow_y_scroll().child(
                                    div().flex().flex_col().gap_1().children(
                                        conns.into_iter().map(|conn| {
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .p_2()
                                                .bg(rgb(0x232323))
                                                .rounded_md()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_3()
                                                        .child(
                                                            div()
                                                                .w_4()
                                                                .h_4()
                                                                .bg(rgb(0x1677ff))
                                                                .rounded_sm(),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0xcccccc))
                                                                .child(
                                                                    conn.process_name.unwrap_or(
                                                                        "Unknown".to_string(),
                                                                    ),
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x666666))
                                                        .child(format!(
                                                            "{} → {}",
                                                            conn.src_port, conn.dst_ip
                                                        )),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x52c41a))
                                                        .child("Direct"),
                                                )
                                        }),
                                    ),
                                ),
                            ),
                    ),
            )
    }

    fn render_stat_card(
        &self,
        label: &'static str,
        value: String,
        unit: &'static str,
        color: Rgba,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .p_3()
            .min_w(px(140.0))
            .bg(rgba(0xffffff05))
            .rounded_lg()
            .border_1()
            .border_color(rgba(0xffffff0a))
            .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0x666666)).mb_1().child(label.to_uppercase()))
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap_1()
                    .child(div().text_2xl().font_weight(FontWeight::BOLD).text_color(color).child(value))
                    .child(div().text_xs().text_color(rgb(0x444444)).child(unit)),
            )
    }

    fn render_proxies(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity().clone();
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .mb_6()
                    .child(
                        div()
                            .text_2xl()
                            .text_color(rgb(0xffffff))
                            .child("Proxy Nodes"),
                    )
                    .child(
                        div()
                            .id("test-all-btn")
                            .px_4()
                            .py_1()
                            .bg(rgb(0x1677ff))
                            .rounded_md()
                            .text_xs()
                            .cursor_pointer()
                            .on_click(move |_, window, cx| {
                                entity.update(cx, |workspace, cx| {
                                    workspace.test_all_latencies(window, cx);
                                });
                            })
                            .child("Test All"),
                    ),
            )
            .child(
                div()
                    .id("proxies-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(ProxyList::render(&self.profile_store, cx)),
            )
    }

    fn render_profiles(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity().clone();
        let store = self.profile_store.read();
        let is_loading = store.is_loading;
        let url = store.url.clone();
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_2xl()
                    .text_color(rgb(0xffffff))
                    .mb_6()
                    .child("Subscriptions"),
            )
            .child(
                div()
                    .bg(rgb(0x141414))
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(0x303030))
                    .p_6()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .mb_2()
                            .child("Active Subscription URL"),
                    )
                    .child(
                        div()
                            .p_3()
                            .bg(rgb(0x000000))
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(0x303030))
                            .text_xs()
                            .text_color(rgb(0x52c41a))
                            .child(url),
                    )
                    .child(
                        div().mt_6().flex().justify_end().child(
                            div()
                                .id("refresh-btn")
                                .px_6()
                                .py_2()
                                .bg(if is_loading {
                                    rgb(0x303030)
                                } else {
                                    rgb(0x1677ff)
                                })
                                .rounded_lg()
                                .text_color(rgb(0xffffff))
                                .cursor_pointer()
                                .hover(|s| if !is_loading { s.bg(rgb(0x4096ff)) } else { s })
                                .on_click(move |_, window, cx| {
                                    if !is_loading {
                                        entity.update(cx, |workspace, cx| {
                                            workspace.fetch_subscription(window, cx);
                                        });
                                    }
                                })
                                .child(if is_loading {
                                    "Updating..."
                                } else {
                                    "Update Now"
                                }),
                        ),
                    ),
            )
    }
}

pub fn run_app() {
    application().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1024.), px(768.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Narya Proxy Engine".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Workspace::new(cx)),
        )
        .unwrap();

        cx.activate(true);
    });
}
