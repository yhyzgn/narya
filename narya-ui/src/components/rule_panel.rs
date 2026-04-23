use crate::ipc_client;
use crate::models::rule::{AppInfo, SharedRuleStore};
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
        let unassigned = store_read.unassigned.clone();
        let direct = store_read.direct.clone();
        let proxy = store_read.proxy.clone();
        let entity_id = cx.entity_id();
        drop(store_read);

        div()
            .flex()
            .gap_6()
            .size_full()
            .child(
                // 左侧：待分配应用池 (30% 宽度)
                div().w_1_3().h_full().child(Self::render_column(
                    "Available Apps",
                    unassigned,
                    "pool",
                    store,
                    entity_id,
                )),
            )
            .child(
                // 右侧：决策区 (70% 宽度)
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(Self::render_column(
                        "Direct Connection (No Proxy)",
                        direct,
                        "direct",
                        store,
                        entity_id,
                    ))
                    .child(Self::render_column(
                        "Global Proxy (Tunneled)",
                        proxy,
                        "proxy",
                        store,
                        entity_id,
                    )),
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

        div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(rgb(0x141414)) // 内部背景稍微深一点
            .rounded_xl()
            .border_1()
            .border_color(rgb(0x303030))
            .p_4()
            .id(zone_id)
            // 增强 Drop 视觉：悬停时边框变亮
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
                    .items_center()
                    .gap_2()
                    .mb_4()
                    .child(
                        // 区域标识小图标 (模拟)
                        div().w_2().h_2().rounded_full().bg(match zone_id {
                            "direct" => rgb(0x52c41a),
                            "proxy" => rgb(0xfaad14),
                            _ => rgb(0x1677ff),
                        }),
                    )
                    .child(div().text_sm().text_color(rgb(0x888888)).child(title)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
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
                            .bg(rgb(0x2d2d2d))
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(0x383838))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x353535)).border_color(rgb(0x555555)))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        // 应用小图标占位符
                                        div().w_4().h_4().bg(rgba(0xffffff1a)).rounded_sm(),
                                    )
                                    .child(
                                        div().text_xs().text_color(rgb(0xcccccc)).child(app.name),
                                    ),
                            )
                    })),
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
            .bg(rgba(0x1677ffcc)) // 拖拽时呈现品牌色
            .rounded_md()
            .shadow_lg()
            .text_color(rgb(0xffffff))
            .text_xs()
            .child(self.name.clone())
    }
}
