use crate::components::{icon, IconName};
use crate::state::AppState;
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};

pub fn render_settings_view(
    model: &Entity<AppState>,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let state = model.read(cx);
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
        .gap_4()
        .child(
            // --- 1. Top Metrics Row (5 Cards) ---
            div()
                .flex()
                .w_full()
                .gap_4()
                .child(top_metric_card(
                    "应用版本",
                    "1.0.0",
                    "当前为最新版本",
                    IconName::Logs,
                    rgb(0x3B82F6),
                ))
                .child(top_metric_card_highlight(
                    "当前内核",
                    "sing-box 1.11.x",
                    "运行中",
                    IconName::Terminal,
                    color_brand,
                ))
                .child(top_metric_card(
                    "系统代理",
                    "7890 / 7891",
                    "HTTP / SOCKS",
                    IconName::Dashboard,
                    rgb(0x6366F1),
                ))
                .child(top_metric_card_highlight(
                    "IPv6 状态",
                    "自动 / 防泄漏",
                    "已启用",
                    IconName::Nodes,
                    rgb(0x10B981),
                ))
                .child(top_metric_card(
                    "更新通道",
                    "Stable",
                    "稳定版更新",
                    IconName::Config,
                    rgb(0x8B5CF6),
                )),
        )
        .child(
            // --- 2. Main Content (3 Columns) ---
            div()
                .flex()
                .flex_1()
                .gap_4()
                .overflow_hidden()
                .child(
                    // Left Column: Categories
                    div()
                        .flex()
                        .flex_col()
                        .w(px(220.0))
                        .flex_shrink_0()
                        .bg(color_card)
                        .border_1()
                        .border_color(color_border)
                        .rounded_2xl()
                        .p_4()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .px_3()
                                .py_2()
                                .mb_2()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(color_text_primary)
                                .child("设置分类"),
                        )
                        .child(category_item(IconName::Settings, "常规", true))
                        .child(category_item(IconName::About, "外观", false))
                        .child(category_item(IconName::Dashboard, "网络", false))
                        .child(category_item(IconName::Nodes, "IPv6", false))
                        .child(category_item(IconName::Terminal, "内核", false))
                        .child(category_item(IconName::Connections, "TUN", false))
                        .child(category_item(IconName::Logs, "DNS", false))
                        .child(category_item(IconName::Rules, "安全", false))
                        .child(category_item(IconName::Bell, "通知", false))
                        .child(category_item(IconName::Config, "更新", false))
                        .child(category_item(IconName::Settings, "高级", false)),
                )
                .child(
                    // Center Column: Main Settings Grid
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .gap_4()
                        .overflow_hidden()
                        .child(
                            // General Settings
                            div()
                                .flex()
                                .flex_col()
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
                                        .child("常规设置"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_8()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .gap_4()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .child("应用行为"),
                                                )
                                                .child(setting_toggle("开机自启", false))
                                                .child(setting_toggle("启动后最小化", false))
                                                .child(setting_toggle("关闭到托盘", true))
                                                .child(setting_toggle("启动时恢复代理", true)),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .gap_4()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .child("语言与区域"),
                                                )
                                                .child(setting_dropdown("语言", "简体中文"))
                                                .child(setting_dropdown("时区", "Asia/Shanghai"))
                                                .child(setting_dropdown("延迟单位", "ms")),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_4()
                                .child(
                                    // Default Config
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
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
                                                .child("默认配置"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .flex_1()
                                                        .gap_2()
                                                        .child(setting_dropdown_stacked(
                                                            "默认 Profile",
                                                            "Narya Default",
                                                        ))
                                                        .child(setting_dropdown_stacked(
                                                            "启动模式",
                                                            "规则模式",
                                                        )),
                                                )
                                                .child(
                                                    div().flex().flex_col().flex_1().gap_2().child(
                                                        setting_dropdown_stacked(
                                                            "默认策略组",
                                                            "🚀 Proxy / 自动选择",
                                                        ),
                                                    ),
                                                ),
                                        ),
                                )
                                .child(
                                    // Local Ports
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
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
                                                .child("本地端口"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_2()
                                                .child(setting_input_port("HTTP 端口", "7890"))
                                                .child(setting_input_port("SOCKS 端口", "7891"))
                                                .child(setting_input_port("API 端口", "9090")),
                                        ),
                                ),
                        )
                        .child(
                            // IPv6 Quick Settings
                            div()
                                .flex()
                                .flex_col()
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
                                        .child("IPv6 快捷设置"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(setting_toggle_inline("启用 IPv6", true))
                                        .child(setting_dropdown_inline("策略 (IPv6)", "自动"))
                                        .child(setting_toggle_inline("防 IPv6 泄漏", true)),
                                )
                                .child(
                                    div().flex().justify_end().child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .px_3()
                                            .py_1p5()
                                            .bg(rgb(0xF1F5F9))
                                            .border_1()
                                            .border_color(color_border)
                                            .rounded_lg()
                                            .child(icon(
                                                IconName::Dashboard,
                                                14.0,
                                                color_text_secondary.into(),
                                            ))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(color_text_primary)
                                                    .child("测试 IPv6 连接"),
                                            ),
                                    ),
                                ),
                        )
                        .child(
                            // Appearance Preview
                            div()
                                .flex()
                                .flex_col()
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
                                        .child("外观预览"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_1()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .child("主题模式"),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .gap_3()
                                                        .child(theme_mode_card(
                                                            IconName::Moon,
                                                            "浅色",
                                                            true,
                                                        ))
                                                        .child(theme_mode_card(
                                                            IconName::Moon,
                                                            "深色",
                                                            false,
                                                        ))
                                                        .child(theme_mode_card(
                                                            IconName::Settings,
                                                            "跟随系统",
                                                            false,
                                                        )),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .child("主色调"),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .gap_2()
                                                        .child(color_swatch(rgb(0x3B82F6), true)) // Blue
                                                        .child(color_swatch(rgb(0x8B5CF6), false)) // Purple
                                                        .child(color_swatch(rgb(0x10B981), false)) // Green
                                                        .child(color_swatch(rgb(0xF59E0B), false)) // Orange
                                                        .child(color_swatch(rgb(0xEF4444), false)) // Red
                                                        .child(
                                                            div()
                                                                .size(px(32.0))
                                                                .rounded_md()
                                                                .bg(rgb(0x6366F1)), // Placeholder for gradient
                                                        ),
                                                ),
                                        ),
                                ),
                        ),
                )
                .child(
                    // Right Column: System & Management
                    div()
                        .flex()
                        .flex_col()
                        .w(px(320.0))
                        .flex_shrink_0()
                        .gap_4()
                        .overflow_hidden()
                        .child(
                            // Kernel Management
                            div()
                                .flex()
                                .flex_col()
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
                                        .child("内核管理"),
                                )
                                .child(div().flex().flex_col().gap_2().children(
                                    state.kernels.iter().map(|k| {
                                        let is_running = k.running;
                                        let (status, color) = if k.installed {
                                            if is_running {
                                                ("运行中", color_success)
                                            } else {
                                                ("已安装", color_brand)
                                            }
                                        } else {
                                            ("未安装", color_text_muted)
                                        };

                                        let ver = k.version.as_deref().unwrap_or(if k.installed {
                                            "未知版本"
                                        } else {
                                            "未安装"
                                        });

                                        if k.installed {
                                            kernel_item(
                                                k.name.clone(),
                                                status,
                                                ver.to_string(),
                                                color,
                                                is_running,
                                            )
                                            .into_any_element()
                                        } else {
                                            let model = model.clone();
                                            let name_clone = k.name.clone();
                                            kernel_item_action(
                                                k.name.clone(),
                                                status,
                                                "安装",
                                                color,
                                                move |_, _, app| {
                                                    AppState::install_kernel(
                                                        model.clone(),
                                                        app,
                                                        name_clone.clone(),
                                                    );
                                                },
                                            )
                                            .into_any_element()
                                        }
                                    }),
                                ))
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(mini_btn_full("检查版本", IconName::Dashboard))
                                        .child(mini_btn_full("管理内核", IconName::Settings)),
                                ),
                        )
                        .child(
                            // Permission Status
                            div()
                                .flex()
                                .flex_col()
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
                                        .child("权限状态"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(permission_item("系统代理权限", true))
                                        .child(permission_item("TUN 权限", true))
                                        .child(permission_item("通知权限", true))
                                        .child(permission_item("IPv6 路由权限", true))
                                        .child(permission_item_warn("开机自启权限")),
                                ),
                        )
                        .child(
                            // Security & Privacy
                            div()
                                .flex()
                                .flex_col()
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
                                        .child("安全与隐私"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(setting_toggle("日志脱敏", true))
                                        .child(setting_toggle("本地 API Token", true))
                                        .child(setting_toggle("配置文件加密", false)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_center()
                                        .items_center()
                                        .gap_2()
                                        .mt_2()
                                        .child(icon(
                                            IconName::Rules,
                                            14.0,
                                            color_text_secondary.into(),
                                        ))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(color_text_secondary)
                                                .child("管理密钥"),
                                        ),
                                ),
                        )
                        .child(
                            // Update Settings
                            div()
                                .flex()
                                .flex_col()
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
                                        .child("更新设置"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(setting_toggle("自动检查更新", true))
                                        .child(
                                            div()
                                                .flex()
                                                .justify_between()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .child("更新通道"),
                                                )
                                                .child(setting_dropdown_inline_no_label("Stable")),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_secondary)
                                                        .child("上次检查"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color_text_muted)
                                                        .child("刚刚"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .mt_2()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .gap_2()
                                        .h(px(40.0))
                                        .w_full()
                                        .bg(color_brand)
                                        .text_color(white())
                                        .rounded_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .cursor_pointer()
                                        .child(icon(IconName::Dashboard, 16.0, white()))
                                        .child(div().text_sm().child("检查更新")),
                                ),
                        ),
                ),
        )
}

