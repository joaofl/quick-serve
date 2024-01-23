use eframe::egui;
use crate::ui::toggle_switch::toggle;


#[derive(Default)]
pub struct MyApp {
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
    toggle_sw: bool,
    selected: i32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ctx.set_pixels_per_point(1.3);
            // ctx.set
            ctx.request_repaint();
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    };
                    egui::ComboBox::from_label("")
                        .selected_text("Text Size")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected, 1, "Big");
                            ui.selectable_value(&mut self.selected, 2, "Medium");
                            ui.selectable_value(&mut self.selected, 3, "Small");
                        });
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                // Large button text:
                if self.selected == 1 {
                    ctx.style_mut(|style| {
                        style.text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(20.0, egui::FontFamily::Proportional),
                        );
                    });
                } else if self.selected == 2 {
                    ctx.style_mut(|style| {
                        style.text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(15.0, egui::FontFamily::Proportional),
                        );
                    });
                } else {
                    ctx.style_mut(|style| {
                        style.text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(10.0, egui::FontFamily::Proportional),
                        );
                    });
                }

                // ui.label("Drag-and-drop files onto the window!");
    
                if ui.button("Open file…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.picked_path = Some(path.display().to_string());
                    }
                }
    
                ui.add(toggle(&mut self.toggle_sw));
    
                if let Some(picked_path) = &self.picked_path {
                    ui.horizontal(|ui| {
                        ui.label("Picked path:");
                        ui.monospace(picked_path);
                    });
                }

                ui.heading("Text:");
                ui.heading("--------------------");
                ui.label("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod");

            });


            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }

                        ui.label(info);
                    }
                });
            }
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files = i.raw.dropped_files.clone();
            }
        });
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}
