use crate::state::AppState;
use crate::ui_kit as narya_ui;
use crate::ui_kit::{
    NaryaButton, NaryaCard, NaryaMetric, NaryaPage, NaryaStatus, NavTarget, PageKind,
};
use crate::views::ActiveView;
use liora::components::{
    Drawer, Flex, Input, LocalizedText, NavigationMenu, NavigationMenuMode, Segmented,
    SegmentedOption, Select, SettingsGroup, SettingsItem, SettingsPage, Switch, Text,
};
use liora_icons_lucide::IconName;
use narya_ui::{
    px, App, Context, NaryaAppContext, NaryaEntity as Entity, NaryaFluentBuilder, NaryaIntoElement,
    NaryaRenderOnce, NaryaViewElement, Render, Window,
};

fn narya_text(content: impl Into<LocalizedText>) -> Text {
    Text::new(content)
}

pub struct AppShell {
    pub(super) active_view: ActiveView,
    pub(super) state: Entity<AppState>,
    settings: SettingsControls,
}

struct SettingsControls {
    category_menu: Entity<NavigationMenu>,
    autostart: Entity<Switch>,
    start_minimized: Entity<Switch>,
    close_to_tray: Entity<Switch>,
    restore_proxy: Entity<Switch>,
    auto_update: Entity<Switch>,
    appearance: Entity<Segmented>,
    update_channel: Entity<Select>,
}

impl SettingsControls {
    fn new(state: Entity<AppState>, cx: &mut App) -> Self {
        let category_state = state.clone();
        let category_menu = cx.new(|_| {
            NavigationMenu::new()
                .id("narya-settings-categories")
                .mode(NavigationMenuMode::Vertical)
                .default_active("narya-settings-category-0")
                .on_select(move |id, _, cx| {
                    let Some(index) = id.rsplit('-').next().and_then(|value| value.parse().ok())
                    else {
                        return;
                    };
                    category_state.update(cx, |state, cx| state.set_settings_category(index, cx));
                })
                .item(
                    "narya-settings-category-0",
                    "常规",
                    Some(IconName::Settings),
                )
                .item("narya-settings-category-1", "外观", Some(IconName::Palette))
                .item("narya-settings-category-2", "网络", Some(IconName::Network))
                .item("narya-settings-category-3", "IPv6", Some(IconName::Route))
                .item("narya-settings-category-4", "内核", Some(IconName::Cpu))
                .item("narya-settings-category-5", "TUN", Some(IconName::Shield))
                .item("narya-settings-category-6", "DNS", Some(IconName::Server))
                .item(
                    "narya-settings-category-7",
                    "安全",
                    Some(IconName::LockKeyhole),
                )
                .item("narya-settings-category-8", "通知", Some(IconName::Bell))
                .item(
                    "narya-settings-category-9",
                    "更新",
                    Some(IconName::RefreshCw),
                )
                .item(
                    "narya-settings-category-10",
                    "高级",
                    Some(IconName::SlidersHorizontal),
                )
        });
        let switch = |key: &'static str, value: bool, state: &Entity<AppState>, cx: &mut App| {
            let state = state.clone();
            cx.new(|cx| {
                Switch::new(value, cx)
                    .id(format!("narya-setting-{key}"))
                    .on_change(move |checked, _, cx| {
                        state.update(cx, |state, cx| state.set_setting_value(key, checked, cx));
                    })
            })
        };
        let autostart = switch("autostart", state.read(cx).setting_autostart, &state, cx);
        let start_minimized = switch(
            "start_minimized",
            state.read(cx).setting_start_minimized,
            &state,
            cx,
        );
        let close_to_tray = switch(
            "close_to_tray",
            state.read(cx).setting_close_to_tray,
            &state,
            cx,
        );
        let restore_proxy = switch(
            "restore_proxy",
            state.read(cx).setting_restore_proxy,
            &state,
            cx,
        );
        let auto_update = switch(
            "auto_update",
            state.read(cx).setting_auto_update,
            &state,
            cx,
        );
        let appearance_state = state.clone();
        let appearance_mode = state.read(cx).appearance_mode;
        let appearance = cx.new(|_| {
            Segmented::new(vec![
                SegmentedOption::new("浅色", "light"),
                SegmentedOption::new("深色", "dark"),
                SegmentedOption::new("跟随系统", "system"),
            ])
            .id("narya-settings-appearance")
            .value(match appearance_mode {
                1 => "dark",
                2 => "system",
                _ => "light",
            })
            .on_change(move |value, _, cx| {
                appearance_state.update(cx, |state, cx| {
                    state.set_appearance_mode(
                        match value.as_ref() {
                            "dark" => 1,
                            "system" => 2,
                            _ => 0,
                        },
                        cx,
                    )
                });
            })
        });
        let update_channel = cx.new(|cx| {
            Select::new(vec!["Stable", "Beta", "Nightly"], Some(0), cx)
                .id("narya-settings-update-channel")
        });
        Self {
            category_menu,
            autostart,
            start_minimized,
            close_to_tray,
            restore_proxy,
            auto_update,
            appearance,
            update_channel,
        }
    }

    fn sync(&self, _snapshot: &ShellSnapshot, _cx: &mut Context<AppShell>) {}
}

impl AppShell {
    pub fn open(cx: &mut App) {
        crate::ipc::ensure_daemon();
        let state = cx.new(|_| AppState::load_or_default());
        let settings = SettingsControls::new(state.clone(), cx);
        AppState::start_traffic_monitor(state.clone(), cx);
        AppState::fetch_kernel_status(state.clone(), cx);
        AppState::fetch_routing_status(state.clone(), cx);

        narya_ui::open_shell_window(cx, move |_, cx| {
            cx.new(|_| AppShell {
                active_view: ActiveView::Dashboard,
                state,
                settings,
            })
        });
    }
}

