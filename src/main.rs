mod models;
mod patcher;

use eframe::egui;
use models::{GameType, PatcherConfig};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

struct UgieApp {
    selected_game: GameType,
    config: PatcherConfig,
    percentage: f32,
    status_msg: String,
    is_error: bool,
    show_wrong_folder_popup: bool,
}

impl UgieApp {
    fn load() -> Self {
        let config = if let Ok(data) = fs::read_to_string("itc_config.ini") {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            PatcherConfig::default()
        };

        Self {
            selected_game: GameType::Attila,
            config,
            percentage: 25.0,
            status_msg: "SYSTEM READY".to_string(),
            is_error: false,
            show_wrong_folder_popup: false,
        }
    }

    fn play_audio(&self, audio_bytes: &'static [u8]) {
        std::thread::spawn(move || {
            if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() {
                if let Ok(sink) = rodio::Sink::try_new(&stream_handle) {
                    let cursor = Cursor::new(audio_bytes);
                    if let Ok(source) = rodio::Decoder::new_wav(cursor) {
                        sink.append(source);
                        sink.sleep_until_end();
                    }
                }
            }
        });
    }

    fn save_config(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.config) {
            let _ = fs::write("itc_config.ini", data);
        }
    }

    fn get_current_path(&self) -> Option<PathBuf> {
        match self.selected_game {
            GameType::Attila => self.config.path_attila.clone(),
            GameType::Rome2 => self.config.path_rome2.clone(),
            GameType::Britain => self.config.path_britain.clone(),
        }
    }
}

