use crate::models::traffic::SharedTrafficStore;
use gpui::*;

pub struct TrafficChart;

impl TrafficChart {
    pub fn render(
        store: SharedTrafficStore,
        _cx: &mut Context<crate::Workspace>,
    ) -> impl IntoElement {
        canvas(
            // Prepaint 闭包
            move |_bounds, _window, _cx| {
                let store_read = store.read();
                let history = store_read.get_history();
                drop(store_read);
                history
            },
            // Paint 闭包
            move |bounds, data, window, _cx| {
                if data.is_empty() {
                    return;
                }

                let width: f32 = bounds.size.width.into();
                let height: f32 = bounds.size.height.into();
                let padding = 10.0;
                let chart_height = height - padding * 2.0;

                let mut max_val: f32 = 100.0;
                for d in &data {
                    if d.up > max_val {
                        max_val = d.up;
                    }
                    if d.down > max_val {
                        max_val = d.down;
                    }
                }
                max_val *= 1.2;

                let x_step = width / (60.0 - 1.0);
                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px(val / max_val * chart_height)
                };

                // 1. 网格
                for i in 0..=4 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 4.0));
                    let mut path = Path::new(point(bounds.left(), y));
                    path.line_to(point(bounds.right(), y));
                    window.paint_path(path, rgb(0x222222));
                }

                // 2. 波形
                let draw_wave =
                    |data_slice: Vec<f32>, stroke: Rgba, fill: Rgba, window: &mut Window| {
                        if data_slice.len() < 2 {
                            return;
                        }

                        let start_y = scale_y(data_slice[0]);
                        let mut path = Path::new(point(bounds.left(), start_y));

                        for i in 1..data_slice.len() {
                            let x1 = bounds.left() + px(i as f32 * x_step);
                            let y1 = scale_y(data_slice[i]);
                            path.line_to(point(x1, y1));
                        }

                        let mut fill_path = path.clone();
                        fill_path.line_to(point(bounds.right(), bounds.bottom() - px(padding)));
                        fill_path.line_to(point(bounds.left(), bounds.bottom() - px(padding)));
                        window.paint_path(fill_path, fill);
                        window.paint_path(path, stroke);
                    };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                draw_wave(down_data, rgb(0x1677ff), rgba(0x1677ff1a), window);
                draw_wave(up_data, rgb(0x52c41a), rgba(0x52c41a15), window);
            },
        )
        .size_full()
    }
}
