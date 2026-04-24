use crate::models::profile::SharedProfileStore;
use gpui::*;

pub struct ProxyList;

impl ProxyList {
    pub fn render(
        store: &SharedProfileStore,
        cx: &mut Context<crate::Workspace>,
    ) -> impl IntoElement {
        let store_handle = store.clone();
        let store = store.read();
        let entity = cx.entity().clone();

        if store.nodes.is_empty() {
            if store.is_loading {
                return div()
                    .text_color(rgb(0x888888))
                    .child("Loading nodes...")
                    .into_any_element();
            } else {
                return div()
                    .text_color(rgb(0x888888))
                    .child("No nodes available.")
                    .into_any_element();
            }
        }

        div()
            .id("proxy-list-inner")
            .flex()
            .flex_col()
            .gap_2()
            .children(store.nodes.iter().enumerate().map(|(i, node)| {
                let name = node.name.clone();
                let is_active = store.active_node.as_ref() == Some(&name);
                let store_handle = store_handle.clone();
                let entity = entity.clone();

                let delay_color = match node.delay {
                    Some(d) if d < 100 => rgb(0x52c41a),
                    Some(d) if d < 300 => rgb(0xfaad14),
                    Some(_) => rgb(0xff4d4f),
                    None => rgb(0x888888),
                };

                div()
                    .id(i)
                    .flex()
                    .justify_between()
                    .items_center()
                    .p_3()
                    .bg(if is_active {
                        rgba(0x1677ff1a)
                    } else {
                        rgb(0x2d2d2d)
                    })
                    .border_1()
                    .border_color(if is_active {
                        rgb(0x1677ff)
                    } else {
                        rgba(0x00000000)
                    })
                    .rounded_lg()
                    .hover(|style| style.bg(rgb(0x353535)))
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        let name_clone = name.clone();
                        let store_handle = store_handle.clone();
                        entity.update(cx, |workspace, cx| {
                            {
                                let mut s = store_handle.write();
                                s.set_active(&name_clone);
                            }
                            workspace.sync_proxy_selection(&name_clone);
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_color(if is_active {
                                        rgb(0x1677ff)
                                    } else {
                                        rgb(0xffffff)
                                    })
                                    .child(node.name.clone()),
                            )
                            .child(
                                div()
                                    .text_color(rgb(0x888888))
                                    .text_sm()
                                    .child(node.protocol.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_color(delay_color)
                            .text_sm()
                            .child(match node.delay {
                                Some(d) => format!("{}ms", d),
                                None => "- ms".to_string(),
                            }),
                    )
            }))
            .into_any_element()
    }
}
