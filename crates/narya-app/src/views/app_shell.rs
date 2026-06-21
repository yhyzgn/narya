use crate::state::AppState;
use crate::ui_kit as narya_ui;
use crate::ui_kit::{
    NaryaButton, NaryaCard, NaryaMetric, NaryaPage, NaryaStatus, NavTarget, PageKind,
};
use crate::views::ActiveView;
use liora::components::{Flex, Text};
use narya_ui::{
    App, Context, NaryaAppContext, NaryaEntity as Entity, NaryaIntoElement, Render, Window,
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
        NaryaButton::icon("□").into_any_element(),
        NaryaButton::icon("▣").into_any_element(),
        NaryaButton::icon("⚙").into_any_element(),
        NaryaButton::icon("⋮").into_any_element(),
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
        ActiveView::Dashboard => actions.insert(
            0,
            NaryaButton::primary("连接")
                .on_click(move |_, _, cx| AppState::toggle_proxy(model_for_connect.clone(), cx))
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
        ActiveView::Settings => settings_page(snapshot).into_any_element(),
        ActiveView::Config => config_page().into_any_element(),
        ActiveView::Connections => connections_page(snapshot).into_any_element(),
        ActiveView::Rules => rules_page().into_any_element(),
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
                "▣",
                "系统代理",
                "管理系统网络代理设置",
                snapshot.running,
                "规则模式 ›",
                NaryaStatus::Info,
            ),
            narya_ui::hero_toggle_card(
                "☍",
                "TUN 虚拟网卡",
                "拦截并代理所有网络流量（推荐）",
                snapshot.running,
                "智能路由 ›",
                NaryaStatus::Success,
            ),
        ))
        .row(narya_ui::dashboard_middle(
            NaryaCard::titled(
                "快速连接",
                Flex::new()
                    .column()
                    .gap_md()
                    .children(snapshot.nodes.iter().take(4).map(|node| {
                        narya_ui::quick_node(
                            node.name.clone(),
                            node.protocol.clone(),
                            node.latency.unwrap_or(0),
                            latency_status(node.latency.unwrap_or(0)),
                        )
                    })),
            ),
            NaryaCard::titled(
                "网络概览",
                Flex::new()
                    .row()
                    .gap_lg()
                    .child(Flex::new().flex_1().child(narya_ui::trend_chart(
                        latency_values(),
                        214.0,
                        narya_ui::SUCCESS,
                    )))
                    .child(
                        Flex::new()
                            .width_px(270.0)
                            .flex_none()
                            .child(narya_ui::metric_grid(vec![
                                NaryaMetric::card(
                                    "节点延迟",
                                    format!("{} ms", snapshot.active_latency),
                                    "当前节点",
                                    "↯",
                                    NaryaStatus::Info,
                                )
                                .into_any_element(),
                                NaryaMetric::card(
                                    "可用节点",
                                    format!("{} / 128", snapshot.nodes.len() * 9 + 2),
                                    "在线 / 总数",
                                    "◎",
                                    NaryaStatus::Success,
                                )
                                .into_any_element(),
                            ])),
                    ),
            ),
        ))
        .row(narya_ui::dashboard_bottom(
            NaryaCard::titled(
                "流量使用",
                Flex::new()
                    .column()
                    .gap_lg()
                    .child(narya_ui::metric_grid(vec![
                        NaryaMetric::card(
                            "总流量",
                            "1.26 GB",
                            "↓ 842 MB  ↑ 436 MB",
                            "⇅",
                            NaryaStatus::Info,
                        )
                        .into_any_element(),
                        NaryaMetric::card("连接数", "324", "峰值 1280", "☷", NaryaStatus::Success)
                            .into_any_element(),
                    ]))
                    .child(narya_ui::trend_chart(
                        traffic_values(),
                        128.0,
                        narya_ui::BRAND,
                    )),
            ),
            NaryaCard::titled(
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
            NaryaCard::titled(
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
                    .child(NaryaButton::ghost("立即连接").on_click(move |_, _, cx| {
                        AppState::toggle_proxy(model_for_toggle.clone(), cx)
                    })),
            ),
        ))
}

