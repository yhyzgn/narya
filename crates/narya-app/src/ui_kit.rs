use gpui::{div, prelude::*, px, rgb, AnyElement, IntoElement, Rgba};
use liora::components::{Button, Card, Flex, Progress, Space, Tag, Text};

pub const BG: u32 = 0xF5F7FB;
pub const SURFACE: u32 = 0xFFFFFF;
pub const BORDER: u32 = 0xE5E7EB;
pub const TEXT: u32 = 0x111827;
pub const MUTED: u32 = 0x6B7280;
pub const BRAND: u32 = 0x3B82F6;
pub const SUCCESS: u32 = 0x10B981;
pub const WARNING: u32 = 0xF59E0B;
pub const DANGER: u32 = 0xEF4444;

pub fn color(hex: u32) -> Rgba {
    rgb(hex)
}

pub struct NaryaPage {
    title: String,
    subtitle: String,
    children: Vec<AnyElement>,
}

impl NaryaPage {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl IntoElement for NaryaPage {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .flex()
            .flex_col()
            .size_full()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(24.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(color(TEXT))
                            .child(self.title),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(color(MUTED))
                            .child(self.subtitle),
                    ),
            )
            .children(self.children)
    }
}

pub struct NaryaCard;

impl NaryaCard {
    pub fn panel(body: impl IntoElement) -> Card {
        Card::new(body)
    }

    pub fn titled(title: impl Into<gpui::SharedString>, body: impl IntoElement) -> Card {
        Card::new(body).title(title)
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

    pub fn danger(label: impl Into<gpui::SharedString>) -> Button {
        Button::new(label).danger().rounded_md()
    }
}

pub struct NaryaMetric;

impl NaryaMetric {
    pub fn card(
        label: impl Into<String>,
        value: impl Into<String>,
        hint: impl Into<String>,
        accent: Rgba,
    ) -> impl IntoElement {
        let accent: gpui::Hsla = accent.into();
        NaryaCard::panel(
            Flex::new()
                .column()
                .gap_md()
                .child(
                    Text::new(label.into())
                        .sm()
                        .text_color(color(MUTED).into())
                        .selectable(false),
                )
                .child(
                    Text::new(value.into())
                        .size(px(24.0))
                        .weight(gpui::FontWeight::BOLD)
                        .text_color(color(TEXT).into())
                        .selectable(false),
                )
                .child(
                    Space::new()
                        .gap_sm()
                        .child(Tag::new("LIVE").small().round(true).info())
                        .child(
                            Text::new(hint.into())
                                .xs()
                                .text_color(accent)
                                .selectable(false),
                        ),
                ),
        )
        .no_shadow()
    }
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

pub fn progress(percent: f32) -> Progress {
    Progress::new(percent)
        .show_text(false)
        .stroke_width(px(7.0))
}

#[derive(Clone, Copy)]
pub enum NaryaStatus {
    Info,
    Success,
    Warning,
    Danger,
}
