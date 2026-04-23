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
            .gap_4()
            .size_full()
            .child(
                // 左侧：应用池
                Self::render_column("App Pool", unassigned, "pool", store, entity_id),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .flex_1()
                    .child(Self::render_column(
                        "Direct (Whitelist)",
                        direct,
                        "direct",
                        store,
                        entity_id,
                    ))
                    .child(Self::render_column(
                        "Proxy (Overseas)",
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
            .bg(rgb(0x2d2d2d))
            .rounded_lg()
            .p_4()
            .id(zone_id)
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
                    // 同步到后端
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
            .child(div().text_lg().mb_4().child(title))
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
                            .p_3()
                            .bg(rgb(0x3d3d3d))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x4d4d4d)))
                            .child(app.name)
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
            .p_2()
            .bg(rgba(0x1677ffcc))
            .rounded_md()
            .text_color(rgb(0xffffff))
            .child(self.name.clone())
    }
}
