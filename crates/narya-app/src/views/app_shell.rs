use crate::state::AppState;
use crate::ui_kit::{
    color, narya_tag, progress, NaryaButton, NaryaCard, NaryaMetric, NaryaPage, NaryaStatus, BG,
    BORDER, BRAND, MUTED, SUCCESS, SURFACE, TEXT, WARNING,
};
use crate::views::ActiveView;
use gpui::{prelude::*, *};
use liora::components::{Button, Flex, Space, Tag, Text};

pub struct AppShell {
    pub(super) active_view: ActiveView,
    pub(super) state: Entity<AppState>,
}

impl AppShell {
    pub fn open(cx: &mut App) {
        let state = cx.new(|_| AppState::init_or_mock());
        AppState::start_traffic_monitor(state.clone(), cx);
        AppState::fetch_kernel_status(state.clone(), cx);

        let size = size(px(1536.0), px(1000.0));
        let bounds = Bounds::centered(None, size, cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size),
                titlebar: Some(TitlebarOptions {
                    title: Some("Narya".into()),
                    ..Default::default()
                }),
                app_id: Some("narya".into()),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|_| AppShell {
                    active_view: ActiveView::Dashboard,
                    state,
                })
            },
        )
        .expect("failed to open Narya main window");
    }
}

impl Render for AppShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = self.active_view;
        let state = self.state.read(cx);
        let active_node = state
            .active_node_id
            .as_ref()
            .and_then(|id| state.nodes.iter().find(|node| node.id == *id));
        let active_node_name = active_node
            .map(|node| node.name.clone())
            .unwrap_or_else(|| "未连接".to_string());
        let active_latency = active_node.and_then(|node| node.latency).unwrap_or(0);
        let download_speed = active_node.map(|node| node.download_speed).unwrap_or(0.0);
        let upload_speed = active_node.map(|node| node.upload_speed).unwrap_or(0.0);
        let nodes = state.nodes.clone();
        let subscriptions = state.subscriptions.clone();
        let logs = state.log_lines.clone();
        let kernels = state.kernels.clone();
        let running = state.kernel_running;
        let _ = state;

        Flex::new()
            .row()
            .size_full()
            .bg(color(SURFACE).into())
            .text_color(color(TEXT).into())
            .child(sidebar(
                view,
                cx,
                running,
                active_node_name,
                active_latency,
                download_speed,
                upload_speed,
            ))
            .child(
                Flex::new()
                    .column()
                    .flex_1()
                    .h_full()
                    .min_h_0()
                    .bg(color(BG).into())
                    .child(header(view, &self.state))
                    .child(
                        Flex::new()
                            .flex_1()
                            .min_h_0()
                            .padding_lg()
                            .overflow_hidden()
                            .child(route_page(
                                view,
                                &self.state,
                                nodes,
                                subscriptions,
                                logs,
                                kernels,
                                running,
                            )),
                    )
                    .child(footer(running)),
            )
    }
}

