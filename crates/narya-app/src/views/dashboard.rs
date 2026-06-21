use crate::components::{icon, IconName};
use crate::state::AppState;
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};

pub fn render_dashboard_view(
    model: &Entity<AppState>,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let _state = model.read(cx);

    // --- High-Fidelity Design System ---
    let color_bg = rgb(0xF8FAFC);
    let color_card = rgb(0xFFFFFF);
    let color_border = rgb(0xE2E8F0);
    let color_text_primary = rgb(0x0F172A);
    let color_text_secondary = rgb(0x64748B);
    let color_text_muted = rgb(0x94A3B8);
    let color_brand = rgb(0x4F46E5);
    let color_success = rgb(0x10B981);

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(color_bg)
        .gap_6()
        .child(
            // --- 1. Top Row: Control Cards ---
            div()
                .flex()
                .w_full()
                .gap_6()
                .child(status_control_card(
                    "系统代理",
                    "管理系统网络代理设置",
                    "已启用",
                    "规则模式",
                    IconName::Settings,
                    color_brand,
                    true,
                ))
                .child(status_control_card(
                    "TUN 虚拟网卡",
                    "拦截并代理所有网络流量 (推荐)",
                    "已启用",
                    "智能路由",
                    IconName::Nodes,
                    color_success,
                    true,
                )),
        )
        .child(
            // --- 2. Middle Row: Quick Connect & Overview ---
            div()
                .flex()
                .w_full()
                .gap_6()
                .child(
                    // Quick Connect Panel
                    div()
                        .flex()
                        .flex_col()
                        .w(px(400.0))
                        .flex_shrink_0()
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
                                        .child("快速连接"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(color_brand)
                                        .cursor_pointer()
                                        .child("管理"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(quick_node_item(
                                    "香港 · HK 01",
                                    "Shadowsocks",
                                    "48 ms",
                                    color_success,
                                ))
                                .child(quick_node_item(
                                    "日本 · JP 01",
                                    "Shadowsocks",
                                    "62 ms",
                                    color_success,
                                ))
                                .child(quick_node_item(
                                    "美国 · US 01",
                                    "Vmess",
                                    "110 ms",
                                    rgb(0xEF4444),
                                ))
                                .child(quick_node_item(
                                    "新加坡 · SG 01",
                                    "Shadowsocks",
                                    "55 ms",
                                    color_success,
                                )),
                        ),
                )
                .child(
                    // Network Overview Panel
                    div()
                        .flex()
                        .flex_1()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_6()
                        .gap_6()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .gap_4()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("网络概览"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_1()
                                        .bg(rgb(0xFAFBFE))
                                        .rounded_xl()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(color_text_muted)
                                                .child("Network Latency Chart Placeholder"),
                                        ),
                                ),
                        )
                        .child(
                            // Side Stats
                            div()
                                .flex()
                                .flex_col()
                                .w(px(140.0))
                                .flex_shrink_0()
                                .gap_6()
                                .child(overview_stat("节点延迟", "48", "ms", "当前节点"))
                                .child(overview_stat("可用节点", "38", "/ 128", "在线 / 总数"))
                                .child(overview_stat("负载", "23", "%", "当前节点负载"))
                                .child(overview_stat("丢包率", "0.2", "%", "当前节点")),
                        ),
                ),
        )
        .child(
            // --- 3. Bottom Row: Three columns ---
            div()
                .flex()
                .w_full()
                .gap_6()
                .child(
                    // Traffic Usage
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
                                        .child("流量使用"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(color_text_secondary)
                                        .bg(color_bg)
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .child("今日 ▾"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_8()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_4()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .child("总流量"),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_baseline()
                                                        .gap_1()
                                                        .child(
                                                            div()
                                                                .text_2xl()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(color_text_primary)
                                                                .child("1.26"),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(color_text_primary)
                                                                .child("GB"),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_success)
                                                        .child("↓ 842 MB ↑ 436 MB"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .child("连接数"),
                                                )
                                                .child(
                                                    div()
                                                        .text_2xl()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(color_text_primary)
                                                        .child("324"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_muted)
                                                        .child("峰值 1280"),
                                                ),
                                        ),
                                )
                                .child(div().flex().flex_1().bg(rgb(0xFAFBFE)).rounded_xl()),
                        ),
                )
                .child(
                    // Connection Stats
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
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
                                        .child("连接统计"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(color_text_secondary)
                                        .bg(color_bg)
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .child("按协议 ▾"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(protocol_row("Shadowsocks", 63.5, color_brand))
                                .child(protocol_row("Vmess", 23.4, color_success))
                                .child(protocol_row("Trojan", 8.7, rgb(0x3B82F6)))
                                .child(protocol_row("Hysteria2", 4.4, rgb(0xF59E0B))),
                        )
                        .child(
                            div()
                                .mt_auto()
                                .flex()
                                .justify_between()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(color_text_secondary)
                                        .child("总连接数"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("324"),
                                ),
                        ),
                )
                .child(
                    // Activity Logs
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
                                        .child("活动日志"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(color_brand)
                                        .cursor_pointer()
                                        .child("更多 >"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(log_item("17:25:21", "已连接到 香港 · HK 01", color_success))
                                .child(log_item("17:25:20", "系统代理已启用", color_brand))
                                .child(log_item("17:25:18", "TUN 模式已启用", color_success))
                                .child(log_item("17:25:10", "应用启动成功", color_text_muted))
                                .child(log_item(
                                    "17:25:09",
                                    "正在更新 GeoIP 数据库",
                                    rgb(0x3B82F6),
                                )),
                        ),
                ),
        )
}

