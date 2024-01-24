use eframe::egui;
use crate::ui::toggle_switch::toggle;
use egui::Vec2;

#[derive(Default)]
pub struct MyApp {
    dropped_files: Vec<egui::DroppedFile>,
    pick_folder: Option<String>,
    toggle_sw: bool,
    selected: f32,
    portFtp: String,
    logs: String,
}

// impl MyApp {
//     fn new(cc: &eframe::CreationContext<'_>) -> Self {
//         setup_custom_fonts(&cc.egui_ctx);
//         Self {
//             text: "Edit this text field if you want".to_owned(),
//         }
//     }
// }

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            //TODO: Set this at the contructor
            if self.selected == 0.0 {
                self.selected = 1.5;
            };

            ctx.set_pixels_per_point(self.selected);
            ctx.request_repaint();
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {

                egui::menu::bar(ui, |ui| {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    };
                    egui::ComboBox::from_label("")
                        .selected_text("Gui size")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected, 2.0, "Big");
                            ui.selectable_value(&mut self.selected, 1.5, "Medium");
                            ui.selectable_value(&mut self.selected, 1.0, "Small");
                        });

                        ui.horizontal(|ui| {
                            // ui.label("egui theme:");
                            egui::widgets::global_dark_light_mode_buttons(ui);
                        });
                });



                ui.horizontal(|ui| {
                    if ui.button("Select path…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.pick_folder = Some(path.display().to_string());
                        }
                    }

                    if let Some(pick_folder) = &self.pick_folder {
                        // ui.text_edit_singleline(&mut pick_folder);
                        ui.monospace(pick_folder);

                        // ui.horizontal(|ui| {
                        //     ui.label("Picked path:");
                        // });
                    }
                });


                ui.horizontal(|ui| {
                    ui.label("FTP");
                    // ui.text_edit_singleline(&mut self.portFtp).desired_width(10.0);

                    ui.add(
                        egui::TextEdit::singleline(&mut self.portFtp)
                            .lock_focus(true)
                            .desired_width(60   .0),
                    );

                    ui.add(toggle(&mut self.toggle_sw));
                });


            });

            egui::CentralPanel::default().show(ctx, |ui| {

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.logs)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(false)
                            .desired_width(f32::INFINITY),
                            // .layouter(&mut layouter),
                    );
                });
            });

        });

    }
}