impl Render for AppShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl NaryaIntoElement {
        let view = self.active_view;
        let snapshot = ShellSnapshot::from_state(self.state.clone(), cx);
        self.settings.sync(&snapshot, cx);
        let active_category = format!("narya-settings-category-{}", snapshot.settings_category);
        self.settings.category_menu.update(cx, |menu, cx| {
            menu.set_active_index(active_category, cx);
        });
        let footer = narya_ui::FooterBar {
            kernel: if snapshot.kernel_healthy {
                format!("● {}", snapshot.active_kernel)
            } else {
                format!("○ {} 未运行", snapshot.active_kernel)
            },
            config: format!("{} 条规则", snapshot.rules.len()),
            subscriptions: format!("{} 个订阅", snapshot.subscriptions.len()),
        };
        let weak_shell = cx.entity().downgrade();
        let on_nav = move |target: NavTarget, cx: &mut App| {
            let _ = weak_shell.update(cx, |shell, cx| {
                shell.active_view = ActiveView::from(target);
                cx.notify();
            });
        };

        // Overlay portals are rendered from the window layer after the shell
        // builds its normal content tree.
        liora::core::render_active_drawer_in_window(_window, cx);

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
            header(view, &self.state, cx.entity().downgrade()),
            route_page(view, &self.state, snapshot, &self.settings),
            footer,
        )
    }
}

#[derive(Clone)]
struct ShellSnapshot {
    nodes: Vec<narya_core::Node>,
    subscriptions: Vec<narya_core::Subscription>,
    selected_subscription_id: Option<String>,
    subscription_draft_name: String,
    subscription_draft_url: String,
    subscription_error: Option<String>,
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
    kernel_operation: Option<String>,
    kernel_error: Option<String>,
    rule_set_draft_id: String,
    rule_set_draft_source: String,
    rule_set_draft_version: String,
    rule_set_draft_sha256: String,
    rule_set_draft_format: String,
    rule_set_draft_signature: String,
    rule_set_draft_public_key: String,
    rule_set_error: Option<String>,
    group_error: Option<String>,
    rule_editor_error: Option<String>,
    rule_io_path: String,
    rule_io_status: Option<String>,
    settings_category: usize,
    running: bool,
    active_node_id: Option<String>,
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
            selected_subscription_id: state.selected_subscription_id.clone(),
            subscription_draft_name: state.subscription_draft_name.clone(),
            subscription_draft_url: state.subscription_draft_url.clone(),
            subscription_error: state.subscription_error.clone(),
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
            kernel_operation: state.kernel_operation.clone(),
            kernel_error: state.kernel_error.clone(),
            rule_set_draft_id: state.rule_set_draft_id.clone(),
            rule_set_draft_source: state.rule_set_draft_source.clone(),
            rule_set_draft_version: state.rule_set_draft_version.clone(),
            rule_set_draft_sha256: state.rule_set_draft_sha256.clone(),
            rule_set_draft_format: state.rule_set_draft_format.clone(),
            rule_set_draft_signature: state.rule_set_draft_signature.clone(),
            rule_set_draft_public_key: state.rule_set_draft_public_key.clone(),
            rule_set_error: state.rule_set_error.clone(),
            group_error: state.group_error.clone(),
            rule_editor_error: state.rule_editor_error.clone(),
            rule_io_path: state.rule_io_path.clone(),
            rule_io_status: state.rule_io_status.clone(),
            settings_category: state.settings_category,
            running: state.kernel_running,
            active_node_id: state.active_node_id.clone(),
            active_node_name: active_node
                .map(|node| node.name.clone())
                .unwrap_or_else(|| "未连接".to_string()),
            active_latency: active_node.and_then(|node| node.latency).unwrap_or(0),
            download_speed: active_node.map(|node| node.download_speed).unwrap_or(0.0),
            upload_speed: active_node.map(|node| node.upload_speed).unwrap_or(0.0),
        }
    }

    fn measured_latency_values(nodes: &[narya_core::Node]) -> Vec<f64> {
        let values = nodes
            .iter()
            .filter_map(|node| node.latency.map(f64::from))
            .collect::<Vec<_>>();
        if values.is_empty() {
            latency_values()
        } else {
            values
        }
    }

    fn log_tone(level: &str) -> NaryaStatus {
        match level.to_ascii_uppercase().as_str() {
            "ERROR" | "FATAL" => NaryaStatus::Danger,
            "WARN" => NaryaStatus::Warning,
            _ => NaryaStatus::Info,
        }
    }
}

fn header(
    view: ActiveView,
    model: &Entity<AppState>,
    shell: gpui::WeakEntity<AppShell>,
) -> impl NaryaIntoElement {
    let page = PageKind::from(view);
    let model_for_connect = model.clone();
    let mut actions = vec![
        NaryaButton::icon_name(IconName::Fullscreen).into_any_element(),
        NaryaButton::icon_name(IconName::ClipboardList).into_any_element(),
        NaryaButton::icon_name(IconName::Settings).into_any_element(),
        NaryaButton::icon_name(IconName::EllipsisVertical).into_any_element(),
    ];
    match view {
        ActiveView::Config => {
            let import_model = model.clone();
            actions = vec![NaryaButton::primary("导入配置")
                .id("narya-config-import-menu")
                .on_click(move |_, window, cx| {
                    let shell = shell.clone();
                    let import_model = import_model.clone();
                    Drawer::new()
                        .id("narya-config-import-drawer")
                        .title("导入配置")
                        .width(px(440.0))
                        .content(move |_, _| {
                            Flex::new()
                                .column()
                                .gap_lg()
                                .child(narya_text("选择一种方式添加配置订阅").sm())
                                .child(narya_ui::detail_field(
                                    "远程订阅 URL",
                                    "HTTPS V2Ray / Clash / sing-box",
                                ))
                                .child(narya_ui::detail_field(
                                    "本地配置文件",
                                    "JSON、YAML 或 Base64 文本",
                                ))
                                .child(
                                    NaryaButton::primary("读取剪贴板并导入")
                                        .id("narya-config-import-clipboard")
                                        .on_click({
                                            let import_model = import_model.clone();
                                            move |_, _, app| {
                                                let Some(item) = app.read_from_clipboard() else {
                                                    return;
                                                };
                                                let Some(text) = item.text() else {
                                                    return;
                                                };
                                                import_model.update(app, |state, cx| {
                                                    state.set_subscription_draft_name(
                                                        "剪贴板配置".into(),
                                                        cx,
                                                    );
                                                    state.set_subscription_draft_url(
                                                        text.to_string(),
                                                        cx,
                                                    );
                                                });
                                                AppState::add_subscription(
                                                    import_model.clone(),
                                                    app,
                                                );
                                                Drawer::close(app);
                                            }
                                        }),
                                )
                                .child(
                                    narya_text("导入后会先解析和校验，成功后才加入配置列表").xs(),
                                )
                        })
                        .show(cx);
                    window.refresh();
                    let _ = shell.update(cx, |_, cx| cx.notify());
                })
                .into_any_element()];
        }
        ActiveView::Nodes => actions.insert(
            0,
            NaryaButton::primary("一键测速")
                .on_click(move |_, _, cx| AppState::test_all_latency(model_for_connect.clone(), cx))
                .into_any_element(),
        ),
        ActiveView::Subscriptions => {}
        _ => {}
    }
    narya_ui::HeaderBar::new(page, actions)
}

