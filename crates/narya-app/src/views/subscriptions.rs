use crate::components::{icon, IconName};
use crate::state::{AppState, SubscriptionTab};
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};
use narya_core::Subscription as NaryaSubscription;

pub fn render_subscriptions_view(
    model: &Entity<AppState>,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let state = model.read(cx);

    // --- Definitive Colors from PNG ---
    let color_bg = rgb(0xF8FAFC);
    let color_card = rgb(0xFFFFFF);
    let color_border = rgb(0xE2E8F0);
    let color_brand = rgb(0x4F46E5);
    let color_success = rgb(0x10B981);
    let color_text_primary = rgb(0x0F172A);
    let color_text_secondary = rgb(0x64748B);
    let color_text_muted = rgb(0x94A3B8);

    let selected_sub_id = state.selected_subscription_id.clone();
    let selected_sub = selected_sub_id
        .as_ref()
        .and_then(|id| state.subscriptions.iter().find(|s| s.id == *id));

    // Pre-calculate owned strings
    let current_sub_name = selected_sub
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "未选择".to_string());
    let node_count_str = selected_sub
        .map(|s| s.node_count.to_string())
        .unwrap_or_else(|| "0".to_string());
    let node_stats_str = format!(
        "{} 可用 / 0 失败",
        selected_sub.map(|s| s.used_nodes).unwrap_or(0)
    );
    let remaining_traffic_str = format!(
        "{:.0} GB",
        selected_sub
            .map(|s| s.traffic_total - s.traffic_used)
            .unwrap_or(0.0)
    );
    let traffic_stats_str = format!(
        "已用 {:.0} GB / 总量 {:.1} TB",
        selected_sub.map(|s| s.traffic_used).unwrap_or(0.0),
        selected_sub
            .map(|s| s.traffic_total / 1000.0)
            .unwrap_or(0.0)
    );
    let expiration_str = selected_sub
        .map(|s| format!("{} 到期", s.expiration))
        .unwrap_or_else(|| "---".to_string());
    let sub_url_display = selected_sub
        .map(|s| s.url.clone())
        .unwrap_or_else(|| "---".to_string());

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(color_bg)
        .gap_4()
        .child(
            // 1. Top Metrics Row
            div()
                .flex()
                .w_full()
                .h(px(100.0))
                .gap_4()
                .child(metric_card(
                    "当前订阅",
                    current_sub_name.clone(),
                    Some("运行中"),
                    "类型: 远程订阅".to_string(),
                    IconName::Dashboard,
                    color_brand,
                ))
                .child(metric_card(
                    "节点总数",
                    node_count_str,
                    None,
                    node_stats_str,
                    IconName::Nodes,
                    rgb(0x3B82F6),
                ))
                .child(metric_card(
                    "剩余流量",
                    remaining_traffic_str,
                    None,
                    traffic_stats_str,
                    IconName::Config,
                    rgb(0x6366F1),
                ))
                .child(metric_card(
                    "到期时间",
                    "42 天".to_string(),
                    None,
                    expiration_str,
                    IconName::Subscriptions,
                    rgb(0x8B5CF6),
                ))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .w(px(160.0))
                        .flex_shrink_0()
                        .child(
                            div()
                                .h(px(42.0))
                                .bg(color_brand)
                                .text_color(white())
                                .rounded_lg()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .cursor_pointer()
                                .child(icon(IconName::Github, 14.0, white()))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .child("添加订阅"),
                                ),
                        )
                        .child(
                            div()
                                .h(px(42.0))
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_lg()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, {
                                    let model = model.clone();
                                    let active_id = selected_sub_id.clone();
                                    move |_, _, cx| {
                                        if let Some(id) = active_id.clone() {
                                            AppState::refresh_subscription(model.clone(), cx, id);
                                        }
                                    }
                                })
                                .child(icon(IconName::Github, 14.0, color_text_primary.into()))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("手动刷新"),
                                ),
                        ),
                ),
        )
        .child(
            // 2. Main Middle Section - Respecting user's mt_4
            div()
                .flex()
                .w_full()
                .gap_4()
                .flex_1()
                .overflow_hidden()
                .child(
                    // Column 1: Subscription Source List
                    div()
                        .flex()
                        .w(px(380.0))
                        .h_full() // FIX: Fixed height matching parent layout
                        .flex_shrink_0()
                        .flex_col()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_4()
                        .gap_2()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(color_text_primary)
                                .child("订阅源列表"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .h(px(40.0))
                                .bg(rgb(0xF8FAFC))
                                .border_1()
                                .border_color(color_border)
                                .rounded_lg()
                                .px_3()
                                .gap_2()
                                .child(icon(IconName::Github, 16.0, color_text_muted.into()))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if state.subscription_filter_text.is_empty() {
                                            color_text_muted
                                        } else {
                                            color_text_primary
                                        })
                                        .child(if state.subscription_filter_text.is_empty() {
                                            "搜索订阅名称或 URL".to_string()
                                        } else {
                                            state.subscription_filter_text.clone()
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .h_full()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .mt_2()
                                .overflow_hidden()
                                .child({
                                    let model = model.clone();
                                    list(
                                        state.subscription_list_state.clone(),
                                        move |index, _, cx| {
                                            let state = model.read(cx);
                                            if let Some(sub) = state.subscriptions.get(index) {
                                                let sub = sub.clone();
                                                let is_selected =
                                                    state.selected_subscription_id.as_ref()
                                                        == Some(&sub.id);
                                                let sub_id = sub.id.clone();
                                                let model = model.clone();

                                                return div()
                                                    .pb_2()
                                                    .child(subscription_card(
                                                        &sub,
                                                        is_selected,
                                                        move |_, _, cx| {
                                                            model.update(cx, |state, cx| {
                                                                state.selected_subscription_id =
                                                                    Some(sub_id.clone());
                                                                cx.notify();
                                                            });
                                                        },
                                                    ))
                                                    .into_any_element();
                                            }
                                            div().into_any_element()
                                        },
                                    )
                                    .flex_1()
                                }),
                        ),
                )
                .child(
                    // Column 2: Details Panel
                    div()
                        .flex()
                        .min_w(px(480.0))
                        .flex_1()
                        .flex_col()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_4()
                        .gap_2()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(color_text_primary)
                                .child("订阅详情"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .justify_between()
                                .items_baseline()
                                .flex()
                                .gap_4()
                                .border_b_1()
                                .border_color(color_border)
                                .child(tab_item(
                                    "概览",
                                    state.active_subscription_tab == SubscriptionTab::Overview,
                                    {
                                        let model = model.clone();
                                        move |_, _, cx| {
                                            model.update(cx, |state, cx| {
                                                state.set_subscription_tab(
                                                    SubscriptionTab::Overview,
                                                    cx,
                                                )
                                            });
                                        }
                                    },
                                ))
                                .child(tab_item(
                                    "节点",
                                    state.active_subscription_tab == SubscriptionTab::Nodes,
                                    {
                                        let model = model.clone();
                                        move |_, _, cx| {
                                            model.update(cx, |state, cx| {
                                                state.set_subscription_tab(
                                                    SubscriptionTab::Nodes,
                                                    cx,
                                                )
                                            });
                                        }
                                    },
                                ))
                                .child(tab_item(
                                    "规则",
                                    state.active_subscription_tab == SubscriptionTab::Rules,
                                    {
                                        let model = model.clone();
                                        move |_, _, cx| {
                                            model.update(cx, |state, cx| {
                                                state.set_subscription_tab(
                                                    SubscriptionTab::Rules,
                                                    cx,
                                                )
                                            });
                                        }
                                    },
                                ))
                                .child(tab_item(
                                    "转换",
                                    state.active_subscription_tab == SubscriptionTab::Conversion,
                                    {
                                        let model = model.clone();
                                        move |_, _, cx| {
                                            model.update(cx, |state, cx| {
                                                state.set_subscription_tab(
                                                    SubscriptionTab::Conversion,
                                                    cx,
                                                )
                                            });
                                        }
                                    },
                                ))
                                .child(tab_item(
                                    "高级",
                                    state.active_subscription_tab == SubscriptionTab::Advanced,
                                    {
                                        let model = model.clone();
                                        move |_, _, cx| {
                                            model.update(cx, |state, cx| {
                                                state.set_subscription_tab(
                                                    SubscriptionTab::Advanced,
                                                    cx,
                                                )
                                            });
                                        }
                                    },
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(form_row(
                                    "名称".to_string(),
                                    current_sub_name.clone(),
                                    false,
                                ))
                                .child(form_row("订阅 URL".to_string(), sub_url_display, true))
                                .child(form_row(
                                    "User-Agent".to_string(),
                                    "Narya/1.0.0 (Windows; sing-box)".to_string(),
                                    true,
                                ))
                                .child(form_row(
                                    "更新间隔".to_string(),
                                    "30 分钟".to_string(),
                                    true,
                                ))
                                .child(form_row(
                                    "目标内核".to_string(),
                                    "sing-box".to_string(),
                                    true,
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .mt_2()
                                .gap_2()
                                .p_4()
                                .border_1()
                                .border_color(color_border)
                                .rounded_lg()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("流量配额"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_8()
                                        .child(
                                            div()
                                                .size(px(72.0))
                                                .rounded_full()
                                                .border_4()
                                                .border_color(color_success)
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(color_success)
                                                        .font_weight(FontWeight::BOLD)
                                                        .child("35%"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .flex_col()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .justify_between()
                                                        .items_baseline()
                                                        .child(
                                                            div()
                                                                .flex_col()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(
                                                                            color_text_secondary,
                                                                        )
                                                                        .child("已用流量"),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_base()
                                                                        .font_weight(
                                                                            FontWeight::BOLD,
                                                                        )
                                                                        .text_color(color_success)
                                                                        .child(format!(
                                                                            "↓ {:.0} GB",
                                                                            selected_sub
                                                                                .map(|s| s
                                                                                    .traffic_used)
                                                                                .unwrap_or(0.0)
                                                                        )),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex_col()
                                                                .items_end()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(
                                                                            color_text_secondary,
                                                                        )
                                                                        .child("总量"),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_base()
                                                                        .font_weight(
                                                                            FontWeight::BOLD,
                                                                        )
                                                                        .text_color(
                                                                            color_text_primary,
                                                                        )
                                                                        .child(format!(
                                                                            "{:.1} TB",
                                                                            selected_sub
                                                                                .map(|s| s
                                                                                    .traffic_total
                                                                                    / 1000.0)
                                                                                .unwrap_or(0.0)
                                                                        )),
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .h(px(8.0))
                                                        .bg(rgb(0xF1F5F9))
                                                        .rounded_full()
                                                        .child(
                                                            div()
                                                                .w(relative(
                                                                    selected_sub
                                                                        .map(|s| {
                                                                            s.traffic_used
                                                                                / s.traffic_total
                                                                        })
                                                                        .unwrap_or(0.0)
                                                                        as f32,
                                                                ))
                                                                .h_full()
                                                                .bg(color_success)
                                                                .rounded_full(),
                                                        ),
                                                ),
                                        ),
                                )
                                .child(info_row(
                                    "到期时间".to_string(),
                                    selected_sub
                                        .map(|s| s.expiration.clone())
                                        .unwrap_or_else(|| "---".to_string()),
                                    false,
                                ))
                                .child(info_row(
                                    "上次更新".to_string(),
                                    selected_sub
                                        .map(|s| s.update_time.clone())
                                        .unwrap_or_else(|| "---".to_string()),
                                    true,
                                )),
                        ),
                )
                .child(
                    // Column 3: Status & Recognition
                    div()
                        .flex()
                        .w(px(320.0))
                        .flex_shrink_0()
                        .flex_col()
                        .gap_4()
                        .child(
                            // 3a. Update Status
                            div()
                                .flex()
                                .flex_col()
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_2xl()
                                .p_4()
                                .gap_1()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("更新状态"),
                                )
                                .child(
                                    div()
                                        .mt_1()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div().size(px(14.0)).bg(color_success).rounded_full(),
                                        )
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(color_success)
                                                .child("更新成功"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .items_baseline()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("延迟"),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(color_text_primary)
                                                .child("128 ms"),
                                        ),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .h(px(38.0))
                                        .bg(rgb(0xF1F5F9))
                                        .rounded_lg()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .gap_2()
                                        .child(icon(
                                            IconName::Github,
                                            14.0,
                                            color_text_secondary.into(),
                                        ))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(color_text_secondary)
                                                .child("查看更新日志"),
                                        ),
                                ),
                        )
                        .child(
                            // 3b. Auto Update
                            div()
                                .flex()
                                .flex_col()
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_2xl()
                                .p_4()
                                .gap_1()
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
                                                .child("自动更新"),
                                        )
                                        .child(
                                            div()
                                                .w(px(38.0))
                                                .h(px(20.0))
                                                .bg(color_brand)
                                                .rounded_full()
                                                .flex()
                                                .items_center()
                                                .px_1()
                                                .child(
                                                    div()
                                                        .size(px(14.0))
                                                        .bg(white())
                                                        .rounded_full()
                                                        .ml_auto(),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("更新间隔"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .bg(rgb(0xF8FAFC))
                                                .border_1()
                                                .border_color(rgb(0xE2E8F0))
                                                .px_2()
                                                .py_1()
                                                .rounded_md()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(0x0F172A))
                                                        .child("每 30 分钟"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x94A3B8))
                                                        .child("▾"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("流量触发"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .bg(rgb(0xF8FAFC))
                                                .border_1()
                                                .border_color(rgb(0xE2E8F0))
                                                .px_2()
                                                .py_1()
                                                .rounded_md()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(0x0F172A))
                                                        .child("低于 5% 时"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x94A3B8))
                                                        .child("▾"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("启动时更新"),
                                        )
                                        .child(
                                            div()
                                                .w(px(38.0))
                                                .h(px(20.0))
                                                .bg(color_brand)
                                                .rounded_full()
                                                .flex()
                                                .items_center()
                                                .px_1()
                                                .child(
                                                    div()
                                                        .size(px(14.0))
                                                        .bg(white())
                                                        .rounded_full()
                                                        .ml_auto(),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("静默失败通知"),
                                        )
                                        .child(
                                            div()
                                                .w(px(38.0))
                                                .h(px(20.0))
                                                .bg(color_brand)
                                                .rounded_full()
                                                .flex()
                                                .items_center()
                                                .px_1()
                                                .child(
                                                    div()
                                                        .size(px(14.0))
                                                        .bg(white())
                                                        .rounded_full()
                                                        .ml_auto(),
                                                ),
                                        ),
                                ),
                        )
                        .child(
                            // 3c. Format Recognition - COMPACT INLINE TAGS
                            div()
                                .flex()
                                .flex_col()
                                .bg(color_card)
                                .border_1()
                                .border_color(color_border)
                                .rounded_2xl()
                                .p_4()
                                .gap_1()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color_text_primary)
                                        .child("格式识别"),
                                )
                                .child({
                                    let detected_format = selected_sub
                                        .and_then(|s| s.format.as_deref())
                                        .unwrap_or("");
                                    div()
                                        .mt_2()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .flex_shrink_0()
                                                        .child("识别结果："),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .gap_1p5()
                                                        .child(recognition_tag(
                                                            "Clash",
                                                            detected_format.contains("Clash"),
                                                            color_success,
                                                        ))
                                                        .child(recognition_tag(
                                                            "V2Ray",
                                                            detected_format.contains("V2Ray"),
                                                            color_success,
                                                        ))
                                                        .child(recognition_tag(
                                                            "sing-box",
                                                            detected_format.contains("Sing-box"),
                                                            color_success,
                                                        ))
                                                        .child(recognition_tag(
                                                            "Base64",
                                                            detected_format.contains("Base64"),
                                                            color_success,
                                                        )),
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
                                                        .text_color(color_text_secondary)
                                                        .flex_shrink_0()
                                                        .child("当前格式："),
                                                )
                                                .child(recognition_tag(
                                                    detected_format,
                                                    !detected_format.is_empty(),
                                                    color_brand,
                                                )),
                                        )
                                }),
                        ),
                ),
        )
        .child(
            // 3. Bottom Sections
            div()
                .flex()
                .w_full()
                .h(px(238.0))
                .gap_6()
                .child(
                    // Traffic Analytics
                    div()
                        .flex_1()
                        .flex_col()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_4()
                        .gap_6()
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_end()
                                .child(
                                    div()
                                        .flex()
                                        .items_baseline()
                                        .gap_3()
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(color_text_primary)
                                                .child("流量趋势"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("(最近 30 天)"),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(color_text_primary)
                                        .bg(rgb(0xF1F5F9))
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .child("30 天 ▾"),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .w_full()
                                .bg(rgb(0xFAFBFE))
                                .rounded_xl()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(color_text_muted)
                                        .child("Enhanced Analytics Plot"),
                                ),
                        ),
                )
                .child(
                    // Subscription Priority
                    div()
                        .w(px(500.0))
                        .flex_shrink_0()
                        .flex_col()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_4()
                        .gap_6()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(color_text_primary)
                                .child("订阅优先级"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(priority_item("1", "远程订阅", "机场 A", true))
                                .child(div().text_xl().text_color(color_border).child("→"))
                                .child(priority_item("2", "本地覆写", "Narya Default", false))
                                .child(div().text_xl().text_color(color_border).child("→"))
                                .child(priority_item("3", "UI 临时", "活动中", false)),
                        ),
                ),
        )
}

