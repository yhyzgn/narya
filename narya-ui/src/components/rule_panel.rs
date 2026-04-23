use crate::ipc_client;
use crate::models::rule::{AppInfo, SharedRuleStore};
use gpui::prelude::*;
use gpui::*;

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
        let q = store_read.search_query.to_lowercase();

        let filter = |apps: &[AppInfo]| {
            apps.iter()
                .filter(|a| q.is_empty() || a.name.to_lowercase().contains(&q))
                .cloned()
                .collect::<Vec<_>>()
        };

        let unassigned = filter(&store_read.unassigned);
        let direct = filter(&store_read.direct);
        let proxy = filter(&store_read.proxy);

        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .child(
                // 顶部状态栏
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .bg(rgb(0x141414))
                    .border_1()
                    .border_color(rgb(0x303030))
                    .rounded_lg()
                    .px_4()
                    .py_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().text_color(rgb(0x888888)).child("Filter:"))
                            .child(div().text_sm().text_color(rgb(0x1677ff)).child(
                                if q.is_empty() {
                                    "Search applications...".to_string()
                                } else {
                                    q.clone()
                                },
                            )),
                    )
                    .child(div().text_xs().text_color(rgb(0x555555)).child(format!(
                        "Total: {} apps detected",
                        store_read.unassigned.len()
                            + store_read.direct.len()
                            + store_read.proxy.len()
                    ))),
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
            .bg(rgb(0x141414))
            .rounded_xl()
            .border_1()
            .border_color(rgb(0x303030))
            .p_4()
            .min_h_0()
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
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w_2().h_2().rounded_full().bg(match zone_id {
                                "direct" => rgb(0x52c41a),
                                "proxy" => rgb(0xfaad14),
                                _ => rgb(0x1677ff),
                            }))
                            .child(div().text_sm().text_color(rgb(0x888888)).child(title)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x555555))
                            .child(count.to_string()),
                    ),
            )
            .child(
                // 滚动区优化：垂直列表模式
                div()
                    .id(format!("{}-scroll", zone_id))
                    .flex_1()
                    .min_h_0()
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
                                                div().w_5().h_5().bg(rgba(0xffffff0d)).rounded_sm(),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0xcccccc))
                                                    .child(app.name),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x444444))
                                            .child(app.id.to_string()),
                                    )
                            })),
                    ),
            )
    }
}

struct AppDragView {
    name: String,
}

impl Render for AppDragView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .py_2()
            .bg(rgba(0x1677ffcc))
            .rounded_md()
            .shadow_lg()
            .text_color(rgb(0xffffff))
            .text_xs()
            .child(self.name.clone())
    }
}