fn sidebar(
    active: ActiveView,
    cx: &mut Context<AppShell>,
    running: bool,
    active_node_name: String,
    latency: u32,
    down: f32,
    up: f32,
) -> impl IntoElement {
    Flex::new()
        .column()
        .width_px(220.0)
        .h_full()
        .flex_none()
        .justify_between()
        .border()
        .border_color(color(BORDER).into())
        .bg(color(SURFACE).into())
        .child(
            Flex::new()
                .column()
                .child(
                    Flex::new()
                        .row()
                        .align_center()
                        .gap_sm()
                        .height_px(60.0)
                        .padding_x_px(20.0)
                        .child(
                            Text::new("◈")
                                .size(px(28.0))
                                .text_color(color(BRAND).into())
                                .selectable(false),
                        )
                        .child(
                            Flex::new()
                                .column()
                                .child(
                                    Text::new("Narya")
                                        .bold()
                                        .text_color(color(TEXT).into())
                                        .selectable(false),
                                )
                                .child(
                                    Text::new("Liora Native")
                                        .xs()
                                        .text_color(color(MUTED).into())
                                        .selectable(false),
                                ),
                        ),
                )
                .child(
                    Flex::new()
                        .column()
                        .gap_sm()
                        .padding_sm()
                        .child(nav_button("仪表盘", ActiveView::Dashboard, active, cx))
                        .child(nav_button("节点", ActiveView::Nodes, active, cx))
                        .child(nav_button("配置", ActiveView::Config, active, cx))
                        .child(nav_button("订阅", ActiveView::Subscriptions, active, cx))
                        .child(nav_button("连接", ActiveView::Connections, active, cx))
                        .child(nav_button("规则", ActiveView::Rules, active, cx))
                        .child(nav_button("日志", ActiveView::Logs, active, cx))
                        .child(nav_button("工具箱", ActiveView::Tools, active, cx))
                        .child(nav_button("设置", ActiveView::Settings, active, cx)),
                ),
        )
        .child(
            Flex::new()
                .column()
                .gap_md()
                .padding_md()
                .child(
                    NaryaCard::panel(
                        Flex::new()
                            .column()
                            .gap_sm()
                            .child(
                                Space::new().gap_sm().child(status_dot(running)).child(
                                    Text::new(if running { "已连接" } else { "待连接" })
                                        .sm()
                                        .bold()
                                        .selectable(false),
                                ),
                            )
                            .child(
                                Text::new(active_node_name)
                                    .sm()
                                    .text_color(color(TEXT).into())
                                    .selectable(false),
                            )
                            .child(narya_tag(
                                format!("{} ms", latency),
                                if running {
                                    NaryaStatus::Success
                                } else {
                                    NaryaStatus::Info
                                },
                            ))
                            .child(
                                Space::new()
                                    .gap_sm()
                                    .child(
                                        Text::new(format!("↓ {:.1} MB/s", down))
                                            .xs()
                                            .text_color(color(MUTED).into())
                                            .selectable(false),
                                    )
                                    .child(
                                        Text::new(format!("↑ {:.1} MB/s", up))
                                            .xs()
                                            .text_color(color(MUTED).into())
                                            .selectable(false),
                                    ),
                            )
                            .child(progress(if running { 72.0 } else { 8.0 })),
                    )
                    .no_shadow(),
                )
                .child(
                    Space::new()
                        .gap_sm()
                        .child(NaryaButton::ghost("GitHub").small())
                        .child(NaryaButton::ghost("Theme").small())
                        .child(NaryaButton::ghost("Bell").small()),
                ),
        )
}

fn nav_button(
    label: &'static str,
    target: ActiveView,
    active: ActiveView,
    cx: &mut Context<AppShell>,
) -> Button {
    let weak = cx.entity().downgrade();
    let button = if active == target {
        Button::new(label).primary()
    } else {
        Button::new(label).tertiary()
    };

    button
        .rounded_md()
        .background(active == target)
        .border(false)
        .on_click(move |_, _, cx| {
            let _ = weak.update(cx, |this, cx| {
                this.active_view = target;
                cx.notify();
            });
        })
}

fn status_dot(running: bool) -> impl IntoElement {
    Flex::new()
        .width_px(8.0)
        .height_px(8.0)
        .rounded_pill()
        .bg(color(if running { SUCCESS } else { MUTED }).into())
}

fn header(view: ActiveView, model: &Entity<AppState>) -> impl IntoElement {
    let model_for_connect = model.clone();
    Flex::new()
        .row()
        .align_center()
        .justify_between()
        .height_px(64.0)
        .w_full()
        .flex_none()
        .padding_x_px(20.0)
        .bg(color(SURFACE).into())
        .border()
        .border_color(color(BORDER).into())
        .child(
            Flex::new()
                .column()
                .gap_px(4.0)
                .child(
                    Text::new(view_title(view))
                        .size(px(18.0))
                        .bold()
                        .text_color(color(TEXT).into())
                        .selectable(false),
                )
                .child(
                    Text::new(view_subtitle(view))
                        .xs()
                        .text_color(color(MUTED).into())
                        .selectable(false),
                ),
        )
        .child(
            Space::new()
                .gap_sm()
                .child(
                    NaryaButton::primary("连接")
                        .small()
                        .on_click(move |_, _, cx| {
                            AppState::toggle_proxy(model_for_connect.clone(), cx)
                        }),
                )
                .child(NaryaButton::ghost("刷新全部").small())
                .child(NaryaButton::ghost("导入").small())
                .child(NaryaButton::ghost("导出").small()),
        )
}