fn metric_card(
    title: &'static str,
    val: String,
    badge_text: Option<&'static str>,
    sub: String,
    icon_name: IconName,
    color: Rgba,
) -> impl IntoElement {
    let mut icon_bg: Hsla = color.into();
    icon_bg.a = 0.1;

    div()
        .flex_1()
        .min_w(px(260.0))
        .bg(rgb(0xFFFFFF))
        .border_1()
        .border_color(rgb(0xE2E8F0))
        .rounded_2xl()
        .p_5()
        .flex()
        .items_center()
        .gap_4()
        .child(
            div()
                .size(px(52.0))
                .bg(icon_bg)
                .rounded_2xl()
                .flex()
                .items_center()
                .justify_center()
                .child(icon(icon_name, 28.0, color.into())),
        )
        .child(
            div()
                .flex_col()
                .gap_1()
                .overflow_hidden()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x64748B))
                        .child(title),
                )
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap_3()
                        .child(
                            div()
                                .text_2xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x0F172A))
                                .child(val),
                        )
                        .child(if let Some(t) = badge_text {
                            let mut bg: Hsla = rgb(0xDCFCE7).into();
                            bg.a = 1.0;
                            div()
                                .bg(bg)
                                .px_2()
                                .py_0p5()
                                .rounded_full()
                                .child(
                                    div()
                                        .text_color(rgb(0x10B981))
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .child(t),
                                )
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        }),
                )
                .child(div().text_xs().text_color(rgb(0x94A3B8)).child(sub)),
        )
}

