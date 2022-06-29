use egui::plot::{Line, Plot, Value, Values};
use notan::{
    egui::{self, *},
    prelude::Graphics,
};

use crate::{
    update,
    utils::{
        disp_col, disp_col_norm, highlight_bleed, highlight_semitrans, ExtendedImageInfo, ImageExt,
        OculanteState,
    },
};

pub fn info_ui(ctx: &Context, state: &mut OculanteState, gfx: &mut Graphics) {
    if state.info_enabled {
        egui::SidePanel::left("side_panel").show(&ctx, |ui| {
            ui.label(format!(
                "Size: {}x{}",
                state.image_dimension.0, state.image_dimension.1
            ));

            if let Some(path) = &state.current_path {
                ui.label(format!("Path: {}", path.display()));
            }

            if let Some(img) = &state.current_image {
                if let Some(p) = img.get_pixel_checked(
                    state.cursor_relative.x as u32,
                    state.cursor_relative.y as u32,
                ) {
                    state.sampled_color = [p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32];
                }
            }

            if let Some(texture) = &state.current_texture {
                ui.horizontal(|ui| {
                    ui.label("🌗 RGBA");
                    ui.label(
                        RichText::new(format!("{}", disp_col(state.sampled_color)))
                            .monospace()
                            .background_color(Color32::from_rgba_unmultiplied(255, 255, 255, 6)),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("🌗 RGBA");
                    ui.label(
                        RichText::new(format!("{}", disp_col_norm(state.sampled_color, 255.)))
                            .monospace()
                            .background_color(Color32::from_rgba_unmultiplied(255, 255, 255, 6)),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("⊞ Pos");
                    ui.label(
                        RichText::new(format!(
                            "{:.0},{:.0}",
                            state.cursor_relative.x, state.cursor_relative.y
                        ))
                        .monospace()
                        .background_color(Color32::from_rgba_unmultiplied(255, 255, 255, 6)),
                    );
                });

                // texture.
                let tex_id = gfx.egui_register_texture(&texture);

                // width of image widget
                let desired_width = 200.;

                let scale = (desired_width / 8.) / texture.size().0;
                let img_size = egui::Vec2::new(desired_width, desired_width);

                let uv_center = (
                    state.cursor_relative.x / state.image_dimension.0 as f32,
                    state.cursor_relative.y / state.image_dimension.1 as f32,
                );

                ui.horizontal(|ui| {
                    ui.label(" UV");
                    ui.label(
                        RichText::new(format!("{:.3},{:.3}", uv_center.0, uv_center.1))
                            .monospace()
                            .background_color(Color32::from_rgba_unmultiplied(255, 255, 255, 6)),
                    );
                });

                // make sure aspect ratio is compensated for the square preview
                let ratio = texture.size().0 / texture.size().1;
                let uv_size = (scale, scale * ratio);
                let x = ui
                    .add(
                        egui::Image::new(tex_id, img_size).uv(egui::Rect::from_x_y_ranges(
                            uv_center.0 - uv_size.0..=uv_center.0 + uv_size.0,
                            uv_center.1 - uv_size.1..=uv_center.1 + uv_size.1,
                        )), // .bg_fill(egui::Color32::RED),
                    )
                    .rect;

                let stroke_color = Color32::from_white_alpha(240);
                let bg_color = Color32::BLACK.linear_multiply(0.5);
                ui.painter_at(x).line_segment(
                    [x.center_bottom(), x.center_top()],
                    Stroke::new(4., bg_color),
                );
                ui.painter_at(x).line_segment(
                    [x.left_center(), x.right_center()],
                    Stroke::new(4., bg_color),
                );
                ui.painter_at(x).line_segment(
                    [x.center_bottom(), x.center_top()],
                    Stroke::new(1., stroke_color),
                );
                ui.painter_at(x).line_segment(
                    [x.left_center(), x.right_center()],
                    Stroke::new(1., stroke_color),
                );
                // ui.image(tex_id, img_size);
            }

            ui.vertical_centered_justified(|ui| {
                if let Some(img) = &state.current_image {
                    if ui
                        .button("Calculate extended info")
                        .on_hover_text("Count unique colors in image")
                        .clicked()
                    {
                        state.image_info = Some(ExtendedImageInfo::from_image(img));
                    }
                    if ui
                        .button("Show alpha bleed")
                        .on_hover_text("Highlight pixels with zero alpha and color information")
                        .clicked()
                    {
                        state.current_texture = highlight_bleed(img).to_texture(gfx);
                    }
                    if ui
                        .button("Show semi-transparent pixels")
                        .on_hover_text(
                            "Highlight pixels that are neither fully opaque not fully transparent",
                        )
                        .clicked()
                    {
                        state.current_texture = highlight_semitrans(img).to_texture(gfx);
                    }
                    if ui.button("Reset image").clicked() {
                        state.current_texture = img.to_texture(gfx);
                    }

                    ui.add(egui::Slider::new(&mut state.tiling, 1..=10).text("Image tiling"));
                }
            });

            advanced_ui(ui, state);
        });
    }
}

pub fn settings_ui(ctx: &Context, state: &mut OculanteState) {
    if state.settings_enabled {
        egui::Window::new("Settings")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .default_width(400.)
            // .title_bar(false)
            .show(&ctx, |ui| {
                if ui.button("Check for updates").clicked() {
                    state.message = Some("Checking for updates...".into());
                    update::update(Some(state.message_channel.0.clone()));
                    state.settings_enabled = false;
                }

                if ui.button("Close").clicked() {
                    state.settings_enabled = false;
                }
            });
    }
}

pub fn advanced_ui(ui: &mut Ui, state: &mut OculanteState) {
    if let Some(info) = &state.image_info {
        ui.label(format!("Number of colors: {}", info.num_colors));
        ui.label(format!(
            "Fully transparent: {:.2}%",
            (info.num_transparent_pixels as f32 / info.num_pixels as f32) * 100.
        ));
        ui.label(format!("Pixels: {}", info.num_pixels));

        let grey_vals = Line::new(Values::from_values_iter(
            info.grey_histogram
                .iter()
                .map(|(k, v)| Value::new(*k as f64, *v as f64)),
        ))
        .color(Color32::GRAY);

        let red_vals = Line::new(Values::from_values_iter(
            info.red_histogram
                .iter()
                .map(|(k, v)| Value::new(*k as f64, *v as f64)),
        ))
        .fill(0.)
        .color(Color32::RED);

        let green_vals = Line::new(Values::from_values_iter(
            info.green_histogram
                .iter()
                .map(|(k, v)| Value::new(*k as f64, *v as f64)),
        ))
        .fill(0.)
        .color(Color32::GREEN);

        let blue_vals = Line::new(Values::from_values_iter(
            info.blue_histogram
                .iter()
                .map(|(k, v)| Value::new(*k as f64, *v as f64)),
        ))
        .fill(0.)
        .color(Color32::BLUE);

        ui.label("Histogram");
        Plot::new("my_plot")
            .allow_zoom(false)
            .allow_drag(false)
            .show(ui, |plot_ui| {
                plot_ui.line(grey_vals);
                plot_ui.line(red_vals);
                plot_ui.line(green_vals);
                plot_ui.line(blue_vals);
            });
    }
}

pub fn edit_ui(ctx: &Context, state: &mut OculanteState, gfx: &mut Graphics) {
    //    ui.color_edit_button_rgb(rgb)
    if !state.edit_enabled {
        return;
    }

    egui::SidePanel::right("edit_panel")
        .min_width(300.)
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(img) = &mut state.current_image {
                    if ui
                        .button("⟳")
                        .on_hover_text("Rotate 90 deg right")
                        .clicked()
                    {
                        *img = image::imageops::rotate90(img);
                        state.current_texture = img.to_texture(gfx);
                    }
                    if ui.button("⟲").on_hover_text("Rotate 90 deg left").clicked() {
                        *img = image::imageops::rotate270(img);
                        state.current_texture = img.to_texture(gfx);
                    }
                }
            });

            // Blur
            if let Some(img) = &mut state.current_image {
                let response = ui
                    .add(egui::Slider::new(&mut state.edit_state.blur, 0.0..=10.).text("💧 blur"));
                if response.changed() {
                    let img_blurred = image::imageops::blur(img, state.edit_state.blur);
                    state.current_texture = img_blurred.to_texture(gfx);
                    state.edit_state.result = img_blurred;
                }
                if response.drag_released() {
                    *img = state.edit_state.result.clone();
                }
            }

            // // Unsharp
            // if let Some(img) = &mut state.current_image {
            //     let response_amt = ui.add(
            //         egui::Slider::new(&mut state.edit_state.unsharpen, 0.0..=20.)
            //             .text("💧 unsharpen amt"),
            //     );
            //     let response_thresh = ui.add(
            //         egui::Slider::new(&mut state.edit_state.unsharpen_threshold, 0..=20)
            //             .text("💧 unsharpen threshold"),
            //     );
            //     if response_amt.changed() {
            //         let img_blurred = image::imageops::unsharpen(
            //             img,
            //             state.edit_state.unsharpen,
            //             state.edit_state.unsharpen_threshold,
            //         );
            //         state.current_texture = img_blurred.to_texture(gfx);
            //         state.edit_state.result = img_blurred;
            //     }
            //     if response_amt.drag_released() {
            //         *img = state.edit_state.result.clone();
            //     }

            //     if response_thresh.changed() {
            //         let img_blurred = image::imageops::unsharpen(
            //             img,
            //             state.edit_state.unsharpen,
            //             state.edit_state.unsharpen_threshold,
            //         );
            //         state.current_texture = img_blurred.to_texture(gfx);
            //         state.edit_state.result = img_blurred;
            //     }
            //     if response_thresh.drag_released() {
            //         *img = state.edit_state.result.clone();
            //     }
            // }

            // Contrast
            if let Some(img) = &mut state.current_image {
                let response = ui.add(
                    egui::Slider::new(&mut state.edit_state.contrast, -100.0..=100.)
                        .text("◑ Contrast"),
                );
                if response.changed() {
                    let img_contrasted = image::imageops::contrast(img, state.edit_state.contrast);
                    state.current_texture = img_contrasted.to_texture(gfx);
                    state.edit_state.result = img_contrasted;
                }
                if response.drag_released() {
                    *img = state.edit_state.result.clone();
                }
            }

                        // Contrast
                        if let Some(img) = &mut state.current_image {
                            let response = ui.add(
                                egui::Slider::new(&mut state.edit_state.brightness, -255..=255)
                                    .text("☀ Brightness"),
                            );
                            if response.changed() {
                                let img_brightness = image::imageops::brighten(img, state.edit_state.brightness);
                                state.current_texture = img_brightness.to_texture(gfx);
                                state.edit_state.result = img_brightness;
                            }
                            if response.drag_released() {
                                *img = state.edit_state.result.clone();
                            }
                        }

            ui.horizontal(|ui| {
                ui.label("Mult color");
                if let Some(img) = &mut state.current_image {
                    let response = ui.color_edit_button_rgb(&mut state.edit_state.color);

                    if response.changed() {
                        let mut e = img.clone();

                        for p in e.pixels_mut() {
                            p[0] = (p[0] as f32 * state.edit_state.color[0]) as u8;
                            p[1] = (p[1] as f32 * state.edit_state.color[1]) as u8;
                            p[2] = (p[2] as f32 * state.edit_state.color[2]) as u8;
                        }
                        state.current_texture = e.to_texture(gfx);
                        state.edit_state.result = e;
                    }

                    if ui.button("Apply").clicked() {
                        // dbg!("rels clr");
                        *img = state.edit_state.result.clone();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Add  color");
                if let Some(img) = &mut state.current_image {
                    if ui
                        .color_edit_button_rgb(&mut state.edit_state.color)
                        .changed()
                    {
                        let mut e = img.clone();

                        for p in e.pixels_mut() {
                            p[0] = (p[0] as f32 + state.edit_state.color[0] * 255.) as u8;
                            p[1] = (p[1] as f32 + state.edit_state.color[1] * 255.) as u8;
                            p[2] = (p[2] as f32 + state.edit_state.color[2] * 255.) as u8;
                        }
                        state.current_texture = e.to_texture(gfx);
                        state.edit_state.result = e;
                    }
                    if ui.button("Apply").clicked() {
                        *img = state.edit_state.result.clone();
                    }
                }
            });
            ui.horizontal(|ui| {
                if let Some(img) = &mut state.current_image {
                    if ui.button("Invert").clicked() {
                        image::imageops::invert(img);
                        state.current_texture = img.to_texture(gfx);
                    }
                }
            });
            ui.horizontal(|ui| {
                if let Some(img) = &mut state.current_image {
                    if ui.button("↔ Flip horizontally").clicked() {
                        *img = image::imageops::flip_horizontal(img);
                        state.current_texture = img.to_texture(gfx);
                    }

                    if ui.button("↕ Flip vertically").clicked() {
                        *img = image::imageops::flip_vertical(img);
                        state.current_texture = img.to_texture(gfx);
                    }
                }
            });

            if let Some(img) = &mut state.current_image {
                if let Some(path) = &state.current_path {
                    if ui.button("💾 Save").clicked() {
                        let _ = img.save(path);
                    }
                }
            }
        });
}

pub fn tooltip(r: Response, tooltip: &str, hotkey: &str, ui: &mut Ui) -> Response {
    r.on_hover_ui(|ui| {
        ui.horizontal(|ui| {
            ui.label(tooltip);
            ui.label(
                RichText::new(hotkey)
                    .monospace()
                    .color(Color32::WHITE)
                    .background_color(ui.style().visuals.selection.bg_fill),
            );
        });
    })
}

pub fn unframed_button(text: impl Into<WidgetText>, ui: &mut Ui) -> Response {
    ui.add(egui::Button::new(text).frame(false))
}
