use gpui::*;
use gpui::prelude::*;
use gpui_platform::application;
use std::time::Duration;
use std::sync::Arc;
use parking_lot::RwLock;

mod models;
mod components;

use crate::models::traffic::{TrafficData, TrafficStore, SharedTrafficStore};
use crate::models::profile::{ProfileStore, SharedProfileStore, ProxyNode};
use crate::models::rule::{RuleStore, SharedRuleStore, AppInfo};
use crate::components::traffic_chart::TrafficChart;
use crate::components::proxy_list::ProxyList;
use crate::components::rule_panel::RulePanel;
use config::parser::SubscriptionParser;

pub struct Workspace {
    selected_tab: usize,
    traffic_store: SharedTrafficStore,
    profile_store: SharedProfileStore,
    rule_store: SharedRuleStore,
}

impl Workspace {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let store = Arc::new(RwLock::new(TrafficStore::new(60)));
        let store_clone = store.clone();
        
        let profile_store = Arc::new(RwLock::new(ProfileStore::new(
            "https://jsjc.cfd/api/v1/client/subscribe?token=a6db043ed2bd5771205036c514290aa0".to_string()
        )));

        let rule_store = Arc::new(RwLock::new(RuleStore::new()));
        {
            let mut r_store = rule_store.write();
            r_store.unassigned.push(AppInfo { id: "chrome".to_string(), name: "Google Chrome".to_string(), icon: None });
            r_store.unassigned.push(AppInfo { id: "telegram".to_string(), name: "Telegram".to_string(), icon: None });
            r_store.unassigned.push(AppInfo { id: "discord".to_string(), name: "Discord".to_string(), icon: None });
            r_store.unassigned.push(AppInfo { id: "spotify".to_string(), name: "Spotify".to_string(), icon: None });
        }

        // 模拟数据生成
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
        }
    }

    fn select_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected_tab = index;
        cx.notify();
    }

    fn fetch_subscription(&self, cx: &mut Context<Self>) {
        tracing::info!("Starting to fetch subscription...");
        {
            let mut store = self.profile_store.write();
            store.is_loading = true;
            store.last_error = None;
        }
        cx.notify();

        let profile_store = self.profile_store.clone();
        let url = profile_store.read().url.clone();

        // 核心修复：使用 GPUI 的 background_executor 而不是直接调用 tokio::spawn
        cx.background_executor().spawn(async move {
            tracing::info!("Subscription task running via GPUI executor...");
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
                        tracing::warn!("Parsed nodes are empty, using fallback mock nodes");
                        nodes.push(ProxyNode { name: "HK-IEPL-1".to_string(), protocol: "Vmess".to_string(), delay: Some(32) });
                        nodes.push(ProxyNode { name: "SG-Standard-1".to_string(), protocol: "Shadowsocks".to_string(), delay: Some(58) });
                        nodes.push(ProxyNode { name: "US-GIA-Premium".to_string(), protocol: "Trojan".to_string(), delay: Some(145) });
                    }
                    
                    tracing::info!("Fetched {} nodes successfully", nodes.len());
                    p_store.nodes = nodes;
                }
                Err(e) => {
                    tracing::error!("Subscription fetch error: {}", e);
                    p_store.last_error = Some(e.to_string());
                    p_store.nodes = vec![
                        ProxyNode { name: "DEBUG: HK-Node".to_string(), protocol: "Vmess".to_string(), delay: Some(999) }
                    ];
                }
            }
        }).detach();
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        cx.on_next_frame(_window, |_, _, cx| cx.notify());

        let entity = cx.entity().clone();
        
        div()
            .size_full()
            .flex()
            .bg(rgb(0x1e1e1e))
            .child(
                // 侧边栏
                div()
                    .w_64()
                    .h_full()
                    .bg(rgb(0x252526))
                    .flex()
                    .flex_col()
                    .child(self.render_tab(0, "Dashboard", &entity, cx))
                    .child(self.render_tab(1, "Proxies", &entity, cx))
                    .child(self.render_tab(2, "Profiles", &entity, cx))
                    .child(self.render_tab(3, "Rules", &entity, cx))
                    .child(self.render_tab(4, "Settings", &entity, cx))
            )
            .child(
                // 主内容区
                div()
                    .flex_1()
                    .h_full()
                    .p_4()
                    .text_color(rgb(0xffffff))
                    .child(match self.selected_tab {
                        0 => self.render_dashboard(cx).into_any_element(),
                        1 => self.render_proxies(cx).into_any_element(),
                        2 => self.render_profiles(cx).into_any_element(),
                        3 => RulePanel::render(&self.rule_store, cx).into_any_element(),
                        4 => div().child("App Settings").into_any_element(),
                        _ => div().child("Under Construction").into_any_element(),
                    })
            )
    }
}

impl Workspace {
    fn render_tab(
        &self, 
        index: usize, 
        label: &'static str, 
        entity: &Entity<Self>,
        _cx: &mut Context<Self>
    ) -> impl IntoElement {
        let is_selected = self.selected_tab == index;
        let entity = entity.clone();
        
        div()
            .id(index)
            .p_2()
            .m_1()
            .rounded_md()
            .bg(if is_selected { rgba(0x37373dff) } else { rgba(0x00000000) })
            .hover(|style| style.bg(rgb(0x2a2d2e)))
            .cursor_pointer()
            .on_click(move |_, _, cx| {
                entity.update(cx, |workspace, cx| {
                    workspace.select_tab(index, cx);
                });
            })
            .text_color(if is_selected { rgb(0xffffff) } else { rgb(0xcccccc) })
            .child(label)
    }

    fn render_dashboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.traffic_store.read();
        let current_speed = store.last();
        
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_xl().child("Narya Dashboard"))
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(div().text_color(rgb(0x52c41a)).child(format!("↑ {:.1} KB/s", current_speed.up)))
                            .child(div().text_color(rgb(0x1677ff)).child(format!("↓ {:.1} KB/s", current_speed.down)))
                    )
            )
            .child(
                div()
                    .mt_4()
                    .h_64()
                    .w_full()
                    .bg(rgb(0x2d2d2d))
                    .rounded_lg()
                    .p_2()
                    .child(TrafficChart::render(self.traffic_store.clone(), cx))
            )
    }

    fn render_proxies(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(div().text_xl().mb_4().child("Proxy Nodes"))
            .child(
                div().id("proxies-scroll").flex_1().overflow_y_scroll()
                    .child(ProxyList::render(&self.profile_store, cx))
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
            .child(div().text_xl().child("Profile Management"))
            .child(
                div()
                    .mt_4()
                    .p_4()
                    .bg(rgb(0x2d2d2d))
                    .rounded_lg()
                    .child(div().text_color(rgb(0xcccccc)).child(format!("URL: {}", url)))
                    .child(
                        div()
                            .mt_4()
                            .id("refresh-btn")
                            .p_2()
                            .bg(rgb(0x1677ff))
                            .rounded_md()
                            .cursor_pointer()
                            .on_click(move |_, _, cx| {
                                entity.update(cx, |workspace, cx| {
                                    workspace.fetch_subscription(cx);
                                });
                            })
                            .child(if is_loading { "Fetching..." } else { "Update from URL" })
                    )
            )
    }
}

pub fn run_app() {
    application().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions::default(),
            |_, cx| cx.new(|cx| Workspace::new(cx)),
        ).unwrap();
        
        cx.activate(true);
    });
}
