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
                let padding = 8.0;
                let chart_height = height - padding * 2.0;

                // 动态高度缩放，确保波峰不会撞顶
                let mut max_val: f32 = 50.0; // 最小 50KB/s 刻度
                for d in &data {
                    if d.up > max_val {
                        max_val = d.up;
                    }
                    if d.down > max_val {
                        max_val = d.down;
                    }
                }
                max_val *= 1.25;

                let max_samples = 60;
                let sub_segments = 10;
                let total_render_points = (max_samples - 1) * sub_segments;
                let sub_x_step = width / (total_render_points as f32);

                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px((val / max_val) * chart_height)
                };

                // 1. 绘制极简背景刻度线
                for i in 1..=3 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 4.0));
                    let mut grid_path = Path::new(point(bounds.left(), y));
                    grid_path.line_to(point(bounds.right(), y));
                    window.paint_path(grid_path, rgba(0xffffff0a));
                }

                // 2. 超采样 Liquid 渲染函数
                let render_layer = |data_slice: &[f32], color: Rgba, window: &mut Window| {
                    if data_slice.len() < 2 {
                        return;
                    }

                    // A. 生成平滑点集
                    let mut wave_points = Vec::with_capacity(total_render_points + 1);
                    for i in 0..data_slice.len() - 1 {
                        let y1 = data_slice[i];
                        let y2 = data_slice[i + 1];
                        for s in 0..sub_segments {
                            let mu = s as f32 / sub_segments as f32;
                            // 余弦插值：极致丝滑
                            let mu2 = (1.0 - (mu * PI).cos()) / 2.0;
                            let y_interp = y1 * (1.0 - mu2) + y2 * mu2;

                            let x = bounds.left() + px((i * sub_segments + s) as f32 * sub_x_step);
                            wave_points.push(point(x, scale_y(y_interp)));
                        }
                    }
                    wave_points.push(point(bounds.right(), scale_y(*data_slice.last().unwrap())));

                    // B. 绘制填充区域 (关键：独立 Path 且严格闭合)
                    let bl = point(bounds.left(), bounds.bottom() - px(padding));
                    let br = point(bounds.right(), bounds.bottom() - px(padding));

                    let mut fill_path = Path::new(bl);
                    for &p in &wave_points {
                        fill_path.line_to(p);
                    }
                    fill_path.line_to(br);
                    fill_path.line_to(bl); // 强制闭合到左下角

                    window.paint_path(
                        fill_path,
                        Rgba {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: 0.12,
                        },
                    );

                    // C. 绘制波形描边 (独立 Path)
                    let mut stroke_path = Path::new(wave_points[0]);
                    for &p in wave_points.iter().skip(1) {
                        stroke_path.line_to(p);
                    }
                    window.paint_path(stroke_path, color);
                };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 渲染下载 (蓝色)
                render_layer(&down_data, rgb(0x3584e4), window);
                // 渲染上传 (绿色)
                render_layer(&up_data, rgb(0x2ec27e), window);
            },
        )
        .size_full()
    }
}
