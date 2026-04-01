use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::{DragValue, TextEdit};
use egui::{Label, TextStyle};
use crate::ui::toggle_switch::toggle;
use crate::{DefaultChannel, PROTOCOL_LIST};

use crate::messages::CommandMsg;

pub struct UI {
    protocols: Vec<CommandMsg>,
    bind_ip: String,
    path: String,

    pub channel: DefaultChannel<CommandMsg>,
    pub logs: Arc<Mutex<Vec<String>>>,
}

impl UI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = UI {
            protocols: Vec::new(),
            bind_ip: "127.0.0.1".into(),
            path: "/tmp/".into(),
            channel: Default::default(),
            logs: Default::default(),
        };
        for protocol in PROTOCOL_LIST {
            s.protocols.push(CommandMsg::new(protocol));
        }
        s
    }
}

impl eframe::App for UI {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("View", |ui| {
                    let zoom = ui.ctx().zoom_factor();
                    ui.horizontal(|ui| {
                        if ui.button("−").clicked() {
                            ui.ctx().set_zoom_factor((zoom - 0.1).max(0.5));
                            ui.close();
                        }
                        ui.label(format!("{:.0}%", zoom * 100.0));
                        if ui.button("+").clicked() {
                            ui.ctx().set_zoom_factor((zoom + 0.1).min(2.0));
                            ui.close();
                        }
                    });

                    ui.separator();

                    let theme = ui.ctx().theme();
                    let label = if theme == egui::Theme::Dark { "☀  Light mode" } else { "🌙  Dark mode" };
                    if ui.button(label).clicked() {
                        ui.ctx().set_theme(if theme == egui::Theme::Dark {
                            egui::Theme::Light
                        } else {
                            egui::Theme::Dark
                        });
                        ui.close();
                    }
                });
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
                        let _name_label = ui.label("Bind IP: ");
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
                        

                        // Some protocols do not allow changing ports (and may be set to 0)
                        // so we only show the port field if it is not 0
                        if p.port != 0 {
                            ui.add(DragValue::new(&mut p.port).range(1..=50000));
                        }

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

            let text_style = TextStyle::Monospace;
            let logs = self.logs.lock().unwrap();
            let row_height = ui.text_style_height(&text_style);
            let num_rows = logs.len();

            egui::ScrollArea::both()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .show_rows(ui, row_height, num_rows, |ui, row_range| {
                    for row in row_range {
                        ui.label( egui::RichText::new(&logs[row]).text_style(text_style.clone()) );
                    }
                });
        }); // CentralPanel
    }
}
