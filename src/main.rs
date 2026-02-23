/* ================================================================
        UGIE'S ANTYLAZY v2.5 - CUTTER ENGINE + AUTO BACKUP
        (Ko-fi Donation Integrated)
================================================================ */

use eframe::egui;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
struct PatcherConfig {
    path_attila: Option<PathBuf>,
    path_rome2: Option<PathBuf>,
    path_britain: Option<PathBuf>,
}

#[derive(PartialEq, Clone, Copy, Debug)] 
enum GameType { Attila, Rome2, Britain }

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
        let config = if let Ok(data) = fs::read_to_string("antylazy.ini") {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            PatcherConfig::default()
        };

        Self {
            selected_game: GameType::Attila,
            config,
            percentage: 25.0, 
            status_msg: "READY".to_string(),
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
            let _ = fs::write("antylazy.ini", data);
        }
    }
}

impl eframe::App for UgieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
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
            ui.vertical_centered(|ui| {
                ui.heading("Total War Anty Lazy V.1");
            });
            ui.separator();

            ui.add_space(10.0);
            ui.label("1. Select Game:");
            ui.horizontal(|ui| {
                // Hitung: (Lebar Window 550 - (3 ikon * 90) - (2 spasi * 20)) / 2
                let total_icons_width = (90.0 * 3.0) + (20.0 * 2.0);
                let centering_space = (ui.available_width() - total_icons_width) / 2.0;
                
                ui.add_space(centering_space); // Dorong ke tengah
                ui.spacing_mut().item_spacing = egui::vec2(20.0, 0.0);

                // --- ICON ATTILA ---
                let att_img = egui::Image::new(egui::include_image!("../assets/Icon/Attila.png"))
                    .fit_to_exact_size(egui::vec2(90.0, 90.0))
                    .tint(if self.selected_game == GameType::Attila { egui::Color32::WHITE } else { egui::Color32::GRAY });
                if ui.add(egui::Button::image(att_img).frame(self.selected_game == GameType::Attila)).on_hover_text("Select Attila").clicked() {
                    self.selected_game = GameType::Attila;
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                }

                // --- ICON ROME 2 ---
                let rome_img = egui::Image::new(egui::include_image!("../assets/Icon/Rome2.png"))
                    .fit_to_exact_size(egui::vec2(90.0, 90.0))
                    .tint(if self.selected_game == GameType::Rome2 { egui::Color32::WHITE } else { egui::Color32::GRAY });
                if ui.add(egui::Button::image(rome_img).frame(self.selected_game == GameType::Rome2)).on_hover_text("Select Rome 2").clicked() {
                    self.selected_game = GameType::Rome2;
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                }

                // --- ICON BRITANNIA ---
                let brit_img = egui::Image::new(egui::include_image!("../assets/Icon/Britain.png"))
                    .fit_to_exact_size(egui::vec2(90.0, 90.0))
                    .tint(if self.selected_game == GameType::Britain { egui::Color32::WHITE } else { egui::Color32::GRAY });
                if ui.add(egui::Button::image(brit_img).frame(self.selected_game == GameType::Britain)).on_hover_text("Select Britannia").clicked() {
                    self.selected_game = GameType::Britain;
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                }
            });

            ui.add_space(20.0);
            ui.label("2. Game Installation Path:");
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

                let current_path = match self.selected_game {
                    GameType::Attila => &self.config.path_attila,
                    GameType::Rome2 => &self.config.path_rome2,
                    GameType::Britain => &self.config.path_britain,
                };

                if let Some(p) = current_path {
                    ui.label(egui::RichText::new(" OK ").color(egui::Color32::GREEN).strong());
                    ui.label(egui::RichText::new(p.to_string_lossy()).size(12.0).strong());
                } else {
                    ui.label(egui::RichText::new(" NOT SET ").color(egui::Color32::RED));
                }
            });

            ui.add_space(20.0);
            ui.label("3. Adjust Patch Percentage:");
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.percentage, 0.0..=100.0).suffix("%"));
                if ui.button("SET DEFAULT").clicked() {
                    self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                    self.percentage = 25.0;
                }
            });

            ui.add_space(30.0);
            ui.horizontal(|ui| {
                let current_path = match self.selected_game {
                    GameType::Attila => &self.config.path_attila,
                    GameType::Rome2 => &self.config.path_rome2,
                    GameType::Britain => &self.config.path_britain,
                };

                ui.add_enabled_ui(current_path.is_some(), |ui| {
                    if ui.add_sized([ui.available_width() / 2.0 - 5.0, 40.0], egui::Button::new(egui::RichText::new("APPLY PATCH").strong())).clicked() {
                        self.execute_patch();
                    }
                    if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(egui::RichText::new("RESTORE ORIGINAL").strong())).clicked() {
                        self.execute_restore();
                    }
                });
            });

            ui.add_space(20.0);
            ui.separator();

            // --- BAGIAN STATUS & KO-FI (MODIFIKASI DI SINI) ---
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                
                // Baris paling bawah: Link Donasi di kanan
                ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                    if ui.button(egui::RichText::new("☕ Tip me ").strong().color(egui::Color32::WHITE)).on_hover_text("Support Me!").clicked() {
                        let _ = webbrowser::open("https://ko-fi.com/ugiejago");
                    }
                });

                // Status Message tetap di tengah bawah
                let color = if self.is_error { egui::Color32::LIGHT_RED } else { egui::Color32::LIGHT_GREEN };
                ui.label(egui::RichText::new(&self.status_msg).color(color).size(18.0).strong());
            });
        });
    }
}