fn route_page(
    view: ActiveView,
    model: &Entity<AppState>,
    snapshot: ShellSnapshot,
    settings: &SettingsControls,
) -> impl NaryaIntoElement {
    match view {
        ActiveView::Dashboard => dashboard_page(model, snapshot).into_any_element(),
        ActiveView::Nodes => nodes_page(model, snapshot).into_any_element(),
        ActiveView::Subscriptions => subscriptions_page(model, snapshot).into_any_element(),
        ActiveView::Settings => settings_page(model, snapshot, settings).into_any_element(),
        ActiveView::Config => config_page(snapshot).into_any_element(),
        ActiveView::Connections => connections_page(snapshot).into_any_element(),
        ActiveView::Rules => rules_page(model, snapshot).into_any_element(),
        ActiveView::Logs => logs_page(snapshot).into_any_element(),
        ActiveView::Tools => tools_page().into_any_element(),
        ActiveView::About => about_page().into_any_element(),
    }
}

fn dashboard_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let model_for_toggle = model.clone();
    let model_for_system_proxy = model.clone();
    let model_for_tun = model.clone();
    let measured_nodes = snapshot
        .nodes
        .iter()
        .filter(|node| node.latency.is_some())
        .count();
    let latency_series = ShellSnapshot::measured_latency_values(&snapshot.nodes);
    let traffic_series = vec![
        0.0,
        f64::from(snapshot.download_speed),
        f64::from(snapshot.upload_speed),
    ];
    let node_total = snapshot.nodes.len();
    let protocol_pct = |names: &[&str]| {
        if node_total == 0 {
            return 0.0;
        }
        let count = snapshot
            .nodes
            .iter()
            .filter(|node| {
                names
                    .iter()
                    .any(|name| node.protocol.eq_ignore_ascii_case(name))
            })
            .count();
        (count as f32 / node_total as f32) * 100.0
    };
    let activity_rows = if snapshot.logs.is_empty() {
        vec![narya_ui::log_line(
            "--:--:--",
            if snapshot.running {
                "内核已连接，等待运行日志"
            } else {
                "Daemon 尚未推送运行日志"
            },
            NaryaStatus::Info,
        )
        .into_any_element()]
    } else {
        snapshot
            .logs
            .iter()
            .rev()
            .take(4)
            .map(|log| {
                narya_ui::log_line(
                    log.time.clone(),
                    log.content.clone(),
                    ShellSnapshot::log_tone(&log.level),
                )
                .into_any_element()
            })
            .collect()
    };
    let toggle_label = if snapshot.running {
        "断开连接"
    } else {
        "连接当前节点"
    };
    NaryaPage::new()
        .row(narya_ui::dashboard_top(
            narya_ui::hero_toggle_card_with_click(
                IconName::Monitor,
                "系统代理",
                "管理系统网络代理设置",
                snapshot.routing_active == narya_platform::ProxyMode::SystemProxy,
                "规则模式 ›",
                NaryaStatus::Info,
                Box::new(move |_, _, cx| {
                    toggle_routing_mode(
                        model_for_system_proxy.clone(),
                        narya_platform::ProxyMode::SystemProxy,
                        cx,
                    )
                }),
            ),
            narya_ui::hero_toggle_card_with_click(
                IconName::Network,
                "TUN 虚拟网卡",
                "拦截并代理所有网络流量（推荐）",
                snapshot.routing_active == narya_platform::ProxyMode::Tun,
                "智能路由 ›",
                NaryaStatus::Success,
                Box::new(move |_, _, cx| {
                    toggle_routing_mode(model_for_tun.clone(), narya_platform::ProxyMode::Tun, cx)
                }),
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
                narya_ui::soft_trend(latency_series, 212.0, narya_ui::SUCCESS),
                vec![
                    narya_ui::compact_metric(
                        "节点延迟",
                        if snapshot.active_latency == 0 {
                            "--".to_string()
                        } else {
                            format!("{} ms", snapshot.active_latency)
                        },
                        "当前节点",
                    )
                    .into_any_element(),
                    narya_ui::compact_metric(
                        "可用节点",
                        format!("{measured_nodes} / {node_total}"),
                        "已测 / 总数",
                    )
                    .into_any_element(),
                    narya_ui::compact_metric(
                        "内核健康",
                        if snapshot.kernel_healthy {
                            "正常"
                        } else {
                            "未连接"
                        },
                        snapshot.active_kernel.clone(),
                    )
                    .into_any_element(),
                    narya_ui::compact_metric(
                        "路由模式",
                        routing_mode_label(snapshot.routing_active),
                        "Daemon 确认状态",
                    )
                    .into_any_element(),
                ],
            ),
        ))
        .row(narya_ui::dashboard_bottom(
            narya_ui::dashboard_traffic_panel(
                vec![
                    narya_ui::compact_metric(
                        "实时下行",
                        format!("{:.2} MB/s", snapshot.download_speed),
                        "Daemon 统计",
                    )
                    .into_any_element(),
                    narya_ui::compact_metric(
                        "实时上行",
                        format!("{:.2} MB/s", snapshot.upload_speed),
                        "Daemon 统计",
                    )
                    .into_any_element(),
                ],
                narya_ui::soft_trend(traffic_series, 188.0, narya_ui::BRAND),
            ),
            narya_ui::titled_panel(
                "节点协议",
                Flex::new()
                    .column()
                    .gap_lg()
                    .child(narya_ui::ratio_row(
                        "Shadowsocks",
                        protocol_pct(&["ss", "shadowsocks"]),
                        NaryaStatus::Info,
                    ))
                    .child(narya_ui::ratio_row(
                        "VMess",
                        protocol_pct(&["vmess"]),
                        NaryaStatus::Success,
                    ))
                    .child(narya_ui::ratio_row(
                        "Trojan",
                        protocol_pct(&["trojan"]),
                        NaryaStatus::Warning,
                    ))
                    .child(narya_ui::ratio_row(
                        "Hysteria2",
                        protocol_pct(&["hysteria2", "hy2"]),
                        NaryaStatus::Danger,
                    ))
                    .child(narya_ui::detail_field("节点总数", node_total.to_string())),
            ),
            narya_ui::titled_panel(
                "活动日志",
                Flex::new()
                    .column()
                    .gap_md()
                    .children(activity_rows)
                    .child(NaryaButton::ghost(toggle_label).on_click(move |_, _, cx| {
                        AppState::toggle_proxy(model_for_toggle.clone(), cx)
                    }))
                    .when_some(snapshot.kernel_operation.clone(), |element, operation| {
                        element.child(narya_text(operation))
                    })
                    .when_some(snapshot.kernel_error.clone(), |element, error| {
                        element.child(narya_text(error))
                    }),
            ),
        ))
}

