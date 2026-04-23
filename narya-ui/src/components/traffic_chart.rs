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
                max_val *= 1.2;

                let x_step = width / (60.0 - 1.0);
                let scale_y = |val: f32| {
                    bounds.top() + px(chart_height + padding) - px(val / max_val * chart_height)
                };

                // 1. 绘制极简背景网格 (GNOME Monitor 风格)
                for i in 0..=2 {
                    let y = bounds.top() + px(padding + (i as f32 * chart_height / 2.0));
                    let mut path = Path::new(point(bounds.left(), y));
                    path.line_to(point(bounds.right(), y));
                    window.paint_path(path, rgba(0xffffff0a)); // 极淡的白色参考线
                }

                // 2. 极致平滑绘制算法 (中点平滑法)
                let draw_smooth_wave =
                    |data_slice: Vec<f32>,
                     stroke_color: Rgba,
                     fill_color: Rgba,
                     window: &mut Window| {
                        if data_slice.len() < 3 {
                            return;
                        }

                        let get_pt = |i: usize| {
                            let x = bounds.left() + px(i as f32 * x_step);
                            let y = scale_y(data_slice[i]);
                            point(x, y)
                        };

                        let mut path = Path::new(get_pt(0));

                        // 算法：对于每两个点，找到它们的中点作为二次曲线的终点
                        // 原始点作为控制点。这能产生非常圆润且无折角的波形。
                        for i in 1..data_slice.len() - 1 {
                            let p_curr = get_pt(i);
                            let p_next = get_pt(i + 1);
                            let mid =
                                point((p_curr.x + p_next.x) / 2.0, (p_curr.y + p_next.y) / 2.0);

                            path.curve_to(mid, p_curr);
                        }

                        // 闭合到最后一个点
                        path.line_to(get_pt(data_slice.len() - 1));

                        // 绘制填充阴影 (Area)
                        let mut fill_path = path.clone();
                        fill_path.line_to(point(bounds.right(), bounds.bottom() - px(padding)));
                        fill_path.line_to(point(bounds.left(), bounds.bottom() - px(padding)));
                        window.paint_path(fill_path, fill_color);

                        // 绘制高亮描边 (COSMIC Style Stroke)
                        window.paint_path(path, stroke_color);
                    };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 渲染下载：COSMIC 鲜蓝色
                draw_smooth_wave(
                    down_data,
                    rgb(0x3584e4),    // GNOME/COSMIC Blue
                    rgba(0x3584e422), // 底部阴影
                    window,
                );

                // 渲染上传：COSMIC 鲜绿色
                draw_smooth_wave(
                    up_data,
                    rgb(0x2ec27e), // GNOME/COSMIC Green
                    rgba(0x2ec27e22),
                    window,
                );
            },
        )
        .size_full()
    }
}
