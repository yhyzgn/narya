use gpui::{div, prelude::*, px, rgb, AnyElement, Entity, IntoElement, ParentElement, Rgba};
pub use gpui::{
    App, AppContext as NaryaAppContext, Context, Entity as NaryaEntity,
    IntoElement as NaryaIntoElement, Render, Window,
};
use liora::components::{
    Button, Card, Flex, Image, LineChart, Progress, SignalMeter, Space, Sparkline, Tag, Text,
};
use liora_icons::Icon;
use liora_icons_lucide::IconName;

const SIDEBAR_W: f32 = 256.0;
const HEADER_H: f32 = 120.0;
const FOOTER_H: f32 = 68.0;
const CONTENT_X_PAD: f32 = 36.0;
const CONTENT_BOTTOM_PAD: f32 = 16.0;
const GAP: f32 = 24.0;

pub const APP_BG: u32 = 0xF8FBFF;
pub const SURFACE: u32 = 0xFFFFFF;
pub const BORDER: u32 = 0xDDE6F5;
pub const TEXT: u32 = 0x10203D;
pub const MUTED: u32 = 0x637392;
pub const SOFT: u32 = 0xF1F5FF;
pub const BRAND: u32 = 0x2F66FF;
pub const VIOLET: u32 = 0x7C4DFF;
pub const SUCCESS: u32 = 0x10B981;
pub const WARNING: u32 = 0xF59E0B;
pub const DANGER: u32 = 0xFF4D2E;
pub const INFO: u32 = 0x0EA5E9;

pub fn color(hex: u32) -> Rgba {
    rgb(hex)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NaryaStatus {
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NavTarget {
    Dashboard,
    Nodes,
    Config,
    Subscriptions,
    Connections,
    Rules,
    Logs,
    Tools,
    Settings,
}

impl NavTarget {
    fn icon(self) -> IconName {
        match self {
            NavTarget::Dashboard => IconName::House,
            NavTarget::Nodes => IconName::Database,
            NavTarget::Config => IconName::ClipboardList,
            NavTarget::Subscriptions => IconName::ClipboardCheck,
            NavTarget::Connections => IconName::SlidersHorizontal,
            NavTarget::Rules => IconName::ListFilter,
            NavTarget::Logs => IconName::List,
            NavTarget::Tools => IconName::BriefcaseBusiness,
            NavTarget::Settings => IconName::Settings,
        }
    }
}

type NavHandler = std::rc::Rc<dyn Fn(NavTarget, &mut gpui::App)>;
pub type ClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PageKind {
    Dashboard,
    Nodes,
    Config,
    Subscriptions,
    Connections,
    Rules,
    Logs,
    Tools,
    Settings,
    About,
}

pub struct ShellFrame {
    sidebar: AnyElement,
    header: AnyElement,
    content: AnyElement,
    footer: AnyElement,
}

impl ShellFrame {
    pub fn new(
        sidebar: impl IntoElement,
        header: impl IntoElement,
        content: impl IntoElement,
        footer: impl IntoElement,
    ) -> Self {
        Self {
            sidebar: sidebar.into_any_element(),
            header: header.into_any_element(),
            content: content.into_any_element(),
            footer: footer.into_any_element(),
        }
    }
}

impl IntoElement for ShellFrame {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .flex()
            .size_full()
            .bg(color(APP_BG))
            .text_color(color(TEXT))
            .child(self.sidebar)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .h_full()
                    .min_h_0()
                    .child(self.header)
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .overflow_hidden()
                            .px(px(CONTENT_X_PAD))
                            .pb(px(CONTENT_BOTTOM_PAD))
                            .child(self.content),
                    )
                    .child(self.footer),
            )
    }
}

pub struct Sidebar {
    active: NavTarget,
    running: bool,
    node: String,
    latency: u32,
    down: f32,
    up: f32,
    on_nav: NavHandler,
}

impl Sidebar {
    pub fn new(
        active: NavTarget,
        running: bool,
        node: impl Into<String>,
        latency: u32,
        down: f32,
        up: f32,
        on_nav: impl Fn(NavTarget, &mut gpui::App) + 'static,
    ) -> Self {
        Self {
            active,
            running,
            node: node.into(),
            latency,
            down,
            up,
            on_nav: std::rc::Rc::new(on_nav),
        }
    }
}