fn toggle_routing_mode(model: Entity<AppState>, mode: narya_platform::ProxyMode, cx: &mut App) {
    let (running, active_mode) = {
        let state = model.read(cx);
        (state.kernel_running, state.routing_active)
    };
    if !running {
        model.update(cx, |state, cx| state.set_routing_mode(mode, cx));
        AppState::set_proxy_running(model, cx, true);
    } else if active_mode == mode {
        AppState::set_proxy_running(model, cx, false);
    } else {
        AppState::switch_routing_mode(model, cx, mode);
    }
}

fn nodes_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let measured = snapshot
        .nodes
        .iter()
        .filter(|node| node.latency.is_some())
        .count();
    let failed = snapshot.nodes.len().saturating_sub(measured);
    let average = if measured == 0 {
        None
    } else {
        Some(
            snapshot
                .nodes
                .iter()
                .filter_map(|node| node.latency)
                .sum::<u32>()
                / u32::try_from(measured).unwrap_or(1),
        )
    };
    let fastest = snapshot
        .nodes
        .iter()
        .filter_map(|node| node.latency.map(|latency| (latency, node.name.clone())))
        .min_by_key(|(latency, _)| *latency);
    let active = snapshot
        .active_node_id
        .as_ref()
        .and_then(|id| snapshot.nodes.iter().find(|node| node.id == *id));
    let active_address = active
        .map(|node| node.details.address.clone())
        .unwrap_or_else(|| "--".into());
    let active_protocol = active
        .map(|node| node.protocol.clone())
        .unwrap_or_else(|| "--".into());
    let credential_status = active
        .map(|node| {
            if node.details.encryption.trim().is_empty()
                || node.details.encryption.eq_ignore_ascii_case("none")
            {
                "未配置"
            } else {
                "已配置"
            }
        })
        .unwrap_or("--");
    let active_udp = active
        .map(|node| {
            if node.details.udp {
                "已启用"
            } else {
                "未启用"
            }
        })
        .unwrap_or("--");
    let group_rows = if snapshot.groups.is_empty() {
        vec![narya_ui::detail_field("分流组", "尚未配置").into_any_element()]
    } else {
        snapshot
            .groups
            .iter()
            .map(|group| {
                narya_ui::detail_field(group.id.clone(), format!("{} 个成员", group.members.len()))
                    .into_any_element()
            })
            .collect()
    };
    let active_node_id = snapshot.active_node_id.clone();
    let node_cards = snapshot
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
                    active_node_id.as_deref() == Some(id.as_str()),
                ),
                Box::new(move |_, _, cx| AppState::connect_node(model.clone(), cx, id.clone())),
            )
            .into_any_element()
        })
        .collect();
    NaryaPage::new()
        .row(narya_ui::nodes_top_controls(vec![
            narya_ui::control_card(
                "当前策略组",
                snapshot
                    .groups
                    .first()
                    .map(|group| group.id.clone())
                    .unwrap_or_else(|| "尚未配置".into()),
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
                routing_mode_label(snapshot.routing_mode),
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
        .row(narya_ui::nodes_main(
            narya_ui::titled_panel("策略组", Flex::new().column().gap_md().children(group_rows)),
            narya_ui::titled_panel("节点列表", narya_ui::node_grid(node_cards)),
            narya_ui::titled_panel(
                "测速概览",
                Flex::new()
                    .column()
                    .gap_lg()
                    .child(NaryaMetric::card(
                        "平均延迟",
                        average
                            .map(|latency| format!("{latency} ms"))
                            .unwrap_or_else(|| "--".into()),
                        fastest
                            .map(|(latency, name)| format!("最快：{name} · {latency} ms"))
                            .unwrap_or_else(|| "尚未测速".into()),
                        IconName::Gauge,
                        NaryaStatus::Info,
                    ))
                    .child(narya_ui::detail_field(
                        "已测节点",
                        format!("{measured} / {}", snapshot.nodes.len()),
                    ))
                    .child(narya_ui::detail_field("未连接", failed.to_string())),
            ),
        ))
        .row(narya_ui::nodes_bottom(
            narya_ui::chart_card(
                "节点延迟",
                ShellSnapshot::measured_latency_values(&snapshot.nodes),
                128.0,
                narya_ui::SUCCESS,
            ),
            NaryaCard::titled(
                format!("节点详情（{}）", snapshot.active_node_name),
                Flex::new()
                    .column()
                    .gap_md()
                    .child(narya_ui::detail_field("地址", active_address))
                    .child(narya_ui::detail_field("协议", active_protocol))
                    .child(narya_ui::detail_field("节点凭据", credential_status))
                    .child(narya_ui::detail_field("UDP", active_udp)),
            ),
        ))
}

fn subscriptions_page(model: &Entity<AppState>, snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let selected = snapshot
        .selected_subscription_id
        .as_ref()
        .and_then(|id| snapshot.subscriptions.iter().find(|item| item.id == *id))
        .or_else(|| snapshot.subscriptions.first());
    let selected_name = selected
        .map(|item| item.name.clone())
        .unwrap_or_else(|| "尚未添加".into());
    let selected_status = selected
        .map(|item| item.status.clone())
        .unwrap_or_else(|| "等待订阅".into());
    let selected_format = selected
        .and_then(|item| item.format.clone())
        .unwrap_or_else(|| "未知".into());
    let selected_updated = selected
        .map(|item| item.update_time.clone())
        .unwrap_or_else(|| "--".into());
    let total_nodes = snapshot
        .subscriptions
        .iter()
        .map(|item| usize::try_from(item.node_count).unwrap_or(0))
        .sum::<usize>();
    let selected_id = selected.map(|item| item.id.clone());
    let model_for_name = model.clone();
    let model_for_url = model.clone();
    let model_for_add = model.clone();
    let subscription_rows = if snapshot.subscriptions.is_empty() {
        vec![narya_ui::detail_field("订阅源", "请在上方添加 HTTPS 订阅").into_any_element()]
    } else {
        snapshot
            .subscriptions
            .iter()
            .map(|sub| {
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
                    selected_id.as_deref() == Some(sub.id.as_str()),
                )
                .into_any_element()
            })
            .collect()
    };
    let refresh_button = if let Some(subscription_id) = selected_id {
        let model = model.clone();
        NaryaButton::ghost("刷新当前订阅")
            .on_click(move |_, _, cx| {
                AppState::refresh_subscription(model.clone(), cx, subscription_id.clone())
            })
            .into_any_element()
    } else {
        NaryaButton::ghost("刷新当前订阅")
            .disabled(true)
            .into_any_element()
    };
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "当前订阅",
                selected_name,
                &selected_status,
                IconName::ClipboardList,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "节点总数",
                total_nodes.to_string(),
                format!("{} 个订阅源", snapshot.subscriptions.len()),
                IconName::CircleGauge,
                NaryaStatus::Success,
            )
            .into_any_element(),
            NaryaMetric::card(
                "订阅格式",
                &selected_format,
                "最近一次成功解析",
                IconName::ChartNoAxesCombined,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "更新时间",
                &selected_updated,
                "由实际刷新结果更新",
                IconName::SquareStack,
                NaryaStatus::Warning,
            )
            .into_any_element(),
        ]))
        .row(NaryaCard::titled(
            "添加订阅",
            Flex::new().w_full().min_w_0().child(SubscriptionDraftForm {
                model_for_name,
                model_for_url,
                model_for_add,
                name: snapshot.subscription_draft_name.clone(),
                url: snapshot.subscription_draft_url.clone(),
                error: snapshot.subscription_error.clone(),
            }),
        ))
        .row(NaryaCard::titled(
            "订阅源列表",
            Flex::new()
                .column()
                .gap_md()
                .min_w_0()
                .children(subscription_rows),
        ))
        .row(NaryaCard::titled(
            "更新状态",
            Flex::new()
                .column()
                .gap_lg()
                .min_w_0()
                .child(narya_ui::detail_field("状态", selected_status))
                .child(narya_ui::detail_field("格式", selected_format))
                .child(narya_ui::detail_field("更新时间", selected_updated))
                .child(refresh_button),
        ))
}

