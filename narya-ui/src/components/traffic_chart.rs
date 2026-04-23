use crate::models::traffic::SharedTrafficStore;
use gpui::*;
use std::f32::consts::PI;

pub struct TrafficChart;

impl TrafficChart {
    pub fn render(
        store: SharedTrafficStore,
        _cx: &mut Context<crate::Workspace>,
    ) -> impl IntoElement {
        canvas(
            move |_bounds, _window, _cx| {
                let store_read = store.read();
                let history = store_read.get_history();
                drop(store_read);
                history
            },
            move |bounds, data, window, _cx| {
                if data.is_empty() {
                    return;
                }

                let width: f32 = bounds.size.width.into();
                let height: f32 = bounds.size.height.into();
                let padding = 10.0;
                let chart_height = height - padding * 2.0;

                let mut max_val: f32 = 50.0;
                for d in &data {
                    if d.up > max_val {
                        max_val = d.up;
                    }
                    if d.down > max_val {
                        max_val = d.down;
                    }
                }
                max_val *= 1.25;

                let sub_segments = 12;
                let total_points = (60 - 1) * sub_segments;
                let sub_x_step = width / (total_points as f32);

                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px((val / max_val) * chart_height)
                };

                // 1. 绘制极简背景网格
                for i in 1..=3 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 4.0));
                    let mut path = Path::new(point(bounds.left(), y));
                    path.line_to(point(bounds.right(), y));
                    window.paint_path(path, rgba(0xffffff08));
                }

                // 2. 液体分段绘制 (彻底修复三角形伪影)
                let render_segments = |data_slice: &[f32], color: Rgba, window: &mut Window| {
                    if data_slice.len() < 2 {
                        return;
                    }

                    let bottom_y = bounds.bottom() - px(padding);

                    for i in 0..data_slice.len() - 1 {
                        let y_start = data_slice[i];
                        let y_end = data_slice[i + 1];
                        let base_x = bounds.left() + px(i as f32 * (width / 59.0));

                        // 在两个采样点之间进行超采样
                        for s in 0..sub_segments {
                            let mu = s as f32 / sub_segments as f32;
                            let mu_next = (s + 1) as f32 / sub_segments as f32;

                            // 余弦插值
                            let f = |m: f32| (1.0 - (m * PI).cos()) / 2.0;
                            let p1_y = scale_y(y_start * (1.0 - f(mu)) + y_end * f(mu));
                            let p2_y = scale_y(y_start * (1.0 - f(mu_next)) + y_end * f(mu_next));

                            let p1_x = base_x + px(s as f32 * sub_x_step);
                            let p2_x = base_x + px((s + 1) as f32 * sub_x_step);

                            // 绘制极小的独立填充块 (梯形)
                            let mut fill = Path::new(point(p1_x, bottom_y));
                            fill.line_to(point(p1_x, p1_y));
                            fill.line_to(point(p2_x, p2_y));
                            fill.line_to(point(p2_x, bottom_y));
                            window.paint_path(
                                fill,
                                Rgba {
                                    r: color.r,
                                    g: color.g,
                                    b: color.b,
                                    a: 0.15,
                                },
                            );

                            // 绘制极小的独立线条段
                            let mut line = Path::new(point(p1_x, p1_y));
                            line.line_to(point(p2_x, p2_y));
                            // 给线条增加一点伪厚度
                            line.line_to(point(p2_x, p2_y + px(1.2)));
                            line.line_to(point(p1_x, p1_y + px(1.2)));
                            window.paint_path(line, color);
                        }
                    }
                };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                render_segments(&down_data, rgb(0x3584e4), window);
                render_segments(&up_data, rgb(0x2ec27e), window);
            },
        )
        .size_full()
    }
}