// --- Helper Functions ---

fn top_metric_card(
    title: &'static str,
    val: &'static str,
    sub: &'static str,
    icon_name: IconName,
    accent: Rgba,
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
        .p_4()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(40.0))
                .bg(bg)
                .rounded_xl()
                .child(icon(icon_name, 20.0, accent.into())),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_xs().text_color(rgb(0x64748B)).child(title))
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .child(val),
                )
                .child(div().text_xs().text_color(rgb(0x94A3B8)).child(sub)),
        )
}

fn top_metric_card_highlight(
    title: &'static str,
    val: &'static str,
    sub: &'static str,
    icon_name: IconName,
    accent: Rgba,
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
        .p_4()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(40.0))
                .bg(bg)
                .rounded_xl()
                .child(icon(icon_name, 20.0, accent.into())),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_xs().text_color(rgb(0x64748B)).child(title))
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0F172A))
                        .child(val),
                )
                .child(div().text_xs().text_color(rgb(0x10B981)).child(sub)), // Highlighted sub text
        )
}

fn category_item(icon_name: IconName, label: &'static str, active: bool) -> impl IntoElement {
    let bg: Hsla = if active {
        rgb(0xEEF2FF).into()
    } else {
        white()
    };
    div()
        .flex()
        .items_center()
        .gap_3()
        .px_4()
        .py_2p5()
        .rounded_xl()
        .bg(bg)
        .cursor_pointer()
        .child(icon(
            icon_name,
            18.0,
            if active {
                rgb(0x4F46E5).into()
            } else {
                rgb(0x64748B).into()
            },
        ))
        .child(
            div()
                .flex()
                .text_sm()
                .font_weight(if active {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(if active { rgb(0x4F46E5) } else { rgb(0x0F172A) })
                .child(label),
        )
}

fn setting_toggle(label: &'static str, active: bool) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_center()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(div().text_sm().text_color(rgb(0x0F172A)).child(label))
                .child(icon(IconName::About, 12.0, rgb(0x94A3B8).into())),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(if active { rgb(0x10B981) } else { rgb(0x94A3B8) })
                        .child(if active { "已启用" } else { "已关闭" }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .w(px(32.0))
                        .h(px(18.0))
                        .bg(if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) })
                        .rounded_full()
                        .px_0p5()
                        .child({
                            let mut dot = div().flex().size(px(14.0)).bg(white()).rounded_full();
                            if active {
                                dot = dot.ml_auto();
                            }
                            dot
                        }),
                ),
        )
}

