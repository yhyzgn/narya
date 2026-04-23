use crate::models::traffic::SharedTrafficStore;
use gpui::*;

pub struct TrafficChart;

impl TrafficChart {
    pub fn render(
        store: SharedTrafficStore,
        _cx: &mut Context<crate::Workspace>,
    ) -> impl IntoElement {
        canvas(
            move |_bounds, _window, _cx| {
                // prepaint: 从共享存储中拉取历史数据
                let store = store.read();
                store.get_history()
            },
            move |bounds, history, window, _cx| {
                let width = bounds.size.width;
                let height = bounds.size.height;
                let len = history.len();

                if len < 2 {
                    return;
                }

                let width_f32: f32 = width.into();
                let height_f32: f32 = height.into();
                let step_x = width_f32 / (len as f32 - 1.0);
                let max_kb = 1024.0f32;

                // 绘制下行流量 (Down) - 面积图
                let mut down_path = Path::new(bounds.origin + point(px(0.0), height));
                for (i, data) in history.iter().enumerate() {
                    let x = px(i as f32 * step_x);
                    let ratio = (data.down / max_kb).min(1.0);
                    let y = height_f32 * (1.0 - ratio);
                    down_path.line_to(bounds.origin + point(x, px(y)));
                }
                down_path.line_to(bounds.origin + point(width, height));
                down_path.line_to(bounds.origin + point(px(0.0), height));

                window.paint_path(down_path, rgba(0x1677ff33));

                // 绘制上行流量 (Up) - 描边线
                let start_ratio = (history[0].up / max_kb).min(1.0);
                let start_y = height_f32 * (1.0 - start_ratio);
                let mut up_path = Path::new(bounds.origin + point(px(0.0), px(start_y)));
                for (i, data) in history.iter().enumerate() {
                    let x = px(i as f32 * step_x);
                    let ratio = (data.up / max_kb).min(1.0);
                    let y = height_f32 * (1.0 - ratio);
                    up_path.line_to(bounds.origin + point(x, px(y)));
                }

                window.paint_path(up_path, rgba(0x52c41aff));
            },
        )
        .size_full()
    }
}