fn nodes_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "当前策略组",
                "Proxy / 自动选择",
                "38 / 128",
                "🚀",
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "当前节点",
                snapshot.active_node_name.clone(),
                format!("{} ms", snapshot.active_latency),
                "✤",
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card("模式", "规则模式", "智能分流", "⌘", NaryaStatus::Info)
                .into_any_element(),
            NaryaButton::primary("一键测速")
                .on_click({
                    let model = model.clone();
                    move |_, _, cx| AppState::test_all_latency(model.clone(), cx)
                })
                .into_any_element(),
        ]))
        .row(narya_ui::toolbar(vec![
            NaryaButton::ghost("搜索节点、地区、协议或标签").into_any_element(),
            NaryaButton::primary("全部").into_any_element(),
            NaryaButton::ghost("低延迟").into_any_element(),
            NaryaButton::ghost("香港").into_any_element(),
            NaryaButton::ghost("日本").into_any_element(),
            NaryaButton::ghost("美国").into_any_element(),
            NaryaButton::ghost("Shadowsocks").into_any_element(),
            NaryaButton::ghost("按延迟排序⌄").into_any_element(),
        ]))
        .row(narya_ui::page_columns(
            NaryaCard::titled(
                "节点列表",
                Flex::new()
                    .row()
                    .wrap()
                    .gap_lg()
                    .children(snapshot.nodes.iter().cloned().map(|node| {
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
                    })),
            ),
            NaryaCard::titled(
                "测速概览",
                Flex::new()
                    .column()
                    .gap_lg()
                    .child(NaryaMetric::card(
                        "平均延迟",
                        "82 ms",
                        "最快节点：香港 · HK 01",
                        "◴",
                        NaryaStatus::Info,
                    ))
                    .child(narya_ui::detail_field("可用节点", "38 / 128"))
                    .child(narya_ui::detail_field("失败", "3"))
                    .child(narya_ui::detail_field("上次测速时间", "17:26:12"))
                    .child(NaryaButton::ghost("查看测速日志")),
            ),
        ))
        .row(narya_ui::page_row(vec![
            narya_ui::chart_card("延迟趋势", latency_values(), 128.0, narya_ui::SUCCESS)
                .into_any_element(),
            NaryaCard::titled(
                "节点详情（香港 · HK 01）",
                Flex::new()
                    .column()
                    .gap_md()
                    .child(narya_ui::detail_field("地址", "hkg01.narya.net:443"))
                    .child(narya_ui::detail_field("协议", "Shadowsocks"))
                    .child(narya_ui::detail_field("加密", "2022-blake3-aes-128-gcm"))
                    .child(narya_ui::detail_field("UDP", "已启用"))
                    .child(NaryaButton::ghost("设为默认")),
            )
            .into_any_element(),
        ]))
}