pub fn subscription_card(
    sub: &NaryaSubscription,
    active: bool,
    on_click: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let border_color = if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) };
    let border_width = if active { px(2.0) } else { px(1.0) };
    let card_bg = if active { rgb(0xF8F9FF) } else { rgb(0xFFFFFF) };

    let mut icon_bg: Hsla = rgb(0x4F46E5).into();
    icon_bg.a = 0.1;

    div()
        .bg(card_bg)
        .border(border_width)
        .border_color(border_color)
        .rounded_xl()
        .p_2()
        .flex()
        .gap_2()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_click)
        .child(
            div()
                .size(px(46.0))
                .bg(icon_bg)
                .rounded_xl()
                .flex()
                .items_center()
                .justify_center()
                .child(icon(IconName::Dashboard, 24.0, rgb(0x4F46E5).into())),
        )
        .child(
            div()
                .flex()
                .flex_1()
                .flex_col()
                .gap_1()
                .overflow_hidden()
                .child(
                    div().flex().justify_between().items_baseline().child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap_2()
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x0F172A))
                                    .child(sub.name.clone()),
                            )
                            .child(if active {
                                let mut bg: Hsla = rgb(0xEEF2FF).into();
                                bg.a = 1.0;
                                div()
                                    .bg(bg)
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .flex_shrink_0()
                                    .child(
                                        div()
                                            .text_color(rgb(0x4F46E5))
                                            .text_size(px(10.0))
                                            .font_weight(FontWeight::BOLD)
                                            .child("当前使用"),
                                    )
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            }),
                    ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x94A3B8))
                        .overflow_hidden()
                        .child("https://***********/sub"),
                )
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .mt_1()
                        .child(
                            div()
                                .text_color(rgb(0x64748B))
                                .text_size(px(11.0))
                                .child(format!(
                                    "{} 节点   更新: {}",
                                    sub.node_count, sub.update_time
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_color(rgb(0x64748B))
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .child("34%"),
                                )
                                .child(
                                    div()
                                        .w(px(40.0))
                                        .h(px(4.0))
                                        .bg(rgb(0xE5E7EB))
                                        .rounded_full()
                                        .child(
                                            div()
                                                .w(relative(0.34))
                                                .h_full()
                                                .bg(rgb(0x10B981))
                                                .rounded_full(),
                                        ),
                                ),
                        ),
                ),
        )
}

