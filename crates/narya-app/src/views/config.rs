use crate::components::{icon, IconName};
use crate::state::AppState;
use crate::views::app_shell::AppShell;
use gpui::{prelude::*, *};

pub fn render_config_view(
    _model: &Entity<AppState>,
    _cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let color_text_primary = rgb(0x0F172A);
    let color_text_secondary = rgb(0x64748B);
    let color_border = rgb(0xE2E8F0);

    div()
        .flex()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(600.0))
                .bg(rgb(0xFFFFFF))
                .border_1()
                .border_color(color_border)
                .rounded_2xl()
                .p_12()
                .items_center()
                .gap_6()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(80.0))
                        .bg(rgb(0xF1F5F9))
                        .rounded_full()
                        .child(icon(IconName::Config, 40.0, color_text_secondary.into())),
                )
                .child(
                    div()
                        .flex()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(color_text_primary)
                        .child("配置编辑模块正在开发中"),
                )
                .child(
                    div()
                        .flex()
                        .text_center()
                        .text_sm()
                        .text_color(color_text_secondary)
                        .child("我们将提供基于 YAML 的可视化编辑器和智能补全功能，敬请期待。"),
                )
                .child(
                    div()
                        .flex()
                        .px_6()
                        .py_3()
                        .bg(rgb(0x4F46E5))
                        .text_color(white())
                        .rounded_lg()
                        .font_weight(FontWeight::BOLD)
                        .child("查看文档"),
                ),
        )
}