// LOGIK CORE PATCHER KAMU (TIDAK ADA PERUBAHAN)
impl UgieApp {
    fn execute_patch(&mut self) {
        let file_target = "empire.retail.dll";
        let file_backup = "empire.retail.dll.bak";
        
        let folder = match self.selected_game {
            GameType::Attila => &self.config.path_attila,
            GameType::Rome2 => &self.config.path_rome2,
            GameType::Britain => &self.config.path_britain,
        };

        if let Some(f) = folder {
            let target_path = f.join(file_target);
            let backup_path = f.join(file_backup);

            if !backup_path.exists() {
                if let Err(_) = fs::copy(&target_path, &backup_path) {
                    self.play_audio(include_bytes!("../assets/Sound/Error.wav"));
                    self.status_msg = "ERROR: Failed to create backup!".to_string();
                    self.is_error = true;
                    return;
                }
            }

            if let Ok(mut data) = fs::read(&target_path) {
                let (head, body_prefix, tail) = match self.selected_game {
                    GameType::Attila => (
                        vec![0x68, 0xA8, 0x34, 0xAC, 0x11, 0x8D, 0x44, 0x24, 0x18], 
                        vec![0xC7, 0x44, 0x24, 0x18], 
                        vec![0x50, 0x68, 0xC4, 0x35, 0xAC, 0x11]
                    ),
                    GameType::Rome2 => (
                        vec![0x68, 0x68, 0x68, 0x74, 0x11, 0x8D, 0x44, 0x24, 0x18], 
                        vec![0xC7, 0x44, 0x24, 0x18], 
                        vec![0x50, 0x68, 0xC0, 0x68, 0x74, 0x11]
                    ),
                    GameType::Britain => (
                        vec![0x68, 0x88, 0xA5, 0xC1, 0x11, 0x8D, 0x44, 0x24, 0x18],
                        vec![0xC7, 0x44, 0x24, 0x18],
                        vec![0x50, 0x68, 0xAC, 0xA6, 0xC1, 0x11]
                    ),
                };

                let mut found = false;
                let new_val_bytes = (self.percentage / 100.0).to_le_bytes();
                let full_pattern_len = head.len() + body_prefix.len() + 4 + tail.len();
                
                if data.len() > full_pattern_len {
                    for i in 0..(data.len() - full_pattern_len) {
                        if &data[i..i+head.len()] == head.as_slice() {
                            let body_pos = i + head.len();
                            if &data[body_pos..body_pos+body_prefix.len()] == body_prefix.as_slice() {
                                let val_pos = body_pos + body_prefix.len();
                                let tail_pos = val_pos + 4;
                                if &data[tail_pos..tail_pos+tail.len()] == tail.as_slice() {
                                    data[val_pos..val_pos+4].copy_from_slice(&new_val_bytes);
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if found {
                    if fs::write(&target_path, data).is_ok() {
                        self.play_audio(include_bytes!("../assets/Sound/Ding.wav"));
                        self.status_msg = format!("Successfully Patching {:?} To {:.0}%", self.selected_game, self.percentage);      
                        self.is_error = false;
                    } else {
                        self.play_audio(include_bytes!("../assets/Sound/Error.wav"));
                        self.status_msg = "ERROR: File is Write Protected!".to_string();
                        self.is_error = true;
                    }
                } else {
                    self.play_audio(include_bytes!("../assets/Sound/Error.wav"));
                    self.status_msg = "ERROR: Unique Pattern Not Found!".to_string();
                    self.is_error = true;
                }
            }
        }
    }

    fn execute_restore(&mut self) {
        let file_target = "empire.retail.dll";
        let file_backup = "empire.retail.dll.bak";
        
        let folder = match self.selected_game {
            GameType::Attila => &self.config.path_attila,
            GameType::Rome2 => &self.config.path_rome2,
            GameType::Britain => &self.config.path_britain,
        };

        if let Some(f) = folder {
            let target_path = f.join(file_target);
            let backup_path = f.join(file_backup);

            if backup_path.exists() {
                let _ = fs::remove_file(&target_path);
                if let Ok(_) = fs::rename(&backup_path, &target_path) {
                    self.play_audio(include_bytes!("../assets/Sound/Ding.wav"));
                    self.status_msg = "RESTORE SUCCESS: Original file restored!".to_string();
                    self.is_error = false;
                    self.percentage = 25.0;
                } else {
                    self.play_audio(include_bytes!("../assets/Sound/Error.wav"));
                    self.status_msg = "RESTORE ERROR: Access Denied!".to_string();
                    self.is_error = true;
                }
            } else {
                self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                self.status_msg = "INFO: File is already original!".to_string();
                self.is_error = false;
                self.percentage = 25.0;
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 520.0])
            .with_resizable(false)
            .with_position(egui::pos2(600.0, 300.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "UGIE'S TW Anty Lazy V.1", 
        options, 
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx); 
            Ok(Box::new(UgieApp::load())) 
        })
    )
}