impl eframe::App for UgieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // POPUP ERROR
        if self.show_wrong_folder_popup {
            egui::Window::new("DIRECTORY ERROR").anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.label(format!("The selected folder is not valid for {:?}.", self.selected_game));
                if ui.button("OK").clicked() { 
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                    self.show_wrong_folder_popup = false; 
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            // --- HEADER (Dari versi 2.5 Abang) ---
            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../Assets/Icon/Logo.png"))
                        .fit_to_exact_size(egui::vec2(110.0, 110.0))
                );
                ui.add_space(5.0);
                ui.heading(egui::RichText::new("Total War Idle Threshold Changer").strong().size(15.0));
                ui.label(egui::RichText::new("By Ugie").italics().color(egui::Color32::GRAY));
            });
            
            ui.add_space(10.0);
            ui.separator();

            // --- 1. SELECT GAME ENGINE (Centering & Tint Logic) ---
            ui.add_space(10.0);
            ui.label("1. Select Game Engine:");
            ui.horizontal(|ui| {
                let total_icons_width = (90.0 * 3.0) + (20.0 * 2.0);
                let centering_space = (ui.available_width() - total_icons_width) / 2.0;
                
                ui.add_space(centering_space); 
                ui.spacing_mut().item_spacing = egui::vec2(20.0, 0.0);

                // ATTILA
                let att_img = egui::Image::new(egui::include_image!("../Assets/Icon/Attila.png"))
                    .fit_to_exact_size(egui::vec2(90.0, 90.0))
                    .tint(if self.selected_game == GameType::Attila { egui::Color32::WHITE } else { egui::Color32::GRAY });
                if ui.add(egui::Button::image(att_img).frame(self.selected_game == GameType::Attila)).clicked() {
                    self.selected_game = GameType::Attila;
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                }

                // ROME 2
                let rome_img = egui::Image::new(egui::include_image!("../Assets/Icon/Rome2.png"))
                    .fit_to_exact_size(egui::vec2(90.0, 90.0))
                    .tint(if self.selected_game == GameType::Rome2 { egui::Color32::WHITE } else { egui::Color32::GRAY });
                if ui.add(egui::Button::image(rome_img).frame(self.selected_game == GameType::Rome2)).clicked() {
                    self.selected_game = GameType::Rome2;
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                }

                // BRITAIN
                let brit_img = egui::Image::new(egui::include_image!("../Assets/Icon/Britain.png"))
                    .fit_to_exact_size(egui::vec2(90.0, 90.0))
                    .tint(if self.selected_game == GameType::Britain { egui::Color32::WHITE } else { egui::Color32::GRAY });
                if ui.add(egui::Button::image(brit_img).frame(self.selected_game == GameType::Britain)).clicked() {
                    self.selected_game = GameType::Britain;
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                }
            });

            // --- 2. TARGET DIRECTORY ---
            ui.add_space(20.0);
            ui.label("2. Target Directory:");
            ui.horizontal(|ui| {
                if ui.button("📁 Browse Folder").clicked() {
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let exe_name = match self.selected_game {
                            GameType::Attila => "attila.exe",
                            GameType::Rome2 => "rome2.exe",
                            GameType::Britain => "thrones.exe",
                        };
                        
                        if path.join(exe_name).exists() {
                            match self.selected_game {
                                GameType::Attila => self.config.path_attila = Some(path),
                                GameType::Rome2 => self.config.path_rome2 = Some(path),
                                GameType::Britain => self.config.path_britain = Some(path),
                            }
                            self.save_config();
                            self.status_msg = "DIRECTORY VERIFIED".to_string();
                            self.is_error = false;
                        } else {
                            self.play_audio(include_bytes!("../assets/Sound/Error.wav"));
                            self.show_wrong_folder_popup = true;
                        }
                    }
                }

                if let Some(p) = self.get_current_path() {
                    ui.label(egui::RichText::new(" OK ").color(egui::Color32::GREEN).strong());
                    ui.label(egui::RichText::new(p.to_string_lossy()).size(12.0).strong());
                } else {
                    ui.label(egui::RichText::new(" NOT SET ").color(egui::Color32::RED));
                }
            });

            // --- 3. SLIDER ---
            ui.add_space(20.0);
            ui.label("3. Adjust Idle Fatigue Threshold:");
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.percentage, 0.0..=100.0).suffix("%"));
                if ui.button("SET DEFAULT").clicked() {
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                    self.percentage = 25.0;
                }
            });

            // --- 4. ACTION BUTTONS ---
            ui.add_space(30.0);
            ui.horizontal(|ui| {
                let current_path = self.get_current_path();
                ui.add_enabled_ui(current_path.is_some(), |ui| {
                    if ui.add_sized([ui.available_width() / 2.0 - 5.0, 40.0], egui::Button::new(egui::RichText::new("APPLY PATCH").strong())).clicked() {
                        if let Some(path) = &current_path {
                            match patcher::execute_patch(self.selected_game, path, self.percentage) {
                                Ok(msg) => {
                                    self.status_msg = msg;
                                    self.is_error = false;
                                    self.play_audio(include_bytes!("../assets/Sound/Ding.wav"));
                                }
                                Err(err) => {
                                    self.status_msg = err;
                                    self.is_error = true;
                                    self.play_audio(include_bytes!("../assets/Sound/Error.wav"));
                                }
                            }
                        }
                    }
                    if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(egui::RichText::new("RESTORE ORIGINAL").strong())).clicked() {
                        if let Some(path) = &current_path {
                            match patcher::execute_restore(path) {
                                Ok(msg) => {
                                    self.status_msg = msg;
                                    self.is_error = false;
                                    self.play_audio(include_bytes!("../assets/Sound/Ding.wav"));
                                }
                                Err(err) => {
                                    self.status_msg = err;
                                    self.is_error = true;
                                }
                            }
                        }
                    }
                });
            });

            // --- FOOTER & TIP ME ---
            ui.add_space(20.0);
            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                    if ui.button(egui::RichText::new("☕ Tip me ").strong().color(egui::Color32::WHITE)).on_hover_text("Support via Ko-fi").clicked() {
                        let _ = webbrowser::open("https://ko-fi.com/ugiejago");
                    }
                });

                let color = if self.is_error { egui::Color32::LIGHT_RED } else { egui::Color32::LIGHT_GREEN };
                ui.label(egui::RichText::new(&self.status_msg).color(color).size(18.0).strong());
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 600.0])
            .with_resizable(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "Total War Idle Threshold Changer",
        options, 
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx); 
            Ok(Box::new(UgieApp::load())) 
        })
    )
}