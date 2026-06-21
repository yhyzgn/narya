use crate::components::{icon, IconName};
use crate::state::AppState;
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};

pub fn render_rules_view(
    _model: &Entity<AppState>,
    _cx: &mut Context<AppShell>,
) -> impl IntoElement {
    // --- Design System ---
    let color_bg = rgb(0xF8FAFC);
    let color_card = rgb(0xFFFFFF);
    let color_border = rgb(0xE2E8F0);
    let color_brand = rgb(0x4F46E5);
    let color_success = rgb(0x10B981);
    let color_danger = rgb(0xEF4444);
    let color_warning = rgb(0xF59E0B);
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
                .child(rule_metric_card(
                    "规则总数",
                    "1,284",
                    "较昨日 +32",
                    rgb(0x6366F1),
                    IconName::Rules,
                ))
                .child(rule_metric_card(
                    "规则组",
                    "12",
                    "8 启用 / 4 禁用",
                    rgb(0x8B5CF6),
                    IconName::Config,
                ))
                .child(rule_metric_card(
                    "今日命中",
                    "18,642",
                    "较昨日 +12.6%",
                    color_success,
                    IconName::Dashboard,
                ))
                .child(rule_metric_card(
                    "冲突检测",
                    "0 error / 2 warning",
                    "需要注意的规则冲突",
                    color_warning,
                    IconName::About,
                ))
                .child(rule_metric_card(
                    "默认动作",
                    "Proxy",
                    "Proxy / 自动选择",
                    color_brand,
                    IconName::Nodes,
                )),
        )
        .child(
            // --- 2. Main Content: Groups + Editor + Simulator ---
            div()
                .flex()
                .flex_1()
                .gap_6()
                .overflow_hidden()
                .child(
                    // Left Column: Rule Groups
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
                                        .child("规则组"),
                                )
                                .child(div().text_lg().text_color(color_text_muted).child("+")),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(rule_group_item(
                                    "国内直连",
                                    "428 条规则",
                                    "命中 8,642",
                                    color_success,
                                    true,
                                ))
                                .child(rule_group_item(
                                    "AI Services",
                                    "86 条规则",
                                    "命中 2,173",
                                    rgb(0x8B5CF6),
                                    false,
                                ))
                                .child(rule_group_item(
                                    "Streaming",
                                    "132 条规则",
                                    "命中 4,218",
                                    color_brand,
                                    false,
                                ))
                                .child(rule_group_item(
                                    "Gaming",
                                    "54 条规则",
                                    "命中 1,942",
                                    rgb(0x6366F1),
                                    false,
                                ))
                                .child(rule_group_item(
                                    "Ads Block",
                                    "312 条规则",
                                    "命中 6,321",
                                    color_danger,
                                    false,
                                )),
                        ),
                )
                .child(
                    // Center Column: Editor
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
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(color_text_primary)
                                                .child("规则编辑器 >"),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(color_brand)
                                                .child("Streaming"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(mini_btn("添加规则", color_brand, true))
                                        .child(mini_btn("批量导入", color_text_secondary, false)),
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
                                                .w(px(60.0))
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("优先级"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .w(px(120.0))
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("类型"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_1()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("匹配内容"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .w(px(80.0))
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("动作"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .w(px(100.0))
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("状态"),
                                        ),
                                )
                                .child(
                                    div().flex().flex_col().flex_1().overflow_hidden().children(
                                        (1..9).map(|i| {
                                            rule_editor_row(
                                                i,
                                                if i % 3 == 0 {
                                                    "domain-suffix"
                                                } else if i % 3 == 1 {
                                                    "ip-cidr"
                                                } else {
                                                    "domain-keyword"
                                                },
                                                if i % 2 == 0 { "youtube.com" } else { "netflix" },
                                                if i % 2 == 0 { "proxy" } else { "direct" },
                                            )
                                        }),
                                    ),
                                ),
                        ),
                )
                .child(
                    // Right Column: Simulator & Alerts
                    div()
                        .flex()
                        .flex_col()
                        .w(px(320.0))
                        .flex_shrink_0()
                        .gap_6()
                        .child(
                            // Simulator
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
                                        .child("规则模拟器"),
                                )
                                .child(sim_input("目标", "youtube.com"))
                                .child(sim_input("进程", "Chrome.exe"))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .h(px(40.0))
                                        .bg(color_brand)
                                        .text_color(white())
                                        .rounded_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .child("运行模拟"),
                                ),
                        )
                        .child(
                            // Alerts
                            div()
                                .flex()
                                .flex_col()
                                .bg(rgb(0xFFFBEB))
                                .border_1()
                                .border_color(rgb(0xFEF3C7))
                                .rounded_2xl()
                                .p_5()
                                .gap_3()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(icon(IconName::About, 14.0, rgb(0xD97706).into()))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0xD97706))
                                                .child("冲突提示"),
                                        ),
                                )
                                .child(div().text_xs().text_color(rgb(0xB45309)).child(
                                    "存在 2 条域名规则重叠：youtube.com 同时匹配多个规则。",
                                )),
                        ),
                ),
        )
}

