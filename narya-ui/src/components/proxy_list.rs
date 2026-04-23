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
                    .child("Loading nodes from subscription...")
                    .into_any_element();
            } else if let Some(ref err) = store.last_error {
                return div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_color(rgb(0xff4d4f))
                            .child(format!("Error: {}", err)),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x888888))
                            .child("Go to 'Profiles' to check your URL."),
                    )
                    .into_any_element();
            } else {
                return div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_color(rgb(0x888888))
                            .child("No nodes found. Did you refresh the profile?"),
                    )
                    .into_any_element();
            }
        }

        for (i, node) in store.nodes.iter().enumerate() {
            let delay_color = match node.delay {
                Some(d) if d < 100 => rgb(0x52c41a), // Green
                Some(d) if d < 300 => rgb(0xfaad14), // Orange
                Some(_) => rgb(0xff4d4f),            // Red
                None => rgb(0x888888),               // Gray
            };

            let delay_text = match node.delay {
                Some(d) => format!("{}ms", d),
                None => "timeout".to_string(),
            };

            list_container = list_container.child(
                div()
                    .id(i)
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
            .id("proxy-list-container")
            .h_full()
            .child(list_container)
            .into_any_element()
    }
}
