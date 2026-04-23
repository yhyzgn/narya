use crate::models::traffic::SharedTrafficStore;
use gpui::*;

pub struct TrafficChart;

impl TrafficChart {
    pub fn render(
        store: SharedTrafficStore,
        _cx: &mut Context<crate::Workspace>,
    ) -> impl IntoElement {
        canvas(
            move |bounds, cx, _| {
                let store_read = store.read();
                let data = store_read.get_history();
                let max_samples = 60.0;
                drop(store_read);

                if data.is_empty() {
                    return;
                }

                // GPUI Pixels 转换
                let width: f32 = bounds.size.width.into();
                let height: f32 = bounds.size.height.into();
                let padding = 10.0;
                let chart_height = height - padding * 2.0;

                let max_val: f32 = data
                    .iter()
                    .map(|d| d.up.max(d.down))
                    .fold(100.0f32, |a, b| a.max(b))
                    * 1.2;

                let x_step = width / (max_samples - 1.0);
                let scale_y = |val: f32| height - padding - (val / max_val * chart_height);

                // 1. 绘制背景网格
                for i in 0..=4 {
                    let y = padding + (i as f32 * chart_height / 4.0);
                    let mut path = Path::new(point(px(0.0), px(y)));
                    path.line_to(point(px(width), px(y)));
                    cx.paint_path(path, rgb(0x222222));
                }

                // 2. 绘制平滑波形 (COSMIC 风格)
                let mut draw_smooth_area =
                    |data_slice: &[f32], stroke_color: Rgba, fill_color: Rgba| {
                        if data_slice.len() < 2 {
                            return;
                        }

                        let start_y = scale_y(data_slice[0]);
                        let mut path = Path::new(point(px(0.0), px(start_y)));

                        for i in 1..data_slice.len() {
                            let x0 = (i as f32 - 1.0) * x_step;
                            let x1 = i as f32 * x_step;
                            let y0 = scale_y(data_slice[i - 1]);
                            let y1 = scale_y(data_slice[i]);

                            let mid_x = (x0 + x1) / 2.0;
                            path.curve_to(
                                point(px(mid_x), px((y0 + y1) / 2.0)),
                                point(px(x0), px(y0)),
                            );
                            path.curve_to(point(px(x1), px(y1)), point(px(mid_x), px(y1)));
                        }

                        // 绘制填充 (Area)
                        let mut fill_path = path.clone();
                        fill_path.line_to(point(px(width), px(height - padding)));
                        fill_path.line_to(point(px(0.0), px(height - padding)));
                        cx.paint_path(fill_path, fill_color);

                        // 绘制明亮描边 (Line)
                        cx.paint_path(path, stroke_color);
                    };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 渲染下载 (蓝色)
                draw_smooth_area(&down_data, rgb(0x1677ff), rgba(0x1677ff1a));
                // 渲染上传 (绿色)
                draw_smooth_area(&up_data, rgb(0x52c41a), rgba(0x52c41a15));
            },
            |_, _, _, _| {},
        )
        .size_full()
    }
}
