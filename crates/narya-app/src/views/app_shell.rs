use crate::state::AppState;
use crate::ui_kit as narya_ui;
use crate::ui_kit::{
    NaryaButton, NaryaCard, NaryaMetric, NaryaPage, NaryaStatus, NavTarget, PageKind,
};
use crate::views::ActiveView;
use liora::components::{Flex, Input, Select, Text};
use liora_icons_lucide::IconName;
use narya_ui::{
    px, App, Context, NaryaAppContext, NaryaEntity as Entity, NaryaFluentBuilder, NaryaIntoElement,
    NaryaRenderOnce, NaryaViewElement, Render, Window,
};

pub struct AppShell {
    pub(super) active_view: ActiveView,
    pub(super) state: Entity<AppState>,
}

impl AppShell {
    pub fn open(cx: &mut App) {
        let state = cx.new(|_| AppState::init_or_mock());
        AppState::start_traffic_monitor(state.clone(), cx);
        AppState::fetch_kernel_status(state.clone(), cx);
        AppState::fetch_routing_status(state.clone(), cx);

        narya_ui::open_shell_window(cx, move |_, cx| {
            cx.new(|_| AppShell {
                active_view: ActiveView::Dashboard,
                state,
            })
        });
    }
}

impl Render for AppShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl NaryaIntoElement {
        let view = self.active_view;
        let snapshot = ShellSnapshot::from_state(self.state.clone(), cx);
        let weak_shell = cx.entity().downgrade();
        let on_nav = move |target: NavTarget, cx: &mut App| {
            let _ = weak_shell.update(cx, |shell, cx| {
                shell.active_view = ActiveView::from(target);
                cx.notify();
            });
        };

        narya_ui::ShellFrame::new(
            narya_ui::Sidebar::new(
                NavTarget::from(view),
                snapshot.running,
                snapshot.active_node_name.clone(),
                snapshot.active_latency,
                snapshot.download_speed,
                snapshot.upload_speed,
                on_nav,
            ),
            header(view, &self.state),
            route_page(view, &self.state, snapshot),
            narya_ui::FooterBar,
        )
    }
}

#[derive(Clone)]
struct ShellSnapshot {
    nodes: Vec<narya_core::Node>,
    subscriptions: Vec<narya_core::Subscription>,
    logs: Vec<crate::state::LogMessage>,
    kernels: Vec<narya_ipc::KernelInfo>,
    rules: Vec<narya_rules::Rule>,
    groups: Vec<narya_rules::RoutingGroup>,
    rule_sets: Vec<narya_rules::RuleSetSource>,
    rule_filter_text: String,
    rule_action_filter: String,
    routing_mode: narya_platform::ProxyMode,
    routing_active: narya_platform::ProxyMode,
    kernel_healthy: bool,
    active_kernel: String,
    kernel_artifact_kernel: String,
    kernel_artifact_version: String,
    kernel_artifact_source: String,
    kernel_artifact_sha256: String,
    kernel_artifact_signature: String,
    kernel_artifact_public_key: String,
    kernel_operation: Option<String>,
    kernel_error: Option<String>,
    rule_set_draft_id: String,
    rule_set_draft_source: String,
    rule_set_draft_version: String,
    rule_set_draft_sha256: String,
    rule_set_draft_signature: String,
    rule_set_draft_public_key: String,
    rule_set_error: Option<String>,
    group_error: Option<String>,
    rule_editor_error: Option<String>,
    rule_io_path: String,
    rule_io_status: Option<String>,
    running: bool,
    active_node_name: String,
    active_latency: u32,
    download_speed: f32,
    upload_speed: f32,
}

impl ShellSnapshot {
    fn from_state(model: Entity<AppState>, cx: &mut Context<AppShell>) -> Self {
        let state = model.read(cx);
        let active_node = state
            .active_node_id
            .as_ref()
            .and_then(|id| state.nodes.iter().find(|node| node.id == *id));
        Self {
            nodes: state.nodes.clone(),
            subscriptions: state.subscriptions.clone(),
            logs: state.log_lines.clone(),
            kernels: state.kernels.clone(),
            rules: state.rules.clone(),
            groups: state.groups.clone(),
            rule_sets: state.rule_sets.clone(),
            rule_filter_text: state.rule_filter_text.clone(),
            rule_action_filter: state.rule_action_filter.clone(),
            routing_mode: state.routing_mode,
            routing_active: state.routing_active,
            kernel_healthy: state.kernel_healthy,
            active_kernel: state.active_kernel.clone(),
            kernel_artifact_kernel: state.kernel_artifact_kernel.clone(),
            kernel_artifact_version: state.kernel_artifact_version.clone(),
            kernel_artifact_source: state.kernel_artifact_source.clone(),
            kernel_artifact_sha256: state.kernel_artifact_sha256.clone(),
            kernel_artifact_signature: state.kernel_artifact_signature.clone(),
            kernel_artifact_public_key: state.kernel_artifact_public_key.clone(),
            kernel_operation: state.kernel_operation.clone(),
            kernel_error: state.kernel_error.clone(),
            rule_set_draft_id: state.rule_set_draft_id.clone(),
            rule_set_draft_source: state.rule_set_draft_source.clone(),
            rule_set_draft_version: state.rule_set_draft_version.clone(),
            rule_set_draft_sha256: state.rule_set_draft_sha256.clone(),
            rule_set_draft_signature: state.rule_set_draft_signature.clone(),
            rule_set_draft_public_key: state.rule_set_draft_public_key.clone(),
            rule_set_error: state.rule_set_error.clone(),
            group_error: state.group_error.clone(),
            rule_editor_error: state.rule_editor_error.clone(),
            rule_io_path: state.rule_io_path.clone(),
            rule_io_status: state.rule_io_status.clone(),
            running: state.kernel_running,
            active_node_name: active_node
                .map(|node| node.name.clone())
                .unwrap_or_else(|| "未连接".to_string()),
            active_latency: active_node.and_then(|node| node.latency).unwrap_or(0),
            download_speed: active_node.map(|node| node.download_speed).unwrap_or(0.0),
            upload_speed: active_node.map(|node| node.upload_speed).unwrap_or(0.0),
        }
    }
}

fn header(view: ActiveView, model: &Entity<AppState>) -> impl NaryaIntoElement {
    let page = PageKind::from(view);
    let model_for_connect = model.clone();
    let mut actions = vec![
        NaryaButton::icon_name(IconName::Fullscreen).into_any_element(),
        NaryaButton::icon_name(IconName::ClipboardList).into_any_element(),
        NaryaButton::icon_name(IconName::Settings).into_any_element(),
        NaryaButton::icon_name(IconName::EllipsisVertical).into_any_element(),
    ];
    match view {
        ActiveView::Nodes => actions.insert(
            0,
            NaryaButton::primary("一键测速")
                .on_click(move |_, _, cx| AppState::test_all_latency(model_for_connect.clone(), cx))
                .into_any_element(),
        ),
        ActiveView::Subscriptions => actions.insert(
            0,
            NaryaButton::primary("＋ 添加订阅")
                .disabled(true)
                .into_any_element(),
        ),
        _ => {}
    }
    narya_ui::HeaderBar::new(page, actions)
}

