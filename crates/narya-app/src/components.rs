use gpui::{prelude::*, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconName {
    Dashboard,
    Nodes,
    Config,
    Subscriptions,
    Connections,
    Rules,
    Logs,
    Tools,
    Settings,
    About,
    Github,
    Moon,
    Bell,
    ExternalLink,
    Terminal,
}

impl IconName {
    pub fn path(&self) -> &'static str {
        match self {
            IconName::Dashboard => "resources/assets/ic_dashboard.svg",
            IconName::Nodes => "resources/assets/ic_nodes.svg",
            IconName::Config => "resources/assets/ic_config.svg",
            IconName::Subscriptions => "resources/assets/ic_subscriptions.svg",
            IconName::Connections => "resources/assets/ic_connections.svg",
            IconName::Rules => "resources/assets/ic_rules.svg",
            IconName::Logs => "resources/assets/ic_logs.svg",
            IconName::Tools => "resources/assets/ic_tools.svg",
            IconName::Settings => "resources/assets/ic_settings.svg",
            IconName::About => "resources/assets/ic_about.svg",
            IconName::Github => "resources/assets/ic_github.svg",
            IconName::Moon => "resources/assets/ic_moon.svg",
            IconName::Bell => "resources/assets/ic_bell.svg",
            IconName::ExternalLink => "resources/assets/ic_external-link.svg",
            IconName::Terminal => "resources/assets/ic_terminal.svg",
        }
    }
}

pub fn icon(name: IconName, size: f32, color: Hsla) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size))
        .child(
            svg()
                .path(name.path())
                .size(px(size * 0.8))
                .text_color(color), // Use text_color to set currentColor for SVG stroke/fill
        )
}

pub fn glass_card() -> Div {
    div()
        .bg(rgb(0xFFFFFF))
        .border_1()
        .border_color(rgb(0xE5E7EB))
        .rounded_xl()
        .shadow_sm()
}

pub fn badge(text: impl Into<String>, color: Hsla) -> impl IntoElement {
    let mut bg = color;
    bg.a = 0.1;

    div().bg(bg).px_2().py_0p5().rounded_md().child(
        div()
            .text_color(color)
            .text_size(px(11.0))
            .font_weight(FontWeight::BOLD)
            .child(text.into()),
    )
}

pub fn search_input() -> impl IntoElement {
    div()
        .w(px(320.0))
        .h(px(36.0))
        .bg(rgb(0xffffff))
        .border_1()
        .border_color(rgb(0xe5e7eb))
        .rounded_md()
        .flex()
        .items_center()
        .px_3()
        .gap_2()
        .child("🔍")
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x9ca3af))
                .child("搜索节点..."),
        )
}

pub fn toast(message: impl Into<String>, _kind: ToastKind) -> impl IntoElement {
    div().absolute().bottom_10().right_10().child(
        glass_card().p_4().shadow_lg().child(
            div().flex().items_center().gap_3().child("🔔").child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .child(message.into()),
            ),
        ),
    )
}

pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}
