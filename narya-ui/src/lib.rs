use gpui::prelude::*;
use gpui::*;
use gpui_platform::application;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

mod components;
mod models;

use crate::components::traffic_chart::TrafficChart;
use crate::models::traffic::{SharedTrafficStore, TrafficData, TrafficStore};

pub struct Workspace {
    selected_tab: usize,
    traffic_store: SharedTrafficStore,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let store = Arc::new(RwLock::new(TrafficStore::new(60)));
        let store_clone = store.clone();

        // 纯后台线程模拟数据，不涉及 GPUI 上下文
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
        }
    }

    fn select_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected_tab = index;
        cx.notify();
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // 关键点：强制下一帧重绘，实现 60FPS 动画效果
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
                    .child(self.render_tab(3, "Settings", &entity, cx)),
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
                        1 => div().child("Proxy Nodes List").into_any_element(),
                        2 => div().child("Profile Management").into_any_element(),
                        3 => div().child("App Settings").into_any_element(),
                        _ => div().child("Under Construction").into_any_element(),
                    }),
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
            .p_2()
            .m_1()
            .rounded_md()
            .bg(if is_selected {
                rgba(0x37373dff)
            } else {
                rgba(0x00000000)
            })
            .hover(|style| style.bg(rgb(0x2a2d2e)))
            .cursor_pointer()
            .on_click(move |_, _, cx| {
                entity.update(cx, |workspace, cx| {
                    workspace.select_tab(index, cx);
                });
            })
            .text_color(if is_selected {
                rgb(0xffffff)
            } else {
                rgb(0xcccccc)
            })
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
                            .child(
                                div()
                                    .text_color(rgb(0x52c41a))
                                    .child(format!("↑ {:.1} KB/s", current_speed.up)),
                            )
                            .child(
                                div()
                                    .text_color(rgb(0x1677ff))
                                    .child(format!("↓ {:.1} KB/s", current_speed.down)),
                            ),
                    ),
            )
            .child(
                div()
                    .mt_4()
                    .h_64()
                    .w_full()
                    .bg(rgb(0x2d2d2d))
                    .rounded_lg()
                    .p_2()
                    .child(TrafficChart::render(self.traffic_store.clone(), cx)),
            )
    }
}

pub fn run_app() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|cx| Workspace::new(cx))
        })
        .unwrap();

        cx.activate(true);
    });
}