impl IntoElement for Sidebar {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let nav_items = [
            ("仪表盘", NavTarget::Dashboard),
            ("节点", NavTarget::Nodes),
            ("配置", NavTarget::Config),
            ("订阅", NavTarget::Subscriptions),
            ("连接", NavTarget::Connections),
            ("规则", NavTarget::Rules),
            ("日志", NavTarget::Logs),
            ("工具箱", NavTarget::Tools),
            ("设置", NavTarget::Settings),
        ];
        let on_nav = self.on_nav.clone();

        div()
            .w(px(SIDEBAR_W))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .justify_between()
            .bg(color(0xFBFDFF))
            .border_r_1()
            .border_color(color(BORDER))
            .child(
                div().flex().flex_col().child(brand_block()).child(
                    div().flex().flex_col().gap_2().px(px(22.0)).children(
                        nav_items.into_iter().map(|(label, target)| {
                            let active = self.active == target;
                            let on_nav = on_nav.clone();
                            nav_item(target.icon(), label, active)
                                .on_click(move |_, _, cx| on_nav(target, cx))
                        }),
                    ),
                ),
            )
            .child(sidebar_status(
                self.running,
                self.node,
                self.latency,
                self.down,
                self.up,
            ))
    }
}

fn brand_block() -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap_4()
        .h(px(146.0))
        .px(px(36.0))
        .child(
            Image::local("ui/icons/narya-logo-v2.png")
                .width(px(62.0))
                .height(px(62.0))
                .shadow(false)
                .bordered(false),
        )
        .child(
            Flex::new()
                .column()
                .gap_px(2.0)
                .child(
                    Text::new("Narya")
                        .size(px(26.0))
                        .bold()
                        .text_color(color(TEXT).into())
                        .selectable(false),
                )
                .child(
                    Text::new("v1.0.0")
                        .sm()
                        .text_color(color(MUTED).into())
                        .selectable(false),
                ),
        )
}

fn nav_item(icon: IconName, label: &'static str, active: bool) -> Button {
    let fg = color(if active { BRAND } else { TEXT });
    let bg = color(if active { 0xEEF0FF } else { 0xFBFDFF });
    Button::new(label)
        .icon_start(Icon::new(icon).size(px(20.0)).color(fg.into()))
        .custom_color(bg.into(), fg.into())
        .rounded_md()
        .border(false)
        .large()
}

fn sidebar_status(running: bool, node: String, latency: u32, down: f32, up: f32) -> gpui::Div {
    div()
        .px(px(22.0))
        .pb(px(26.0))
        .flex()
        .flex_col()
        .gap_5()
        .child(NaryaCard::plain(
            Flex::new()
                .column()
                .gap_md()
                .child(
                    Space::new().gap_sm().child(status_dot(running)).child(
                        Text::new(if running { "已连接" } else { "未连接" })
                            .sm()
                            .bold()
                            .selectable(false),
                    ),
                )
                .child(
                    Text::new("当前节点")
                        .xs()
                        .text_color(color(MUTED).into())
                        .selectable(false),
                )
                .child(
                    Space::new()
                        .gap_sm()
                        .child(flag("✤"))
                        .child(
                            Text::new(node)
                                .sm()
                                .text_color(color(TEXT).into())
                                .selectable(false),
                        )
                        .child(narya_tag(format!("{} ms", latency), NaryaStatus::Info)),
                )
                .child(key_value("代理模式", "规则模式 ›"))
                .child(
                    Space::new()
                        .gap_lg()
                        .child(
                            Text::new(format!("↓ {:.2} MB/s", down))
                                .xs()
                                .text_color(color(SUCCESS).into())
                                .selectable(false),
                        )
                        .child(
                            Text::new(format!("↑ {:.2} MB/s", up))
                                .xs()
                                .text_color(color(VIOLET).into())
                                .selectable(false),
                        ),
                )
                .child(
                    Sparkline::new([6.0, 8.0, 7.0, 12.0, 9.0, 15.0, 10.0, 13.0, 8.0, 11.0])
                        .height(px(52.0))
                        .color(color(BRAND).into())
                        .area_fill(true),
                ),
        ))
        .child(sidebar_footer_icons())
}

pub struct HeaderBar {
    title: &'static str,
    subtitle: &'static str,
    actions: Vec<AnyElement>,
}