fn footer(running: bool) -> impl IntoElement {
    Flex::new()
        .row()
        .align_center()
        .justify_between()
        .height_px(36.0)
        .w_full()
        .flex_none()
        .padding_x_px(16.0)
        .bg(color(SURFACE).into())
        .border()
        .border_color(color(BORDER).into())
        .child(
            Space::new()
                .gap_lg()
                .child(
                    Text::new("内核: sing-box")
                        .xs()
                        .text_color(color(MUTED).into())
                        .selectable(false),
                )
                .child(
                    Text::new(if running {
                        "状态: Active"
                    } else {
                        "状态: Standby"
                    })
                    .xs()
                    .text_color(color(MUTED).into())
                    .selectable(false),
                )
                .child(
                    Text::new("配置: Narya Default")
                        .xs()
                        .text_color(color(MUTED).into())
                        .selectable(false),
                ),
        )
        .child(
            Text::new("v0.1.0 · Liora UI")
                .xs()
                .text_color(color(MUTED).into())
                .selectable(false),
        )
}

fn route_page(
    view: ActiveView,
    model: &Entity<AppState>,
    nodes: Vec<narya_core::Node>,
    subscriptions: Vec<narya_core::Subscription>,
    logs: Vec<crate::state::LogMessage>,
    kernels: Vec<narya_ipc::KernelInfo>,
    running: bool,
) -> AnyElement {
    match view {
        ActiveView::Dashboard => {
            dashboard_page(model, &nodes, &subscriptions, running).into_any_element()
        }
        ActiveView::Nodes => nodes_page(model, nodes).into_any_element(),
        ActiveView::Subscriptions => subscriptions_page(model, subscriptions).into_any_element(),
        ActiveView::Config => config_page().into_any_element(),
        ActiveView::Connections => connections_page(&nodes).into_any_element(),
        ActiveView::Rules => rules_page().into_any_element(),
        ActiveView::Logs => logs_page(logs).into_any_element(),
        ActiveView::Tools => tools_page().into_any_element(),
        ActiveView::Settings => settings_page(model, kernels).into_any_element(),
        ActiveView::About => simple_page("关于", "Narya GPUI 客户端").into_any_element(),
    }
}

fn dashboard_page(
    model: &Entity<AppState>,
    nodes: &[narya_core::Node],
    subscriptions: &[narya_core::Subscription],
    running: bool,
) -> impl IntoElement {
    let model_for_connect = model.clone();
    NaryaPage::new("Dashboard", "实时代理状态、流量趋势与关键动作入口。")
        .child(
            Flex::new()
                .row()
                .gap_lg()
                .w_full()
                .child(Flex::new().flex_1().child(NaryaMetric::card("连接状态", if running { "已连接" } else { "未连接" }, "System Proxy · Rule", color(BRAND))))
                .child(Flex::new().flex_1().child(NaryaMetric::card("可用节点", nodes.len().to_string(), "按延迟智能选择", color(SUCCESS))))
                .child(Flex::new().flex_1().child(NaryaMetric::card("订阅源", subscriptions.len().to_string(), "自动更新已就绪", color(WARNING))))
                .child(Flex::new().flex_1().child(NaryaMetric::card("今日流量", "1.28 GB", "下载 1.02 / 上传 0.26", color(BRAND)))),
        )
        .child(
            Flex::new()
                .row()
                .gap_lg()
                .flex_1()
                .min_h_0()
                .child(
                    Flex::new()
                        .flex_1()
                        .child(NaryaCard::titled(
                            "快速控制",
                            Flex::new()
                                .column()
                                .gap_lg()
                                .child(Space::new().gap_md().child(NaryaButton::primary(if running { "断开连接" } else { "立即连接" }).on_click(move |_, _, cx| AppState::toggle_proxy(model_for_connect.clone(), cx))).child(NaryaButton::ghost("测速全部节点")))
                                .child(Text::new("系统代理、TUN 智能路由、DNS 保护和规则模式都在主控面板集中呈现。").sm().text_color(color(MUTED).into()).selectable(false))
                                .child(progress(if running { 88.0 } else { 12.0 })),
                        )),
                )
                .child(
                    Flex::new()
                        .width_px(360.0)
                        .flex_none()
                        .child(NaryaCard::titled(
                            "推荐节点",
                            Flex::new().column().gap_md().children(nodes.iter().take(5).map(node_row)),
                        )),
                ),
        )
}