fn tab_item(
    label: &'static str,
    active: bool,
    on_click: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .w(px(80.0))
        .pb_3()
        .border_b(if active { px(2.5) } else { px(0.0) })
        .border_color(rgb(0x4F46E5))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_click)
        .flex()
        .justify_center()
        .child(
            div()
                .text_sm()
                .font_weight(if active {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(if active { rgb(0x4F46E5) } else { rgb(0x64748B) })
                .child(label),
        )
}

fn recognition_tag(label: &str, active: bool, active_color: Rgba) -> impl IntoElement {
    let mut bg: Hsla = active_color.into();
    bg.a = 0.1; // Translucent background

    let (bg_final, text_color, border_color) = if active {
        (bg, active_color, active_color)
    } else {
        (rgb(0xF1F5F9).into(), rgb(0x94A3B8), rgb(0xE2E8F0))
    };

    div()
        .bg(bg_final)
        .border_1()
        .border_color(border_color)
        .px_1p5()
        .py_0p5()
        .rounded_md()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px(10.0))
                .font_weight(if active {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(text_color)
                .child(label.to_string()),
        )
}

fn form_row(label: String, val: String, has_icon: bool) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_baseline()
        .child(
            div()
                .w(px(100.0))
                .flex_shrink_0()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x64748B))
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .items_baseline()
                .gap_3()
                .overflow_hidden()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .overflow_hidden()
                        .child(val),
                )
                .child(if has_icon {
                    icon(IconName::ExternalLink, 14.0, rgb(0x94A3B8).into()).into_any_element()
                } else {
                    div().into_any_element()
                }),
        )
}