fn route_page(
    view: ActiveView,
    model: &Entity<AppState>,
    snapshot: ShellSnapshot,
) -> impl NaryaIntoElement {
    match view {
        ActiveView::Dashboard => dashboard_page(model, snapshot).into_any_element(),
        ActiveView::Nodes => nodes_page(model, snapshot).into_any_element(),
        ActiveView::Subscriptions => subscriptions_page(model, snapshot).into_any_element(),
        ActiveView::Settings => settings_page(model, snapshot).into_any_element(),
        ActiveView::Config => config_page().into_any_element(),
        ActiveView::Connections => connections_page(snapshot).into_any_element(),
        ActiveView::Rules => rules_page(model, snapshot).into_any_element(),
        ActiveView::Logs => logs_page(snapshot).into_any_element(),
        ActiveView::Tools => tools_page().into_any_element(),
        ActiveView::About => about_page().into_any_element(),
    }
}

fn dashboard_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let model_for_toggle = model.clone();
    NaryaPage::new()
        .row(narya_ui::dashboard_top(
            narya_ui::hero_toggle_card(
                IconName::Monitor,
                "系统代理",
                "管理系统网络代理设置",
                snapshot.routing_active == narya_platform::ProxyMode::SystemProxy,
                "规则模式 ›",
                NaryaStatus::Info,
            ),
            narya_ui::hero_toggle_card(
                IconName::Network,
                "TUN 虚拟网卡",
                "拦截并代理所有网络流量（推荐）",
                snapshot.routing_active == narya_platform::ProxyMode::Tun,
                "智能路由 ›",
                NaryaStatus::Success,
            ),
        ))
        .row(narya_ui::dashboard_middle(
            narya_ui::dashboard_quick_panel(
                snapshot
                    .nodes
                    .iter()
                    .take(4)
                    .map(|node| {
                        narya_ui::quick_node(
                            node.name.clone(),
                            node.protocol.clone(),
                            node.latency.unwrap_or(0),
                            latency_status(node.latency.unwrap_or(0)),
                        )
                        .into_any_element()
                    })
                    .collect(),
            ),
            narya_ui::dashboard_network_panel(
                narya_ui::soft_trend(latency_values(), 212.0, narya_ui::SUCCESS),
                vec![
                    narya_ui::compact_metric(
                        "节点延迟",
                        format!("{} ms", snapshot.active_latency),
                        "当前节点",
                    )
                    .into_any_element(),
                    narya_ui::compact_metric(
                        "可用节点",
                        format!("{} / 128", snapshot.nodes.len() * 9 + 2),
                        "在线 / 总数",
                    )
                    .into_any_element(),
                    narya_ui::compact_metric("负载", "23%", "当前节点负载").into_any_element(),
                    narya_ui::compact_metric("丢包率", "0.2%", "当前节点").into_any_element(),
                ],
            ),
        ))
        .row(narya_ui::dashboard_bottom(
            narya_ui::dashboard_traffic_panel(
                vec![
                    narya_ui::compact_metric("总流量", "1.26 GB", "↓ 842 MB  ↑ 436 MB")
                        .into_any_element(),
                    narya_ui::compact_metric("连接数", "324", "峰值 1280").into_any_element(),
                ],
                narya_ui::soft_trend(traffic_values(), 188.0, narya_ui::BRAND),
            ),
            narya_ui::titled_panel(
                "连接统计",
                Flex::new()
                    .column()
                    .gap_lg()
                    .child(narya_ui::ratio_row("Shadowsocks", 63.5, NaryaStatus::Info))
                    .child(narya_ui::ratio_row("Vmess", 23.4, NaryaStatus::Success))
                    .child(narya_ui::ratio_row("Trojan", 8.7, NaryaStatus::Warning))
                    .child(narya_ui::ratio_row("Hysteria2", 4.4, NaryaStatus::Danger))
                    .child(narya_ui::detail_field("总连接数", "324")),
            ),
            narya_ui::titled_panel(
                "活动日志",
                Flex::new()
                    .column()
                    .gap_md()
                    .child(narya_ui::log_line(
                        "17:25:21",
                        format!("已连接到 {}", snapshot.active_node_name),
                        NaryaStatus::Success,
                    ))
                    .child(narya_ui::log_line(
                        "17:25:20",
                        "系统代理已启用",
                        NaryaStatus::Success,
                    ))
                    .child(narya_ui::log_line(
                        "17:25:18",
                        "TUN 模式已启用",
                        NaryaStatus::Info,
                    ))
                    .child(narya_ui::log_line(
                        "17:25:09",
                        "正在更新 GeoIP 数据库",
                        NaryaStatus::Info,
                    ))
                    .child(narya_ui::log_line(
                        "17:25:08",
                        "配置加载成功",
                        NaryaStatus::Info,
                    ))
                    .child(NaryaButton::ghost("断开连接").on_click(move |_, _, cx| {
                        AppState::toggle_proxy(model_for_toggle.clone(), cx)
                    })),
            ),
        ))
}

fn nodes_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    NaryaPage::new()
        .row(narya_ui::nodes_top_controls(vec![
            narya_ui::control_card(
                "当前策略组",
                "Proxy / 自动选择",
                IconName::Rocket,
                300.0,
                NaryaStatus::Info,
            )
            .into_any_element(),
            narya_ui::control_card(
                "当前节点",
                format!(
                    "{}    {} ms",
                    snapshot.active_node_name, snapshot.active_latency
                ),
                IconName::MapPinHouse,
                356.0,
                NaryaStatus::Success,
            )
            .into_any_element(),
            narya_ui::control_card(
                "模式",
                "规则模式",
                IconName::Settings2,
                262.0,
                NaryaStatus::Info,
            )
            .into_any_element(),
            narya_ui::gradient_action("一键测速", IconName::Gauge)
                .on_click({
                    let model = model.clone();
                    move |_, _, cx| AppState::test_all_latency(model.clone(), cx)
                })
                .into_any_element(),
        ]))
        .row(narya_ui::toolbar(vec![
            narya_ui::search_input("搜索节点、地区、协议或标签", 286.0).into_any_element(),
            narya_ui::filter_segmented(
                &[
                    "全部",
                    "低延迟",
                    "香港",
                    "日本",
                    "美国",
                    "新加坡",
                    "Hysteria2",
                    "Vmess",
                    "Shadowsocks",
                ],
                "全部",
            )
            .into_any_element(),
            narya_ui::sort_select(&["按延迟排序", "按名称排序", "按负载排序"], 0, 132.0)
                .into_any_element(),
        ]))
        .row(narya_ui::nodes_main(
            narya_ui::titled_panel(
                "策略组",
                narya_ui::category_menu(
                    vec![
                        ("1   Proxy   自动选择      38 / 128", IconName::Rocket),
                        ("2   Global   全局代理      36 / 128", IconName::Globe),
                        (
                            "3   Direct   国内直连      1 / 128",
                            IconName::MousePointer2,
                        ),
                        ("4   AI Services          28 / 128", IconName::ShieldCheck),
                        ("5   Streaming            24 / 128", IconName::MonitorPlay),
                        ("6   Gaming               20 / 128", IconName::Gamepad2),
                        ("7   Fallback             8 / 128", IconName::FlaskConical),
                    ],
                    0,
                ),
            ),
            narya_ui::titled_panel(
                "节点列表",
                narya_ui::node_grid(
                    snapshot
                        .nodes
                        .iter()
                        .cloned()
                        .map(|node| {
                            let id = node.id.clone();
                            let model = model.clone();
                            narya_ui::node_card(
                                narya_ui::NodeCardData::new(
                                    node.name,
                                    node.protocol,
                                    node.latency.unwrap_or(0),
                                    node.usage_pct,
                                    node.download_speed,
                                    node.upload_speed,
                                    snapshot.active_node_name == id,
                                ),
                                Box::new(move |_, _, cx| {
                                    AppState::connect_node(model.clone(), cx, id.clone())
                                }),
                            )
                            .into_any_element()
                        })
                        .collect(),
                ),
            ),
            narya_ui::titled_panel(
                "测速概览",
                Flex::new()
                    .column()
                    .gap_lg()
                    .child(NaryaMetric::card(
                        "平均延迟",
                        "82 ms",
                        "最快节点：香港 · HK 01",
                        IconName::Gauge,
                        NaryaStatus::Info,
                    ))
                    .child(narya_ui::detail_field("可用节点", "38 / 128"))
                    .child(narya_ui::detail_field("失败", "3"))
                    .child(narya_ui::detail_field("上次测速时间", "17:26:12"))
                    .child(NaryaButton::ghost("查看测速日志")),
            ),
        ))
        .row(narya_ui::nodes_bottom(
            narya_ui::chart_card("延迟趋势", latency_values(), 128.0, narya_ui::SUCCESS),
            narya_ui::titled_panel(
                "节点详情（香港 · HK 01）",
                Flex::new()
                    .column()
                    .gap_md()
                    .child(narya_ui::detail_field("地址", "hkg01.narya.net:443"))
                    .child(narya_ui::detail_field("协议", "Shadowsocks"))
                    .child(narya_ui::detail_field("加密", "2022-blake3-aes-128-gcm"))
                    .child(narya_ui::detail_field("UDP", "已启用"))
                    .child(NaryaButton::ghost("设为默认")),
            ),
        ))
}

