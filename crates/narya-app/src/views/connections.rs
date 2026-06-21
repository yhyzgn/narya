use crate::components::{icon, IconName};
use crate::state::AppState;
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};

pub fn render_connections_view(
    model: &Entity<AppState>,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let _state = model.read(cx);

    // --- Design System ---
    let color_bg = rgb(0xF8FAFC);
    let color_card = rgb(0xFFFFFF);
    let color_border = rgb(0xE2E8F0);
    let color_brand = rgb(0x4F46E5);
    let color_success = rgb(0x10B981);
    let color_text_primary = rgb(0x0F172A);
    let color_text_secondary = rgb(0x64748B);
    let color_text_muted = rgb(0x94A3B8);

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(color_bg)
        .gap_6()
        .child(
            // --- 1. Metrics Row ---
            div()
                .flex()
                .w_full()
                .gap_4()
                .child(connection_metric_card(
                    "活跃连接",
                    "324",
                    "TCP 286 / UDP 38",
                    rgb(0x6366F1),
                    IconName::Connections,
                ))
                .child(connection_metric_card(
                    "当前速率",
                    "12.45",
                    "↓ 12.4 MB/s ↑ 3.2 MB/s",
                    color_success,
                    IconName::Dashboard,
                ))
                .child(connection_metric_card(
                    "进程数量",
                    "42",
                    "浏览器 / Telegram / Steam",
                    color_brand,
                    IconName::Nodes,
                ))
                .child(connection_metric_card(
                    "命中规则",
                    "51%",
                    "Proxy 51% Direct 42%",
                    rgb(0x3B82F6),
                    IconName::Rules,
                ))
                .child(connection_metric_card(
                    "今日流量",
                    "1.26",
                    "较昨日 +18.6%",
                    rgb(0xF59E0B),
                    IconName::Config,
                )),
        )
        .child(
            // --- 2. Main Content: Processes + Table + Side Details ---
            div()
                .flex()
                .flex_1()
                .gap_6()
                .overflow_hidden()
                .child(
                    // Left Column: Process Overview
                    div()
                        .flex()
                        .flex_col()
                        .w(px(240.0))
                        .flex_shrink_0()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_4()
                        .gap_4()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(color_text_primary)
                                .child("进程概览"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(process_item("Google Chrome", "86", "↓ 5.32 MB/s", true))
                                .child(process_item("Telegram Desktop", "24", "↓ 2.41 MB/s", false))
                                .child(process_item("Steam Client", "18", "↓ 1.83 MB/s", false))
                                .child(process_item(
                                    "Visual Studio Code",
                                    "12",
                                    "↓ 1.02 MB/s",
                                    false,
                                )),
                        ),
                )
                .child(
                    // Center Column: Live Table
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_5()
                        .gap_4()
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("实时连接表 (324)"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(filter_tag_mini("全部", true))
                                        .child(filter_tag_mini("代理", false))
                                        .child(filter_tag_mini("直连", false)),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .overflow_hidden()
                                .child(
                                    // Table Header
                                    div()
                                        .flex()
                                        .items_center()
                                        .px_3()
                                        .py_2()
                                        .bg(rgb(0xF8FAFC))
                                        .rounded_md()
                                        .child(
                                            div()
                                                .flex()
                                                .w(px(120.0))
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("进程"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_1()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("目标"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .w(px(80.0))
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("协议"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .w(px(100.0))
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("规则"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .w(px(120.0))
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("速率"),
                                        ),
                                )
                                .child(
                                    div().flex().flex_col().flex_1().overflow_hidden().children(
                                        (0..8).map(|i| {
                                            connection_row(
                                                if i % 2 == 0 {
                                                    "youtube.com:443"
                                                } else {
                                                    "github.com:443"
                                                },
                                                if i % 2 == 0 { "Streaming" } else { "Proxy" },
                                                if i % 2 == 0 {
                                                    "↓ 4.2 MB/s"
                                                } else {
                                                    "↓ 0.9 MB/s"
                                                },
                                                i % 2 == 0,
                                            )
                                        }),
                                    ),
                                ),
                        ),
                )
                .child(
                    // Right Column: Path & Details
                    div()
                        .flex()
                        .flex_col()
                        .w(px(320.0))
                        .flex_shrink_0()
                        .gap_6()
                        .child(
                            // Connection Path
                            div()
                                .flex()
                                .flex_col()
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_2xl()
                                .p_5()
                                .gap_4()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("连接路径"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_around()
                                        .child(path_node("App", "Chrome"))
                                        .child(
                                            div().text_xs().text_color(color_text_muted).child("→"),
                                        )
                                        .child(path_node("Rule", "Streaming"))
                                        .child(
                                            div().text_xs().text_color(color_text_muted).child("→"),
                                        )
                                        .child(path_node("Node", "HK 01")),
                                ),
                        )
                        .child(
                            // Details
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_2xl()
                                .p_5()
                                .gap_4()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("连接详情"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_3()
                                        .child(detail_row("域名", "youtube.com:443"))
                                        .child(detail_row("远程 IP", "142.250.72.78"))
                                        .child(detail_row("本地端口", "51562"))
                                        .child(detail_row("加密", "TLS")),
                                )
                                .child(
                                    div()
                                        .mt_auto()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .h(px(40.0))
                                        .w_full()
                                        .bg(rgb(0xFEF2F2))
                                        .text_color(rgb(0xEF4444))
                                        .rounded_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .child("断开连接"),
                                ),
                        ),
                ),
        )
}

fn connection_metric_card(
    label: &'static str,
    val: &'static str,
    sub: &'static str,
    color: Rgba,
    icon_name: IconName,
) -> impl IntoElement {
    let mut bg: Hsla = color.into();
    bg.a = 0.1;
    div()
        .flex()
        .flex_1()
        .bg(white())
        .border_1()
        .border_color(rgb(0xE2E8F0))
        .rounded_2xl()
        .p_4()
        .items_center()
        .gap_4()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(44.0))
                .bg(bg)
                .rounded_xl()
                .child(icon(icon_name, 22.0, color.into())),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_xs().text_color(rgb(0x64748B)).child(label))
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .child(val),
                )
                .child(div().text_xs().text_color(rgb(0x94A3B8)).child(sub)),
        )
}

