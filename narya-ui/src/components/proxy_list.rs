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
                    .mb_1()
                    .bg(if is_active {
                        rgba(0x1677ff1a)
                    } else {
                        rgba(0xffffff05)
                    })
                    .border_1()
                    .border_color(if is_active {
                        rgb(0x1677ff)
                    } else {
                        rgba(0xffffff0a)
                    })
                    .rounded_lg()
                    .hover(|style| {
                        if is_active {
                            style
                        } else {
                            style.bg(rgba(0xffffff0a)).border_color(rgba(0xffffff1a))
                        }
                    })
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
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .w_1()
                                    .h_8()
                                    .rounded_full()
                                    .bg(if is_active {
                                        rgb(0x1677ff)
                                    } else {
                                        rgba(0xffffff0a)
                                    })
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                            .text_color(if is_active {
                                                rgb(0x1677ff)
                                            } else {
                                                rgb(0xffffff)
                                            })
                                            .child(node.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x666666))
                                            .child(node.protocol.to_uppercase()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .bg(Rgba {
                                r: delay_color.r,
                                g: delay_color.g,
                                b: delay_color.b,
                                a: 0.1,
                            })
                            .child(
                                div()
                                    .text_color(delay_color)
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .child(match node.delay {
                                        Some(d) => format!("{}ms", d),
                                        None => "---".to_string(),
                                    }),
                            ),
                    )
            }))
            .into_any_element()
    }
}
