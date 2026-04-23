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

                let mut max_val: f32 = 100.0;
                for d in &data {
                    if d.up > max_val {
                        max_val = d.up;
                    }
                    if d.down > max_val {
                        max_val = d.down;
                    }
                }
                max_val *= 1.15;

                let _x_step = width / (60.0 - 1.0);
                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px(val / max_val * chart_height)
                };

                // 1. 背景网格
                for i in 1..=3 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 4.0));
                    let mut path = Path::new(point(bounds.left(), y));
                    path.line_to(point(bounds.right(), y));
                    window.paint_path(path, rgba(0xffffff08));
                }

                // 2. 超采样平滑绘制逻辑
                let draw_liquid_wave = |data_slice: Vec<f32>, color: Rgba, window: &mut Window| {
                    if data_slice.len() < 2 {
                        return;
                    }

                    let sub_segments = 12;
                    let total_points = (data_slice.len() - 1) * sub_segments;
                    let sub_x_step = width / (total_points as f32);

                    let mut points = Vec::with_capacity(total_points + 1);
                    for i in 0..data_slice.len() - 1 {
                        let y1 = data_slice[i];
                        let y2 = data_slice[i + 1];
                        for s in 0..sub_segments {
                            let mu = s as f32 / sub_segments as f32;
                            // 余弦插值：确保波形在点之间是正弦平滑的
                            let mu2 = (1.0 - (mu * PI).cos()) / 2.0;
                            let y_interp = y1 * (1.0 - mu2) + y2 * mu2;

                            let x = bounds.left() + px((i * sub_segments + s) as f32 * sub_x_step);
                            points.push(point(x, scale_y(y_interp)));
                        }
                    }
                    points.push(point(bounds.right(), scale_y(*data_slice.last().unwrap())));

                    let mut path = Path::new(points[0]);
                    for p in points.iter().skip(1) {
                        path.line_to(*p);
                    }

                    // 绘制填充
                    let mut fill_path = path.clone();
                    fill_path.line_to(point(bounds.right(), bounds.bottom() - px(padding)));
                    fill_path.line_to(point(bounds.left(), bounds.bottom() - px(padding)));
                    window.paint_path(
                        fill_path,
                        Rgba {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: 0.15,
                        },
                    );

                    // 绘制线条阴影层 (Glow)
                    window.paint_path(
                        path.clone(),
                        Rgba {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: 0.4,
                        },
                    );

                    // 绘制顶层实色线
                    window.paint_path(path, color);
                };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 下载：丝滑蓝
                draw_liquid_wave(down_data, rgb(0x3584e4), window);

                // 上传：丝滑绿
                draw_liquid_wave(up_data, rgb(0x2ec27e), window);
            },
        )
        .size_full()
    }
}
