use crate::components::{icon, IconName};
use crate::state::AppState;
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};

pub fn render_tools_view(
    _model: &Entity<AppState>,
    _cx: &mut Context<AppShell>,
) -> impl IntoElement {
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
                .child(tool_metric_card(
                    "当前网络",
                    "正常",
                    "延迟 48 ms · 丢包 0.2%",
                    color_success,
                    IconName::Dashboard,
                ))
                .child(tool_metric_card(
                    "DNS 延迟",
                    "18 ms",
                    "1.1.1.1 · 解析正常",
                    rgb(0x3B82F6),
                    IconName::Nodes,
                ))
                .child(tool_metric_card(
                    "出口 IP",
                    "香港 · HK",
                    "103.77.12.34",
                    color_brand,
                    IconName::Github,
                ))
                .child(tool_metric_card(
                    "最近诊断",
                    "17:34:20",
                    "Ping 测试 · 成功",
                    rgb(0x6366F1),
                    IconName::Terminal,
                ))
                .child(tool_metric_card(
                    "报告数量",
                    "12",
                    "本月生成 8 份报告",
                    rgb(0x8B5CF6),
                    IconName::Logs,
                )),
        )
        .child(
            // --- 2. Main Content ---
            div()
                .flex()
                .flex_1()
                .gap_6()
                .overflow_hidden()
                .child(
                    // Left Column: Tools List
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
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(color_text_primary)
                                .child("诊断工具"),
                        )
                        .child(tool_item(
                            "Ping 测试",
                            "检测主机可达性与网络延迟",
                            IconName::Dashboard,
                            rgb(0x3B82F6),
                            true,
                        ))
                        .child(tool_item(
                            "DNS 查询",
                            "查询 DNS 记录与解析延迟",
                            IconName::Nodes,
                            rgb(0x6366F1),
                            false,
                        ))
                        .child(tool_item(
                            "GeoIP 查询",
                            "查询 IP 地理位置与 ASN",
                            IconName::About,
                            rgb(0x8B5CF6),
                            false,
                        ))
                        .child(tool_item(
                            "Speed Test",
                            "网络带宽实测",
                            IconName::Terminal,
                            rgb(0xF59E0B),
                            false,
                        )),
                )
                .child(
                    // Center Column: Work Area (Ping Test Example)
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_5()
                        .gap_6()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("Ping 测试"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_4()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_1()
                                                .child(tool_input("目标地址", "google.com")),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_1()
                                                .child(tool_input("出口节点", "香港 · HK 01")),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .h(px(42.0))
                                        .w_full()
                                        .bg(color_brand)
                                        .text_color(white())
                                        .rounded_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .child("开始测试"),
                                ),
                        )
                        .child(
                            div().flex().flex_1().overflow_hidden().child(
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
                                                    .w(px(40.0))
                                                    .text_xs()
                                                    .text_color(color_text_secondary)
                                                    .child("#"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_1()
                                                    .text_xs()
                                                    .text_color(color_text_secondary)
                                                    .child("IP 地址"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .w(px(80.0))
                                                    .text_xs()
                                                    .text_color(color_text_secondary)
                                                    .child("延迟 (ms)"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .w(px(80.0))
                                                    .text_xs()
                                                    .text_color(color_text_secondary)
                                                    .child("状态"),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .flex_1()
                                            .overflow_hidden()
                                            .children((1..6).map(|i| {
                                                ping_row(i, "142.250.72.78", "44.2", true)
                                            })),
                                    ),
                            ),
                        ),
                )
                .child(
                    // Right Column: Recommendations
                    div()
                        .flex()
                        .flex_col()
                        .w(px(320.0))
                        .flex_shrink_0()
                        .gap_6()
                        .child(
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
                                        .child("诊断建议"),
                                )
                                .child(suggestion_item(
                                    "网络正常，延迟和丢包率表现良好",
                                    color_success,
                                ))
                                .child(suggestion_item("GeoSite 数据库 2 天前更新", rgb(0xF59E0B))),
                        ),
                ),
        )
        .child(
            // --- 3. Bottom Panels ---
            div()
                .flex()
                .w_full()
                .h(px(220.0))
                .gap_6()
                .child(
                    // Routing Path
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
                                .child("路由路径 (google.com)"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_1()
                                .items_center()
                                .justify_around()
                                .child(path_step("本机", "192.168.1.100", IconName::Settings))
                                .child(div().text_xs().text_color(color_text_muted).child("→"))
                                .child(path_step("Narya", "规则模式", IconName::Dashboard))
                                .child(div().text_xs().text_color(color_text_muted).child("→"))
                                .child(path_step("香港 HK 01", "103.77.12.34", IconName::Nodes))
                                .child(div().text_xs().text_color(color_text_muted).child("→"))
                                .child(path_step("目标", "google.com", IconName::Github)),
                        ),
                )
                .child(
                    // One-click Report
                    div()
                        .flex()
                        .flex_col()
                        .w(px(400.0))
                        .flex_shrink_0()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_5()
                        .gap_6()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(color_text_primary)
                                .child("一键报告"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_3()
                                .child(checkbox_tool("系统代理", true))
                                .child(checkbox_tool("TUN 状态", true))
                                .child(checkbox_tool("内核日志", true))
                                .child(checkbox_tool("DNS 缓存", false)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .h(px(40.0))
                                .w_full()
                                .bg(color_brand)
                                .text_color(white())
                                .rounded_lg()
                                .font_weight(FontWeight::BOLD)
                                .child("生成报告"),
                        ),
                ),
        )
}