struct SubscriptionDraftForm {
    model_for_name: Entity<AppState>,
    model_for_url: Entity<AppState>,
    model_for_add: Entity<AppState>,
    name: String,
    url: String,
    error: Option<String>,
}

impl NaryaRenderOnce for SubscriptionDraftForm {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let model_for_name = self.model_for_name;
        let model_for_url = self.model_for_url;
        let model_for_add = self.model_for_add;
        Flex::new()
            .column()
            .gap_md()
            .child(
                Flex::new()
                    .row()
                    .wrap()
                    .gap_md()
                    .min_w_0()
                    .child(cx.new(|cx| {
                        Input::new(self.name, cx)
                            .id("narya-subscription-name")
                            .placeholder("订阅名称")
                            .width(px(220.0))
                            .on_change(move |value, input_cx| {
                                model_for_name.update(input_cx, |state, state_cx| {
                                    state.set_subscription_draft_name(value.to_string(), state_cx)
                                });
                            })
                    }))
                    .child(cx.new(|cx| {
                        Input::new(self.url, cx)
                            .id("narya-subscription-url")
                            .placeholder("https://example.com/subscription")
                            .width(px(320.0))
                            .on_change(move |value, input_cx| {
                                model_for_url.update(input_cx, |state, state_cx| {
                                    state.set_subscription_draft_url(value.to_string(), state_cx)
                                });
                            })
                    }))
                    .child(
                        NaryaButton::primary("添加并刷新").on_click(move |_, _, app| {
                            AppState::add_subscription(model_for_add.clone(), app)
                        }),
                    ),
            )
            .when_some(self.error, |element, error| {
                element.child(narya_text(error))
            })
    }
}

impl NaryaIntoElement for SubscriptionDraftForm {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
}