fn status_control_card(
    title: &'static str,
    desc: &'static str,
    status: &'static str,
    mode: &'static str,
    icon_name: IconName,
    accent: Rgba,
    active: bool,
) -> impl IntoElement {
    let mut bg: Hsla = accent.into();
    bg.a = 0.1;

    div()
        .flex()
        .flex_1()
        .bg(rgb(0xFFFFFF))
        .border_1()
        .border_color(rgb(0xE2E8F0))
        .rounded_2xl()
        .p_6()
        .flex_col()
        .gap_6()
        .child(
            div()
                .flex()
                .justify_between()
                .items_start()
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(48.0))
                                .bg(bg)
                                .rounded_xl()
                                .child(icon(icon_name, 24.0, accent.into())),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x0F172A))
                                        .child(title),
                                )
                                .child(div().text_xs().text_color(rgb(0x64748B)).child(desc)),
                        ),
                )
                .child(
                    // Large Toggle
                    div()
                        .flex()
                        .items_center()
                        .w(px(52.0))
                        .h(px(28.0))
                        .bg(if active { accent } else { rgb(0xE2E8F0) })
                        .rounded_full()
                        .px_1()
                        .child(div().size(px(22.0)).bg(white()).rounded_full().ml_auto()),
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
                        .gap_2()
                        .child(icon(IconName::About, 14.0, rgb(0x10B981).into()))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x10B981))
                                .child(status),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .cursor_pointer()
                        .child(div().text_sm().text_color(rgb(0x0F172A)).child(mode))
                        .child(div().text_sm().text_color(rgb(0x94A3B8)).child(">")),
                ),
        )
}

fn quick_node_item(
    name: &'static str,
    proto: &'static str,
    latency: &'static str,
    latency_color: Rgba,
) -> impl IntoElement {
    let mut bg: Hsla = latency_color.into();
    bg.a = 0.1;

    div()
        .flex()
        .items_center()
        .justify_between()
        .py_3()
        .border_b_1()
        .border_color(rgb(0xF1F5F9))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().size(px(24.0)).bg(rgb(0xEF4444)).rounded_full()) // Flag placeholder
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
            div().flex().px_2().py_0p5().bg(bg).rounded_md().child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(latency_color)
                    .child(latency),
            ),
        )
}

fn overview_stat(
    label: &'static str,
    val: &'static str,
    unit: &'static str,
    desc: &'static str,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(rgb(0x64748B)).child(label))
        .child(
            div()
                .flex()
                .items_baseline()
                .gap_1()
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .child(val),
                )
                .child(div().text_xs().text_color(rgb(0x64748B)).child(unit)),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(rgb(0x94A3B8))
                .child(desc),
        )
}

fn protocol_row(name: &'static str, pct: f32, color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .justify_between()
                .child(div().text_xs().text_color(rgb(0x64748B)).child(name))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .child(format!("{:.1}%", pct)),
                ),
        )
        .child(
            div()
                .flex()
                .w_full()
                .h(px(4.0))
                .bg(rgb(0xF1F5F9))
                .rounded_full()
                .child(
                    div()
                        .flex()
                        .w(relative(pct / 100.0))
                        .h_full()
                        .bg(color)
                        .rounded_full(),
                ),
        )
}

fn log_item(time: &'static str, msg: &'static str, dot_color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(div().size(px(6.0)).bg(dot_color).rounded_full())
        .child(div().text_xs().text_color(rgb(0x94A3B8)).child(time))
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(rgb(0x64748B))
                .child(msg),
        )
}