fn subscriptions_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "当前订阅",
                "机场 A",
                "类型：远程订阅",
                IconName::ClipboardList,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "节点总数",
                "128",
                "38 可用 / 3 失败",
                IconName::CircleGauge,
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card(
                "剩余流量",
                "842 GB",
                "已用 436 GB",
                IconName::ChartNoAxesCombined,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "到期时间",
                "42 天",
                "2026-06-10 到期",
                IconName::SquareStack,
                NaryaStatus::Warning,
            )
            .into_any_element(),
            NaryaButton::primary("＋ 添加订阅")
                .disabled(true)
                .into_any_element(),
        ]))
        .row(narya_ui::page_columns(
            NaryaCard::titled(
                "订阅源列表",
                Flex::new().column().gap_md().children(
                    snapshot.subscriptions.iter().enumerate().map(|(idx, sub)| {
                        let usage = if sub.traffic_total > 0.0 {
                            ((sub.traffic_used / sub.traffic_total) * 100.0) as f32
                        } else {
                            0.0
                        };
                        narya_ui::subscription_item(
                            sub.name.clone(),
                            sub.url.clone(),
                            sub.node_count,
                            usage,
                            idx == 0,
                        )
                    }),
                ),
            ),
            NaryaCard::titled(
                "更新状态",
                Flex::new()
                    .column()
                    .gap_lg()
                    .child(NaryaMetric::card(
                        "更新成功",
                        "128 ms",
                        "下载时间 1.82s",
                        IconName::Check,
                        NaryaStatus::Success,
                    ))
                    .child(narya_ui::metric_grid(vec![
                        NaryaMetric::card(
                            "新增节点",
                            "+4",
                            "",
                            IconName::Plus,
                            NaryaStatus::Success,
                        )
                        .into_any_element(),
                        NaryaMetric::card(
                            "移除节点",
                            "-1",
                            "",
                            IconName::Minus,
                            NaryaStatus::Danger,
                        )
                        .into_any_element(),
                        NaryaMetric::card("未变更", "125", "", IconName::Equal, NaryaStatus::Info)
                            .into_any_element(),
                    ]))
                    .child(NaryaButton::ghost("查看更新日志")),
            ),
        ))
        .row(narya_ui::page_row(vec![
            narya_ui::chart_card(
                "流量趋势（最近 30 天）",
                traffic_values(),
                132.0,
                narya_ui::BRAND,
            )
            .into_any_element(),
            NaryaCard::titled(
                "订阅优先级",
                Flex::new()
                    .row()
                    .gap_lg()
                    .child(NaryaMetric::card(
                        "1",
                        "远程订阅",
                        "机场 A · 128 节点",
                        IconName::Badge,
                        NaryaStatus::Info,
                    ))
                    .child(NaryaMetric::card(
                        "2",
                        "本地覆写",
                        "Narya Default",
                        IconName::Badge,
                        NaryaStatus::Info,
                    ))
                    .child(NaryaMetric::card(
                        "3",
                        "UI 临时规则",
                        "活动中",
                        IconName::Badge,
                        NaryaStatus::Info,
                    )),
            )
            .into_any_element(),
            NaryaCard::titled(
                "自动更新",
                Flex::new()
                    .column()
                    .gap_md()
                    .child(narya_ui::setting_row("更新间隔：每 30 分钟", true))
                    .child(narya_ui::setting_row("启动时更新", true))
                    .child(narya_ui::setting_row("静默失败通知", true)),
            )
            .into_any_element(),
        ]))
        .row(NaryaButton::ghost("手动刷新").on_click({
            let model = model.clone();
            move |_, _, cx| AppState::refresh_subscription(model.clone(), cx, "sub-1".to_string())
        }))
}

