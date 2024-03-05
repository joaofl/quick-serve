// use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::{DragValue, TextEdit};
use egui::{Label, TextStyle};
// use log::info;
use crate::ui::toggle_switch::toggle;
use crate::servers::server::Protocol;
use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::messages::CommandMsg;

// Define a struct to hold both the sender and receiver
pub struct DefaultChannel<T> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}

impl<T: Clone> Default for DefaultChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = channel (50);
        DefaultChannel { sender, receiver }
    }
}


#[derive(Default)]
pub struct UI {
    aspect_ratio: f32,
    protocols: Vec<CommandMsg>,
    bind_ip: String,
    path: String,

    pub channel: DefaultChannel<CommandMsg>,
    pub logs: Arc<Mutex<String>>,
}

impl UI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        let mut s = UI {
            aspect_ratio: 1.8,
            bind_ip: "127.0.0.1".into(),
            path: "/tmp/".into(),
            ..Default::default()
        };

        s.protocols.push(CommandMsg::new(&Protocol::Http));
        s.protocols.push(CommandMsg::new(&Protocol::Ftp));
        s.protocols.push(CommandMsg::new(&Protocol::Tftp));
        s
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
                        ui.selectable_value(&mut self.aspect_ratio, 2.0, "L");
                        ui.selectable_value(&mut self.aspect_ratio, 1.5, "M");
                        ui.selectable_value(&mut self.aspect_ratio, 1.0, "S");
                    });

                    egui::widgets::global_dark_light_mode_switch(ui);

                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    };
            });

            // #######################################################################

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let name_label = ui.label("Directory: ");
                    ui.text_edit_singleline(&mut self.path.clone())
                        .labelled_by(name_label.id);

                    if ui.button("📂").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.path = path.display().to_string();
                        }
                    }
                    // ui.monospace(self.path.clone());
                    // ui.label(self.path.clone());
                });
            });

            // #######################################################################
            ui.add_space(5.0);
            ui.horizontal(|ui| {

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let name_label = ui.label("Bind IP: ");
                        ui.add(
                            TextEdit::singleline(&mut self.bind_ip)
                            .char_limit(15)
                            .desired_width(100.0)
                        );
                    });
                });

                // #######################################################################
                // Iterate over each known protocol, and draw its elements
                for p in self.protocols.iter_mut() {
                    ui.group(|ui| {
                        ui.add(Label::new(format!("{}", p.protocol.to_string())));
                        ui.add(DragValue::new(&mut p.port).clamp_range(1..=50000));

                        if ui.add(toggle(&mut p.start)).clicked() {

                            let mut msg = p.clone();
                            msg.bind_ip = self.bind_ip.clone();
                            msg.path = self.path.clone();

                            self.channel.sender
                                .send(msg)
                                .expect("Failed to send message");
                        }
                    });
                }
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