fn subscriptions_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "当前订阅",
                "机场 A",
                "类型：远程订阅",
                "▤",
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "节点总数",
                "128",
                "38 可用 / 3 失败",
                "◎",
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card("剩余流量", "842 GB", "已用 436 GB", "◍", NaryaStatus::Info)
                .into_any_element(),
            NaryaMetric::card(
                "到期时间",
                "42 天",
                "2026-06-10 到期",
                "▣",
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
                        "✓",
                        NaryaStatus::Success,
                    ))
                    .child(narya_ui::metric_grid(vec![
                        NaryaMetric::card("新增节点", "+4", "", "+", NaryaStatus::Success)
                            .into_any_element(),
                        NaryaMetric::card("移除节点", "-1", "", "−", NaryaStatus::Danger)
                            .into_any_element(),
                        NaryaMetric::card("未变更", "125", "", "=", NaryaStatus::Info)
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
                        "①",
                        NaryaStatus::Info,
                    ))
                    .child(NaryaMetric::card(
                        "2",
                        "本地覆写",
                        "Narya Default",
                        "②",
                        NaryaStatus::Info,
                    ))
                    .child(NaryaMetric::card(
                        "3",
                        "UI 临时规则",
                        "活动中",
                        "③",
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

fn settings_page(snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let kernel_label = snapshot
        .kernels
        .first()
        .and_then(|k| k.version.clone())
        .unwrap_or_else(|| "sing-box 1.11.x".to_string());
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "应用版本",
                "1.0.0",
                "当前为最新版本",
                "▤",
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "当前内核",
                kernel_label,
                "运行中",
                "◈",
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card(
                "系统代理",
                "7890 / 7891",
                "HTTP / SOCKS",
                "▣",
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "IPv6 状态",
                "自动 / 防泄漏",
                "已启用",
                "IPv6",
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card("更新通道", "Stable", "稳定版更新", "◉", NaryaStatus::Info)
                .into_any_element(),
        ]))
        .row(narya_ui::page_columns(
            narya_ui::page_row(vec![
                NaryaCard::titled(
                    "设置分类",
                    Flex::new()
                        .column()
                        .gap_md()
                        .child(narya_ui::category("常规", true))
                        .child(narya_ui::category("外观", false))
                        .child(narya_ui::category("网络", false))
                        .child(narya_ui::category("IPv6", false))
                        .child(narya_ui::category("内核", false))
                        .child(narya_ui::category("TUN", false))
                        .child(narya_ui::category("DNS", false))
                        .child(narya_ui::category("安全", false))
                        .child(narya_ui::category("通知", false))
                        .child(narya_ui::category("更新", false))
                        .child(narya_ui::category("高级", false)),
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
                        .children(snapshot.kernels.into_iter().map(|k| {
                            narya_ui::detail_field(
                                k.name,
                                if k.installed {
                                    "已安装"
                                } else {
                                    "未安装"
                                },
                            )
                        }))
                        .child(NaryaButton::primary("安装未实现").disabled(true)),
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
                Flex::new()
                    .row()
                    .gap_lg()
                    .child(NaryaButton::primary("浅色"))
                    .child(NaryaButton::ghost("深色"))
                    .child(NaryaButton::ghost("跟随系统")),
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

fn config_page() -> impl NaryaIntoElement {
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "当前配置",
                "Narya Default",
                "规则模式",
                "▤",
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "链式代理",
                "未启用",
                "可视化编排",
                "⇄",
                NaryaStatus::Warning,
            )
            .into_any_element(),
            NaryaMetric::card("YAML", "只读预览", "编辑器待接入", "{}", NaryaStatus::Info)
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
                    "⇄",
                    NaryaStatus::Info,
                ))
                .child(NaryaMetric::card(
                    "规则命中",
                    "12,840",
                    "DIRECT 62%",
                    "☷",
                    NaryaStatus::Success,
                )),
        ),
    ))
}

fn rules_page() -> impl NaryaIntoElement {
    NaryaPage::new()
        .row(narya_ui::metric_grid(vec![
            NaryaMetric::card("规则集", "8", "GeoSite / GeoIP", "☷", NaryaStatus::Info)
                .into_any_element(),
            NaryaMetric::card(
                "今日命中",
                "12,840",
                "DIRECT 62% · PROXY 38%",
                "◎",
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card(
                "最后更新",
                "2 天前",
                "可手动刷新",
                "↻",
                NaryaStatus::Warning,
            )
            .into_any_element(),
        ]))
        .row(NaryaCard::titled(
            "规则模拟器",
            Flex::new()
                .column()
                .gap_md()
                .child(narya_ui::detail_field("apple.com", "DIRECT"))
                .child(narya_ui::detail_field("github.com", "PROXY"))
                .child(narya_ui::detail_field("openai.com", "PROXY")),
        ))
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
            "↯",
            NaryaStatus::Info,
        )
        .into_any_element(),
        NaryaMetric::card(
            "DNS 查询",
            "就绪",
            "查看解析链路",
            "◎",
            NaryaStatus::Success,
        )
        .into_any_element(),
        NaryaMetric::card(
            "MTR Trace",
            "就绪",
            "追踪链路质量",
            "⇄",
            NaryaStatus::Warning,
        )
        .into_any_element(),
        NaryaMetric::card("端口检查", "就绪", "验证远端端口", "▣", NaryaStatus::Info)
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