fn settings_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let kernel_infos = snapshot.kernels.clone();
    let kernel_label = snapshot
        .kernels
        .first()
        .filter(|kernel| kernel.name == snapshot.active_kernel)
        .or_else(|| {
            snapshot
                .kernels
                .iter()
                .find(|kernel| kernel.name == snapshot.active_kernel)
        })
        .and_then(|k| k.version.clone())
        .map(|version| format!("{} {version}", snapshot.active_kernel))
        .unwrap_or_else(|| format!("{} 未安装", snapshot.active_kernel));
    let kernel_status = if snapshot.kernel_healthy {
        "健康运行"
    } else if snapshot.kernels.iter().any(|kernel| kernel.installed) {
        "已安装，未运行"
    } else {
        "需要可信工件"
    };
    let kernel_tone = if snapshot.kernel_healthy {
        NaryaStatus::Success
    } else {
        NaryaStatus::Warning
    };
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "应用版本",
                "1.0.0",
                "当前为最新版本",
                IconName::ClipboardList,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "当前内核",
                kernel_label,
                kernel_status,
                IconName::Cpu,
                kernel_tone,
            )
            .into_any_element(),
            NaryaMetric::card(
                "系统代理",
                "2080 / 1080",
                "HTTP / SOCKS",
                IconName::SquareStack,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "IPv6 状态",
                "自动 / 防泄漏",
                "已启用",
                IconName::Route,
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card(
                "更新通道",
                "Stable",
                "稳定版更新",
                IconName::RefreshCw,
                NaryaStatus::Info,
            )
            .into_any_element(),
        ]))
        .row(narya_ui::page_columns(
            narya_ui::page_row(vec![
                NaryaCard::titled(
                    "设置分类",
                    narya_ui::category_menu(
                        vec![
                            ("常规", IconName::Settings),
                            ("外观", IconName::Palette),
                            ("网络", IconName::Network),
                            ("IPv6", IconName::Route),
                            ("内核", IconName::Cpu),
                            ("TUN", IconName::Shield),
                            ("DNS", IconName::Server),
                            ("安全", IconName::LockKeyhole),
                            ("通知", IconName::Bell),
                            ("更新", IconName::RefreshCw),
                            ("高级", IconName::SlidersHorizontal),
                        ],
                        0,
                    ),
                )
                .into_any_element(),
                NaryaCard::titled(
                    "常规设置",
                    Flex::new()
                        .column()
                        .gap_lg()
                        .child(narya_ui::setting_row("开机自启", false))
                        .child(narya_ui::setting_row("启动后最小化", false))
                        .child(narya_ui::setting_row("关闭到托盘", true))
                        .child(narya_ui::setting_row("启动时恢复代理", true))
                        .child(narya_ui::detail_field("语言", "简体中文⌄"))
                        .child(narya_ui::detail_field("时区", "Asia/Shanghai⌄"))
                        .child(narya_ui::detail_field("HTTP 端口", "7890"))
                        .child(narya_ui::detail_field("SOCKS 端口", "7891"))
                        .child(narya_ui::detail_field("API 端口", "9090")),
                )
                .into_any_element(),
            ]),
            Flex::new()
                .column()
                .gap_lg()
                .child(NaryaCard::titled(
                    "内核管理",
                    Flex::new()
                        .column()
                        .gap_md()
                        .children(kernel_infos.into_iter().map(|k| {
                            narya_ui::detail_field(
                                k.name,
                                if k.installed {
                                    "已安装"
                                } else {
                                    "未安装"
                                },
                            )
                        }))
                        .child(KernelArtifactForm {
                            model: model.clone(),
                            selected_kernel: snapshot.kernel_artifact_kernel,
                            version: snapshot.kernel_artifact_version,
                            source: snapshot.kernel_artifact_source,
                            sha256: snapshot.kernel_artifact_sha256,
                            signature: snapshot.kernel_artifact_signature,
                            public_key: snapshot.kernel_artifact_public_key,
                            operation: snapshot.kernel_operation,
                            error: snapshot.kernel_error,
                        }),
                ))
                .child(NaryaCard::titled(
                    "权限状态",
                    Flex::new()
                        .column()
                        .gap_md()
                        .child(narya_ui::setting_row("系统代理权限", true))
                        .child(narya_ui::setting_row("TUN 权限", true))
                        .child(narya_ui::setting_row("通知权限", true))
                        .child(narya_ui::setting_row("开机自启权限", false)),
                ))
                .child(NaryaCard::titled(
                    "安全与隐私",
                    Flex::new()
                        .column()
                        .gap_md()
                        .child(narya_ui::setting_row("日志脱敏", true))
                        .child(narya_ui::setting_row("本地 API Token", true))
                        .child(narya_ui::setting_row("配置文件加密", false)),
                )),
        ))
        .row(narya_ui::page_row(vec![
            NaryaCard::titled(
                "外观预览",
                narya_ui::segmented_control(&["浅色", "深色", "跟随系统"], "浅色", 260.0),
            )
            .into_any_element(),
            NaryaCard::titled(
                "更新设置",
                Flex::new()
                    .column()
                    .gap_lg()
                    .child(narya_ui::setting_row("自动检查更新", true))
                    .child(narya_ui::detail_field("更新通道", "Stable⌄"))
                    .child(NaryaButton::primary("检查更新")),
            )
            .into_any_element(),
        ]))
}

struct KernelArtifactForm {
    model: Entity<AppState>,
    selected_kernel: String,
    version: String,
    source: String,
    sha256: String,
    signature: String,
    public_key: String,
    operation: Option<String>,
    error: Option<String>,
}

impl NaryaRenderOnce for KernelArtifactForm {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let selected_index = match self.selected_kernel.as_str() {
            "mihomo" => 1,
            "xray-core" => 2,
            _ => 0,
        };
        let model_for_select = self.model.clone();
        let model_for_version = self.model.clone();
        let model_for_source = self.model.clone();
        let model_for_sha = self.model.clone();
        let model_for_signature = self.model.clone();
        let model_for_public_key = self.model.clone();
        let model_for_install = self.model.clone();
        let version = self.version;
        let source = self.source;
        let sha256 = self.sha256;
        let signature = self.signature;
        let public_key = self.public_key;
        let selected_kernel = self.selected_kernel;
        let operation = self.operation;
        let error = self.error;
        let installed = self
            .model
            .read(cx)
            .kernels
            .iter()
            .any(|kernel| kernel.name == selected_kernel && kernel.installed);
        let action_label = if installed {
            "升级内核"
        } else {
            "安装内核"
        };

        Flex::new()
            .column()
            .gap_md()
            .child(Text::new("仅接受明确来源与 SHA-256 的可信工件"))
            .child(cx.new(|cx| {
                Select::new(
                    vec!["sing-box", "mihomo", "xray-core"],
                    Some(selected_index),
                    cx,
                )
                .width(px(180.0))
                .on_change(move |index, _, app| {
                    let kernel = match index {
                        1 => "mihomo",
                        2 => "xray-core",
                        _ => "sing-box",
                    };
                    model_for_select.update(app, |state, state_cx| {
                        state.set_kernel_artifact_kernel(kernel.into(), state_cx);
                    });
                })
            }))
            .child(cx.new(|cx| {
                Input::new(version, cx)
                    .placeholder("版本，例如 1.11.0")
                    .width(px(260.0))
                    .on_change(move |value, input_cx| {
                        model_for_version.update(input_cx, |state, state_cx| {
                            state.set_kernel_artifact_version(value.to_string(), state_cx);
                        });
                    })
            }))
            .child(cx.new(|cx| {
                Input::new(source, cx)
                    .placeholder("绝对路径、file:// 或 HTTPS")
                    .width(px(420.0))
                    .on_change(move |value, input_cx| {
                        model_for_source.update(input_cx, |state, state_cx| {
                            state.set_kernel_artifact_source(value.to_string(), state_cx);
                        });
                    })
            }))
            .child(cx.new(|cx| {
                Input::new(sha256, cx)
                    .placeholder("SHA-256 64 位十六进制摘要")
                    .width(px(420.0))
                    .on_change(move |value, input_cx| {
                        model_for_sha.update(input_cx, |state, state_cx| {
                            state.set_kernel_artifact_sha256(value.to_string(), state_cx);
                        });
                    })
            }))
            .child(cx.new(|cx| {
                Input::new(signature, cx)
                    .placeholder("可选：Ed25519 签名（HTTPS 必填）")
                    .width(px(420.0))
                    .on_change(move |value, input_cx| {
                        model_for_signature.update(input_cx, |state, state_cx| {
                            state.set_kernel_artifact_signature(value.to_string(), state_cx);
                        });
                    })
            }))
            .child(cx.new(|cx| {
                Input::new(public_key, cx)
                    .placeholder("可选：Ed25519 公钥（HTTPS 必填）")
                    .width(px(420.0))
                    .on_change(move |value, input_cx| {
                        model_for_public_key.update(input_cx, |state, state_cx| {
                            state.set_kernel_artifact_public_key(value.to_string(), state_cx);
                        });
                    })
            }))
            .child(
                NaryaButton::primary(action_label).on_click(move |_, _, app| {
                    AppState::install_kernel(model_for_install.clone(), app);
                }),
            )
            .when_some(operation, |element, text| element.child(Text::new(text)))
            .when_some(error, |element, text| {
                element.child(Text::new(format!("错误：{text}")))
            })
    }
}

impl NaryaIntoElement for KernelArtifactForm {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

fn config_page() -> impl NaryaIntoElement {
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "当前配置",
                "Narya Default",
                "规则模式",
                IconName::ClipboardList,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "链式代理",
                "未启用",
                "可视化编排",
                IconName::ArrowLeftRight,
                NaryaStatus::Warning,
            )
            .into_any_element(),
            NaryaMetric::card(
                "YAML",
                "只读预览",
                "编辑器待接入",
                IconName::Braces,
                NaryaStatus::Info,
            )
            .into_any_element(),
        ]))
        .row(NaryaCard::titled(
            "配置工作台",
            Flex::new()
                .row()
                .gap_lg()
                .child(NaryaButton::ghost("可视化编辑").disabled(true))
                .child(NaryaButton::ghost("YAML 编辑器").disabled(true))
                .child(NaryaButton::ghost("链式代理").disabled(true)),
        ))
}

