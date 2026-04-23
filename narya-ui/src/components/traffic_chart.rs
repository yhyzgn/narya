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

                // 提高精度：20倍超采样
                let sub_segments = 20;
                let total_points = (60 - 1) * sub_segments;
                let sub_x_step = width / (total_points as f32);

                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px((val / max_val) * chart_height)
                };

                // 1. 背景网格
                for i in 1..=4 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 4.0));
                    let mut path = Path::new(point(bounds.left(), y));
                    path.line_to(point(bounds.right(), y));
                    window.paint_path(path, rgba(0xffffff05));
                }

                // 2. 连续长路径渲染逻辑 (杜绝接缝)
                let render_continuous_wave =
                    |data_slice: &[f32], color: Rgba, window: &mut Window| {
                        if data_slice.len() < 2 {
                            return;
                        }

                        // A. 预计算所有平滑轨迹点 (不分段)
                        let mut points = Vec::with_capacity(total_points + 1);
                        for i in 0..data_slice.len() - 1 {
                            let y_start = data_slice[i];
                            let y_end = data_slice[i + 1];
                            let base_x = bounds.left() + px(i as f32 * (width / 59.0));

                            for s in 0..sub_segments {
                                let mu = s as f32 / sub_segments as f32;
                                let f = |m: f32| (1.0 - (m * PI).cos()) / 2.0;
                                let py = scale_y(y_start * (1.0 - f(mu)) + y_end * f(mu));
                                let px = base_x + px(s as f32 * sub_x_step);
                                points.push(point(px, py));
                            }
                        }
                        points.push(point(bounds.right(), scale_y(*data_slice.last().unwrap())));

                        // B. 绘制填充 (单一封闭 Path)
                        // 关键：为了不产生从 (0,0) 开始的三角形，填充 Path 必须从左下角显式开始
                        let bottom_left = point(bounds.left(), bounds.bottom() - px(padding));
                        let bottom_right = point(bounds.right(), bounds.bottom() - px(padding));

                        let mut fill_path = Path::new(bottom_left);
                        for &p in &points {
                            fill_path.line_to(p);
                        }
                        fill_path.line_to(bottom_right);
                        fill_path.line_to(bottom_left);

                        window.paint_path(
                            fill_path,
                            Rgba {
                                r: color.r,
                                g: color.g,
                                b: color.b,
                                a: 0.15,
                            },
                        );

                        // C. 绘制连续亮线 (单一 Path，彻底消除虚线感)
                        let mut line_path = Path::new(points[0]);
                        for &p in points.iter().skip(1) {
                            line_path.line_to(p);
                        }

                        // 为了让 GPUI 认为这是一个闭合形状以增加厚度，我们拉出一个微小的厚度层
                        let mut stroke_path = line_path;
                        // 我们通过给描边路径增加一像素的垂直厚度并反向拉回，来实现真正的“粗线”且无虚线
                        for &p in points.iter().rev() {
                            stroke_path.line_to(point(p.x, p.y + px(1.8)));
                        }
                        window.paint_path(stroke_path, color);
                    };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 下载：电光青
                render_continuous_wave(&down_data, rgb(0x00f0ff), window);
                // 上传：荧光绿
                render_continuous_wave(&up_data, rgb(0xb0ff00), window);
            },
        )
        .size_full()
    }
}
