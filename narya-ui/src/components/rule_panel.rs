use crate::ipc_client;
use crate::models::rule::{AppInfo, SharedRuleStore};
use gpui::prelude::*;
use gpui::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RulePanel;

#[derive(Clone)]
pub struct AppDrag {
    pub app_id: String,
    pub name: String,
}

impl RulePanel {
    pub fn render(store: &SharedRuleStore, cx: &mut Context<crate::Workspace>) -> impl IntoElement {
        let store_read = store.read();
        let entity_id = cx.entity_id();
        let q = store_read.search_query.clone();

        // 计算闪烁状态 (500ms 翻转一次)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let cursor_visible = (now / 500) % 2 == 0;

        let filter = |apps: &[AppInfo]| {
            let q_lower = q.to_lowercase();
            apps.iter()
                .filter(|a| q_lower.is_empty() || a.name.to_lowercase().contains(&q_lower))
                .take(150)
                .cloned()
                .collect::<Vec<_>>()
        };

        let unassigned = filter(&store_read.unassigned);
        let direct = filter(&store_read.direct);
        let proxy = filter(&store_read.proxy);

        let total_count =
            store_read.unassigned.len() + store_read.direct.len() + store_read.proxy.len();
        drop(store_read);

        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .child(
                // 彻底修复：输入框 UI 与光标逻辑
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x141414))
                    .border_1()
                    .border_color(if q.is_empty() {
                        rgb(0x303030)
                    } else {
                        rgb(0x1677ff)
                    })
                    .rounded_lg()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_color(rgb(0x888888)).text_sm().child(""))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .relative()
                                    // 修正：光标始终处于文字或占位符的最前端/末端
                                    .child(if q.is_empty() {
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x555555))
                                            .child("Search applications...")
                                    } else {
                                        div().text_sm().text_color(rgb(0xffffff)).child(q.clone())
                                    })
                                    .child(if cursor_visible {
                                        if q.is_empty() {
                                            // 空白时，光标悬浮在 Placeholder 起始处
                                            div()
                                                .absolute()
                                                .left_0()
                                                .w_0p5()
                                                .h_4()
                                                .bg(rgb(0x1677ff))
                                        } else {
                                            // 有文字时，光标紧跟其后
                                            div().ml_0p5().w_0p5().h_4().bg(rgb(0x1677ff))
                                        }
                                    } else {
                                        div()
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x555555))
                            .child(format!("{} apps", total_count)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_6()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(div().w_1_3().h_full().child(Self::render_column(
                        "Available",
                        unassigned,
                        "pool",
                        store,
                        entity_id,
                    )))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_6()
                            .h_full()
                            .min_h_0()
                            .child(Self::render_column(
                                "Direct", direct, "direct", store, entity_id,
                            ))
                            .child(Self::render_column(
                                "Proxy", proxy, "proxy", store, entity_id,
                            )),
                    ),
            )
    }

    fn render_column(
        title: &'static str,
        apps: Vec<AppInfo>,
        zone_id: &'static str,
        store: &SharedRuleStore,
        entity_id: EntityId,
    ) -> impl IntoElement {
        let store = store.clone();
        let count = apps.len();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            .bg(rgb(0x141414))
            .rounded_xl()
            .border_1()
            .border_color(rgb(0x303030))
            .p_4()
            .id(zone_id)
            .hover(|s| s.border_color(rgb(0x1677ff)))
            .on_drop(move |drag: &AppDrag, _, cx| {
                let mut rules_changed = false;
                {
                    let mut store_write = store.write();
                    match zone_id {
                        "direct" => {
                            store_write.assign_to_direct(&drag.app_id);
                            rules_changed = true;
                        }
                        "proxy" => {
                            store_write.assign_to_proxy(&drag.app_id);
                            rules_changed = true;
                        }
                        _ => {
                            store_write.unassign(&drag.app_id);
                            rules_changed = true;
                        }
                    }
                }
                if rules_changed {
                    let store_read = store.read();
                    let bypass_rules = api::tracker::BypassRules {
                        whitelist: store_read.direct.iter().map(|a| a.id.clone()).collect(),
                        blacklist: store_read.proxy.iter().map(|a| a.id.clone()).collect(),
                    };
                    if let Ok(json) = serde_json::to_string(&bypass_rules) {
                        let cmd = format!("update_rules {}", json);
                        utils::TOKIO_RUNTIME.spawn(async move {
                            let _ = ipc_client::send_command(&cmd).await;
                        });
                    }
                }
                cx.notify(entity_id);
            })
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .mb_4()
                    .child(div().text_sm().text_color(rgb(0xcccccc)).child(title))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x555555))
                            .child(count.to_string()),
                    ),
            )
            .child(
                div().flex_1().min_h_0().relative().child(
                    div()
                        .id(format!("{}-scroll", zone_id))
                        .absolute()
                        .inset_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(apps.into_iter().map(|app| {
                                    let app_id = app.id.clone();
                                    let app_name = app.name.clone();

                                    div()
                                        .id(app.id.clone())
                                        .on_drag(
                                            AppDrag {
                                                app_id: app_id.clone(),
                                                name: app_name.clone(),
                                            },
                                            |drag, _, _, cx| {
                                                // 修复不跟手：使用最简化的渲染，强制移除所有外部偏移
                                                cx.new(|_| AppDragView {
                                                    name: drag.name.clone(),
                                                })
                                            },
                                        )
                                        .px_3()
                                        .py_2()
                                        .bg(rgb(0x232323))
                                        .rounded_md()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(0x2d2d2d)))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .w_5()
                                                        .h_5()
                                                        .bg(rgba(0xffffff0d))
                                                        .rounded_sm(),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xcccccc))
                                                        .child(app.name),
                                                ),
                                        )
                                })),
                        ),
                ),
            )
    }
}

struct AppDragView {
    name: String,
}

impl Render for AppDragView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 极致对齐修复：使用 absolute 居中或者是极简布局
        // 在 GPUI 中，最好的“跟手”方式是创建一个不带内外边距的根节点
        div()
            .px_2()
            .py_1()
            .bg(rgb(0x1677ff))
            .rounded_md()
            .text_color(rgb(0xffffff))
            .text_xs()
            .child(self.name.clone())
    }
}