fn info_row(label: String, val: String, success: bool) -> impl IntoElement {
    div()
        .flex()
        .items_baseline()
        .gap_4()
        .child(
            div()
                .w(px(100.0))
                .flex_shrink_0()
                .text_xs()
                .text_color(rgb(0x64748B))
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .items_baseline()
                .gap_3()
                .overflow_hidden()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x0F172A))
                        .overflow_hidden()
                        .child(val),
                )
                .child(if success {
                    div()
                        .bg(rgb(0xDCFCE7))
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .flex_shrink_0()
                        .child(
                            div()
                                .text_color(rgb(0x10B981))
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .child("成功"),
                        )
                        .into_any_element()
                } else {
                    div().into_any_element()
                }),
        )
}

fn priority_item(
    index: &'static str,
    title: &'static str,
    sub: &'static str,
    active: bool,
) -> impl IntoElement {
    let bg = if active { rgb(0xEEF2FF) } else { rgb(0xFFFFFF) };
    let border = if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) };

    div()
        .flex_1()
        .bg(bg)
        .border_1()
        .border_color(border)
        .rounded_2xl()
        .p_4()
        .flex()
        .items_center()
        .gap_3()
        .overflow_hidden()
        .child(
            div()
                .size(px(26.0))
                .bg(border)
                .rounded_full()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_color(white())
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .child(index),
                ),
        )
        .child(
            div()
                .flex_1()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .overflow_hidden()
                        .child(title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x64748B))
                        .overflow_hidden()
                        .child(sub.to_string()),
                ),
        )
}
