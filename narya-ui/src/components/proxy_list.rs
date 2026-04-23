use crate::models::profile::SharedProfileStore;
use gpui::*;

pub struct ProxyList;

impl ProxyList {
    pub fn render(
        store: &SharedProfileStore,
        _cx: &mut Context<crate::Workspace>,
    ) -> impl IntoElement {
        let store = store.read();

        let mut list_container = div().flex().flex_col().gap_2().w_full();

        if store.nodes.is_empty() {
            if store.is_loading {
                return div()
                    .text_color(rgb(0x888888))
                    .child("Loading nodes...")
                    .into_any_element();
            } else if let Some(ref err) = store.last_error {
                return div()
                    .text_color(rgb(0xff4d4f))
                    .child(format!("Error: {}", err))
                    .into_any_element();
            } else {
                return div()
                    .text_color(rgb(0x888888))
                    .child("No nodes available. Please update profile.")
                    .into_any_element();
            }
        }

        for node in &store.nodes {
            let delay_color = match node.delay {
                Some(d) if d < 100 => rgb(0x52c41a), // Green
                Some(d) if d < 300 => rgb(0xfaad14), // Orange
                Some(_) => rgb(0xff4d4f),            // Red
                None => rgb(0x888888),               // Gray
            };

            let delay_text = match node.delay {
                Some(d) => format!("{}ms", d),
                None => "- ms".to_string(),
            };

            list_container = list_container.child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .p_3()
                    .bg(rgb(0x2d2d2d))
                    .rounded_md()
                    .hover(|style| style.bg(rgb(0x353535)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_color(rgb(0xffffff)).child(node.name.clone()))
                            .child(
                                div()
                                    .text_color(rgb(0x888888))
                                    .text_sm()
                                    .child(node.protocol.clone()),
                            ),
                    )
                    .child(div().text_color(delay_color).text_sm().child(delay_text)),
            );
        }

        div()
            .id("proxy-list")
            .h_full()
            .overflow_y_scroll()
            .child(list_container)
            .into_any_element()
    }
}
