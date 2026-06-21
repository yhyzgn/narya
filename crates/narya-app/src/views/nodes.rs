use crate::components::{icon, IconName};
use crate::state::AppState;
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};

pub fn render_nodes_view(model: &Entity<AppState>, cx: &mut Context<AppShell>) -> impl IntoElement {
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
            // --- 1. Filter & Action Row ---
            div()
                .flex()
                .w_full()
                .gap_4()
                .child(filter_card(
                    "当前策略组",
                    "Proxy / 自动选择",
                    IconName::Dashboard,
                    color_brand,
                ))
                .child(filter_card(
                    "当前节点",
                    "香港 · HK 01",
                    IconName::Nodes,
                    color_brand,
                ))
                .child(filter_card(
                    "模式",
                    "规则模式",
                    IconName::Rules,
                    color_brand,
                ))
                .child(
                    // Speed Test Button
                    div()
                        .flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .h(px(80.0))
                        .bg(rgb(0x6366F1))
                        .rounded_2xl()
                        .cursor_pointer()
                        .gap_3()
                        .child(icon(IconName::Dashboard, 24.0, white()))
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(white())
                                .child("一键测速"),
                        ),
                ),
        )
        .child(
            // --- 2. Sub-Filter Row ---
            div()
                .flex()
                .w_full()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .flex()
                        .gap_6()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .w(px(280.0))
                                .h(px(40.0))
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_lg()
                                .px_3()
                                .gap_2()
                                .child(icon(IconName::About, 16.0, color_text_muted.into()))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(color_text_muted)
                                        .child("搜索节点、地区、协议..."),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(filter_tag("全部", true))
                                .child(filter_tag("低延迟", false))
                                .child(filter_tag("香港", false))
                                .child(filter_tag("日本", false))
                                .child(filter_tag("美国", false)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .child(dropdown_mini("按延迟排序"))
                        .child(dropdown_mini("全部协议")),
                ),
        )
        .child(
            // --- 3. Main Content: Strategy Groups + Node Grid ---
            div()
                .flex()
                .flex_1()
                .gap_6()
                .overflow_hidden()
                .child(
                    // Left Column: Strategy Groups
                    div()
                        .flex()
                        .flex_col()
                        .w(px(260.0))
                        .flex_shrink_0()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_4()
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
                                        .child("策略组"),
                                )
                                .child(div().text_lg().text_color(color_text_muted).child("+")),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(strategy_item(
                                    "Proxy",
                                    "自动选择",
                                    "38 / 128",
                                    "香港 · HK 01",
                                    true,
                                ))
                                .child(strategy_item(
                                    "Global",
                                    "全局代理",
                                    "36 / 128",
                                    "香港 · HK 01",
                                    false,
                                ))
                                .child(strategy_item(
                                    "Direct",
                                    "国内直连",
                                    "1 / 128",
                                    "DIRECT",
                                    false,
                                ))
                                .child(strategy_item(
                                    "AI Services",
                                    "分流模式",
                                    "28 / 128",
                                    "美国 · US 01",
                                    false,
                                ))
                                .child(strategy_item(
                                    "Streaming",
                                    "自动选择",
                                    "24 / 128",
                                    "日本 · JP 01",
                                    false,
                                )),
                        ),
                )
                .child(
                    // Right Column: Node Grid
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
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
                                        .child("节点列表"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_4()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("共 38 / 128 个节点"),
                                        )
                                        .child(icon(
                                            IconName::Dashboard,
                                            14.0,
                                            color_text_secondary.into(),
                                        )),
                                ),
                        )
                        .child(
                            // Two-Column Grid of Nodes
                            div()
                                .flex()
                                .flex_1()
                                .gap_4()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .gap_3()
                                        .child(node_card_full(
                                            "香港 · HK 01",
                                            "Shadowsocks",
                                            "48 ms",
                                            23,
                                            12.4,
                                            4.6,
                                            true,
                                        ))
                                        .child(node_card_full(
                                            "新加坡 · SG 01",
                                            "Hysteria2",
                                            "55 ms",
                                            20,
                                            11.2,
                                            4.1,
                                            false,
                                        ))
                                        .child(node_card_full(
                                            "台湾 · TW 02",
                                            "Trojan",
                                            "74 ms",
                                            21,
                                            6.3,
                                            2.8,
                                            false,
                                        )),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .gap_3()
                                        .child(node_card_full(
                                            "日本 · JP 01",
                                            "Vmess",
                                            "62 ms",
                                            18,
                                            9.8,
                                            3.2,
                                            false,
                                        ))
                                        .child(node_card_full(
                                            "美国 · US 01",
                                            "VLESS Reality",
                                            "110 ms",
                                            32,
                                            8.7,
                                            2.9,
                                            false,
                                        ))
                                        .child(node_card_full(
                                            "德国 · DE 01",
                                            "TUIC",
                                            "156 ms",
                                            41,
                                            5.1,
                                            1.6,
                                            false,
                                        )),
                                ),
                        ),
                ),
        )
        .child(
            // --- 4. Bottom Panels ---
            div()
                .flex()
                .w_full()
                .h(px(260.0))
                .gap_6()
                .child(
                    // Latency Trend
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_6()
                        .gap_4()
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("延迟趋势"),
                                )
                                .child(dropdown_mini("1 分钟")),
                        )
                        .child(div().flex().flex_1().bg(rgb(0xFAFBFE)).rounded_xl()),
                )
                .child(
                    // Node Details
                    div()
                        .flex()
                        .flex_col()
                        .w(px(480.0))
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_6()
                        .gap_6()
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("节点详情 (香港 · HK 01)"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(div().size(px(6.0)).bg(color_success).rounded_full())
                                        .child(
                                            div().text_xs().text_color(color_success).child("可用"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(node_info_row("地址", "hkg01.narya.net:443"))
                                .child(node_info_row("协议", "Shadowsocks"))
                                .child(node_info_row("加密", "2022-blake3-aes-128-gcm")),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_3()
                                .child(action_button("设为默认", color_brand, true))
                                .child(action_button("复制信息", color_text_secondary, false))
                                .child(action_button("编辑", color_text_secondary, false)),
                        ),
                ),
        )
}

