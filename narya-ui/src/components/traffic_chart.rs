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

                // 极致平滑：每 0.5 像素绘制一个点，物理重叠消除虚线
                let render_vibrant_wave = |data_slice: &[f32], color: Rgba, window: &mut Window| {
                    if data_slice.len() < 2 {
                        return;
                    }

                    let step_size = 0.5;
                    let total_steps = (width / step_size) as usize;
                    let samples_per_point = total_steps as f32 / 59.0;
                    let bottom_y = bounds.bottom() - px(padding);

                    for i in 0..total_steps {
                        let sample_idx_float = i as f32 / samples_per_point;
                        let idx = sample_idx_float.floor() as usize;
                        let next_idx = (idx + 1).min(data_slice.len() - 1);
                        let mu = sample_idx_float.fract();

                        let f = (1.0 - (mu * PI).cos()) / 2.0;
                        let y_val = data_slice[idx] * (1.0 - f) + data_slice[next_idx] * f;

                        let x = bounds.left() + px(i as f32 * step_size);
                        let top_y = bounds.top() + px(chart_height + padding)
                            - px((y_val / max_val) * chart_height);
                        let slice_w = px(step_size + 0.2);

                        // 1. 区域填充 (10% 透明)
                        if bottom_y > top_y {
                            window.paint_quad(PaintQuad {
                                bounds: Bounds {
                                    origin: point(x, top_y),
                                    size: size(slice_w, bottom_y - top_y),
                                },
                                background: Rgba {
                                    r: color.r,
                                    g: color.g,
                                    b: color.b,
                                    a: 0.1,
                                }
                                .into(),
                                corner_radii: Default::default(),
                                border_widths: Default::default(),
                                border_color: Default::default(),
                                border_style: Default::default(),
                            });
                        }

                        // 2. 霓虹光晕层 (4.5px, 25% 透明)
                        window.paint_quad(PaintQuad {
                            bounds: Bounds {
                                origin: point(x, top_y - px(1.0)),
                                size: size(slice_w, px(4.5)),
                            },
                            background: Rgba {
                                r: color.r,
                                g: color.g,
                                b: color.b,
                                a: 0.25,
                            }
                            .into(),
                            corner_radii: Default::default(),
                            border_widths: Default::default(),
                            border_color: Default::default(),
                            border_style: Default::default(),
                        });

                        // 3. 核心亮线层 (2.0px, 100% 不透明)
                        window.paint_quad(PaintQuad {
                            bounds: Bounds {
                                origin: point(x, top_y),
                                size: size(slice_w, px(2.0)),
                            },
                            background: color.into(),
                            corner_radii: Default::default(),
                            border_widths: Default::default(),
                            border_color: Default::default(),
                            border_style: Default::default(),
                        });
                    }
                };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 下载：鲜艳蓝 (#007AFF)
                render_vibrant_wave(&down_data, rgb(0x007aff), window);
                // 上传：鲜艳绿 (#30D158)
                render_vibrant_wave(&up_data, rgb(0x30d158), window);
            },
        )
        .size_full()
    }
}