fn rule_metric_card(
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

fn rule_group_item(
    name: &'static str,
    rules: &'static str,
    hits: &'static str,
    color: Rgba,
    active: bool,
) -> impl IntoElement {
    let mut bg: Hsla = color.into();
    bg.a = 0.1;
    let item_bg: Hsla = if active { bg } else { white() };
    div()
        .flex()
        .items_center()
        .justify_between()
        .p_3()
        .bg(item_bg)
        .rounded_xl()
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .size(px(32.0))
                        .bg(bg)
                        .rounded_lg()
                        .items_center()
                        .justify_center()
                        .child(icon(IconName::Rules, 18.0, color.into())),
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
                                .child(rules),
                        ),
                ),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(rgb(0x64748B))
                .child(hits),
        )
}

fn rule_editor_row(
    priority: usize,
    rule_type: &'static str,
    payload: &'static str,
    action: &'static str,
) -> impl IntoElement {
    let action_color = if action == "proxy" {
        rgb(0x4F46E5)
    } else {
        rgb(0x10B981)
    };
    let mut action_bg: Hsla = action_color.into();
    action_bg.a = 0.1;

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
                .w(px(60.0))
                .text_xs()
                .text_color(rgb(0x94A3B8))
                .child(priority.to_string()),
        )
        .child(
            div()
                .flex()
                .w(px(120.0))
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x64748B))
                .child(rule_type),
        )
        .child(
            div()
                .flex()
                .flex_1()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x0F172A))
                .child(payload),
        )
        .child(
            div().flex().w(px(80.0)).child(
                div()
                    .flex()
                    .px_2()
                    .py_0p5()
                    .bg(action_bg)
                    .rounded_md()
                    .child(
                        div()
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(action_color)
                            .child(action),
                    ),
            ),
        )
        .child(
            div()
                .flex()
                .w(px(100.0))
                .items_center()
                .gap_2()
                .child(div().size(px(6.0)).bg(rgb(0x10B981)).rounded_full())
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(rgb(0x10B981))
                        .child("启用"),
                ),
        )
}

fn sim_input(label: &'static str, val: &'static str) -> impl IntoElement {
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

fn mini_btn(label: &'static str, color: Rgba, primary: bool) -> impl IntoElement {
    let bg: Hsla = if primary { color.into() } else { white() };
    let text: Hsla = if primary { white() } else { color.into() };
    div()
        .flex()
        .px_3()
        .py_1()
        .bg(bg)
        .border_1()
        .border_color(if primary { color } else { rgb(0xE2E8F0) })
        .rounded_md()
        .cursor_pointer()
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(text)
                .child(label),
        )
}
