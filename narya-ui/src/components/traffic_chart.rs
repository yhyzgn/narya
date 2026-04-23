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

                // 动态高度缩放
                let mut max_val: f32 = 50.0;
                for d in &data {
                    if d.up > max_val {
                        max_val = d.up;
                    }
                    if d.down > max_val {
                        max_val = d.down;
                    }
                }
                max_val *= 1.2;

                let sub_segments = 10;
                let total_points = (60 - 1) * sub_segments;
                let sub_x_step = width / (total_points as f32);

                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px((val / max_val) * chart_height)
                };

                // 1. 绘制极简背景刻度
                for i in 1..=3 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 4.0));
                    let mut grid_path = Path::new(point(bounds.left(), y));
                    grid_path.line_to(point(bounds.right(), y));
                    window.paint_path(grid_path, rgba(0xffffff08));
                }

                // 2. 液体波形渲染
                let render_layer = |data_slice: &[f32], color: Rgba, window: &mut Window| {
                    if data_slice.len() < 2 {
                        return;
                    }

                    // 生成平滑的顶部波形线点集
                    let mut curve_points = Vec::with_capacity(total_points + 1);
                    for i in 0..data_slice.len() - 1 {
                        let y1 = data_slice[i];
                        let y2 = data_slice[i + 1];
                        for s in 0..sub_segments {
                            let mu = s as f32 / sub_segments as f32;
                            let mu2 = (1.0 - (mu * PI).cos()) / 2.0; // 余弦插值
                            let y_interp = y1 * (1.0 - mu2) + y2 * mu2;
                            let x = bounds.left() + px((i * sub_segments + s) as f32 * sub_x_step);
                            curve_points.push(point(x, scale_y(y_interp)));
                        }
                    }
                    curve_points.push(point(bounds.right(), scale_y(*data_slice.last().unwrap())));

                    // A. 绘制描边 (Stroke) - 必须是独立的 Path
                    let mut line_path = Path::new(curve_points[0]);
                    for &p in curve_points.iter().skip(1) {
                        line_path.line_to(p);
                    }
                    window.paint_path(line_path, color);

                    // B. 绘制填充 (Fill) - 关键修复：基准线闭合算法
                    // 我们构建一个完全闭合的环：底左 -> 波形点集 -> 底右 -> 底左
                    let bl = point(bounds.left(), bounds.bottom() - px(padding));
                    let br = point(bounds.right(), bounds.bottom() - px(padding));

                    let mut fill_path = Path::new(bl);
                    for &p in &curve_points {
                        fill_path.line_to(p);
                    }
                    fill_path.line_to(br);
                    fill_path.line_to(bl); // 回到起点闭合

                    window.paint_path(
                        fill_path,
                        Rgba {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: 0.12,
                        },
                    );
                };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 下载层
                render_layer(&down_data, rgb(0x3584e4), window);
                // 上传层
                render_layer(&up_data, rgb(0x2ec27e), window);
            },
        )
        .size_full()
    }
}