fn setting_toggle_inline(label: &'static str, active: bool) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(div().text_sm().text_color(rgb(0x0F172A)).child(label))
                .child(icon(IconName::About, 12.0, rgb(0x94A3B8).into())),
        )
        .child(
            div()
                .flex()
                .items_center()
                .w(px(32.0))
                .h(px(18.0))
                .bg(if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) })
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
        .child(
            div()
                .text_xs()
                .text_color(if active { rgb(0x10B981) } else { rgb(0x94A3B8) })
                .child(if active { "已启用" } else { "已关闭" }),
        )
}

fn setting_dropdown(label: &'static str, val: &'static str) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_center()
        .child(div().text_sm().text_color(rgb(0x0F172A)).child(label))
        .child(setting_dropdown_inline_no_label(val))
}

fn setting_dropdown_stacked(label: &'static str, val: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(rgb(0x64748B)).child(label))
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .bg(rgb(0xF8FAFC))
                .border_1()
                .border_color(rgb(0xE2E8F0))
                .px_3()
                .py_1p5()
                .rounded_lg()
                .child(div().text_sm().text_color(rgb(0x0F172A)).child(val))
                .child(div().text_xs().text_color(rgb(0x94A3B8)).child("▾")),
        )
}

fn setting_dropdown_inline(label: &'static str, val: &'static str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().text_sm().text_color(rgb(0x0F172A)).child(label))
        .child(setting_dropdown_inline_no_label(val))
}