impl HeaderBar {
    pub fn new(page: PageKind, actions: Vec<AnyElement>) -> Self {
        let (title, subtitle) = match page {
            PageKind::Dashboard => ("仪表盘", "一切运行正常  ●"),
            PageKind::Nodes => ("节点", "选择最快的出口节点，支持自动测速与策略分组"),
            PageKind::Config => ("配置", "管理代理配置、链式代理与 YAML 编辑"),
            PageKind::Subscriptions => ("订阅", "管理远程订阅源、流量信息与自动更新策略"),
            PageKind::Connections => ("连接", "查看活跃连接、目标地址与出口链路"),
            PageKind::Rules => ("规则", "规则分流、模拟器与命中统计"),
            PageKind::Logs => ("日志", "内核日志、诊断导出与错误追踪"),
            PageKind::Tools => ("工具箱", "Ping、DNS、MTR、端口检查与报告导出"),
            PageKind::Settings => ("设置", "调整应用、内核、网络、IPv6、安全与更新偏好"),
            PageKind::About => ("关于", "Narya GPUI + Liora Native"),
        };
        Self {
            title,
            subtitle,
            actions,
        }
    }
}

impl IntoElement for HeaderBar {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .h(px(HEADER_H))
            .flex_none()
            .flex()
            .items_start()
            .justify_between()
            .px(px(CONTENT_X_PAD))
            .pt(px(24.0))
            .child(
                Flex::new()
                    .column()
                    .gap_px(6.0)
                    .child(
                        Text::new(self.title)
                            .size(px(30.0))
                            .bold()
                            .text_color(color(TEXT).into())
                            .selectable(false),
                    )
                    .child(
                        Text::new(self.subtitle)
                            .sm()
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    ),
            )
            .child(
                Flex::new()
                    .column()
                    .align_end()
                    .gap_xl()
                    .child(window_controls())
                    .child(Space::new().gap_md().children(self.actions)),
            )
    }
}

fn window_controls() -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap_5()
        .pr(px(4.0))
        .child(
            Icon::new(IconName::Minus)
                .size(px(16.0))
                .color(color(TEXT).into()),
        )
        .child(
            Icon::new(IconName::Square)
                .size(px(14.0))
                .color(color(TEXT).into()),
        )
        .child(
            Icon::new(IconName::X)
                .size(px(18.0))
                .color(color(TEXT).into()),
        )
}

pub struct FooterBar;

impl IntoElement for FooterBar {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .h(px(FOOTER_H))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px(px(CONTENT_X_PAD))
            .bg(color(SURFACE))
            .border_t_1()
            .border_color(color(BORDER))
            .child(
                Space::new()
                    .gap_xl()
                    .child(status_line("内核", "● sing-box"))
                    .child(status_line("配置", "▤ Narya Default"))
                    .child(status_line("订阅", "▣ 机场 A · 128 节点")),
            )
            .child(
                Space::new()
                    .gap_xl()
                    .child(
                        Text::new("检查更新")
                            .sm()
                            .text_color(color(BRAND).into())
                            .selectable(false),
                    )
                    .child(
                        Text::new("1.0.0")
                            .sm()
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    ),
            )
    }
}

pub struct NaryaPage {
    rows: Vec<AnyElement>,
}

impl NaryaPage {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    pub fn row(mut self, row: impl IntoElement) -> Self {
        self.rows.push(row.into_any_element());
        self
    }
}

impl Default for NaryaPage {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for NaryaPage {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .flex()
            .flex_col()
            .gap(px(GAP))
            .size_full()
            .overflow_hidden()
            .children(self.rows)
    }
}

pub struct NaryaMetric;

impl NaryaMetric {
    pub fn card(
        title: &'static str,
        value: impl Into<String>,
        caption: impl Into<String>,
        icon: &'static str,
        status: NaryaStatus,
    ) -> Card {
        NaryaCard::metric(title, value, caption, icon, status)
    }
}

pub struct NaryaCard;
impl NaryaCard {
    pub fn plain(body: impl IntoElement) -> Card {
        Card::new(body).no_shadow()
    }

    pub fn titled(title: impl Into<gpui::SharedString>, body: impl IntoElement) -> Card {
        Card::new(body).title(title).no_shadow()
    }

    pub fn metric(
        title: &'static str,
        value: impl Into<String>,
        caption: impl Into<String>,
        icon: &'static str,
        status: NaryaStatus,
    ) -> Card {
        Self::plain(
            Flex::new()
                .row()
                .align_center()
                .gap_lg()
                .child(metric_icon(icon, status))
                .child(
                    Flex::new()
                        .column()
                        .gap_px(3.0)
                        .child(
                            Text::new(title)
                                .xs()
                                .text_color(color(MUTED).into())
                                .selectable(false),
                        )
                        .child(
                            Text::new(value.into())
                                .size(px(24.0))
                                .text_color(color(TEXT).into())
                                .selectable(false),
                        )
                        .child(
                            Text::new(caption.into())
                                .xs()
                                .text_color(color(MUTED).into())
                                .selectable(false),
                        ),
                ),
        )
    }
}