fn connections_page(snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    NaryaPage::new().row(narya_ui::page_columns(
        NaryaCard::titled(
            "近期连接",
            Flex::new()
                .column()
                .gap_md()
                .children(
                    snapshot
                        .nodes
                        .iter()
                        .take(8)
                        .enumerate()
                        .map(|(idx, node)| {
                            narya_ui::detail_field(
                                format!("10.0.0.{} → {}", idx + 10, node.details.address),
                                node.protocol.clone(),
                            )
                        }),
                ),
        ),
        NaryaCard::titled(
            "连接摘要",
            Flex::new()
                .column()
                .gap_lg()
                .child(NaryaMetric::card(
                    "活跃连接",
                    "324",
                    "TCP / UDP",
                    IconName::ArrowLeftRight,
                    NaryaStatus::Info,
                ))
                .child(NaryaMetric::card(
                    "规则命中",
                    "12,840",
                    "DIRECT 62%",
                    IconName::ListFilter,
                    NaryaStatus::Success,
                )),
        ),
    ))
}

fn rules_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let filter = snapshot.rule_filter_text.to_ascii_lowercase();
    let action_filter = snapshot.rule_action_filter.as_str();
    let filtered_rules: Vec<_> = snapshot
        .rules
        .iter()
        .filter(|rule| {
            (action_filter == "all" || rule_action_key(&rule.action) == action_filter)
                && (filter.is_empty()
                    || rule.id.to_ascii_lowercase().contains(&filter)
                    || rule_action_summary(&rule.action)
                        .to_ascii_lowercase()
                        .contains(&filter)
                    || rule_condition_summary(rule)
                        .to_ascii_lowercase()
                        .contains(&filter))
        })
        .cloned()
        .collect();
    let model_for_add = model.clone();
    let search = RuleSearchBox {
        model: model.clone(),
    };
    let action_select = RuleActionFilterSelect {
        model: model.clone(),
        selected: snapshot.rule_action_filter.clone(),
    };
    let groups = snapshot.groups.clone();
    let model_for_group = model.clone();
    let mode = snapshot.routing_mode;
    NaryaPage::new()
        .row(narya_ui::metric_grid(vec![
            NaryaMetric::card(
                "规则集",
                filtered_rules.len().to_string(),
                "本地规则 · 优先级排序",
                IconName::ListFilter,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "今日命中",
                "12,840",
                "DIRECT 62% · PROXY 38%",
                IconName::CircleGauge,
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card(
                "最后更新",
                "2 天前",
                "规则编译前校验",
                IconName::RefreshCw,
                NaryaStatus::Warning,
            )
            .into_any_element(),
        ]))
        .row(narya_ui::toolbar(vec![
            search.into_any_element(),
            action_select.into_any_element(),
            NaryaButton::primary("新增规则")
                .on_click(move |_, _, cx| AppState::add_rule(model_for_add.clone(), cx))
                .into_any_element(),
        ]))
        .row(NaryaCard::titled(
            "规则配置文件",
            RuleIoControls {
                model: model.clone(),
                path: snapshot.rule_io_path.clone(),
                status: snapshot.rule_io_status.clone(),
            },
        ))
        .row(NaryaCard::titled(
            "分流规则",
            Flex::new()
                .column()
                .gap_md()
                .children(filtered_rules.into_iter().map(|rule| {
                    let rule_id = rule.id.clone();
                    let delete_model = model.clone();
                    let tone = match rule.action {
                        narya_rules::Action::Proxy(_) => NaryaStatus::Info,
                        narya_rules::Action::Direct => NaryaStatus::Success,
                        narya_rules::Action::Block => NaryaStatus::Danger,
                        narya_rules::Action::Dns(_) => NaryaStatus::Warning,
                    };
                    NaryaCard::titled(
                        rule.id.clone(),
                        Flex::new()
                            .row()
                            .gap_lg()
                            .align_center()
                            .child(RulePriorityInput {
                                model: model.clone(),
                                rule_id: rule.id.clone(),
                                priority: rule.priority,
                            })
                            .child(RuleConditionEditor {
                                model: model.clone(),
                                rule_id: rule.id.clone(),
                                conditions: rule.conditions.clone(),
                            })
                            .child(RuleActionSelect {
                                model: model.clone(),
                                rule_id: rule.id.clone(),
                                selected: rule_action_value(&rule.action),
                                groups: snapshot
                                    .groups
                                    .iter()
                                    .map(|group| group.id.clone())
                                    .collect(),
                            })
                            .child(narya_ui::narya_tag(rule_action_summary(&rule.action), tone))
                            .child(NaryaButton::ghost("删除").on_click(move |_, _, cx| {
                                AppState::remove_rule(delete_model.clone(), cx, rule_id.clone())
                            })),
                    )
                    .into_any_element()
                })),
        ))
        .when_some(snapshot.rule_editor_error.clone(), |element, error| {
            element.row(NaryaCard::titled("规则校验", Text::new(error)))
        })
        .row(NaryaCard::titled(
            "分流组",
            Flex::new()
                .column()
                .gap_md()
                .children(groups.into_iter().map(|group| {
                    let group_id = group.id.clone();
                    let removable = group.id != "proxy";
                    let remove_model = model.clone();
                    let editor_group = group.clone();
                    Flex::new()
                        .row()
                        .gap_lg()
                        .align_center()
                        .child(Text::new(group.id.clone()))
                        .child(Text::new(format!(
                            "{} · {}",
                            group_strategy_label(group.strategy),
                            group.members.join(", ")
                        )))
                        .child(NaryaButton::ghost("删除").disabled(!removable).on_click(
                            move |_, _, cx| {
                                AppState::remove_group(remove_model.clone(), cx, group_id.clone())
                            },
                        ))
                        .child(GroupEditor {
                            model: model.clone(),
                            group: editor_group,
                        })
                        .into_any_element()
                }))
                .child(
                    NaryaButton::ghost("新增分流组")
                        .on_click(move |_, _, cx| AppState::add_group(model_for_group.clone(), cx)),
                )
                .when_some(snapshot.group_error.clone(), |element, error| {
                    element.child(Text::new(error))
                }),
        ))
        .row(NaryaCard::titled(
            "规则集",
            Flex::new()
                .column()
                .gap_md()
                .children(snapshot.rule_sets.iter().cloned().map(|source| {
                    let source_id = source.id.clone();
                    let remove_model = model.clone();
                    Flex::new()
                        .row()
                        .gap_lg()
                        .align_center()
                        .child(Text::new(source.id))
                        .child(Text::new(format!(
                            "v{} · SHA-256 {}",
                            source.version,
                            source.sha256.chars().take(12).collect::<String>()
                        )))
                        .child(Text::new(source.source))
                        .child(NaryaButton::ghost("删除").on_click(move |_, _, cx| {
                            AppState::remove_rule_set(remove_model.clone(), cx, source_id.clone())
                        }))
                        .into_any_element()
                }))
                .child(RuleSetForm {
                    model: model.clone(),
                    id: snapshot.rule_set_draft_id,
                    source: snapshot.rule_set_draft_source,
                    version: snapshot.rule_set_draft_version,
                    sha256: snapshot.rule_set_draft_sha256,
                    signature: snapshot.rule_set_draft_signature,
                    public_key: snapshot.rule_set_draft_public_key,
                    error: snapshot.rule_set_error,
                }),
        ))
        .row(NaryaCard::titled(
            "运行模式",
            Flex::new()
                .row()
                .gap_md()
                .align_center()
                .child(Text::new(format!(
                    "目标：{} · 当前：{}{}",
                    routing_mode_label(mode),
                    routing_mode_label(snapshot.routing_active),
                    if snapshot.kernel_healthy {
                        " · 内核健康"
                    } else {
                        " · 等待 daemon 确认"
                    }
                )))
                .child(
                    NaryaButton::ghost("系统代理")
                        .disabled(snapshot.running)
                        .on_click({
                            let model = model.clone();
                            move |_, _, cx| {
                                model.update(cx, |state, cx| {
                                    state.set_routing_mode(
                                        narya_platform::ProxyMode::SystemProxy,
                                        cx,
                                    )
                                });
                            }
                        }),
                )
                .child(
                    NaryaButton::ghost("TUN")
                        .disabled(snapshot.running)
                        .on_click({
                            let model = model.clone();
                            move |_, _, cx| {
                                model.update(cx, |state, cx| {
                                    state.set_routing_mode(narya_platform::ProxyMode::Tun, cx)
                                });
                            }
                        }),
                ),
        ))
}