fn tool_metric_card(
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

fn tool_item(
    name: &'static str,
    desc: &'static str,
    icon_name: IconName,
    color: Rgba,
    active: bool,
) -> impl IntoElement {
    let mut bg: Hsla = color.into();
    bg.a = 0.1;
    let item_bg: Hsla = if active { bg } else { white() };
    div()
        .flex()
        .items_center()
        .gap_3()
        .p_3()
        .bg(item_bg)
        .rounded_xl()
        .child(
            div()
                .flex()
                .size(px(32.0))
                .bg(bg)
                .rounded_lg()
                .items_center()
                .justify_center()
                .child(icon(icon_name, 18.0, color.into())),
        )
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
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(rgb(0x94A3B8))
                        .child(desc),
                ),
        )
}

fn tool_input(label: &'static str, val: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(rgb(0x64748B)).child(label))
        .child(
            div()
                .flex()
                .px_3()
                .py_2()
                .bg(rgb(0xF8FAFC))
                .border_1()
                .border_color(rgb(0xE2E8F0))
                .rounded_lg()
                .text_sm()
                .text_color(rgb(0x0F172A))
                .child(val),
        )
}

fn ping_row(index: usize, ip: &'static str, lat: &'static str, success: bool) -> impl IntoElement {
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
                .w(px(40.0))
                .text_xs()
                .text_color(rgb(0x94A3B8))
                .child(index.to_string()),
        )
        .child(
            div()
                .flex()
                .flex_1()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x0F172A))
                .child(ip),
        )
        .child(
            div()
                .flex()
                .w(px(80.0))
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x10B981))
                .child(lat),
        )
        .child(
            div()
                .flex()
                .w(px(80.0))
                .items_center()
                .gap_2()
                .child(
                    div()
                        .size(px(6.0))
                        .bg(if success {
                            rgb(0x10B981)
                        } else {
                            rgb(0xEF4444)
                        })
                        .rounded_full(),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(if success {
                            rgb(0x10B981)
                        } else {
                            rgb(0xEF4444)
                        })
                        .child("成功"),
                ),
        )
}

fn path_step(label: &'static str, val: &'static str, icon_name: IconName) -> impl IntoElement {
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
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(40.0))
                .bg(rgb(0xF1F5F9))
                .rounded_xl()
                .child(icon(icon_name, 20.0, rgb(0x64748B).into())),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x0F172A))
                .child(val),
        )
}

fn suggestion_item(text: &'static str, color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().size(px(6.0)).bg(color).rounded_full())
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(rgb(0x64748B))
                .child(text),
        )
}

fn checkbox_tool(label: &'static str, checked: bool) -> impl IntoElement {
    let bg: Hsla = if checked {
        rgb(0x4F46E5).into()
    } else {
        white()
    };
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(16.0))
                .bg(bg)
                .border_1()
                .border_color(if checked {
                    rgb(0x4F46E5)
                } else {
                    rgb(0xE2E8F0)
                })
                .rounded_sm()
                .child(if checked {
                    icon(IconName::Dashboard, 10.0, white()).into_any_element()
                } else {
                    div().into_any_element()
                }),
        )
        .child(div().text_xs().text_color(rgb(0x0F172A)).child(label))
}
