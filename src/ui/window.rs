use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::DragValue;
use egui::{Label, TextStyle};
// use log::info;
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

// #[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(Default)]
pub struct ProtocolUI {
    pub toggle: bool, 
    pub port: u16,
    pub name: String,
}

impl ProtocolUI {
    fn new(prot: &Protocol) -> Self {
        Self {
            toggle: false, 
            port: prot.get_default_port(),
            name: prot.to_string().into(),
        }
    }
}


#[derive(Default)]
pub struct UI {
    pick_folder: Option<String>,
    aspect_ratio: f32,

    protocols: Vec<ProtocolUI>,

    pub channel: DefaultChannel<ProtocolUI>,
    pub logs: Arc<Mutex<String>>,
}

impl UI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        let mut s = UI {
            aspect_ratio: 1.8,
            ..Default::default()
        };

        s.protocols.push(ProtocolUI::new(&Protocol::Http));
        s.protocols.push(ProtocolUI::new(&Protocol::Ftp));
        s.protocols.push(ProtocolUI::new(&Protocol::Tftp));
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

                // Iterate over each known protocol, and draw its elements
                for p in self.protocols.iter_mut() {

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(Label::new(format!("{}", p.name)));
                            ui.add(DragValue::new(&mut p.port).clamp_range(1..=50000));

                            if ui.add(toggle(&mut p.toggle)).clicked() {
                                self.channel.sender
                                    .send(p.clone())
                                    .expect("Failed to send message");
                            }
                        });
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
