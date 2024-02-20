// use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::{DragValue, TextEdit};
use egui::{Label, TextStyle};
// use log::info;
use crate::ui::toggle_switch::toggle;
use crate::servers::server::Protocol;
use tokio::sync::broadcast::{channel, Receiver, Sender};


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

#[derive(Clone, Debug, Default)]
pub struct UIProtocolElements {
    pub toggle: bool, 
    pub port: u16,
    pub name: String,
    // pub bind_ip: String,
    // pub path: PathBuf,
}

impl UIProtocolElements {
    fn new(prot: &Protocol) -> Self {
        Self {
            toggle: false, 
            port: prot.get_default_port(),
            name: prot.to_string().into(),
            // bind_ip,
            // path
        }
    }
}


pub type UIEvent = (UIProtocolElements, String, String);

#[derive(Default)]
pub struct UI {
    path: String,
    aspect_ratio: f32,

    protocols: Vec<UIProtocolElements>,

    bind_ip: String,
    // path: PathBuf,

    pub channel: DefaultChannel<UIEvent>,
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

        s.protocols.push(UIProtocolElements::new(&Protocol::Http));
        s.protocols.push(UIProtocolElements::new(&Protocol::Ftp));
        s.protocols.push(UIProtocolElements::new(&Protocol::Tftp));
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
            ui.add_space(5.0);

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
            ui.add_space(5.0);
            ui.horizontal(|ui| {

                // Iterate over each known protocol, and draw its elements
                for p in self.protocols.iter_mut() {

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(Label::new(format!("{}", p.name)));
                            ui.add(DragValue::new(&mut p.port).clamp_range(1..=50000));

                            if ui.add(toggle(&mut p.toggle)).clicked() {

                                let msg = (p.clone(), self.bind_ip.clone(), self.path.clone());
                                self.channel.sender
                                    .send(msg)
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
