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
                let padding = 12.0;
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
                max_val *= 1.25;

                let sub_segments = 12;
                let sub_x_step = width / (59.0 * sub_segments as f32);

                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px((val / max_val) * chart_height)
                };

                // 1. 绘制极简背景网格
                for i in 1..=4 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 4.0));
                    let mut path = Path::new(point(bounds.left(), y));
                    path.line_to(point(bounds.right(), y));
                    window.paint_path(path, rgba(0xffffff05));
                }

                // 2. 霓虹波形渲染函数
                let render_neon_layer = |data_slice: &[f32], color: Rgba, window: &mut Window| {
                    if data_slice.len() < 2 {
                        return;
                    }

                    let bottom_y = bounds.bottom() - px(padding);

                    for i in 0..data_slice.len() - 1 {
                        let y_start = data_slice[i];
                        let y_end = data_slice[i + 1];
                        let base_x = bounds.left() + px(i as f32 * (width / 59.0));

                        for s in 0..sub_segments {
                            let mu = s as f32 / sub_segments as f32;
                            let mu_next = (s + 1) as f32 / sub_segments as f32;

                            let f = |m: f32| (1.0 - (m * PI).cos()) / 2.0;
                            let p1_y = scale_y(y_start * (1.0 - f(mu)) + y_end * f(mu));
                            let p2_y = scale_y(y_start * (1.0 - f(mu_next)) + y_end * f(mu_next));

                            let p1_x = base_x + px(s as f32 * sub_x_step);
                            let p2_x = base_x + px((s + 1) as f32 * sub_x_step);

                            // A. 填充层
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
                                    a: 0.12,
                                },
                            );

                            // B. 霓虹发光层 (Glow Line)
                            let mut glow = Path::new(point(p1_x, p1_y));
                            glow.line_to(point(p2_x, p2_y));
                            glow.line_to(point(p2_x, p2_y + px(4.0)));
                            glow.line_to(point(p1_x, p1_y + px(4.0)));
                            window.paint_path(
                                glow,
                                Rgba {
                                    r: color.r,
                                    g: color.g,
                                    b: color.b,
                                    a: 0.3,
                                },
                            );

                            // C. 核心亮线 (Core Line)
                            let mut core = Path::new(point(p1_x, p1_y));
                            core.line_to(point(p2_x, p2_y));
                            core.line_to(point(p2_x, p2_y + px(1.8)));
                            core.line_to(point(p1_x, p1_y + px(1.8)));
                            window.paint_path(core, color);
                        }
                    }
                };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 下载：电光青 (#00f0ff)
                render_neon_layer(&down_data, rgb(0x00f0ff), window);

                // 上传：荧光绿 (#b0ff00)
                render_neon_layer(&up_data, rgb(0xb0ff00), window);
            },
        )
        .size_full()
    }
}
