use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::DragValue;
use egui::{Label, TextStyle, FontId, Color32};
use log::info;
use crate::ui::toggle_switch::toggle;
use crate::servers::server::Protocol;

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};


// Define a struct to hold both the sender and receiver
pub struct DefaultChannel<T> {
    pub sender: UnboundedSender<T>,
    pub receiver: UnboundedReceiver<T>,
}

impl<T> Default for DefaultChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = unbounded_channel();
        DefaultChannel { sender, receiver }
    }
}


#[derive(Default)]
pub struct UI {
    pick_folder: Option<String>,
    aspect_ratio: f32,

    toggle_sw_ftp: bool,
    port_ftp: u16,

    toggle_sw_tftp: bool,
    port_tftp: u16,

    toggle_sw_http: bool,
    port_http: u16,

    pub channel: DefaultChannel<String>,
    pub logs: Arc<Mutex<String>>,
}

impl UI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        UI {
            aspect_ratio: 1.8,
            port_ftp: Protocol::get_default_port(&Protocol::Ftp),
            port_tftp: Protocol::get_default_port(&Protocol::Tftp),
            port_http: Protocol::get_default_port(&Protocol::Http),
            ..Default::default()
        }
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ctx.set_pixels_per_point(self.aspect_ratio);
            ctx.request_repaint();

            egui::menu::bar(ui, |ui| {
                egui::ComboBox::from_label("")
                    .selected_text("Size")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.aspect_ratio, 2.0, "Big");
                        ui.selectable_value(&mut self.aspect_ratio, 1.5, "Medium");
                        ui.selectable_value(&mut self.aspect_ratio, 1.0, "Small");
                    });

                    egui::widgets::global_dark_light_mode_switch(ui);

                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    };
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
                        // ui.add(toggle(&mut self.toggle_sw_http));

                        if ui.toggle_value(&mut self.toggle_sw_http, "HTTP").clicked() {
                            self.channel.sender.send("Woooooo2222".to_string());
                        }
                    });
                });

            });

            // #######################################################################
            ui.add_space(5.0);
            ui.separator();

            egui::ScrollArea::both()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                        |ui| {
                            // Acquire the lock
                            let logs = self.logs.lock().unwrap().clone();
                            ui.label( egui::RichText::new(logs).text_style(TextStyle::Monospace) );
                        },
                    );
                });
        });

    }
}