fn settings_page(
    model: &Entity<AppState>,
    snapshot: ShellSnapshot,
    settings: &SettingsControls,
) -> impl NaryaIntoElement {
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
    let category_title = [
        "常规设置",
        "外观设置",
        "网络设置",
        "IPv6 设置",
        "内核设置",
        "TUN 设置",
        "DNS 设置",
        "安全设置",
        "通知设置",
        "更新设置",
        "高级设置",
    ]
    .get(snapshot.settings_category)
    .copied()
    .unwrap_or("常规设置");
    let center_page = match snapshot.settings_category {
        1 => SettingsPage::new("外观设置")
            .description("调整应用主题和界面密度")
            .max_width(px(720.0))
            .group(
                SettingsGroup::new("主题")
                    .item(
                        SettingsItem::new("主题模式")
                            .description("选择浅色、深色或跟随系统")
                            .icon(IconName::Palette)
                            .control(settings.appearance.clone())
                            .primary(),
                    )
                    .item(
                        SettingsItem::new("界面语言")
                            .description("当前界面使用简体中文，英文文本遵循系统字体回退")
                            .icon(IconName::Languages)
                            .extra(narya_text("简体中文 · LXGW WenKai / Consolas")),
                    ),
            ),
        9 => SettingsPage::new("更新设置")
            .description("控制版本检查和发布通道")
            .max_width(px(720.0))
            .group(
                SettingsGroup::new("更新")
                    .item(
                        SettingsItem::new("自动检查更新")
                            .description("启动后定期检查新的稳定版本")
                            .icon(IconName::RefreshCw)
                            .control(settings.auto_update.clone()),
                    )
                    .item(
                        SettingsItem::new("更新通道")
                            .description("选择接收稳定版、测试版或 nightly 版本")
                            .icon(IconName::GitBranch)
                            .control(settings.update_channel.clone()),
                    ),
            ),
        _ => SettingsPage::new(category_title)
            .description("Narya 运行时设置")
            .max_width(px(720.0))
            .group(
                SettingsGroup::new("启动行为")
                    .item(
                        SettingsItem::new("开机自启")
                            .description("登录系统后自动启动 Narya")
                            .icon(IconName::Power)
                            .control(settings.autostart.clone()),
                    )
                    .item(
                        SettingsItem::new("启动后最小化")
                            .description("启动完成后隐藏主窗口")
                            .icon(IconName::Minimize2)
                            .control(settings.start_minimized.clone()),
                    )
                    .item(
                        SettingsItem::new("关闭到托盘")
                            .description("关闭窗口时继续在系统托盘运行")
                            .icon(IconName::PanelTopClose)
                            .control(settings.close_to_tray.clone()),
                    )
                    .item(
                        SettingsItem::new("启动时恢复代理")
                            .description("恢复上次确认的系统代理模式")
                            .icon(IconName::RotateCcw)
                            .control(settings.restore_proxy.clone()),
                    ),
            )
            .group(
                SettingsGroup::new("本地服务")
                    .item(
                        SettingsItem::new("语言")
                            .description("当前界面语言")
                            .icon(IconName::Languages)
                            .extra(narya_text("简体中文")),
                    )
                    .item(
                        SettingsItem::new("监听端口")
                            .description("HTTP 7890 · SOCKS 7891 · API 9090")
                            .icon(IconName::Plug),
                    ),
            ),
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
        .row(
            Flex::new()
                .row()
                .gap_lg()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .child(
                    NaryaCard::titled("设置分类", settings.category_menu.clone())
                        .width(px(196.0))
                        .no_shrink(),
                )
                .child(
                    Flex::new()
                        .flex_1()
                        .min_h_0()
                        .min_w_0()
                        .overflow_y_scroll()
                        .child(center_page),
                )
                .child(
                    Flex::new()
                        .column()
                        .width_px(420.0)
                        .h_full()
                        .flex_none()
                        .min_h_0()
                        .min_w_0()
                        .overflow_y_scroll()
                        .child(
                            SettingsPage::new("内核管理")
                                .description("内核仅安装在 Narya 私有目录，不修改系统 PATH")
                                .max_width(px(420.0))
                                .group(
                                    SettingsGroup::new("内核列表")
                                        .description("版本、运行状态与操作集中在单行展示")
                                        .items(
                                            kernel_infos
                                                .into_iter()
                                                .map(|kernel| {
                                                    let name = kernel.name.clone();
                                                    let active = name == snapshot.active_kernel
                                                        && kernel.running;
                                                    let busy = matches!(
                                                        kernel.state.as_str(),
                                                        "installing"
                                                            | "upgrading"
                                                            | "uninstalling"
                                                            | "starting"
                                                            | "stopping"
                                                    );
                                                    let actions = if kernel.installed {
                                                        let upgrade_model = model.clone();
                                                        let upgrade_name = name.clone();
                                                        let uninstall_model = model.clone();
                                                        let uninstall_name = name.clone();
                                                        let switch_model = model.clone();
                                                        let switch_name = name.clone();
                                                        Flex::new()
                                                    .row()
                                                    .wrap()
                                                    .gap_sm()
                                                    .child(
                                                        NaryaButton::ghost("升级")
                                                            .id(format!("narya-kernel-{name}-upgrade"))
                                                            .small()
                                                            .disabled(active || busy)
                                                            .on_click(move |_, _, cx| {
                                                                AppState::install_kernel_named(
                                                                    upgrade_model.clone(),
                                                                    cx,
                                                                    upgrade_name.clone(),
                                                                )
                                                            }),
                                                    )
                                                    .child(
                                                        NaryaButton::ghost("卸载")
                                                            .id(format!("narya-kernel-{name}-uninstall"))
                                                            .danger()
                                                            .small()
                                                            .disabled(active || busy)
                                                            .on_click(move |_, _, cx| {
                                                                AppState::uninstall_kernel(
                                                                    uninstall_model.clone(),
                                                                    cx,
                                                                    uninstall_name.clone(),
                                                                )
                                                            }),
                                                    )
                                                    .child(
                                                        NaryaButton::primary(if active {
                                                            "当前运行"
                                                        } else {
                                                            "切换并启动"
                                                        })
                                                        .id(format!("narya-kernel-{name}-start"))
                                                        .small()
                                                        .disabled(active || busy)
                                                        .on_click(move |_, _, cx| {
                                                            AppState::select_kernel_and_start(
                                                                switch_model.clone(),
                                                                cx,
                                                                switch_name.clone(),
                                                            )
                                                        }),
                                                    )
                                                    } else {
                                                        let install_model = model.clone();
                                                        let install_name = name.clone();
                                                        Flex::new().row().child(
                                                            NaryaButton::primary("安装")
                                                                .id(format!(
                                                                    "narya-kernel-{name}-install"
                                                                ))
                                                                .small()
                                                                .disabled(busy)
                                                                .on_click(move |_, _, cx| {
                                                                    AppState::install_kernel_named(
                                                                        install_model.clone(),
                                                                        cx,
                                                                        install_name.clone(),
                                                                    )
                                                                }),
                                                        )
                                                    };
                                                    let health = if kernel.healthy {
                                                        "健康"
                                                    } else if kernel.running {
                                                        "异常"
                                                    } else {
                                                        "未运行"
                                                    };
                                                    let version = kernel
                                                        .version
                                                        .clone()
                                                        .unwrap_or_else(|| "无版本".into());
                                                    let mut item = SettingsItem::new(name)
                                                        .description(format!(
                                                            "{version} · {} · {health}",
                                                            kernel_state_label(&kernel.state)
                                                        ))
                                                        .icon(IconName::Cpu)
                                                        .control(actions)
                                                        .compact();
                                                    if active {
                                                        item = item.primary();
                                                    }
                                                    if let Some(failure) = kernel.failure {
                                                        item = item.extra(
                                                            narya_text(format!(
                                                                "最近错误：{failure}"
                                                            ))
                                                            .xs()
                                                            .wrap(),
                                                        );
                                                    }
                                                    item
                                                })
                                                .collect(),
                                        ),
                                )
                                .group(
                                    SettingsGroup::new("操作状态").footer(
                                        Flex::new()
                                            .column()
                                            .gap_sm()
                                            .when_some(
                                                snapshot.kernel_operation,
                                                |element, operation| {
                                                    element.child(narya_text(operation).sm())
                                                },
                                            )
                                            .when_some(snapshot.kernel_error, |element, error| {
                                                element.child(
                                                    narya_text(format!("错误：{error}"))
                                                        .sm()
                                                        .wrap(),
                                                )
                                            }),
                                    ),
                                ),
                        ),
                ),
        )
}

fn kernel_state_label(state: &str) -> &'static str {
    match state {
        "installed" => "已安装，未运行",
        "installing" => "正在安装",
        "upgrading" => "正在升级",
        "starting" => "正在启动",
        "running" => "运行中",
        "stopping" => "正在停止",
        "uninstalling" => "正在卸载",
        "failed" => "操作失败",
        _ => "未安装",
    }
}

