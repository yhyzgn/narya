pub use gpui::prelude::FluentBuilder as NaryaFluentBuilder;
pub use gpui::px;
use gpui::{
    div, prelude::*, rgb, AnyElement, Entity, Font, FontFallbacks, FontFeatures, FontStyle,
    FontWeight, IntoElement, ParentElement, Rgba, ViewElement,
};
pub use gpui::{
    App, AppContext as NaryaAppContext, Context, Entity as NaryaEntity,
    IntoElement as NaryaIntoElement, Render, RenderOnce as NaryaRenderOnce,
    ViewElement as NaryaViewElement, Window,
};
use liora::components::{
    Button, Card, Flex, Input, LineChart, LocalizedText, PortalLayer, Progress, Segmented,
    SegmentedOption, Select, SignalMeter, Space, Sparkline, Statistic, Switch, Tag, Text,
};
use liora_icons::Icon;
use liora_icons_lucide::IconName;

const SIDEBAR_W: f32 = 224.0;
const HEADER_H: f32 = 84.0;
const FOOTER_H: f32 = 48.0;
const CONTENT_X_PAD: f32 = 20.0;
const CONTENT_BOTTOM_PAD: f32 = 16.0;
const GAP: f32 = 14.0;

const FS_DISPLAY: f32 = 24.0;
const FS_BRAND: f32 = 22.0;
const FS_CARD_TITLE: f32 = 17.0;
const FS_BODY: f32 = 14.0;
const FS_SMALL: f32 = 13.0;
const FS_CAPTION: f32 = 12.0;
const FS_NUMBER: f32 = 23.0;

fn narya_text(content: impl Into<LocalizedText>) -> Text {
    Text::new(content)
}

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

    fn id(self) -> &'static str {
        match self {
            NavTarget::Dashboard => "dashboard",
            NavTarget::Nodes => "nodes",
            NavTarget::Config => "config",
            NavTarget::Subscriptions => "subscriptions",
            NavTarget::Connections => "connections",
            NavTarget::Rules => "rules",
            NavTarget::Logs => "logs",
            NavTarget::Tools => "tools",
            NavTarget::Settings => "settings",
        }
    }
}

type NavHandler = std::rc::Rc<dyn Fn(NavTarget, &mut gpui::App)>;
pub type ClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App)>;
type BoolChangeHandler = Box<dyn Fn(bool, &mut gpui::Window, &mut gpui::App)>;
type IndexChangeHandler = Box<dyn Fn(usize, &mut gpui::Window, &mut gpui::App)>;
type ValueChangeHandler = Box<dyn Fn(gpui::SharedString, &mut gpui::Window, &mut gpui::App)>;

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
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

impl gpui::RenderOnce for ShellFrame {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .size_full()
            .relative()
            .bg(color(APP_BG))
            .text_color(color(TEXT))
            .font(Font {
                family: "Consolas".into(),
                features: FontFeatures::default(),
                fallbacks: Some(FontFallbacks::from_fonts(vec!["LXGW WenKai".to_string()])),
                weight: FontWeight::default(),
                style: FontStyle::default(),
            })
            .child(
                Flex::new()
                    .row()
                    .size_full()
                    .min_w_0()
                    .child(self.sidebar)
                    .child(
                        Flex::new()
                            .column()
                            .flex_1()
                            .h_full()
                            .min_h_0()
                            .min_w_0()
                            .child(self.header)
                            .child(
                                Flex::new().flex_1().min_h_0().min_w_0().child(
                                    div()
                                        .w_full()
                                        .h_full()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .px(px(CONTENT_X_PAD))
                                        .pb(px(CONTENT_BOTTOM_PAD))
                                        .child(self.content),
                                ),
                            )
                            .child(self.footer),
                    ),
            )
            .child(PortalLayer)
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
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

impl gpui::RenderOnce for Sidebar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
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
            .bg(color(0xFBFDFF))
            .border_r_1()
            .border_color(color(BORDER))
            .child(
                Flex::new()
                    .column()
                    .h_full()
                    .justify_between()
                    .child(
                        Flex::new()
                            .column()
                            .child(brand_block())
                            .child(sidebar_menu(nav_items, self.active, on_nav)),
                    )
                    .child(sidebar_status(
                        self.running,
                        self.node,
                        self.latency,
                        self.down,
                        self.up,
                    )),
            )
    }
}