fn nodes_page(model: &Entity<AppState>, nodes: Vec<narya_core::Node>) -> impl IntoElement {
    let model_for_test = model.clone();
    NaryaPage::new("Nodes", "节点列表、延迟测速、协议与出口状态。")
        .child(
            Space::new()
                .gap_md()
                .child(NaryaButton::primary("一键测速").on_click(move |_, _, cx| {
                    AppState::test_all_latency(model_for_test.clone(), cx)
                }))
                .child(NaryaButton::ghost("智能排序"))
                .child(NaryaButton::ghost("筛选地区")),
        )
        .child(
            Flex::new()
                .column()
                .gap_md()
                .overflow_y_scroll()
                .children(nodes.into_iter().map(|node| node_card(model, node))),
        )
}

fn subscriptions_page(
    model: &Entity<AppState>,
    subscriptions: Vec<narya_core::Subscription>,
) -> impl IntoElement {
    NaryaPage::new("Subscriptions", "订阅导入、更新、流量与节点同步。")
        .child(
            Space::new()
                .gap_md()
                .child(NaryaButton::primary("添加订阅").disabled(true))
                .child(NaryaButton::ghost("从剪贴板导入").disabled(true)),
        )
        .child(
            Flex::new().row().wrap().gap_lg().children(
                subscriptions
                    .into_iter()
                    .map(|sub| subscription_card(model, sub)),
            ),
        )
}

fn config_page() -> impl IntoElement {
    simple_page("Config", "可视化链式代理、YAML 编辑器与配置预览。").child(NaryaCard::titled(
        "配置草案",
        Flex::new()
            .column()
            .gap_md()
            .child(
                Text::new(
                    "当前阶段保留核心配置生成入口，后续可把 sing-box JSON 合成器接入此面板。",
                )
                .sm()
                .text_color(color(MUTED).into())
                .selectable(false),
            )
            .child(NaryaButton::ghost("打开 YAML 编辑器").disabled(true)),
    ))
}

fn connections_page(nodes: &[narya_core::Node]) -> impl IntoElement {
    NaryaPage::new("Connections", "活跃连接、目标地址、规则命中和出口链路。").child(
        NaryaCard::titled(
            "近期连接",
            Flex::new()
                .column()
                .gap_sm()
                .children(nodes.iter().take(6).enumerate().map(|(idx, node)| {
                    Flex::new()
                        .row()
                        .align_center()
                        .justify_between()
                        .padding_sm()
                        .border()
                        .border_color(color(BORDER).into())
                        .rounded_md()
                        .child(
                            Text::new(format!("10.0.0.{} → {}", idx + 10, node.details.address))
                                .sm()
                                .selectable(false),
                        )
                        .child(narya_tag(node.protocol.clone(), NaryaStatus::Info))
                })),
        ),
    )
}

fn rules_page() -> impl IntoElement {
    simple_page("Rules", "规则分流、规则模拟器与命中统计。").child(
        Flex::new()
            .row()
            .gap_lg()
            .child(Flex::new().flex_1().child(NaryaMetric::card(
                "规则集",
                "8",
                "GeoSite / GeoIP",
                color(BRAND),
            )))
            .child(Flex::new().flex_1().child(NaryaMetric::card(
                "今日命中",
                "12,840",
                "DIRECT 62% · PROXY 38%",
                color(SUCCESS),
            )))
            .child(Flex::new().flex_1().child(NaryaMetric::card(
                "最后更新",
                "2 天前",
                "可手动刷新",
                color(WARNING),
            ))),
    )
}

fn logs_page(logs: Vec<crate::state::LogMessage>) -> impl IntoElement {
    let log_rows: Vec<AnyElement> = if logs.is_empty() {
        vec![
            Text::new("Daemon 尚未推送日志，启动 narya-daemon 后这里会实时刷新。")
                .sm()
                .text_color(color(MUTED).into())
                .into_any_element(),
        ]
    } else {
        logs.into_iter()
            .rev()
            .take(18)
            .map(|log| {
                Flex::new()
                    .row()
                    .gap_md()
                    .padding_sm()
                    .border()
                    .border_color(color(BORDER).into())
                    .rounded_md()
                    .child(
                        Text::new(log.time)
                            .xs()
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    )
                    .child(narya_tag(log.level, NaryaStatus::Info))
                    .child(Text::new(log.content).sm().text_color(color(TEXT).into()))
                    .into_any_element()
            })
            .collect()
    };

    NaryaPage::new("Logs", "内核日志、诊断导出与错误追踪。").child(NaryaCard::titled(
        "实时日志",
        Flex::new().column().gap_sm().children(log_rows),
    ))
}

