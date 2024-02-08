use std::default;
use std::sync::Arc;

use eframe::egui;
use egui::DragValue;
use crate::ui::toggle_switch::toggle;
use crate::servers::server::Protocol;
use tokio::sync::broadcast;



use crate::ui::listener;

use super::listener::Logger;


pub struct UI {
    pub sender: broadcast::Sender<String>,

    pick_folder: Option<String>,
    aspect_ratio: f32,
    toggle_sw_ftp: bool,
    port_ftp: u16,

    toggle_sw_tftp: bool,
    port_tftp: u16,

    toggle_sw_http: bool,
    port_http: u16,

    logs: String,
}

impl Default for UI {
    fn default() -> Self {
        UI {
            logs: "Edit this text field if you want".to_owned(),
            pick_folder: Default::default(),
            aspect_ratio: 1.8,
            sender: broadcast::channel(10).0,

            port_ftp: Protocol::get_default_port(&Protocol::Ftp),
            toggle_sw_ftp: Default::default(),

            port_tftp: Protocol::get_default_port(&Protocol::Tftp),
            toggle_sw_tftp: Default::default(),

            port_http: Protocol::get_default_port(&Protocol::Http),
            toggle_sw_http: Default::default(),
        }
    }
}


impl UI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let d = UI::default();
        let e = Arc::new(&d);

        // UI::logger()
        // d.logger();
        // d
        d
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ctx.set_pixels_per_point(self.aspect_ratio);
            ctx.request_repaint();

            egui::menu::bar(ui, |ui| {
                if ui.button("About").clicked() {
                    std::process::exit(0);
                };
                egui::ComboBox::from_label("")
                    .selected_text("Size")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.aspect_ratio, 2.0, "Big");
                        ui.selectable_value(&mut self.aspect_ratio, 1.5, "Medium");
                        ui.selectable_value(&mut self.aspect_ratio, 1.0, "Small");
                    });

                    egui::widgets::global_dark_light_mode_switch(ui);
            });

            // #######################################################################

            ui.add_space(5.0);

            ui.group(|ui| {
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
            });

            // #######################################################################
            ui.add_space(5.0);


            ui.horizontal(|ui| {

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(Protocol::to_string(&Protocol::Tftp));
                        ui.add(DragValue::new(&mut self.port_tftp).clamp_range(1..=50000));
                        ui.add(toggle(&mut self.toggle_sw_tftp));
                    });
                });

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(Protocol::to_string(&Protocol::Ftp));
                        ui.add(DragValue::new(&mut self.port_ftp).clamp_range(1..=50000));
                        ui.add(toggle(&mut self.toggle_sw_ftp));
                    });
                });

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(Protocol::to_string(&Protocol::Http));
                        ui.add(DragValue::new(&mut self.port_http).clamp_range(1..=50000));
                        ui.add(toggle(&mut self.toggle_sw_http));
                    });
                });

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
                            ui.label(&self.logs);
                        },
                    );
                });
        });

    }
}
