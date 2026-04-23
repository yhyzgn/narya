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

                // 20倍超采样，确保极致曲线平滑
                let sub_segments = 20;
                let total_render_points = (60 - 1) * sub_segments;
                let sub_x_step = width / (total_render_points as f32);

                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px((val / max_val) * chart_height)
                };

                // 1. 绘制科技感背景网格
                for i in 1..=4 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 4.0));
                    let mut path = Path::new(point(bounds.left(), y));
                    path.line_to(point(bounds.right(), y));
                    window.paint_path(path, rgba(0xffffff05));
                }

                // 2. 液体霓虹波形渲染器 (分段原子绘制，彻底杜绝三角形伪影)
                let render_wave = |data_slice: Vec<f32>, color: Rgba, window: &mut Window| {
                    if data_slice.len() < 2 {
                        return;
                    }

                    let bottom_y = bounds.bottom() - px(padding);

                    // A. 预计算平滑轨迹
                    let mut points = Vec::with_capacity(total_render_points + 1);
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

                    // B. 绘制单路径填充区域 (只要不画线，填充长路径是安全的)
                    let bl = point(bounds.left(), bottom_y);
                    let br = point(bounds.right(), bottom_y);
                    let mut fill_path = Path::new(bl);
                    for &p in &points {
                        fill_path.line_to(p);
                    }
                    fill_path.line_to(br);
                    fill_path.line_to(bl);
                    window.paint_path(
                        fill_path,
                        Rgba {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: 0.15,
                        },
                    );

                    // C. 绘制原子级“粗线” (分段绘制，每段都是独立的封闭平行四边形)
                    // 这是解决虚线感和三角形伪影的唯一完美方案
                    let thickness = px(2.0);
                    for i in 0..points.len() - 1 {
                        let p1 = points[i];
                        let p2 = points[i + 1];

                        // 增加 0.5px 的重叠以消除虚线感
                        let next_x = p2.x + px(0.5);

                        let mut segment = Path::new(p1);
                        segment.line_to(point(next_x, p2.y));
                        segment.line_to(point(next_x, p2.y + thickness));
                        segment.line_to(point(p1.x, p1.y + thickness));
                        segment.line_to(p1);

                        window.paint_path(segment, color);
                    }
                };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 下载：COSMIC 电光青
                render_wave(down_data, rgb(0x00d2ff), window);
                // 上传：COSMIC 荧光绿
                render_wave(up_data, rgb(0x2ec27e), window);
            },
        )
        .size_full()
    }
}