fn process_item(
    name: &'static str,
    count: &'static str,
    speed: &'static str,
    active: bool,
) -> impl IntoElement {
    let mut bg: Hsla = rgb(0xEEF2FF).into();
    bg.a = 0.5;
    let item_bg: Hsla = if active { bg } else { white() };
    div()
        .flex()
        .items_center()
        .justify_between()
        .p_2()
        .bg(item_bg)
        .rounded_lg()
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().size(px(24.0)).bg(rgb(0xF1F5F9)).rounded_md())
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x0F172A))
                                .child(name),
                        )
                        .child(div().text_xs().text_color(rgb(0x94A3B8)).child(speed)),
                ),
        )
        .child(div().text_xs().text_color(rgb(0x64748B)).child(count))
}

fn connection_row(
    target: &'static str,
    rule: &'static str,
    speed: &'static str,
    _active: bool,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .px_3()
        .py_3()
        .border_b_1()
        .border_color(rgb(0xF1F5F9))
        .child(
            div()
                .flex()
                .w(px(120.0))
                .child(div().size(px(24.0)).bg(rgb(0xF1F5F9)).rounded_full()),
        )
        .child(
            div()
                .flex()
                .flex_1()
                .flex_col()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .child(target),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(rgb(0x94A3B8))
                        .child("142.250.72.78"),
                ),
        )
        .child(
            div()
                .flex()
                .w(px(80.0))
                .text_xs()
                .text_color(rgb(0x64748B))
                .child("TCP"),
        )
        .child(
            div().flex().w(px(100.0)).child(
                div()
                    .flex()
                    .px_2()
                    .py_0p5()
                    .bg(rgb(0xEEF2FF))
                    .rounded_md()
                    .child(
                        div()
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x4F46E5))
                            .child(rule),
                    ),
            ),
        )
        .child(
            div()
                .flex()
                .w(px(120.0))
                .text_xs()
                .text_color(rgb(0x10B981))
                .child(speed),
        )
}

fn path_node(label: &'static str, val: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_size(px(10.0))
                .text_color(rgb(0x94A3B8))
                .child(label),
        )
        .child(div().size(px(32.0)).bg(rgb(0xF1F5F9)).rounded_full())
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x0F172A))
                .child(val),
        )
}

fn detail_row(label: &'static str, value: &'static str) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .child(div().text_xs().text_color(rgb(0x64748B)).child(label))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x0F172A))
                .child(value),
        )
}

fn filter_tag_mini(label: &'static str, active: bool) -> impl IntoElement {
    let bg: Hsla = if active {
        rgb(0xEEF2FF).into()
    } else {
        rgb(0xF8FAFC).into()
    };
    div()
        .flex()
        .px_2()
        .py_1()
        .bg(bg)
        .border_1()
        .border_color(if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) })
        .rounded_md()
        .child(
            div()
                .text_size(px(10.0))
                .text_color(if active { rgb(0x4F46E5) } else { rgb(0x64748B) })
                .child(label),
        )
}