fn brand_block() -> impl IntoElement {
    div().h(px(112.0)).px(px(28.0)).child(
        Flex::new()
            .row()
            .h_full()
            .align_center()
            .gap_lg()
            .child(
                Icon::new(IconName::Shield)
                    .size(px(32.0))
                    .color(color(BRAND).into()),
            )
            .child(
                Flex::new()
                    .column()
                    .gap_px(2.0)
                    .child(
                        narya_text("Narya")
                            .size(px(FS_BRAND))
                            .bold()
                            .text_color(color(TEXT).into())
                            .selectable(false),
                    )
                    .child(
                        narya_text(concat!("v", env!("CARGO_PKG_VERSION")))
                            .size(px(FS_SMALL))
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    ),
            ),
    )
}

fn sidebar_menu(
    nav_items: [(&'static str, NavTarget); 9],
    active: NavTarget,
    on_nav: NavHandler,
) -> SidebarMenu {
    SidebarMenu {
        nav_items,
        active,
        on_nav,
    }
}

struct SidebarMenu {
    nav_items: [(&'static str, NavTarget); 9],
    active: NavTarget,
    on_nav: NavHandler,
}

impl gpui::RenderOnce for SidebarMenu {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let on_nav = self.on_nav.clone();
        Flex::new()
            .column()
            .padding_x_px(22.0)
            .children(self.nav_items.into_iter().map(|(label, target)| {
                let active = target == self.active;
                let on_nav = on_nav.clone();
                div()
                    .id(format!("narya-sidebar-{}", target.id()))
                    .cursor_pointer()
                    .block_mouse_except_scroll()
                    .w_full()
                    .h(px(44.0))
                    .px(px(16.0))
                    .rounded(px(8.0))
                    // Paint the inactive state explicitly. An unpainted GPUI
                    // layer can expose the compositor's black transparent
                    // surface on Linux, which turns the whole row into a
                    // black block.
                    .bg(color(0xFBFDFF))
                    .text_color(color(if active { BRAND } else { TEXT }))
                    .when(active, |style| style.bg(color(0xEEF4FF)))
                    .hover(|style| style.bg(color(0xF1F5FF)))
                    .on_click(move |_, _, cx| on_nav(target, cx))
                    .child(
                        Flex::new()
                            .row()
                            .h_full()
                            .align_center()
                            .gap_sm()
                            .child(Icon::new(target.icon()).size(px(18.0)))
                            .child(narya_text(label).size(px(FS_BODY)).selectable(false)),
                    )
            }))
    }
}

impl IntoElement for SidebarMenu {
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

fn sidebar_status(
    running: bool,
    node: String,
    latency: u32,
    down: f32,
    up: f32,
) -> impl IntoElement {
    div().px(px(22.0)).pb(px(26.0)).child(
        Flex::new()
            .column()
            .gap_lg()
            .child(NaryaCard::plain(
                Flex::new()
                    .column()
                    .gap_md()
                    .child(
                        Space::new().gap_sm().child(status_dot(running)).child(
                            narya_text(if running { "已连接" } else { "未连接" })
                                .size(px(FS_BODY))
                                .bold()
                                .selectable(false),
                        ),
                    )
                    .child(
                        narya_text("当前节点")
                            .size(px(FS_CAPTION))
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    )
                    .child(
                        Space::new()
                            .gap_sm()
                            .child(flag_badge_for_name(&node))
                            .child(
                                narya_text(node)
                                    .size(px(FS_BODY))
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
                                narya_text(format!("↓ {:.2} MB/s", down))
                                    .size(px(FS_CAPTION))
                                    .text_color(color(SUCCESS).into())
                                    .selectable(false),
                            )
                            .child(
                                narya_text(format!("↑ {:.2} MB/s", up))
                                    .size(px(FS_CAPTION))
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
            .child(sidebar_footer_icons()),
    )
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
            PageKind::Rules => ("规则", "规则分流、条件与出口配置"),
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
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

impl gpui::RenderOnce for HeaderBar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .h(px(HEADER_H))
            .px(px(CONTENT_X_PAD))
            .pt(px(16.0))
            .child(
                Flex::new()
                    .row()
                    .h_full()
                    .align_start()
                    .justify_between()
                    .child(
                        Flex::new()
                            .column()
                            .gap_px(6.0)
                            .child(
                                narya_text(self.title)
                                    .size(px(FS_DISPLAY))
                                    .bold()
                                    .text_color(color(TEXT).into())
                                    .selectable(false),
                            )
                            .child(
                                narya_text(self.subtitle)
                                    .size(px(FS_SMALL))
                                    .text_color(color(MUTED).into())
                                    .selectable(false),
                            ),
                    )
                    .child(
                        Flex::new()
                            .column()
                            .align_end()
                            .gap_xl()
                            .child(Space::new().gap_md().children(self.actions)),
                    ),
            )
    }
}

pub struct FooterBar {
    pub kernel: String,
    pub config: String,
    pub subscriptions: String,
}

impl IntoElement for FooterBar {
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

impl gpui::RenderOnce for FooterBar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .h(px(FOOTER_H))
            .bg(color(SURFACE))
            .border_t_1()
            .border_color(color(BORDER))
            .px(px(CONTENT_X_PAD))
            .child(
                Flex::new()
                    .row()
                    .h_full()
                    .align_center()
                    .justify_between()
                    .child(
                        Space::new()
                            .gap_xl()
                            .child(status_line("内核", self.kernel))
                            .child(status_line("配置", self.config))
                            .child(status_line("订阅", self.subscriptions)),
                    )
                    .child(
                        Space::new()
                            .gap_xl()
                            .child(
                                narya_text("检查更新")
                                    .size(px(FS_SMALL))
                                    .text_color(color(BRAND).into())
                                    .selectable(false),
                            )
                            .child(
                                narya_text(env!("CARGO_PKG_VERSION"))
                                    .size(px(FS_SMALL))
                                    .text_color(color(MUTED).into())
                                    .selectable(false),
                            ),
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
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

impl gpui::RenderOnce for NaryaPage {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        Flex::new()
            .column()
            .gap_px(GAP)
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .children(self.rows)
    }
}

pub struct NaryaMetric;

impl NaryaMetric {
    pub fn card(
        title: &'static str,
        value: impl Into<String>,
        caption: impl Into<String>,
        icon: IconName,
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
        icon: IconName,
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
                            Statistic::new(title, value.into())
                                .value_color(color(TEXT).into())
                                .vertical(),
                        )
                        .child(
                            narya_text(caption.into())
                                .size(px(FS_CAPTION))
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
        Button::new(label.into().to_string()).primary().rounded_md()
    }
    pub fn ghost(label: impl Into<gpui::SharedString>) -> Button {
        Button::new(label.into().to_string())
            .tertiary()
            .rounded_md()
    }
    pub fn icon(label: impl Into<gpui::SharedString>) -> Button {
        Button::new(label.into().to_string())
            .tertiary()
            .rounded_md()
            .small()
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
    Flex::new()
        .row()
        .gap_px(GAP)
        .w_full()
        .min_w_0()
        .children(children)
}

pub fn dashboard_top(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    Flex::new()
        .column()
        .gap_px(GAP)
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .child(Flex::new().w_full().min_w_0().child(left))
        .child(Flex::new().w_full().min_w_0().child(right))
}

pub fn dashboard_middle(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    Flex::new()
        .column()
        .gap_px(GAP)
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .child(Flex::new().w_full().min_w_0().child(left))
        .child(Flex::new().w_full().min_w_0().child(right))
}

pub fn dashboard_bottom(
    a: impl IntoElement,
    b: impl IntoElement,
    c: impl IntoElement,
) -> impl IntoElement {
    Flex::new()
        .column()
        .gap_px(GAP)
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .child(Flex::new().w_full().min_w_0().child(a))
        .child(Flex::new().w_full().min_w_0().child(b))
        .child(Flex::new().w_full().min_w_0().child(c))
}

pub fn nodes_main(
    strategy: impl IntoElement,
    list: impl IntoElement,
    overview: impl IntoElement,
) -> impl IntoElement {
    Flex::new()
        .column()
        .gap_md()
        .w_full()
        .min_w_0()
        .child(Flex::new().w_full().min_w_0().child(strategy))
        .child(Flex::new().w_full().min_w_0().child(list))
        .child(Flex::new().w_full().min_w_0().child(overview))
}

pub fn nodes_bottom(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    Flex::new()
        .column()
        .gap_lg()
        .w_full()
        .min_w_0()
        .child(Flex::new().w_full().min_w_0().child(left))
        .child(Flex::new().w_full().min_w_0().child(right))
}

pub fn node_grid(items: Vec<AnyElement>) -> impl IntoElement {
    Flex::new().row().wrap().gap_md().children(
        items
            .into_iter()
            .map(|item| Flex::new().flex_1().min_w_0().child(item)),
    )
}

pub fn page_columns(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    Flex::new()
        .row()
        .gap_lg()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .child(Flex::new().flex_1().min_h_0().min_w_0().child(left))
        .child(
            Flex::new()
                .width_px(384.0)
                .flex_none()
                .min_h_0()
                .min_w_0()
                .child(right),
        )
}

pub fn toolbar(children: Vec<AnyElement>) -> impl IntoElement {
    Flex::new()
        .row()
        .wrap()
        .gap_md()
        .align_center()
        .w_full()
        .min_w_0()
        .height_px(38.0)
        .children(children)
}

pub fn search_input(placeholder: impl Into<gpui::SharedString>, width: f32) -> impl IntoElement {
    LioraInputBox {
        placeholder: placeholder.into(),
        width,
    }
}

pub fn filter_segmented(labels: &[&'static str], active: &'static str) -> impl IntoElement {
    segmented_control(labels, active, 592.0)
}

pub fn segmented_control(
    labels: &[&'static str],
    active: &'static str,
    width: f32,
) -> impl IntoElement {
    LioraSegmentedBox {
        labels: labels.to_vec(),
        active,
        width,
        on_change: None,
    }
}

pub fn segmented_control_with_change(
    labels: &[&'static str],
    active: &'static str,
    width: f32,
    on_change: Option<ValueChangeHandler>,
) -> impl IntoElement {
    LioraSegmentedBox {
        labels: labels.to_vec(),
        active,
        width,
        on_change,
    }
}

pub fn sort_select(options: &[&'static str], selected_idx: usize, width: f32) -> impl IntoElement {
    LioraSelectBox {
        options: options.to_vec(),
        selected_idx,
        width,
    }
}

pub fn grid_two(items: Vec<AnyElement>) -> impl IntoElement {
    Flex::new().row().wrap().gap_lg().children(
        items
            .into_iter()
            .map(|item| Flex::new().width_px(304.0).child(item)),
    )
}

pub fn design_card(body: impl IntoElement) -> gpui::Div {
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
            .min_w_0()
            .padding_px(16.0)
            .gap_md()
            .child(
                narya_text(title)
                    .size(px(FS_CARD_TITLE))
                    .bold()
                    .text_color(color(TEXT).into())
                    .selectable(false),
            )
            .child(Flex::new().flex_1().min_h_0().min_w_0().child(body)),
    )
}

pub fn panel_header(title: &'static str, action: &'static str) -> impl IntoElement {
    Flex::new()
        .row()
        .align_center()
        .justify_between()
        .child(
            narya_text(title)
                .size(px(FS_CARD_TITLE))
                .bold()
                .text_color(color(TEXT).into())
                .selectable(false),
        )
        .child(
            narya_text(action)
                .sm()
                .text_color(color(BRAND).into())
                .selectable(false),
        )
}

pub fn dashboard_quick_panel(items: Vec<AnyElement>) -> impl IntoElement {
    let body = if items.is_empty() {
        Flex::new().flex_1().center().child(
            narya_text("暂无节点，请先导入订阅")
                .size(px(FS_SMALL))
                .text_color(color(MUTED).into())
                .selectable(false),
        )
    } else {
        Flex::new().column().gap_sm().children(items)
    };
    titled_panel("快速连接", body)
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
            .w_full()
            .min_w_0()
            .overflow_hidden()
            .child(Flex::new().flex_1().min_h_0().w_full().child(chart))
            .child(
                Flex::new()
                    .width_px(300.0)
                    .flex_none()
                    .min_w_0()
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
            .w_full()
            .min_w_0()
            .overflow_hidden()
            .child(
                Flex::new()
                    .width_px(126.0)
                    .flex_none()
                    .min_w_0()
                    .child(Flex::new().column().gap_lg().children(stats)),
            )
            .child(Flex::new().flex_1().min_w_0().w_full().child(chart)),
    )
}

pub fn metric_quad(items: Vec<AnyElement>) -> impl IntoElement {
    Flex::new()
        .row()
        .wrap()
        .gap_lg()
        .w_full()
        .min_w_0()
        .children(
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
            Statistic::new(title, value.into())
                .value_color(color(TEXT).into())
                .vertical(),
        )
        .child(
            narya_text(caption.into())
                .size(px(FS_CAPTION))
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
    Flex::new().row().gap_lg().height_px(64.0).children(items)
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
                            .size(px(FS_NUMBER))
                            .color(status_color(tone).into()),
                    )
                    .child(
                        Flex::new()
                            .column()
                            .gap_sm()
                            .child(
                                narya_text(title)
                                    .size(px(FS_CAPTION))
                                    .bold()
                                    .text_color(color(TEXT).into())
                                    .selectable(false),
                            )
                            .child(
                                narya_text(value.into())
                                    .size(px(FS_BODY))
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
) -> gpui::Div {
    design_card(
        Flex::new()
            .column()
            .justify_between()
            .size_full()
            .padding_px(20.0)
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
                                    narya_text(title)
                                        .size(px(FS_CARD_TITLE))
                                        .bold()
                                        .text_color(color(TEXT).into())
                                        .selectable(false),
                                )
                                .child(
                                    narya_text(desc)
                                        .size(px(FS_SMALL))
                                        .text_color(color(MUTED).into())
                                        .selectable(false),
                                ),
                        ),
                    )
                    .child(LioraSwitchView {
                        checked: enabled,
                        disabled: false,
                        on_change: None,
                    }),
            )
            .child(
                Flex::new()
                    .row()
                    .justify_between()
                    .align_center()
                    .child(
                        Space::new().gap_sm().child(status_dot(enabled)).child(
                            narya_text(if enabled { "已启用" } else { "未启用" })
                                .size(px(FS_SMALL))
                                .text_color(color(if enabled { SUCCESS } else { MUTED }).into())
                                .selectable(false),
                        ),
                    )
                    .child(
                        narya_text(mode)
                            .size(px(FS_SMALL))
                            .text_color(color(TEXT).into())
                            .selectable(false),
                    ),
            ),
    )
}

pub fn hero_toggle_card_with_click(
    icon: IconName,
    title: &'static str,
    desc: &'static str,
    enabled: bool,
    mode: &'static str,
    tone: NaryaStatus,
    on_click: ClickHandler,
) -> impl IntoElement {
    div()
        .id(format!("narya-hero-toggle-{title}"))
        .size_full()
        .cursor_pointer()
        .child(hero_toggle_card(icon, title, desc, enabled, mode, tone))
        .on_click(on_click)
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
            Space::new()
                .gap_md()
                .child(flag_badge_for_name(&name))
                .child(
                    Flex::new()
                        .column()
                        .child(
                            narya_text(name)
                                .size(px(FS_SMALL))
                                .text_color(color(TEXT).into())
                                .selectable(false),
                        )
                        .child(
                            narya_text(protocol.into())
                                .size(px(FS_CAPTION))
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
                                narya_text(if data.active { "◉" } else { "○" })
                                    .text_color(
                                        color(if data.active { BRAND } else { MUTED }).into(),
                                    )
                                    .selectable(false),
                            )
                            .child(flag_badge_for_name(&data.name))
                            .child(
                                Flex::new()
                                    .column()
                                    .child(
                                        narya_text(data.name)
                                            .bold()
                                            .text_color(color(TEXT).into())
                                            .selectable(false),
                                    )
                                    .child(
                                        narya_text(data.protocol)
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
                        narya_text(format!("● {}%", data.load))
                            .size(px(FS_CAPTION))
                            .text_color(color(MUTED).into())
                            .selectable(false),
                    )
                    .child(
                        narya_text(format!("↓ {:.1} MB/s", data.down))
                            .size(px(FS_CAPTION))
                            .text_color(color(BRAND).into())
                            .selectable(false),
                    )
                    .child(
                        narya_text(format!("↑ {:.1} MB/s", data.up))
                            .size(px(FS_CAPTION))
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
                        IconName::Plane,
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
                                narya_text(name.into())
                                    .bold()
                                    .text_color(color(TEXT).into())
                                    .selectable(false),
                            )
                            .child(
                                narya_text(url.into())
                                    .xs()
                                    .text_color(color(MUTED).into())
                                    .selectable(false),
                            )
                            .child(
                                narya_text(format!("{} 节点    更新：刚刚", nodes))
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
                        narya_text(format!("流量 {:.0}%", usage))
                            .size(px(FS_CAPTION))
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
            narya_text(label.into())
                .size(px(FS_SMALL))
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            narya_text(value.into())
                .size(px(FS_BODY))
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

pub fn soft_trend(values: Vec<f64>, height: f32, color_hex: u32) -> impl IntoElement {
    Sparkline::new(values)
        .height(px(height))
        .padding(px(8.0))
        .color(color(color_hex).into())
        .area_fill(true)
        .stroke_width(px(2.0))
        .show_last_point(false)
        .smooth(true)
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
            narya_text(label)
                .size(px(FS_BODY))
                .text_color(color(TEXT).into())
                .selectable(false),
        )
        .child(
            Flex::new()
                .flex_1()
                .child(Progress::new(pct).show_text(false).stroke_width(px(6.0))),
        )
        .child(
            narya_text(format!("{:.1}%", pct))
                .size(px(FS_SMALL))
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
            narya_text("●")
                .text_color(status_color(tone).into())
                .selectable(false),
        )
        .child(
            narya_text(time.into())
                .size(px(FS_SMALL))
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            narya_text(message.into())
                .size(px(FS_BODY))
                .text_color(color(TEXT).into())
                .selectable(false),
        )
}

pub fn setting_row(label: &'static str, enabled: bool) -> impl IntoElement {
    setting_row_with_change(label, enabled, None)
}

pub fn setting_row_with_change(
    label: &'static str,
    enabled: bool,
    on_change: Option<BoolChangeHandler>,
) -> impl IntoElement {
    Flex::new()
        .row()
        .justify_between()
        .align_center()
        .child(
            narya_text(label)
                .size(px(FS_BODY))
                .text_color(color(TEXT).into())
                .selectable(false),
        )
        .child(LioraSwitchView {
            checked: enabled,
            disabled: false,
            on_change,
        })
}

pub fn category(label: &'static str, active: bool) -> impl IntoElement {
    category_menu(
        vec![(label, IconName::FolderOpen)],
        if active { 0 } else { usize::MAX },
    )
}

pub fn category_menu(
    items: Vec<(&'static str, IconName)>,
    active_index: usize,
) -> impl IntoElement {
    category_menu_with_change(items, active_index, None)
}

pub fn category_menu_with_change(
    items: Vec<(&'static str, IconName)>,
    active_index: usize,
    on_select: Option<IndexChangeHandler>,
) -> impl IntoElement {
    LioraMenuGroup {
        id: "narya-category-menu",
        items,
        active_index,
        on_select,
    }
}

pub fn narya_tag(label: impl Into<gpui::SharedString>, status: NaryaStatus) -> Tag {
    let tag = Tag::new(label.into().to_string()).small().round(true);
    match status {
        NaryaStatus::Info => tag.info(),
        NaryaStatus::Success => tag.success(),
        NaryaStatus::Warning => tag.warning(),
        NaryaStatus::Danger => tag.danger(),
    }
}

pub fn status_dot(on: bool) -> impl IntoElement {
    narya_text(if on { "●" } else { "○" })
        .text_color(color(if on { SUCCESS } else { MUTED }).into())
        .selectable(false)
}

fn status_line(label: &'static str, value: impl Into<String>) -> impl IntoElement {
    Space::new()
        .gap_sm()
        .child(
            narya_text(label)
                .size(px(FS_SMALL))
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            narya_text(value.into())
                .size(px(FS_BODY))
                .text_color(color(TEXT).into())
                .selectable(false),
        )
}

fn key_value(label: &'static str, value: &'static str) -> impl IntoElement {
    Flex::new()
        .row()
        .justify_between()
        .child(
            narya_text(label)
                .size(px(FS_CAPTION))
                .text_color(color(MUTED).into())
                .selectable(false),
        )
        .child(
            narya_text(value)
                .size(px(FS_CAPTION))
                .text_color(color(TEXT).into())
                .selectable(false),
        )
}

fn hero_icon(icon: IconName, tone: NaryaStatus) -> impl IntoElement {
    div()
        .size(px(56.0))
        .rounded(px(12.0))
        .bg(status_color(tone))
        .child(
            Flex::new()
                .size_full()
                .center()
                .child(Icon::new(icon).size(px(27.0)).color(color(SURFACE).into())),
        )
}

fn flag_badge_for_name(name: &str) -> impl IntoElement {
    let (label, bg, fg) = if name.contains("香港") || name.contains("HK") {
        ("HK", 0xFFEDEB, 0xE23A2E)
    } else if name.contains("日本") || name.contains("JP") {
        ("JP", 0xFFF1F1, 0xE11D48)
    } else if name.contains("美国") || name.contains("US") {
        ("US", 0xEEF4FF, BRAND)
    } else if name.contains("新加坡") || name.contains("SG") {
        ("SG", 0xFFF1F1, 0xE11D48)
    } else if name.contains("台湾") || name.contains("TW") {
        ("TW", 0xEEF4FF, BRAND)
    } else if name.contains("德国") || name.contains("DE") {
        ("DE", 0xFFF7E6, WARNING)
    } else if name.contains("英国") || name.contains("UK") {
        ("UK", 0xEEF4FF, BRAND)
    } else {
        ("GL", 0xEEF4FF, MUTED)
    };
    div().size(px(32.0)).rounded(px(999.0)).bg(color(bg)).child(
        Flex::new().size_full().center().child(
            narya_text(label)
                .size(px(10.5))
                .bold()
                .text_color(color(fg).into())
                .selectable(false),
        ),
    )
}

fn metric_icon(icon: IconName, tone: NaryaStatus) -> impl IntoElement {
    div()
        .size(px(56.0))
        .rounded(px(12.0))
        .bg(status_soft_color(tone))
        .child(
            Flex::new().size_full().center().child(
                Icon::new(icon)
                    .size(px(FS_NUMBER))
                    .color(status_color(tone).into()),
            ),
        )
}

struct LioraInputBox {
    placeholder: gpui::SharedString,
    width: f32,
}

impl gpui::RenderOnce for LioraInputBox {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let placeholder = self.placeholder.clone();
        cx.new(|cx| {
            Input::new("", cx)
                .placeholder(placeholder)
                .icon_prefix(IconName::Search)
                .clearable(false)
                .width(px(self.width))
                .height(px(38.0))
        })
    }
}

impl IntoElement for LioraInputBox {
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

struct LioraSegmentedBox {
    labels: Vec<&'static str>,
    active: &'static str,
    width: f32,
    on_change: Option<ValueChangeHandler>,
}

impl gpui::RenderOnce for LioraSegmentedBox {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let labels_id = self.labels.join("-");
        let segmented_id = format!("narya-filter-segmented-{labels_id}-{}", self.active);
        let options = self
            .labels
            .into_iter()
            .map(|label| SegmentedOption::new(label, label))
            .collect();
        let segmented = cx.new(|_| {
            let segmented = Segmented::new(options)
                .id(segmented_id)
                .value(self.active)
                .block(true);
            match self.on_change {
                Some(on_change) => segmented.on_change(on_change),
                None => segmented,
            }
        });
        div()
            .id(format!("narya-segmented-{labels_id}-{}", self.active))
            .w(px(self.width))
            .child(segmented)
    }
}

impl IntoElement for LioraSegmentedBox {
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

struct LioraSelectBox {
    options: Vec<&'static str>,
    selected_idx: usize,
    width: f32,
}

impl gpui::RenderOnce for LioraSelectBox {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let selected = if self.options.is_empty() {
            None
        } else {
            Some(self.selected_idx.min(self.options.len() - 1))
        };
        cx.new(|cx| {
            Select::new(self.options, selected, cx)
                .width(px(self.width))
                .text_sm()
                .padding_x_sm()
        })
    }
}

impl IntoElement for LioraSelectBox {
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

struct LioraSwitchView {
    checked: bool,
    disabled: bool,
    on_change: Option<BoolChangeHandler>,
}

impl gpui::RenderOnce for LioraSwitchView {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        cx.new(|cx| {
            let switch = Switch::new(self.checked, cx).disabled(self.disabled);
            match self.on_change {
                Some(on_change) => switch.on_change(on_change),
                None => switch,
            }
        })
    }
}

impl IntoElement for LioraSwitchView {
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

struct LioraMenuGroup {
    id: &'static str,
    items: Vec<(&'static str, IconName)>,
    active_index: usize,
    on_select: Option<IndexChangeHandler>,
}

impl gpui::RenderOnce for LioraMenuGroup {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let active_id = if self.active_index < self.items.len() {
            format!("{}-{}", self.id, self.active_index)
        } else {
            String::new()
        };
        let on_select = self.on_select;
        let menu = self.items.into_iter().enumerate().fold(
            liora::components::NavigationMenu::new()
                .id(self.id)
                .default_active(active_id),
            |menu, (index, (label, icon))| {
                menu.item(format!("{}-{}", self.id, index), label, Some(icon))
            },
        );
        cx.new(|_| match on_select {
            Some(on_select) => menu.on_select(move |id, window, app| {
                let Some(index) = id.rsplit('-').next().and_then(|value| value.parse().ok()) else {
                    return;
                };
                on_select(index, window, app);
            }),
            None => menu,
        })
    }
}

impl IntoElement for LioraMenuGroup {
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
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
        window_decorations: Some(gpui::WindowDecorations::Server),
        app_id: Some("narya".into()),
        ..Default::default()
    }
}

pub fn open_shell_window<T: gpui::Render + 'static>(
    cx: &mut gpui::App,
    build: impl FnOnce(&mut gpui::Window, &mut gpui::App) -> Entity<T> + 'static,
) {
    let options = entity_window_options(cx);
    if let Err(error) = cx.open_window(options, build) {
        eprintln!("failed to open Narya main window: {error}");
    }
}
