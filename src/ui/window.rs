use eframe::egui;
use egui::DragValue;
use crate::ui::toggle_switch::toggle;

#[derive(Default)]
pub struct UI {
    dropped_files: Vec<egui::DroppedFile>,
    pick_folder: Option<String>,
    toggle_sw: bool,
    selected: f32,
    port_ftp: u32,
    logs: String,
}

impl UI {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            logs: "Edit this text field if you want".to_owned(),
            selected: 1.8,
            port_ftp: 2121,
            ..Default::default()
        }
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ctx.set_pixels_per_point(self.selected);
            ctx.request_repaint();

            egui::menu::bar(ui, |ui| {
                if ui.button("About").clicked() {
                    std::process::exit(0);
                };
                egui::ComboBox::from_label("")
                    .selected_text("Size")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected, 2.0, "Big");
                        ui.selectable_value(&mut self.selected, 1.5, "Medium");
                        ui.selectable_value(&mut self.selected, 1.0, "Small");
                    });

                    egui::widgets::global_dark_light_mode_switch(ui);
            });

            // #######################################################################

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("Select path…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.pick_folder = Some(path.display().to_string());
                    }
                }

                if let Some(pick_folder) = &self.pick_folder {
                    // ui.text_edit_singleline(&mut pick_folder);
                    ui.monospace(pick_folder);
                }
            });

            // #######################################################################
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("FTP");
                ui.add(DragValue::new(&mut self.port_ftp).clamp_range(1..=50000));
                ui.add(toggle(&mut self.toggle_sw));
            });

            // #######################################################################
            ui.add_space(5.0);
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                        |ui| {
                            ui.label("Some text ".repeat(1000),);
                        },
                    );
                });
        });

    }
}