fn tools_page() -> impl IntoElement {
    NaryaPage::new("Tools", "Ping、DNS 查询、MTR、端口检查与报告导出。").child(
        Flex::new()
            .row()
            .gap_lg()
            .child(tool_card("Ping 测试", "检测主机可达性与往返延迟"))
            .child(tool_card("DNS 查询", "查看解析链路与污染风险"))
            .child(tool_card("Speed Test", "按出口节点测试吞吐")),
    )
}

fn settings_page(
    model: &Entity<AppState>,
    kernels: Vec<narya_ipc::KernelInfo>,
) -> impl IntoElement {
    NaryaPage::new("Settings", "内核、网络、DNS、外观和安全策略。")
        .child(NaryaCard::titled(
            "代理内核",
            Flex::new()
                .column()
                .gap_md()
                .children(kernels.into_iter().map(|kernel| kernel_row(model, kernel))),
        ))
        .child(
            Flex::new()
                .row()
                .gap_lg()
                .child(Flex::new().flex_1().child(NaryaMetric::card(
                    "DNS",
                    "增强模式",
                    "DoH · FakeIP",
                    color(BRAND),
                )))
                .child(Flex::new().flex_1().child(NaryaMetric::card(
                    "安全",
                    "严格",
                    "证书/权限提示启用",
                    color(SUCCESS),
                )))
                .child(Flex::new().flex_1().child(NaryaMetric::card(
                    "更新",
                    "自动",
                    "每日检查",
                    color(WARNING),
                ))),
        )
}

fn simple_page(title: &'static str, subtitle: &'static str) -> NaryaPage {
    NaryaPage::new(title, subtitle)
}

fn node_card(model: &Entity<AppState>, node: narya_core::Node) -> impl IntoElement {
    let node_id = node.id.clone();
    let model_for_connect = model.clone();
    NaryaCard::panel(
        Flex::new()
            .row()
            .align_center()
            .justify_between()
            .gap_lg()
            .child(
                Flex::new()
                    .column()
                    .gap_sm()
                    .child(
                        Text::new(node.name.clone())
                            .bold()
                            .text_color(color(TEXT).into())
                            .selectable(false),
                    )
                    .child(
                        Space::new()
                            .gap_sm()
                            .child(narya_tag(node.protocol.clone(), NaryaStatus::Info))
                            .child(
                                Text::new(node.details.address.clone())
                                    .xs()
                                    .text_color(color(MUTED).into())
                                    .selectable(false),
                            ),
                    ),
            )
            .child(
                Space::new()
                    .gap_md()
                    .child(latency_tag(node.latency))
                    .child(
                        NaryaButton::primary("连接")
                            .small()
                            .on_click(move |_, _, cx| {
                                AppState::connect_node(
                                    model_for_connect.clone(),
                                    cx,
                                    node_id.clone(),
                                )
                            }),
                    ),
            ),
    )
    .no_shadow()
}