fn setting_dropdown_inline_no_label(val: &'static str) -> impl IntoElement {
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
        .child(div().text_xs().text_color(rgb(0x0F172A)).child(val))
        .child(div().text_xs().text_color(rgb(0x94A3B8)).child("▾"))
}

fn setting_input_port(label: &'static str, val: &'static str) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_center()
        .child(div().text_sm().text_color(rgb(0x0F172A)).child(label))
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .bg(rgb(0xF8FAFC))
                .border_1()
                .border_color(rgb(0xE2E8F0))
                .px_3()
                .py_1p5()
                .rounded_lg()
                .child(
                    div()
                        .w(px(40.0))
                        .text_sm()
                        .text_color(rgb(0x0F172A))
                        .child(val),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(8.0))
                                .text_color(rgb(0x94A3B8))
                                .child("▴"),
                        )
                        .child(
                            div()
                                .text_size(px(8.0))
                                .text_color(rgb(0x94A3B8))
                                .child("▾"),
                        ),
                ),
        )
}

fn theme_mode_card(icon_name: IconName, label: &'static str, active: bool) -> impl IntoElement {
    let bg: Hsla = if active {
        rgb(0xEEF2FF).into()
    } else {
        white()
    };
    div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_1p5()
        .bg(bg)
        .border_1()
        .border_color(if active { rgb(0x4F46E5) } else { rgb(0xE2E8F0) })
        .rounded_lg()
        .cursor_pointer()
        .child(icon(
            icon_name,
            16.0,
            if active {
                rgb(0x4F46E5).into()
            } else {
                rgb(0x64748B).into()
            },
        ))
        .child(
            div()
                .text_sm()
                .text_color(if active { rgb(0x4F46E5) } else { rgb(0x0F172A) })
                .child(label),
        )
}

fn color_swatch(color: Rgba, active: bool) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(32.0))
        .bg(color)
        .rounded_md()
        .cursor_pointer()
        .child(if active {
            icon(IconName::Dashboard, 14.0, white()).into_any_element()
        } else {
            div().into_any_element()
        })
}

fn kernel_item(
    name: String,
    status: &'static str,
    ver: String,
    status_color: Rgba,
    is_running: bool,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(28.0))
                        .bg(rgb(0xF1F5F9))
                        .rounded_lg()
                        .child(icon(IconName::Terminal, 16.0, rgb(0x0F172A).into())), // placeholder icon
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(if is_running {
                            FontWeight::BOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(rgb(0x0F172A))
                        .child(name),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_xs().text_color(status_color).child(status))
                .child(div().text_xs().text_color(rgb(0x64748B)).child(ver)),
        )
}

fn kernel_item_action(
    name: String,
    status: &'static str,
    action: &'static str,
    status_color: Rgba,
    on_click: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(28.0))
                        .bg(rgb(0xF1F5F9))
                        .rounded_lg()
                        .child(icon(IconName::Terminal, 16.0, rgb(0x0F172A).into())), // placeholder icon
                )
                .child(div().text_sm().text_color(rgb(0x0F172A)).child(name)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_xs().text_color(status_color).child(status))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x3B82F6))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, on_click)
                        .child(action),
                ),
        )
}

fn mini_btn_full(label: &'static str, icon_name: IconName) -> impl IntoElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .gap_2()
        .h(px(32.0))
        .bg(rgb(0xF8FAFC))
        .border_1()
        .border_color(rgb(0xE2E8F0))
        .rounded_lg()
        .cursor_pointer()
        .child(icon(icon_name, 14.0, rgb(0x64748B).into()))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x0F172A))
                .child(label),
        )
}

fn permission_item(label: &'static str, ok: bool) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_center()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(icon(IconName::Dashboard, 14.0, rgb(0x64748B).into())) // placeholder
                .child(div().text_xs().text_color(rgb(0x0F172A)).child(label))
                .child(icon(IconName::About, 12.0, rgb(0x94A3B8).into())),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .size(px(6.0))
                        .bg(if ok { rgb(0x10B981) } else { rgb(0xEF4444) })
                        .rounded_full(),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(if ok { rgb(0x10B981) } else { rgb(0xEF4444) })
                        .child(if ok { "正常" } else { "异常" }),
                ),
        )
}

fn permission_item_warn(label: &'static str) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_center()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(icon(IconName::Dashboard, 14.0, rgb(0x64748B).into())) // placeholder
                .child(div().text_xs().text_color(rgb(0x0F172A)).child(label))
                .child(icon(IconName::About, 12.0, rgb(0x94A3B8).into())),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(icon(IconName::About, 12.0, rgb(0xF59E0B).into())) // Warning icon approx
                .child(div().text_xs().text_color(rgb(0xF59E0B)).child("未启用")),
        )
}
