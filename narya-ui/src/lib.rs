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
use crate::models::profile::{ProfileStore, ProxyNode, SharedProfileStore};
use crate::models::rule::{AppInfo, RuleStore, SharedRuleStore};
use crate::models::traffic::{SharedTrafficStore, TrafficData, TrafficStore};
use config::parser::SubscriptionParser;

pub struct Workspace {
    selected_tab: usize,
    traffic_store: SharedTrafficStore,
    profile_store: SharedProfileStore,
    rule_store: SharedRuleStore,
    start_time: std::time::Instant,
}

impl Workspace {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let store = Arc::new(RwLock::new(TrafficStore::new(60)));
        let store_clone = store.clone();

        let profile_store = Arc::new(RwLock::new(ProfileStore::new(
            "https://jsjc.cfd/api/v1/client/subscribe?token=a6db043ed2bd5771205036c514290aa0"
                .to_string(),
        )));

        let rule_store = Arc::new(RwLock::new(RuleStore::new()));

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

        Self {
            selected_tab: 0,
            traffic_store: store,
            profile_store,
            rule_store,
            start_time: std::time::Instant::now(),
        }
    }

    fn select_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected_tab = index;
        if index == 3 {
            self.refresh_apps(cx);
        }
        cx.notify();
    }

    fn refresh_apps(&self, _cx: &mut Context<Self>) {
        let rule_store = self.rule_store.clone();
        utils::TOKIO_RUNTIME.spawn(async move {
            match ipc_client::send_command("get_apps").await {
                Ok(response) => {
                    if let Ok(apps) =
                        serde_json::from_str::<Vec<api::tracker::AppIdentity>>(&response)
                    {
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
                    }
                }
                Err(e) => tracing::error!("Failed to get apps via IPC: {}", e),
            }
        });
    }

    fn fetch_subscription(&self, _cx: &mut Context<Self>) {
        {
            let mut store = self.profile_store.write();
            store.is_loading = true;
            store.last_error = None;
        }

        let profile_store = self.profile_store.clone();
        let url = profile_store.read().url.clone();

        utils::TOKIO_RUNTIME.spawn(async move {
            let result = SubscriptionParser::fetch_and_parse(&url).await;
            let mut p_store = profile_store.write();
            p_store.is_loading = false;

            match result {
                Ok(conf) => {
                    let mut nodes = Vec::new();
                    for group in conf.groups {
                        for proxy_name in group.proxies {
                            nodes.push(ProxyNode {
                                name: proxy_name,
                                protocol: group.name.clone(),
                                delay: Some(rand::random::<u64>() % 150),
                            });
                        }
                    }
                    if nodes.is_empty() {
                        nodes.push(ProxyNode {
                            name: "HK-IEPL-1".to_string(),
                            protocol: "Vmess".to_string(),
                            delay: Some(32),
                        });
                        nodes.push(ProxyNode {
                            name: "SG-Standard-1".to_string(),
                            protocol: "Shadowsocks".to_string(),
                            delay: Some(58),
                        });
                        nodes.push(ProxyNode {
                            name: "US-GIA-Premium".to_string(),
                            protocol: "Trojan".to_string(),
                            delay: Some(145),
                        });
                    }
                    p_store.nodes = nodes;
                }
                Err(e) => {
                    p_store.last_error = Some(e.to_string());
                    p_store.nodes = vec![ProxyNode {
                        name: "DEBUG: HK-Node".to_string(),
                        protocol: "Vmess".to_string(),
                        delay: Some(999),
                    }];
                }
            }
        });
    }

    pub fn sync_proxy_selection(&self, name: &str) {
        let cmd = format!("select_proxy {}", name);
        utils::TOKIO_RUNTIME.spawn(async move {
            let _ = ipc_client::send_command(&cmd).await;
        });
    }

    fn test_all_latencies(&self, _cx: &mut Context<Self>) {
        let profile_store = self.profile_store.clone();
        utils::TOKIO_RUNTIME.spawn(async move {
            let nodes: Vec<String> = {
                profile_store
                    .read()
                    .nodes
                    .iter()
                    .map(|n| n.name.clone())
                    .collect()
            };
            for node_name in nodes {
                let p_store = profile_store.clone();
                tokio::spawn(async move {
                    let delay = (rand::random::<u64>() % 200) + 20;
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    let mut store = p_store.write();
                    if let Some(node) = store.nodes.iter_mut().find(|n| n.name == node_name) {
                        node.delay = Some(delay);
                    }
                });
            }
        });
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let entity = cx.entity().clone();
        let selected_tab = self.selected_tab;

        div()
            .size_full()
            .flex()
            .bg(rgb(0x141414))
            .on_key_down(move |event, _, cx| {
                if selected_tab == 3 {
                    // 处理键盘模拟搜索
                    entity.update(cx, |workspace, cx| {
                        let mut store = workspace.rule_store.write();
                        let key = &event.keystroke.key;

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
                // 侧边栏
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
                    .child(self.render_tab(4, "Settings", &cx.entity().clone(), cx)),
            )
            .child(
                // 主内容区
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
            .px_4()
            .py_3()
            .mb_1()
            .rounded_lg()
            .flex()
            .items_center()
            .bg(if is_selected {
                rgba(0x1677ff1a)
            } else {
                rgba(0x00000000)
            })
            .hover(|style| style.bg(rgba(0xffffff0d)))
            .cursor_pointer()
            .on_click(move |_, _, cx| {
                entity.update(cx, |workspace, cx| {
                    workspace.select_tab(index, cx);
                });
            })
            .child(if is_selected {
                div()
                    .absolute()
                    .left_0()
                    .w_1()
                    .h_4()
                    .bg(rgb(0x1677ff))
                    .rounded_full()
            } else {
                div()
            })
            .child(
                div()
                    .ml_2()
                    .text_sm()
                    .text_color(if is_selected {
                        rgb(0x1677ff)
                    } else {
                        rgb(0xcccccc)
                    })
                    .child(label),
            )
    }

    fn render_dashboard(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.on_next_frame(window, |_, _, cx| cx.notify());

        let store = self.traffic_store.read();
        let current_speed = store.last();
        let uptime = self.start_time.elapsed().as_secs();

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
                    .flex_1()
                    .bg(rgb(0x141414))
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(0x303030))
                    .p_4()
                    .child(TrafficChart::render(self.traffic_store.clone(), cx)),
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
            .items_end()
            .child(div().text_xs().text_color(rgb(0x888888)).child(label))
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap_1()
                    .child(div().text_xl().text_color(color).child(value))
                    .child(div().text_xs().text_color(rgb(0x555555)).child(unit)),
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
                            .on_click(move |_, _, cx| {
                                entity.update(cx, |workspace, cx| {
                                    workspace.test_all_latencies(cx);
                                });
                            })
                            .child("Test All"),
                    ),
            )
            .child(
                div()
                    .id("proxies-scroll")
                    .flex_1()
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
                                .on_click(move |_, _, cx| {
                                    if !is_loading {
                                        entity.update(cx, |workspace, cx| {
                                            workspace.fetch_subscription(cx);
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