fn config_page(snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let subscriptions = snapshot
        .subscriptions
        .iter()
        .map(|subscription| {
            narya_ui::subscription_item(
                subscription.name.clone(),
                subscription.url.clone(),
                subscription.node_count,
                if subscription.traffic_total > 0.0 {
                    (subscription.traffic_used / subscription.traffic_total) as f32
                } else {
                    0.0
                },
                snapshot.selected_subscription_id.as_deref() == Some(subscription.id.as_str()),
            )
            .into_any_element()
        })
        .collect::<Vec<_>>();
    NaryaPage::new()
        .row(narya_ui::page_row(vec![
            NaryaMetric::card(
                "当前配置",
                format!("{} 条规则", snapshot.rules.len()),
                routing_mode_label(snapshot.routing_mode),
                IconName::ClipboardList,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "配置订阅",
                snapshot.subscriptions.len().to_string(),
                "已添加配置源",
                IconName::Braces,
                NaryaStatus::Info,
            )
            .into_any_element(),
            NaryaMetric::card(
                "分流组",
                snapshot.groups.len().to_string(),
                format!("{} 个外部规则集", snapshot.rule_sets.len()),
                IconName::Braces,
                NaryaStatus::Info,
            )
            .into_any_element(),
        ]))
        .row(NaryaCard::titled(
            "配置工作台",
            Flex::new()
                .column()
                .gap_lg()
                .min_w_0()
                .child(narya_ui::detail_field(
                    "配置说明",
                    "配置页只管理订阅来源；节点与规则在对应页面维护",
                ))
                .child(narya_text("点击右上角“导入配置”选择远程、本地或剪贴板导入方式").xs()),
        ))
        .row(NaryaCard::titled(
            "配置订阅列表",
            Flex::new()
                .column()
                .gap_md()
                .min_w_0()
                .children(subscriptions),
        ))
}