pub struct NaryaButton;
impl NaryaButton {
    pub fn primary(label: impl Into<gpui::SharedString>) -> Button {
        Button::new(label).primary().rounded_md()
    }
    pub fn ghost(label: impl Into<gpui::SharedString>) -> Button {
        Button::new(label).tertiary().rounded_md()
    }
    pub fn icon(label: impl Into<gpui::SharedString>) -> Button {
        Button::new(label).tertiary().rounded_md().small()
    }

    pub fn icon_name(icon: IconName) -> Button {
        Button::new("")
            .icon_only(icon)
            .tertiary()
            .background(false)
            .border(false)
            .rounded_md()
    }
}

pub fn page_row(children: Vec<AnyElement>) -> impl IntoElement {
    Flex::new().row().gap_px(GAP).w_full().children(children)
}

pub fn dashboard_top(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_px(28.0)
        .w_full()
        .height_px(164.0)
        .child(Flex::new().width_px(548.0).flex_none().child(left))
        .child(Flex::new().flex_1().child(right))
}

pub fn dashboard_middle(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_px(28.0)
        .height_px(310.0)
        .child(Flex::new().width_px(488.0).flex_none().child(left))
        .child(Flex::new().flex_1().child(right))
}

pub fn dashboard_bottom(
    a: impl IntoElement,
    b: impl IntoElement,
    c: impl IntoElement,
) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_px(28.0)
        .height_px(284.0)
        .child(Flex::new().width_px(488.0).flex_none().child(a))
        .child(Flex::new().width_px(306.0).flex_none().child(b))
        .child(Flex::new().flex_1().child(c))
}

pub fn nodes_main(
    strategy: impl IntoElement,
    list: impl IntoElement,
    overview: impl IntoElement,
) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_md()
        .height_px(420.0)
        .child(Flex::new().width_px(280.0).flex_none().child(strategy))
        .child(Flex::new().flex_1().min_h_0().child(list))
        .child(Flex::new().width_px(276.0).flex_none().child(overview))
}

pub fn nodes_bottom(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_lg()
        .height_px(188.0)
        .child(Flex::new().flex_1().child(left))
        .child(Flex::new().width_px(604.0).flex_none().child(right))
}

pub fn node_grid(items: Vec<AnyElement>) -> impl IntoElement {
    Flex::new().row().wrap().gap_md().children(
        items
            .into_iter()
            .map(|item| Flex::new().width_px(296.0).child(item)),
    )
}

pub fn page_columns(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_lg()
        .flex_1()
        .min_h_0()
        .child(Flex::new().flex_1().min_h_0().child(left))
        .child(Flex::new().width_px(366.0).flex_none().child(right))
}

pub fn toolbar(children: Vec<AnyElement>) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_md()
        .w_full()
        .height_px(38.0)
        .children(children)
}

pub fn grid_two(items: Vec<AnyElement>) -> impl IntoElement {
    Flex::new().row().wrap().gap_lg().children(
        items
            .into_iter()
            .map(|item| Flex::new().width_px(304.0).child(item)),
    )
}

pub fn design_card(body: impl IntoElement) -> impl IntoElement {
    div()
        .size_full()
        .rounded(px(12.0))
        .border_1()
        .border_color(color(BORDER))
        .bg(color(SURFACE))
        .overflow_hidden()
        .child(body)
}

pub fn titled_panel(title: &'static str, body: impl IntoElement) -> impl IntoElement {
    design_card(
        Flex::new()
            .column()
            .size_full()
            .padding_px(20.0)
            .gap_md()
            .child(
                Text::new(title)
                    .size(px(17.0))
                    .bold()
                    .text_color(color(TEXT).into())
                    .selectable(false),
            )
            .child(Flex::new().flex_1().min_h_0().child(body)),
    )
}