struct RuleIoControls {
    model: Entity<AppState>,
    path: String,
    status: Option<String>,
}

impl NaryaRenderOnce for RuleIoControls {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let model_path = self.model.clone();
        let model_export = self.model.clone();
        let model_import = self.model.clone();
        Flex::new()
            .row()
            .gap_md()
            .child(cx.new(|cx| {
                Input::new(self.path, cx)
                    .placeholder("绝对路径，例如 /tmp/narya-routes.json")
                    .width(px(420.0))
                    .on_change(move |value, input_cx| {
                        model_path.update(input_cx, |state, state_cx| {
                            state.set_rule_io_path(value.to_string(), state_cx)
                        });
                    })
            }))
            .child(
                NaryaButton::ghost("导入")
                    .on_click(move |_, _, app| AppState::import_rules(model_import.clone(), app)),
            )
            .child(
                NaryaButton::ghost("导出")
                    .on_click(move |_, _, app| AppState::export_rules(model_export.clone(), app)),
            )
            .when_some(self.status, |element, status| {
                element.child(Text::new(status))
            })
    }
}

impl NaryaIntoElement for RuleIoControls {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

struct RuleConditionEditor {
    model: Entity<AppState>,
    rule_id: String,
    conditions: Vec<narya_rules::Condition>,
}

impl NaryaRenderOnce for RuleConditionEditor {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let rule_id = self.rule_id.clone();
        let add_model = self.model.clone();
        let mut editor = Flex::new().column().gap_sm();
        for (index, condition) in self.conditions.into_iter().enumerate() {
            let (kind, value) = condition_editor_value(&condition);
            let kinds = [
                "domain".to_string(),
                "domain_suffix".to_string(),
                "ip_cidr".to_string(),
                "port".to_string(),
                "process".to_string(),
                "rule_set".to_string(),
                "any".to_string(),
            ];
            let labels = kinds
                .iter()
                .map(|kind| condition_kind_label(kind).to_string())
                .collect::<Vec<_>>();
            let selected = kinds.iter().position(|item| item == &kind).unwrap_or(0);
            let model_select = self.model.clone();
            let model_value = self.model.clone();
            let model_remove = self.model.clone();
            let rule_for_select = rule_id.clone();
            let rule_for_value = rule_id.clone();
            let rule_for_remove = rule_id.clone();
            editor = editor.child(
                Flex::new()
                    .row()
                    .gap_sm()
                    .align_center()
                    .child(Text::new(if index == 0 { "条件" } else { "AND" }))
                    .child(cx.new(|cx| {
                        Select::new(labels, Some(selected), cx)
                            .width(px(130.0))
                            .on_change(move |next, _, app| {
                                if let Some(next_kind) = kinds.get(next) {
                                    AppState::set_rule_condition(
                                        model_select.clone(),
                                        app,
                                        rule_for_select.clone(),
                                        index,
                                        next_kind.clone(),
                                        condition_default_value(next_kind),
                                    );
                                }
                            })
                    }))
                    .child(cx.new(|cx| {
                        Input::new(value, cx)
                            .placeholder("条件值")
                            .width(px(210.0))
                            .on_change(move |next, input_cx| {
                                AppState::set_rule_condition(
                                    model_value.clone(),
                                    input_cx,
                                    rule_for_value.clone(),
                                    index,
                                    kind.clone(),
                                    next.to_string(),
                                );
                            })
                    }))
                    .child(NaryaButton::ghost("移除").on_click(move |_, _, app| {
                        AppState::remove_rule_condition(
                            model_remove.clone(),
                            app,
                            rule_for_remove.clone(),
                            index,
                        )
                    })),
            );
        }
        editor.child(
            NaryaButton::ghost("添加 AND 条件").on_click(move |_, _, app| {
                AppState::add_rule_condition(add_model.clone(), app, rule_id.clone())
            }),
        )
    }
}

impl NaryaIntoElement for RuleConditionEditor {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

struct GroupEditor {
    model: Entity<AppState>,
    group: narya_rules::RoutingGroup,
}

impl NaryaRenderOnce for GroupEditor {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let group_id = self.group.id.clone();
        let select_model = self.model.clone();
        let members_model = self.model.clone();
        let url_model = self.model.clone();
        let interval_model = self.model.clone();
        let strategies = vec![
            "手动选择".to_string(),
            "URL 测试".to_string(),
            "故障转移".to_string(),
            "负载均衡".to_string(),
        ];
        let selected = match self.group.strategy {
            narya_rules::GroupStrategy::Select => 0,
            narya_rules::GroupStrategy::UrlTest => 1,
            narya_rules::GroupStrategy::Fallback => 2,
            narya_rules::GroupStrategy::LoadBalance => 3,
        };
        let id_for_select = group_id.clone();
        let id_for_members = group_id.clone();
        let id_for_url = group_id.clone();
        let id_for_interval = group_id;
        Flex::new()
            .row()
            .gap_sm()
            .child(cx.new(|cx| {
                Select::new(strategies, Some(selected), cx)
                    .width(px(130.0))
                    .on_change(move |index, _, app| {
                        AppState::set_group_strategy(
                            select_model.clone(),
                            app,
                            id_for_select.clone(),
                            index,
                        )
                    })
            }))
            .child(cx.new(|cx| {
                Input::new(self.group.members.join(", "), cx)
                    .placeholder("成员 outbound，用逗号分隔")
                    .width(px(260.0))
                    .on_change(move |value, input_cx| {
                        AppState::set_group_members(
                            members_model.clone(),
                            input_cx,
                            id_for_members.clone(),
                            value.to_string(),
                        )
                    })
            }))
            .child(cx.new(|cx| {
                Input::new(self.group.url.unwrap_or_default(), cx)
                    .placeholder("URL 测试地址")
                    .width(px(250.0))
                    .on_change(move |value, input_cx| {
                        AppState::set_group_url(
                            url_model.clone(),
                            input_cx,
                            id_for_url.clone(),
                            value.to_string(),
                        )
                    })
            }))
            .child(cx.new(|cx| {
                Input::new(
                    self.group
                        .interval_secs
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    cx,
                )
                .placeholder("间隔秒")
                .width(px(100.0))
                .on_change(move |value, input_cx| {
                    AppState::set_group_interval(
                        interval_model.clone(),
                        input_cx,
                        id_for_interval.clone(),
                        value.to_string(),
                    )
                })
            }))
    }
}