fn connections_page(snapshot: ShellSnapshot) -> impl NaryaIntoElement {
    let connection_rows = if snapshot.running {
        vec![narya_ui::detail_field(
            snapshot.active_node_name.clone(),
            routing_mode_label(snapshot.routing_active),
        )
        .into_any_element()]
    } else {
        vec![narya_ui::detail_field("连接状态", "当前没有活动连接").into_any_element()]
    };
    NaryaPage::new().row(narya_ui::page_columns(
        NaryaCard::titled(
            "活动连接",
            Flex::new().column().gap_md().children(connection_rows),
        ),
        NaryaCard::titled(
            "连接摘要",
            Flex::new()
                .column()
                .gap_lg()
                .child(NaryaMetric::card(
                    "运行状态",
                    if snapshot.running {
                        "已连接"
                    } else {
                        "未连接"
                    },
                    snapshot.active_kernel,
                    IconName::ArrowLeftRight,
                    NaryaStatus::Info,
                ))
                .child(NaryaMetric::card(
                    "路由模式",
                    routing_mode_label(snapshot.routing_active),
                    if snapshot.kernel_healthy {
                        "内核健康"
                    } else {
                        "等待内核健康确认"
                    },
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
                .id("narya-rules-add")
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
                            .column()
                            .gap_lg()
                            .min_w_0()
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
                            .child(
                                Flex::new()
                                    .row()
                                    .wrap()
                                    .gap_md()
                                    .child(narya_ui::narya_tag(
                                        rule_action_summary(&rule.action),
                                        tone,
                                    ))
                                    .child(
                                        NaryaButton::ghost("删除")
                                            .id(format!("narya-rule-delete-{rule_id}"))
                                            .on_click(move |_, _, cx| {
                                                AppState::remove_rule(
                                                    delete_model.clone(),
                                                    cx,
                                                    rule_id.clone(),
                                                )
                                            }),
                                    ),
                            ),
                    )
                    .into_any_element()
                })),
        ))
        .when_some(snapshot.rule_editor_error.clone(), |element, error| {
            element.row(NaryaCard::titled("规则校验", narya_text(error)))
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
                        .child(narya_text(group.id.clone()))
                        .child(narya_text(format!(
                            "{} · {}",
                            group_strategy_label(group.strategy),
                            group.members.join(", ")
                        )))
                        .child(
                            NaryaButton::ghost("删除")
                                .id(format!("narya-group-delete-{group_id}"))
                                .disabled(!removable)
                                .on_click(move |_, _, cx| {
                                    AppState::remove_group(
                                        remove_model.clone(),
                                        cx,
                                        group_id.clone(),
                                    )
                                }),
                        )
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
                    element.child(narya_text(error))
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
                        .child(narya_text(source.id))
                        .child(narya_text(format!(
                            "v{} · SHA-256 {}",
                            source.version,
                            source.sha256.chars().take(12).collect::<String>()
                        )))
                        .child(narya_text(source.source))
                        .child(RuleSetToggle {
                            model: model.clone(),
                            rule_set_id: source_id.clone(),
                            enabled: source.enabled,
                        })
                        .child(
                            NaryaButton::ghost("删除")
                                .id(format!("narya-ruleset-delete-{source_id}"))
                                .on_click(move |_, _, cx| {
                                    AppState::remove_rule_set(
                                        remove_model.clone(),
                                        cx,
                                        source_id.clone(),
                                    )
                                }),
                        )
                        .into_any_element()
                }))
                .child(RuleSetForm {
                    model: model.clone(),
                    id: snapshot.rule_set_draft_id,
                    source: snapshot.rule_set_draft_source,
                    version: snapshot.rule_set_draft_version,
                    sha256: snapshot.rule_set_draft_sha256,
                    format: snapshot.rule_set_draft_format,
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
                .child(narya_text(format!(
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

struct RuleSetToggle {
    model: Entity<AppState>,
    rule_set_id: String,
    enabled: bool,
}

impl NaryaRenderOnce for RuleSetToggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl NaryaIntoElement {
        let model = self.model;
        let rule_set_id = self.rule_set_id;
        cx.new(|cx| {
            Switch::new(self.enabled, cx)
                .id(format!("narya-ruleset-toggle-{}", rule_set_id))
                .on_change(move |enabled, _, app| {
                    AppState::set_rule_set_enabled(model.clone(), app, rule_set_id.clone(), enabled)
                })
        })
    }
}

impl NaryaIntoElement for RuleSetToggle {
    type Element = NaryaViewElement<Self>;

    fn into_element(self) -> Self::Element {
        NaryaViewElement::new(self)
    }
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
                    .id("narya-rule-io-path")
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
                element.child(narya_text(status))
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
                    .wrap()
                    .gap_sm()
                    .min_w_0()
                    .align_center()
                    .child(narya_text(if index == 0 { "条件" } else { "AND" }))
                    .child(cx.new(|cx| {
                        Select::new(labels, Some(selected), cx)
                            .id(format!("narya-rule-condition-kind-{rule_id}-{index}"))
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
                            .id(format!("narya-rule-condition-value-{rule_id}-{index}"))
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
                    .id(format!("narya-routing-group-strategy-{}", self.group.id))
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
                    .id(format!("narya-routing-group-members-{}", self.group.id))
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
                    .id(format!("narya-routing-group-url-{}", self.group.id))
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
                .id(format!("narya-routing-group-interval-{}", self.group.id))
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
    format: String,
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
        let model_format = self.model.clone();
        let model_add = self.model.clone();
        let model_signature = self.model.clone();
        let model_public_key = self.model.clone();
        Flex::new()
            .column()
            .gap_sm()
            .child(
                Flex::new()
                    .row()
                    .wrap()
                    .gap_md()
                    .min_w_0()
                    .child(cx.new(|cx| {
                        let options = vec![
                            "sing_box_binary".to_string(),
                            "domain".to_string(),
                            "ip_cidr".to_string(),
                            "classical".to_string(),
                        ];
                        let selected = options
                            .iter()
                            .position(|value| value == &self.format)
                            .unwrap_or(0);
                        Select::new(options, Some(selected), cx)
                            .id("narya-ruleset-format")
                            .width(px(170.0))
                            .on_change(move |index, _, app| {
                                let value = match index {
                                    1 => "domain",
                                    2 => "ip_cidr",
                                    3 => "classical",
                                    _ => "sing_box_binary",
                                };
                                model_format.update(app, |state, state_cx| {
                                    state.set_rule_set_draft_format(value.into(), state_cx)
                                });
                            })
                    }))
                    .child(cx.new(|cx| {
                        Input::new(self.id, cx)
                            .id("narya-ruleset-id")
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
                            .id("narya-ruleset-version")
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
                            .id("narya-ruleset-source")
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
                            .id("narya-ruleset-sha256")
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
                            .id("narya-ruleset-signature")
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
                            .id("narya-ruleset-public-key")
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
            .when_some(self.error, |element, error| {
                element.child(narya_text(error))
            })
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
                .id("narya-rules-search")
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
                .id("narya-rule-action-filter")
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
        let control_id = format!("narya-rule-target-{rule_id}");
        cx.new(|cx| {
            Select::new(labels, Some(selected_index), cx)
                .id(control_id)
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
        let control_id = format!("narya-rule-priority-{rule_id}");
        cx.new(|cx| {
            Input::new(self.priority.to_string(), cx)
                .id(control_id)
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
            "未接入",
            "请使用节点页真实测速",
            IconName::Zap,
            NaryaStatus::Info,
        )
        .into_any_element(),
        NaryaMetric::card(
            "DNS 查询",
            "未接入",
            "等待安全诊断实现",
            IconName::CircleGauge,
            NaryaStatus::Success,
        )
        .into_any_element(),
        NaryaMetric::card(
            "MTR Trace",
            "未接入",
            "等待安全诊断实现",
            IconName::ArrowLeftRight,
            NaryaStatus::Warning,
        )
        .into_any_element(),
        NaryaMetric::card(
            "端口检查",
            "未接入",
            "请使用节点页真实测速",
            IconName::SquareStack,
            NaryaStatus::Info,
        )
        .into_any_element(),
    ]))
}

fn about_page() -> impl NaryaIntoElement {
    NaryaPage::new().row(NaryaCard::titled(
        "Narya",
        narya_text(format!(
            "Narya {} · GPUI + Liora 本地代理控制端",
            env!("CARGO_PKG_VERSION")
        ))
        .selectable(false),
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
    vec![0.0]
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
