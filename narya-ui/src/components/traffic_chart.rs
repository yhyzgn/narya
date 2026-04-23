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
                max_val *= 1.25;

                // 算法：像素切片。弃用 Path 系统，使用 PaintQuad 模拟曲线。
                let render_pixel_liquid = |data_slice: &[f32], color: Rgba, window: &mut Window| {
                    if data_slice.len() < 2 {
                        return;
                    }

                    // X 轴分辨率设定为 width，即每像素一根垂直条
                    let step_count = (width as usize).max(100);
                    let px_step = width / step_count as f32;
                    let samples_per_point = step_count as f32 / 59.0;

                    for i in 0..step_count {
                        let sample_idx_float = i as f32 / samples_per_point;
                        let idx = sample_idx_float.floor() as usize;
                        let next_idx = (idx + 1).min(data_slice.len() - 1);
                        let mu = sample_idx_float.fract();

                        // 余弦插值
                        let f = (1.0 - (mu * PI).cos()) / 2.0;
                        let y_val = data_slice[idx] * (1.0 - f) + data_slice[next_idx] * f;

                        let x = bounds.left() + px(i as f32 * px_step);
                        let top_y = bounds.top() + px(chart_height + padding)
                            - px((y_val / max_val) * chart_height);
                        let bar_height = (bounds.bottom() - px(padding)) - top_y;

                        // 仅当高度大于 0 时渲染
                        if bar_height > px(0.0) {
                            // 1. 区域填充块 (15% 透明度)
                            window.paint_quad(PaintQuad {
                                bounds: Bounds {
                                    origin: point(x, top_y),
                                    size: size(px(px_step + 0.3), bar_height),
                                },
                                background: Rgba {
                                    r: color.r,
                                    g: color.g,
                                    b: color.b,
                                    a: 0.15,
                                }
                                .into(),
                                corner_radii: Default::default(),
                                border_widths: Default::default(),
                                border_color: Default::default(),
                                border_style: Default::default(),
                            });

                            // 2. 顶部亮线块 (2px 高度)
                            window.paint_quad(PaintQuad {
                                bounds: Bounds {
                                    origin: point(x, top_y),
                                    size: size(px(px_step + 0.3), px(1.8)),
                                },
                                background: color.into(),
                                corner_radii: Default::default(),
                                border_widths: Default::default(),
                                border_color: Default::default(),
                                border_style: Default::default(),
                            });
                        }
                    }
                };

                let up_data: Vec<f32> = data.iter().map(|d| d.up).collect();
                let down_data: Vec<f32> = data.iter().map(|d| d.down).collect();

                // 渲染下载 (电光青)
                render_pixel_liquid(&down_data, rgb(0x00d2ff), window);
                // 渲染上传 (荧光绿)
                render_pixel_liquid(&up_data, rgb(0x00ffaa), window);
            },
        )
        .size_full()
    }
}
