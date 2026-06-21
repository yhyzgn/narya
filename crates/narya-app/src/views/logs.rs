use crate::components::{icon, IconName};
use crate::state::AppState;
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};

pub fn render_logs_view(model: &Entity<AppState>, _cx: &mut Context<AppShell>) -> impl IntoElement {
    let state = model.read(_cx);

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
                .child(log_metric_card("今日事件", "12,846", "较昨日 +8.6%", rgb(0x3B82F6), IconName::Logs))
                .child(log_metric_card("错误", "3", "较昨日 -2", color_danger, IconName::About))
                .child(log_metric_card("警告", "24", "较昨日 +6", color_warning, IconName::About))
                .child(log_metric_card("当前级别", "Info", "可查看全部日志", color_brand, IconName::About))
                .child(log_metric_card("日志大小", "18.6 MB", "占用磁盘空间", rgb(0x6366F1), IconName::Config))
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
                                .child(div().text_sm().text_color(color_text_muted).child("搜索日志内容、域名..."))
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(filter_tag_log("全部", true))
                                .child(filter_tag_log("Error", false))
                                .child(filter_tag_log("Warn", false))
                                .child(filter_tag_log("Info", false))
                                .child(filter_tag_log("Debug", false))
                        )
                )
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .child(dropdown_log("最近 1 小时"))
                        .child(toggle_log("自动滚动", true))
                )
        )
        .child(
            // --- 3. Main Content ---
            div()
                .flex()
                .flex_1()
                .gap_6()
                .overflow_hidden()
                .child(
                    // Left Column: Log Sources
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
                        .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(color_text_primary).child("日志来源"))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(source_item("sing-box core", "5,128", color_brand, true))
                                .child(source_item("sing-box daemon", "1,256", color_success, false))
                                .child(source_item("rule engine", "2,314", rgb(0x8B5CF6), false))
                                .child(source_item("subscription", "856", rgb(0xF59E0B), false))
                        )
                )
                .child(
                    // Center Column: Live Stream
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
                        .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(color_text_primary).child("实时日志流"))
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
                                        .child(div().flex().w(px(80.0)).text_xs().text_color(color_text_secondary).child("时间"))
                                        .child(div().flex().w(px(70.0)).text_xs().text_color(color_text_secondary).child("级别"))
                                        .child(div().flex().w(px(80.0)).text_xs().text_color(color_text_secondary).child("模块"))
                                        .child(div().flex().flex_1().text_xs().text_color(color_text_secondary).child("内容"))
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .overflow_hidden()
                                        .children(state.log_lines.iter().map(|log| {
                                            log_stream_row(
                                                log.time.clone(),
                                                log.level.clone(),
                                                log.module.clone(),
                                                log.content.clone()
                                            )
                                        }))
                                )
                        )
                )
                .child(
                    // Right Column: Details & Report
                    div()
                        .flex()
                        .flex_col()
                        .w(px(320.0))
                        .flex_shrink_0()
                        .gap_6()
                        .child(
                            // Log Details
                            div()
                                .flex()
                                .flex_col()
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_2xl()
                                .p_5()
                                .gap_4()
                                .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(color_text_primary).child("日志详情"))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .p_3()
                                        .bg(rgb(0xF8FAFC))
                                        .rounded_xl()
                                        .gap_2()
                                        .child(detail_row_log("时间", "2024-04-29 17:32:18"))
                                        .child(detail_row_log("级别", "INFO"))
                                        .child(detail_row_log("模块", "core"))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .mt_2()
                                                .p_3()
                                                .bg(rgb(0x0F172A))
                                                .rounded_lg()
                                                .child(div().text_size(px(10.0)).text_color(rgb(0xE2E8F0)).child("{ \"type\": \"select\", \"outbound\": \"HK 01\" }"))
                                        )
                                )
                        )
                        .child(
                            // Export
                            div()
                                .flex()
                                .flex_col()
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_2xl()
                                .p_5()
                                .gap_4()
                                .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(color_text_primary).child("导出诊断"))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_3()
                                        .child(checkbox_log("核心日志", true))
                                        .child(checkbox_log("配置摘要", true))
                                        .child(checkbox_log("系统信息", true))
                                )
                                .child(
                                    div()
                                        .mt_4()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .h(px(40.0))
                                        .w_full()
                                        .bg(color_brand)
                                        .text_color(white())
                                        .rounded_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .child("生成报告")
                                )
                        )
                )
        )
}

fn log_metric_card(
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

fn source_item(
    name: &'static str,
    count: &'static str,
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
                        .child(icon(IconName::Logs, 18.0, color.into())),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .child(name),
                ),
        )
        .child(div().text_xs().text_color(rgb(0x64748B)).child(count))
}

fn log_stream_row(time: String, level: String, module: String, msg: String) -> impl IntoElement {
    let level_color = match level.trim() {
        "ERROR" => rgb(0xEF4444),
        "WARN" => rgb(0xF59E0B),
        "DEBUG" => rgb(0x3B82F6),
        _ => rgb(0x10B981),
    };
    let mut level_bg: Hsla = level_color.into();
    level_bg.a = 0.1;

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
                .w(px(80.0))
                .text_xs()
                .text_color(rgb(0x94A3B8))
                .child(time),
        )
        .child(
            div().flex().w(px(70.0)).child(
                div()
                    .flex()
                    .px_1p5()
                    .py_0p5()
                    .bg(level_bg)
                    .rounded_md()
                    .child(
                        div()
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(level_color)
                            .child(level),
                    ),
            ),
        )
        .child(
            div()
                .flex()
                .w(px(80.0))
                .text_xs()
                .text_color(rgb(0x64748B))
                .child(module),
        )
        .child(
            div()
                .flex()
                .flex_1()
                .text_xs()
                .text_color(rgb(0x64748B))
                .child(msg),
        )
}

fn detail_row_log(label: &'static str, value: &'static str) -> impl IntoElement {
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

fn filter_tag_log(label: &'static str, active: bool) -> impl IntoElement {
    let bg: Hsla = if active {
        rgb(0xEEF2FF).into()
    } else {
        white()
    };
    let text: Hsla = if active {
        rgb(0x4F46E5).into()
    } else {
        rgb(0x64748B).into()
    };
    div()
        .flex()
        .px_3()
        .py_1p5()
        .bg(bg)
        .border_1()
        .border_color(if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) })
        .rounded_lg()
        .child(div().text_xs().text_color(text).child(label))
}

fn dropdown_log(label: &'static str) -> impl IntoElement {
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

fn toggle_log(label: &'static str, active: bool) -> impl IntoElement {
    let bg: Hsla = if active {
        rgb(0x10B981).into()
    } else {
        rgb(0xE2E8F0).into()
    };
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().text_xs().text_color(rgb(0x64748B)).child(label))
        .child(
            div()
                .flex()
                .items_center()
                .w(px(32.0))
                .h(px(18.0))
                .bg(bg)
                .rounded_full()
                .px_0p5()
                .child({
                    let mut dot = div().flex().size(px(14.0)).bg(white()).rounded_full();
                    if active {
                        dot = dot.ml_auto();
                    }
                    dot
                }),
        )
}

fn checkbox_log(label: &'static str, checked: bool) -> impl IntoElement {
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
