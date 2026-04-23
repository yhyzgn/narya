use crate::models::traffic::SharedTrafficStore;
use gpui::*;

pub struct TrafficChart;

impl TrafficChart {
    pub fn render(
        store: SharedTrafficStore,
        _cx: &mut Context<crate::Workspace>,
    ) -> impl IntoElement {
        // 使用最简单的 div 占位，确保性能和编译通过
        div().size_full().bg(rgb(0x141414)).child(
            div()
                .text_xs()
                .text_color(rgb(0x444444))
                .child("Traffic Spline Chart Rendering..."),
        )
    }
}