impl NaryaIntoElement for GroupEditor {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

fn condition_editor_value(condition: &narya_rules::Condition) -> (String, String) {
    match condition {
        narya_rules::Condition::Domain(value) => ("domain".into(), value.clone()),
        narya_rules::Condition::DomainSuffix(value) => ("domain_suffix".into(), value.clone()),
        narya_rules::Condition::IpCidr { network, prefix } => {
            ("ip_cidr".into(), format!("{network}/{prefix}"))
        }
        narya_rules::Condition::Port(value) => ("port".into(), value.to_string()),
        narya_rules::Condition::Process(value) => ("process".into(), value.clone()),
        narya_rules::Condition::RuleSet(value) => ("rule_set".into(), value.clone()),
        narya_rules::Condition::Any => ("any".into(), String::new()),
    }
}

fn condition_kind_label(kind: &str) -> &'static str {
    match kind {
        "domain" => "域名",
        "domain_suffix" => "域名后缀",
        "ip_cidr" => "IP/CIDR",
        "port" => "端口",
        "process" => "进程",
        "rule_set" => "规则集",
        "any" => "所有请求",
        _ => "条件",
    }
}

fn condition_default_value(kind: &str) -> String {
    match kind {
        "domain" | "domain_suffix" | "process" | "rule_set" => "example.com".into(),
        "ip_cidr" => "10.0.0.0/8".into(),
        "port" => "443".into(),
        "any" => String::new(),
        _ => String::new(),
    }
}

struct RuleSetForm {
    model: Entity<AppState>,
    id: String,
    source: String,
    version: String,
    sha256: String,
    signature: String,
    public_key: String,
    error: Option<String>,
}

impl NaryaRenderOnce for RuleSetForm {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let model_id = self.model.clone();
        let model_source = self.model.clone();
        let model_version = self.model.clone();
        let model_sha = self.model.clone();
        let model_add = self.model.clone();
        let model_signature = self.model.clone();
        let model_public_key = self.model.clone();
        Flex::new()
            .column()
            .gap_sm()
            .child(
                Flex::new()
                    .row()
                    .gap_md()
                    .child(cx.new(|cx| {
                        Input::new(self.id, cx)
                            .placeholder("ID，例如 geosite-ai")
                            .width(px(180.0))
                            .on_change(move |value, input_cx| {
                                model_id.update(input_cx, |state, state_cx| {
                                    state.set_rule_set_draft_id(value.to_string(), state_cx)
                                });
                            })
                    }))
                    .child(cx.new(|cx| {
                        Input::new(self.version, cx)
                            .placeholder("版本")
                            .width(px(120.0))
                            .on_change(move |value, input_cx| {
                                model_version.update(input_cx, |state, state_cx| {
                                    state.set_rule_set_draft_version(value.to_string(), state_cx)
                                });
                            })
                    }))
                    .child(cx.new(|cx| {
                        Input::new(self.source, cx)
                            .placeholder("绝对路径或 file://")
                            .width(px(320.0))
                            .on_change(move |value, input_cx| {
                                model_source.update(input_cx, |state, state_cx| {
                                    state.set_rule_set_draft_source(value.to_string(), state_cx)
                                });
                            })
                    }))
                    .child(cx.new(|cx| {
                        Input::new(self.sha256, cx)
                            .placeholder("SHA-256")
                            .width(px(260.0))
                            .on_change(move |value, input_cx| {
                                model_sha.update(input_cx, |state, state_cx| {
                                    state.set_rule_set_draft_sha256(value.to_string(), state_cx)
                                });
                            })
                    }))
                    .child(cx.new(|cx| {
                        Input::new(self.signature, cx)
                            .placeholder("Ed25519 签名（HTTPS 必填）")
                            .width(px(320.0))
                            .on_change(move |value, input_cx| {
                                model_signature.update(input_cx, |state, state_cx| {
                                    state.set_rule_set_draft_signature(value.to_string(), state_cx)
                                });
                            })
                    }))
                    .child(cx.new(|cx| {
                        Input::new(self.public_key, cx)
                            .placeholder("Ed25519 公钥（HTTPS 必填）")
                            .width(px(260.0))
                            .on_change(move |value, input_cx| {
                                model_public_key.update(input_cx, |state, state_cx| {
                                    state.set_rule_set_draft_public_key(value.to_string(), state_cx)
                                });
                            })
                    }))
                    .child(
                        NaryaButton::primary("导入规则集").on_click(move |_, _, app| {
                            AppState::add_rule_set(model_add.clone(), app)
                        }),
                    ),
            )
            .when_some(self.error, |element, error| element.child(Text::new(error)))
    }
}

impl NaryaIntoElement for RuleSetForm {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

struct RuleSearchBox {
    model: Entity<AppState>,
}

impl NaryaRenderOnce for RuleSearchBox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let model = self.model;
        cx.new(|cx| {
            Input::new("", cx)
                .placeholder("搜索规则、条件或动作")
                .icon_prefix(IconName::Search)
                .clearable(true)
                .width(px(300.0))
                .on_change(move |value, input_cx| {
                    model.update(input_cx, |state, state_cx| {
                        state.set_rule_filter_text(value.to_string(), state_cx);
                    });
                })
        })
    }
}

impl NaryaIntoElement for RuleSearchBox {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

struct RuleActionFilterSelect {
    model: Entity<AppState>,
    selected: String,
}

impl NaryaRenderOnce for RuleActionFilterSelect {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let options = vec!["全部动作", "代理", "直连", "阻断", "DNS"];
        let selected_index = match self.selected.as_str() {
            "proxy" => 1,
            "direct" => 2,
            "block" => 3,
            "dns" => 4,
            _ => 0,
        };
        let model = self.model;
        cx.new(|cx| {
            Select::new(options, Some(selected_index), cx)
                .width(px(144.0))
                .on_change(move |index, _, app| {
                    let filter = match index {
                        1 => "proxy",
                        2 => "direct",
                        3 => "block",
                        4 => "dns",
                        _ => "all",
                    };
                    model.update(app, |state, state_cx| {
                        state.set_rule_action_filter(filter.to_string(), state_cx);
                    });
                })
        })
    }
}

impl NaryaIntoElement for RuleActionFilterSelect {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

struct RuleActionSelect {
    model: Entity<AppState>,
    rule_id: String,
    selected: String,
    groups: Vec<String>,
}

impl NaryaRenderOnce for RuleActionSelect {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let mut values = vec![
            "proxy".to_string(),
            "direct".to_string(),
            "block".to_string(),
            "dns:proxy".to_string(),
        ];
        for group in self.groups {
            if group != "proxy" {
                values.push(group);
            }
        }
        let selected_index = values
            .iter()
            .position(|value| value == &self.selected)
            .unwrap_or(0);
        let labels: Vec<String> = values
            .iter()
            .map(|value| match value.as_str() {
                "proxy" => "代理组 · proxy".to_string(),
                "direct" => "直连".to_string(),
                "block" => "阻断".to_string(),
                "dns:proxy" => "DNS · proxy".to_string(),
                value => format!("代理组 · {value}"),
            })
            .collect();
        let model = self.model;
        let rule_id = self.rule_id;
        cx.new(|cx| {
            Select::new(labels, Some(selected_index), cx)
                .width(px(116.0))
                .on_change(move |index, _, app| {
                    if let Some(value) = values.get(index) {
                        AppState::set_rule_action(
                            model.clone(),
                            app,
                            rule_id.clone(),
                            value.clone(),
                        );
                    }
                })
        })
    }
}