pub fn panel_header(title: &'static str, action: &'static str) -> impl IntoElement {
    Flex::new()
        .row()
        .align_center()
        .justify_between()
        .child(
            Text::new(title)
                .size(px(17.0))
                .bold()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
        .child(
            Text::new(action)
                .sm()
                .text_color(color(BRAND).into())
                .selectable(false),
        )
}

pub fn dashboard_quick_panel(items: Vec<AnyElement>) -> impl IntoElement {
    titled_panel("快速连接", Flex::new().column().gap_sm().children(items))
}

pub fn dashboard_network_panel(
    chart: impl IntoElement,
    metrics: Vec<AnyElement>,
) -> impl IntoElement {
    titled_panel(
        "网络概览",
        Flex::new()
            .row()
            .gap_lg()
            .child(Flex::new().flex_1().min_h_0().child(chart))
            .child(
                Flex::new()
                    .width_px(300.0)
                    .flex_none()
                    .child(metric_quad(metrics)),
            ),
    )
}

pub fn dashboard_traffic_panel(
    stats: Vec<AnyElement>,
    chart: impl IntoElement,
) -> impl IntoElement {
    titled_panel(
        "流量使用",
        Flex::new()
            .row()
            .gap_lg()
            .child(
                Flex::new()
                    .width_px(126.0)
                    .flex_none()
                    .child(Flex::new().column().gap_lg().children(stats)),
            )
            .child(Flex::new().flex_1().child(chart)),
    )
}

pub fn metric_quad(items: Vec<AnyElement>) -> impl IntoElement {
    Flex::new().row().wrap().gap_lg().children(
        items
            .into_iter()
            .map(|item| Flex::new().width_px(126.0).child(item)),
    )
}

pub fn compact_metric(
    title: &'static str,
    value: impl Into<String>,
    caption: impl Into<String>,
) -> impl IntoElement {
    Flex::new()
        .column()
        .gap_px(5.0)
        .child(
            Text::new(title)
                .xs()
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            Text::new(value.into())
                .size(px(28.0))
                .text_color(color(TEXT).into())
                .selectable(false),
        )
        .child(
            Text::new(caption.into())
                .xs()
                .text_color(color(MUTED).into())
                .selectable(false),
        )
}

pub fn sidebar_footer_icons() -> impl IntoElement {
    Flex::new()
        .row()
        .justify_between()
        .align_center()
        .padding_x_px(24.0)
        .height_px(44.0)
        .child(
            Icon::new(IconName::BadgeQuestionMark)
                .size(px(27.0))
                .color(color(TEXT).into()),
        )
        .child(
            Icon::new(IconName::Moon)
                .size(px(27.0))
                .color(color(TEXT).into()),
        )
        .child(
            Icon::new(IconName::Bell)
                .size(px(27.0))
                .color(color(TEXT).into()),
        )
}

pub fn nodes_top_controls(items: Vec<AnyElement>) -> impl IntoElement {
    Flex::new().row().gap_lg().height_px(80.0).children(items)
}

pub fn control_card(
    title: &'static str,
    value: impl Into<String>,
    icon: IconName,
    width: f32,
    tone: NaryaStatus,
) -> impl IntoElement {
    div().w(px(width)).child(design_card(
        Flex::new()
            .row()
            .align_center()
            .justify_between()
            .padding_x_px(18.0)
            .padding_y_px(14.0)
            .size_full()
            .child(
                Space::new()
                    .gap_md()
                    .child(
                        Icon::new(icon)
                            .size(px(24.0))
                            .color(status_color(tone).into()),
                    )
                    .child(
                        Flex::new()
                            .column()
                            .gap_sm()
                            .child(
                                Text::new(title)
                                    .xs()
                                    .bold()
                                    .text_color(color(TEXT).into())
                                    .selectable(false),
                            )
                            .child(
                                Text::new(value.into())
                                    .sm()
                                    .text_color(color(TEXT).into())
                                    .selectable(false),
                            ),
                    ),
            )
            .child(
                Icon::new(IconName::ChevronDown)
                    .size(px(18.0))
                    .color(color(MUTED).into()),
            ),
    ))
}

pub fn gradient_action(label: &'static str, icon: IconName) -> Button {
    Button::new(label)
        .icon_start(Icon::new(icon).size(px(24.0)).color(color(SURFACE).into()))
        .gradient(color(0x5B8CFF).into(), color(0x8B5CF6).into())
        .rounded_md()
        .large()
}

pub fn hero_toggle_card(
    icon: IconName,
    title: &'static str,
    desc: &'static str,
    enabled: bool,
    mode: &'static str,
    tone: NaryaStatus,
) -> impl IntoElement {
    design_card(
        Flex::new()
            .column()
            .justify_between()
            .size_full()
            .padding_px(28.0)
            .child(
                Flex::new()
                    .row()
                    .align_center()
                    .justify_between()
                    .child(
                        Space::new().gap_lg().child(hero_icon(icon, tone)).child(
                            Flex::new()
                                .column()
                                .gap_sm()
                                .child(
                                    Text::new(title)
                                        .size(px(22.0))
                                        .bold()
                                        .text_color(color(TEXT).into())
                                        .selectable(false),
                                )
                                .child(
                                    Text::new(desc)
                                        .sm()
                                        .text_color(color(MUTED).into())
                                        .selectable(false),
                                ),
                        ),
                    )
                    .child(toggle_pill(enabled)),
            )
            .child(
                Flex::new()
                    .row()
                    .justify_between()
                    .align_center()
                    .child(
                        Space::new().gap_sm().child(status_dot(enabled)).child(
                            Text::new(if enabled { "已启用" } else { "未启用" })
                                .sm()
                                .text_color(color(if enabled { SUCCESS } else { MUTED }).into())
                                .selectable(false),
                        ),
                    )
                    .child(
                        Text::new(mode)
                            .sm()
                            .text_color(color(TEXT).into())
                            .selectable(false),
                    ),
            ),
    )
}

pub fn quick_node(
    name: impl Into<String>,
    protocol: impl Into<String>,
    latency: u32,
    tone: NaryaStatus,
) -> impl IntoElement {
    let name = name.into();
    Flex::new()
        .row()
        .align_center()
        .justify_between()
        .height_px(54.0)
        .padding_x_px(10.0)
        .border()
        .border_color(color(BORDER).into())
        .rounded_md()
        .child(
            Space::new().gap_md().child(flag_for_name(&name)).child(
                Flex::new()
                    .column()
                    .child(
                        Text::new(name)
                            .sm()
                            .text_color(color(TEXT).into())
                            .selectable(false),
                    )
                    .child(
                        Text::new(protocol.into())
                            .xs()
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    ),
            ),
        )
        .child(narya_tag(format!("{} ms", latency), tone))
}

pub struct NodeCardData {
    pub name: String,
    pub protocol: String,
    pub latency: u32,
    pub load: u8,
    pub down: f32,
    pub up: f32,
    pub active: bool,
}

impl NodeCardData {
    pub fn new(
        name: impl Into<String>,
        protocol: impl Into<String>,
        latency: u32,
        load: u8,
        down: f32,
        up: f32,
        active: bool,
    ) -> Self {
        Self {
            name: name.into(),
            protocol: protocol.into(),
            latency,
            load,
            down,
            up,
            active,
        }
    }
}

pub fn node_card(data: NodeCardData, on_connect: ClickHandler) -> impl IntoElement {
    NaryaCard::plain(
        Flex::new()
            .column()
            .gap_md()
            .child(
                Flex::new()
                    .row()
                    .align_center()
                    .justify_between()
                    .child(
                        Space::new()
                            .gap_md()
                            .child(
                                Text::new(if data.active { "◉" } else { "○" })
                                    .text_color(
                                        color(if data.active { BRAND } else { MUTED }).into(),
                                    )
                                    .selectable(false),
                            )
                            .child(flag_for_name(&data.name))
                            .child(
                                Flex::new()
                                    .column()
                                    .child(
                                        Text::new(data.name)
                                            .bold()
                                            .text_color(color(TEXT).into())
                                            .selectable(false),
                                    )
                                    .child(
                                        Text::new(data.protocol)
                                            .xs()
                                            .text_color(color(MUTED).into())
                                            .selectable(false),
                                    ),
                            ),
                    )
                    .child(NaryaButton::ghost("连接").small().on_click(on_connect)),
            )
            .child(
                Flex::new()
                    .row()
                    .align_center()
                    .justify_between()
                    .child(narya_tag(
                        format!("{} ms", data.latency),
                        if data.latency < 90 {
                            NaryaStatus::Success
                        } else if data.latency < 140 {
                            NaryaStatus::Warning
                        } else {
                            NaryaStatus::Danger
                        },
                    ))
                    .child(
                        SignalMeter::new(signal_level(data.latency))
                            .height(px(20.0))
                            .active_color(
                                color(if data.latency < 120 { SUCCESS } else { DANGER }).into(),
                            ),
                    ),
            )
            .child(
                Space::new()
                    .gap_lg()
                    .child(
                        Text::new(format!("● {}%", data.load))
                            .xs()
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    )
                    .child(
                        Text::new(format!("↓ {:.1} MB/s", data.down))
                            .xs()
                            .text_color(color(BRAND).into())
                            .selectable(false),
                    )
                    .child(
                        Text::new(format!("↑ {:.1} MB/s", data.up))
                            .xs()
                            .text_color(color(VIOLET).into())
                            .selectable(false),
                    ),
            )
            .child(
                Sparkline::new([4.0, 5.0, 4.8, 6.0, 5.2, 5.8, 4.9, 5.5, 5.3, 5.7])
                    .height(px(24.0))
                    .color(color(SUCCESS).into()),
            ),
    )
}

pub fn subscription_item(
    name: impl Into<String>,
    url: impl Into<String>,
    nodes: u32,
    usage: f32,
    active: bool,
) -> impl IntoElement {
    NaryaCard::plain(
        Flex::new()
            .row()
            .align_center()
            .justify_between()
            .gap_md()
            .child(
                Space::new()
                    .gap_md()
                    .child(metric_icon(
                        "✈",
                        if active {
                            NaryaStatus::Info
                        } else {
                            NaryaStatus::Success
                        },
                    ))
                    .child(
                        Flex::new()
                            .column()
                            .gap_px(4.0)
                            .child(
                                Text::new(name.into())
                                    .bold()
                                    .text_color(color(TEXT).into())
                                    .selectable(false),
                            )
                            .child(
                                Text::new(url.into())
                                    .xs()
                                    .text_color(color(MUTED).into())
                                    .selectable(false),
                            )
                            .child(
                                Text::new(format!("{} 节点    更新：刚刚", nodes))
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
                    .width_px(100.0)
                    .child(
                        Text::new(format!("流量 {:.0}%", usage))
                            .xs()
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    )
                    .child(Progress::new(usage).show_text(false).stroke_width(px(6.0))),
            ),
    )
}

pub fn detail_field(label: impl Into<String>, value: impl Into<String>) -> impl IntoElement {
    Flex::new()
        .row()
        .justify_between()
        .child(
            Text::new(label.into())
                .sm()
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            Text::new(value.into())
                .sm()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
}

pub fn metric_grid(items: Vec<AnyElement>) -> impl IntoElement {
    Flex::new().row().gap_lg().children(
        items
            .into_iter()
            .map(|item| Flex::new().flex_1().child(item)),
    )
}

pub fn trend_chart(values: Vec<f64>, height: f32, color_hex: u32) -> impl IntoElement {
    let points = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| liora::components::ChartPoint::new(format!("{}", index + 1), value));
    LineChart::new([
        liora::components::ChartSeries::new("趋势", points).color(color(color_hex).into())
    ])
    .height(px(height))
    .show_legend(false)
    .show_tooltip(false)
    .show_value_labels(false)
    .max_axis_labels(6)
    .point_markers(false)
    .stroke_width(px(2.2))
}

pub fn chart_card(
    title: &'static str,
    values: Vec<f64>,
    height: f32,
    color_hex: u32,
) -> impl IntoElement {
    NaryaCard::titled(title, trend_chart(values, height, color_hex))
}

pub fn ratio_row(label: &'static str, pct: f32, tone: NaryaStatus) -> impl IntoElement {
    Flex::new()
        .row()
        .align_center()
        .gap_md()
        .child(
            Text::new(label)
                .sm()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
        .child(
            Flex::new()
                .flex_1()
                .child(Progress::new(pct).show_text(false).stroke_width(px(6.0))),
        )
        .child(
            Text::new(format!("{:.1}%", pct))
                .sm()
                .text_color(status_color(tone).into())
                .selectable(false),
        )
}

pub fn log_line(
    time: impl Into<String>,
    message: impl Into<String>,
    tone: NaryaStatus,
) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_lg()
        .align_center()
        .child(
            Text::new("●")
                .text_color(status_color(tone).into())
                .selectable(false),
        )
        .child(
            Text::new(time.into())
                .sm()
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            Text::new(message.into())
                .sm()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
}

pub fn setting_row(label: &'static str, enabled: bool) -> impl IntoElement {
    Flex::new()
        .row()
        .justify_between()
        .align_center()
        .child(
            Text::new(label)
                .sm()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
        .child(toggle_pill(enabled))
}

pub fn category(label: &'static str, active: bool) -> impl IntoElement {
    Flex::new()
        .row()
        .align_center()
        .gap_md()
        .height_px(42.0)
        .padding_x_px(12.0)
        .bg(color(if active { 0xEEF0FF } else { SURFACE }).into())
        .border()
        .border_color(color(if active { BRAND } else { BORDER }).into())
        .rounded_md()
        .child(
            Text::new("⚙")
                .text_color(color(if active { BRAND } else { MUTED }).into())
                .selectable(false),
        )
        .child(
            Text::new(label)
                .sm()
                .text_color(color(if active { BRAND } else { TEXT }).into())
                .selectable(false),
        )
}

pub fn narya_tag(label: impl Into<gpui::SharedString>, status: NaryaStatus) -> Tag {
    let tag = Tag::new(label).small().round(true);
    match status {
        NaryaStatus::Info => tag.info(),
        NaryaStatus::Success => tag.success(),
        NaryaStatus::Warning => tag.warning(),
        NaryaStatus::Danger => tag.danger(),
    }
}

pub fn status_dot(on: bool) -> impl IntoElement {
    Text::new(if on { "●" } else { "○" })
        .text_color(color(if on { SUCCESS } else { MUTED }).into())
        .selectable(false)
}

fn status_line(label: &'static str, value: &'static str) -> impl IntoElement {
    Space::new()
        .gap_sm()
        .child(
            Text::new(label)
                .sm()
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            Text::new(value)
                .sm()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
}

fn key_value(label: &'static str, value: &'static str) -> impl IntoElement {
    Flex::new()
        .row()
        .justify_between()
        .child(
            Text::new(label)
                .xs()
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            Text::new(value)
                .xs()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
}

fn flag(symbol: &'static str) -> impl IntoElement {
    Text::new(symbol)
        .size(px(22.0))
        .text_color(color(DANGER).into())
        .selectable(false)
}

fn hero_icon(icon: IconName, tone: NaryaStatus) -> impl IntoElement {
    div()
        .size(px(56.0))
        .rounded(px(12.0))
        .bg(status_color(tone))
        .flex()
        .items_center()
        .justify_center()
        .child(Icon::new(icon).size(px(27.0)).color(color(SURFACE).into()))
}

fn flag_for_name(name: &str) -> impl IntoElement {
    let emoji = if name.contains("香港") || name.contains("HK") {
        "🇭🇰"
    } else if name.contains("日本") || name.contains("JP") {
        "🇯🇵"
    } else if name.contains("美国") || name.contains("US") {
        "🇺🇸"
    } else if name.contains("新加坡") || name.contains("SG") {
        "🇸🇬"
    } else if name.contains("台湾") || name.contains("TW") {
        "🇹🇼"
    } else if name.contains("德国") || name.contains("DE") {
        "🇩🇪"
    } else if name.contains("英国") || name.contains("UK") {
        "🇬🇧"
    } else {
        "🌐"
    };
    Text::new(emoji).size(px(28.0)).selectable(false)
}

fn metric_icon(icon: &'static str, tone: NaryaStatus) -> impl IntoElement {
    div()
        .size(px(56.0))
        .rounded(px(12.0))
        .bg(status_soft_color(tone))
        .flex()
        .items_center()
        .justify_center()
        .child(
            Text::new(icon)
                .size(px(24.0))
                .text_color(status_color(tone).into())
                .selectable(false),
        )
}

fn toggle_pill(enabled: bool) -> impl IntoElement {
    div()
        .w(px(68.0))
        .h(px(36.0))
        .rounded(px(18.0))
        .bg(color(if enabled { BRAND } else { 0xCBD5E1 }))
        .flex()
        .items_center()
        .justify_end()
        .when(!enabled, |el| el.justify_start())
        .p_0p5()
        .child(div().size(px(30.0)).rounded(px(999.0)).bg(color(SURFACE)))
}

fn status_color(status: NaryaStatus) -> Rgba {
    color(match status {
        NaryaStatus::Info => BRAND,
        NaryaStatus::Success => SUCCESS,
        NaryaStatus::Warning => WARNING,
        NaryaStatus::Danger => DANGER,
    })
}

fn status_soft_color(status: NaryaStatus) -> Rgba {
    color(match status {
        NaryaStatus::Info => 0xEEF4FF,
        NaryaStatus::Success => 0xEAFBF3,
        NaryaStatus::Warning => 0xFFF7E6,
        NaryaStatus::Danger => 0xFFF0ED,
    })
}

fn signal_level(latency: u32) -> usize {
    if latency < 70 {
        4
    } else if latency < 110 {
        3
    } else if latency < 150 {
        2
    } else {
        1
    }
}

pub fn entity_window_options(cx: &mut gpui::App) -> gpui::WindowOptions {
    let size = gpui::size(px(1536.0), px(1024.0));
    let bounds = gpui::Bounds::centered(None, size, cx);
    gpui::WindowOptions {
        window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
        window_min_size: Some(size),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Narya".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        window_decorations: Some(gpui::WindowDecorations::Client),
        app_id: Some("narya".into()),
        ..Default::default()
    }
}

pub fn open_shell_window<T: gpui::Render + 'static>(
    cx: &mut gpui::App,
    build: impl FnOnce(&mut gpui::Window, &mut gpui::App) -> Entity<T> + 'static,
) {
    let options = entity_window_options(cx);
    cx.open_window(options, build)
        .expect("failed to open Narya main window");
}