fn node_row(node: &narya_core::Node) -> impl IntoElement {
    Flex::new()
        .row()
        .align_center()
        .justify_between()
        .padding_sm()
        .border()
        .border_color(color(BORDER).into())
        .rounded_md()
        .child(
            Text::new(node.name.clone())
                .sm()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
        .child(latency_tag(node.latency))
}

fn subscription_card(model: &Entity<AppState>, sub: narya_core::Subscription) -> impl IntoElement {
    let id = sub.id.clone();
    let model = model.clone();
    let usage = if sub.traffic_total > 0.0 {
        ((sub.traffic_used / sub.traffic_total) * 100.0).clamp(0.0, 100.0) as f32
    } else {
        0.0
    };

    NaryaCard::panel(
        Flex::new()
            .column()
            .gap_md()
            .width_px(330.0)
            .child(
                Flex::new()
                    .row()
                    .align_center()
                    .justify_between()
                    .child(
                        Text::new(sub.name)
                            .bold()
                            .text_color(color(TEXT).into())
                            .selectable(false),
                    )
                    .child(narya_tag(sub.status, NaryaStatus::Success)),
            )
            .child(Text::new(sub.url).xs().text_color(color(MUTED).into()))
            .child(progress(usage))
            .child(
                Text::new(format!(
                    "{:.1} / {:.1} GB · {} 节点",
                    sub.traffic_used, sub.traffic_total, sub.node_count
                ))
                .xs()
                .text_color(color(MUTED).into())
                .selectable(false),
            )
            .child(
                NaryaButton::ghost("手动刷新")
                    .small()
                    .on_click(move |_, _, cx| {
                        AppState::refresh_subscription(model.clone(), cx, id.clone())
                    }),
            ),
    )
    .no_shadow()
}

fn tool_card(title: &'static str, body: &'static str) -> impl IntoElement {
    Flex::new().flex_1().child(NaryaCard::titled(
        title,
        Flex::new()
            .column()
            .gap_md()
            .child(
                Text::new(body)
                    .sm()
                    .text_color(color(MUTED).into())
                    .selectable(false),
            )
            .child(NaryaButton::primary("开始").small().disabled(true)),
    ))
}

fn kernel_row(model: &Entity<AppState>, kernel: narya_ipc::KernelInfo) -> impl IntoElement {
    let status = if kernel.running {
        NaryaStatus::Success
    } else if kernel.installed {
        NaryaStatus::Info
    } else {
        NaryaStatus::Warning
    };
    let status_text = if kernel.running {
        "运行中"
    } else if kernel.installed {
        "已安装"
    } else {
        "未安装"
    };
    let name = kernel.name.clone();
    let model = model.clone();

    Flex::new()
        .row()
        .align_center()
        .justify_between()
        .padding_sm()
        .border()
        .border_color(color(BORDER).into())
        .rounded_md()
        .child(
            Flex::new()
                .column()
                .gap_sm()
                .child(
                    Text::new(kernel.name)
                        .bold()
                        .text_color(color(TEXT).into())
                        .selectable(false),
                )
                .child(
                    Text::new(kernel.version.unwrap_or_else(|| "未安装".to_string()))
                        .xs()
                        .text_color(color(MUTED).into())
                        .selectable(false),
                ),
        )
        .child(
            Space::new()
                .gap_md()
                .child(narya_tag(status_text, status))
                .child(if kernel.installed {
                    NaryaButton::ghost("检查").small().disabled(true)
                } else {
                    NaryaButton::primary("安装未实现")
                        .small()
                        .disabled(true)
                        .on_click(move |_, _, cx| {
                            AppState::install_kernel(model.clone(), cx, name.clone())
                        })
                }),
        )
}

fn latency_tag(latency: Option<u32>) -> Tag {
    match latency {
        Some(ms) if ms < 80 => narya_tag(format!("{} ms", ms), NaryaStatus::Success),
        Some(ms) if ms < 160 => narya_tag(format!("{} ms", ms), NaryaStatus::Warning),
        Some(ms) => narya_tag(format!("{} ms", ms), NaryaStatus::Danger),
        None => narya_tag("测试中", NaryaStatus::Info),
    }
}

fn view_title(view: ActiveView) -> &'static str {
    match view {
        ActiveView::Dashboard => "仪表盘",
        ActiveView::Nodes => "节点列表",
        ActiveView::Config => "配置编辑",
        ActiveView::Subscriptions => "订阅管理",
        ActiveView::Connections => "连接追踪",
        ActiveView::Rules => "规则管理",
        ActiveView::Logs => "实时日志",
        ActiveView::Tools => "工具箱",
        ActiveView::Settings => "系统设置",
        ActiveView::About => "关于 Narya",
    }
}

fn view_subtitle(view: ActiveView) -> &'static str {
    match view {
        ActiveView::Dashboard => "主窗口直接进入，无启动页。",
        ActiveView::Nodes => "按延迟、协议和地区管理出口节点。",
        ActiveView::Config => "编辑与合成 sing-box / mihomo / xray 配置。",
        ActiveView::Subscriptions => "导入、更新并解析远程订阅。",
        ActiveView::Connections => "查看连接、目标和规则命中。",
        ActiveView::Rules => "维护分流规则和模拟匹配结果。",
        ActiveView::Logs => "跟踪 daemon 与内核输出。",
        ActiveView::Tools => "网络诊断与导出报告。",
        ActiveView::Settings => "调整内核、网络、DNS、安全与外观。",
        ActiveView::About => "Narya native GPUI client.",
    }
}