impl NaryaIntoElement for RuleActionSelect {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

struct RulePriorityInput {
    model: Entity<AppState>,
    rule_id: String,
    priority: i32,
}

impl NaryaRenderOnce for RulePriorityInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let model = self.model;
        let rule_id = self.rule_id;
        cx.new(|cx| {
            Input::new(self.priority.to_string(), cx)
                .width(px(84.0))
                .on_change(move |value, input_cx| {
                    if let Ok(priority) = value.parse::<i32>() {
                        AppState::set_rule_priority(
                            model.clone(),
                            input_cx,
                            rule_id.clone(),
                            priority,
                        );
                    }
                })
        })
    }
}

impl NaryaIntoElement for RulePriorityInput {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

fn rule_condition_summary(rule: &narya_rules::Rule) -> String {
    rule.conditions
        .iter()
        .map(|condition| match condition {
            narya_rules::Condition::Domain(value) => format!("域名 = {value}"),
            narya_rules::Condition::DomainSuffix(value) => format!("域名后缀 · {value}"),
            narya_rules::Condition::IpCidr { network, prefix } => {
                format!("CIDR · {network}/{prefix}")
            }
            narya_rules::Condition::Port(port) => format!("端口 · {port}"),
            narya_rules::Condition::Process(process) => format!("进程 · {process}"),
            narya_rules::Condition::RuleSet(name) => format!("规则集 · {name}"),
            narya_rules::Condition::Any => "所有请求".to_string(),
        })
        .collect::<Vec<_>>()
        .join("  AND  ")
}

fn rule_action_summary(action: &narya_rules::Action) -> String {
    match action {
        narya_rules::Action::Proxy(outbound) => format!("代理 · {outbound}"),
        narya_rules::Action::Direct => "直连".to_string(),
        narya_rules::Action::Block => "阻断".to_string(),
        narya_rules::Action::Dns(server) => format!("DNS · {server}"),
    }
}

fn rule_action_key(action: &narya_rules::Action) -> &'static str {
    match action {
        narya_rules::Action::Proxy(_) => "proxy",
        narya_rules::Action::Direct => "direct",
        narya_rules::Action::Block => "block",
        narya_rules::Action::Dns(_) => "dns",
    }
}

fn rule_action_value(action: &narya_rules::Action) -> String {
    match action {
        narya_rules::Action::Proxy(outbound) => outbound.clone(),
        narya_rules::Action::Direct => "direct".into(),
        narya_rules::Action::Block => "block".into(),
        narya_rules::Action::Dns(server) => format!("dns:{server}"),
    }
}

fn group_strategy_label(strategy: narya_rules::GroupStrategy) -> &'static str {
    match strategy {
        narya_rules::GroupStrategy::Select => "手动选择",
        narya_rules::GroupStrategy::UrlTest => "URL 测试",
        narya_rules::GroupStrategy::Fallback => "故障转移",
        narya_rules::GroupStrategy::LoadBalance => "负载均衡",
    }
}

fn routing_mode_label(mode: narya_platform::ProxyMode) -> &'static str {
    match mode {
        narya_platform::ProxyMode::Disabled => "关闭",
        narya_platform::ProxyMode::SystemProxy => "系统代理",
        narya_platform::ProxyMode::Tun => "TUN",
    }
}

fn logs_page(snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let rows: Vec<_> = if snapshot.logs.is_empty() {
        vec![
            narya_ui::log_line("--:--:--", "Daemon 尚未推送日志", NaryaStatus::Info)
                .into_any_element(),
        ]
    } else {
        snapshot
            .logs
            .into_iter()
            .rev()
            .take(18)
            .map(|log| {
                narya_ui::log_line(log.time, log.content, NaryaStatus::Info).into_any_element()
            })
            .collect()
    };
    NaryaPage::new().row(NaryaCard::titled(
        "实时日志",
        Flex::new().column().gap_md().children(rows),
    ))
}

fn tools_page() -> impl NaryaIntoElement {
    NaryaPage::new().row(narya_ui::grid_two(vec![
        NaryaMetric::card(
            "Ping 测试",
            "就绪",
            "检测主机可达性",
            IconName::Zap,
            NaryaStatus::Info,
        )
        .into_any_element(),
        NaryaMetric::card(
            "DNS 查询",
            "就绪",
            "查看解析链路",
            IconName::CircleGauge,
            NaryaStatus::Success,
        )
        .into_any_element(),
        NaryaMetric::card(
            "MTR Trace",
            "就绪",
            "追踪链路质量",
            IconName::ArrowLeftRight,
            NaryaStatus::Warning,
        )
        .into_any_element(),
        NaryaMetric::card(
            "端口检查",
            "就绪",
            "验证远端端口",
            IconName::SquareStack,
            NaryaStatus::Info,
        )
        .into_any_element(),
    ]))
}

fn about_page() -> impl NaryaIntoElement {
    NaryaPage::new().row(NaryaCard::titled(
        "Narya",
        Text::new("GPUI native proxy client rebuilt with Liora components.").selectable(false),
    ))
}

fn latency_status(ms: u32) -> NaryaStatus {
    if ms < 90 {
        NaryaStatus::Success
    } else if ms < 140 {
        NaryaStatus::Warning
    } else {
        NaryaStatus::Danger
    }
}

fn latency_values() -> Vec<f64> {
    vec![
        100.0, 72.0, 68.0, 73.0, 61.0, 64.0, 48.0, 36.0, 44.0, 58.0, 42.0, 57.0, 70.0, 54.0, 78.0,
        51.0,
    ]
}

fn traffic_values() -> Vec<f64> {
    vec![
        5.0, 8.0, 13.0, 18.0, 16.0, 25.0, 15.0, 12.0, 18.0, 24.0, 21.0, 31.0, 26.0, 34.0, 20.0,
        25.0,
    ]
}

impl From<ActiveView> for NavTarget {
    fn from(value: ActiveView) -> Self {
        match value {
            ActiveView::Dashboard => NavTarget::Dashboard,
            ActiveView::Nodes => NavTarget::Nodes,
            ActiveView::Config => NavTarget::Config,
            ActiveView::Subscriptions => NavTarget::Subscriptions,
            ActiveView::Connections => NavTarget::Connections,
            ActiveView::Rules => NavTarget::Rules,
            ActiveView::Logs => NavTarget::Logs,
            ActiveView::Tools => NavTarget::Tools,
            ActiveView::Settings | ActiveView::About => NavTarget::Settings,
        }
    }
}

impl From<NavTarget> for ActiveView {
    fn from(value: NavTarget) -> Self {
        match value {
            NavTarget::Dashboard => ActiveView::Dashboard,
            NavTarget::Nodes => ActiveView::Nodes,
            NavTarget::Config => ActiveView::Config,
            NavTarget::Subscriptions => ActiveView::Subscriptions,
            NavTarget::Connections => ActiveView::Connections,
            NavTarget::Rules => ActiveView::Rules,
            NavTarget::Logs => ActiveView::Logs,
            NavTarget::Tools => ActiveView::Tools,
            NavTarget::Settings => ActiveView::Settings,
        }
    }
}

impl From<ActiveView> for PageKind {
    fn from(value: ActiveView) -> Self {
        match value {
            ActiveView::Dashboard => PageKind::Dashboard,
            ActiveView::Nodes => PageKind::Nodes,
            ActiveView::Config => PageKind::Config,
            ActiveView::Subscriptions => PageKind::Subscriptions,
            ActiveView::Connections => PageKind::Connections,
            ActiveView::Rules => PageKind::Rules,
            ActiveView::Logs => PageKind::Logs,
            ActiveView::Tools => PageKind::Tools,
            ActiveView::Settings => PageKind::Settings,
            ActiveView::About => PageKind::About,
        }
    }
}