fn filter_card(
    title: &'static str,
    val: &'static str,
    icon_name: IconName,
    accent: Rgba,
) -> impl IntoElement {
    let mut bg: Hsla = accent.into();
    bg.a = 0.1;

    div()
        .flex()
        .flex_1()
        .bg(white())
        .border_1()
        .border_color(rgb(0xE2E8F0))
        .rounded_2xl()
        .p_5()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(40.0))
                        .bg(bg)
                        .rounded_full()
                        .child(icon(icon_name, 20.0, accent.into())),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(div().text_xs().text_color(rgb(0x64748B)).child(title))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x0F172A))
                                .child(val),
                        ),
                ),
        )
        .child(div().text_xs().text_color(rgb(0x94A3B8)).child("▾"))
}

fn filter_tag(label: &'static str, active: bool) -> impl IntoElement {
    let bg: Hsla = if active {
        rgb(0xEEF2FF).into()
    } else {
        white()
    };
    div()
        .flex()
        .px_4()
        .py_2()
        .bg(bg)
        .border_1()
        .border_color(if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) })
        .rounded_lg()
        .child(
            div()
                .text_xs()
                .font_weight(if active {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(if active { rgb(0x4F46E5) } else { rgb(0x64748B) })
                .child(label),
        )
}

fn dropdown_mini(label: &'static str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .bg(white())
        .border_1()
        .border_color(rgb(0xE2E8F0))
        .px_3()
        .py_2()
        .rounded_lg()
        .child(div().text_xs().text_color(rgb(0x0F172A)).child(label))
        .child(div().text_xs().text_color(rgb(0x94A3B8)).child("▾"))
}

fn strategy_item(
    name: &'static str,
    mode: &'static str,
    count: &'static str,
    active_node: &'static str,
    active: bool,
) -> impl IntoElement {
    let mut bg: Hsla = rgb(0xEEF2FF).into();
    bg.a = 0.5;

    let final_bg: Hsla = if active { bg } else { white() };

    div()
        .flex()
        .flex_col()
        .p_3()
        .bg(final_bg)
        .rounded_xl()
        .gap_1()
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .size(px(10.0))
                                .bg(if active { rgb(0x4F46E5) } else { rgb(0x94A3B8) })
                                .rounded_full(),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x0F172A))
                                .child(name),
                        ),
                )
                .child(div().text_xs().text_color(rgb(0x94A3B8)).child(mode)),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(div().text_xs().text_color(rgb(0x64748B)).child(count))
                .child(div().text_xs().text_color(rgb(0x10B981)).child(active_node)),
        )
}

fn node_card_full(
    name: &'static str,
    proto: &'static str,
    latency: &'static str,
    load: u32,
    down: f32,
    up: f32,
    active: bool,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .bg(white())
        .border_1()
        .border_color(if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) })
        .rounded_xl()
        .p_4()
        .gap_3()
        .child(
            div()
                .flex()
                .justify_between()
                .items_start()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(div().size(px(32.0)).bg(rgb(0xEF4444)).rounded_full())
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x0F172A))
                                        .child(name),
                                )
                                .child(div().text_xs().text_color(rgb(0x94A3B8)).child(proto)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x10B981))
                        .child(latency),
                ),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(div().size(px(6.0)).bg(rgb(0x10B981)).rounded_full())
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x64748B))
                                .child(format!("{}%", load)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x64748B))
                                .child(format!("↓ {:.1} MB/s", down)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x64748B))
                                .child(format!("↑ {:.1} MB/s", up)),
                        ),
                ),
        )
}

fn node_info_row(label: &'static str, value: &'static str) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_center()
        .child(div().text_xs().text_color(rgb(0x64748B)).child(label))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x0F172A))
                .child(value),
        )
}

fn action_button(label: &'static str, color: Rgba, primary: bool) -> impl IntoElement {
    let bg: Hsla = if primary {
        color.into()
    } else {
        rgb(0xF1F5F9).into()
    };
    let text: Hsla = if primary { white() } else { color.into() };
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .h(px(40.0))
        .bg(bg)
        .rounded_lg()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(text)
                .child(label),
        )
}
