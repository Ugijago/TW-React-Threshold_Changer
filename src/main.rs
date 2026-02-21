/* ================================================================
          UGIE'S ANTYLAZY v2.5 - CUTTER ENGINE + AUTO BACKUP
================================================================ */

use eframe::egui;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use rodio::{Decoder, OutputStream, Sink};

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
            if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    let cursor = Cursor::new(audio_bytes);
                    if let Ok(source) = Decoder::new_wav(cursor) {
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
                    self.play_audio(include_bytes!("../Click.wav"));
                    self.show_wrong_folder_popup = false; 
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.heading("UGIE'S ANTYLAZY v2.5");
            });
            ui.separator();

            // 1. SELECT GAME (WITH ICONS)
            ui.add_space(10.0);
            ui.label("1. Select Game:");
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(20.0, 0.0);

                // --- ICON ATTILA ---
                let attila_btn = egui::ImageButton::new(egui::include_image!("../Attila.png"))
                    .tint(if self.selected_game == GameType::Attila { egui::Color32::WHITE } else { egui::Color32::GRAY })
                    .frame(self.selected_game == GameType::Attila);
                
                if ui.add_sized([90.0, 90.0], attila_btn).on_hover_text("Select Attila").clicked() {
                    self.selected_game = GameType::Attila;
                    self.play_audio(include_bytes!("../Click.wav"));
                }

                // --- ICON ROME 2 ---
                let rome_btn = egui::ImageButton::new(egui::include_image!("../Rome2.png"))
                    .tint(if self.selected_game == GameType::Rome2 { egui::Color32::WHITE } else { egui::Color32::GRAY })
                    .frame(self.selected_game == GameType::Rome2);

                if ui.add_sized([90.0, 90.0], rome_btn).on_hover_text("Select Rome 2").clicked() {
                    self.selected_game = GameType::Rome2;
                    self.play_audio(include_bytes!("../Click.wav"));
                }

                /* --- COMMENT DULU: ICON BRITANNIA (Belum ada filenya) ---
                let brit_btn = egui::ImageButton::new(egui::include_image!("../Britain.png"))
                    .tint(if self.selected_game == GameType::Britain { egui::Color32::WHITE } else { egui::Color32::GRAY })
                    .frame(self.selected_game == GameType::Britain);

                if ui.add_sized([90.0, 90.0], brit_btn).on_hover_text("Select Britannia").clicked() {
                    self.selected_game = GameType::Britain;
                    self.play_audio(include_bytes!("../Click.wav"));
                }
                --------------------------------------------------------- */
            });

            // 2. FOLDER SELECTION
            ui.add_space(20.0);
            ui.label("2. Game Installation Path:");
            ui.horizontal(|ui| {
                if ui.button("📁 Browse Folder").clicked() {
                    self.play_audio(include_bytes!("../Click.wav"));
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
                            self.play_audio(include_bytes!("../Error.wav"));
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

            // 3. SLIDER
            ui.add_space(20.0);
            ui.label("3. Adjust Patch Percentage:");
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.percentage, 0.0..=100.0).suffix("%"));
                if ui.button("SET DEFAULT").clicked() {
                    self.play_audio(include_bytes!("../Click.wav"));
                    self.percentage = 25.0;
                }
            });

            // 4. ACTION BUTTONS
            ui.add_space(30.0);
            ui.horizontal(|ui| {
                let current_path = match self.selected_game {
                    GameType::Attila => &self.config.path_attila,
                    GameType::Rome2 => &self.config.path_rome2,
                    GameType::Britain => &self.config.path_britain,
                };

                ui.add_enabled_ui(current_path.is_some(), |ui| {
                    // Tombol Apply (Kiri)
                    if ui.add_sized([ui.available_width() / 2.0 - 5.0, 40.0], egui::Button::new(egui::RichText::new("APPLY PATCH").strong())).clicked() {
                        self.execute_patch();
                    }
                    
                    // Tombol Restore (Kanan)
                    if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(egui::RichText::new("RESTORE ORIGINAL").strong())).clicked() {
                        self.execute_restore();
                    }
                });
            });

            ui.add_space(30.0);
            ui.separator();

            // STATUS
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                let color = if self.is_error { egui::Color32::LIGHT_RED } else { egui::Color32::LIGHT_GREEN };
                ui.label(egui::RichText::new(&self.status_msg).color(color).size(18.0).strong());
            });
        });
    }
}

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

            // --- AUTO BACKUP LOGIC ---
            if !backup_path.exists() {
                if let Err(_) = fs::copy(&target_path, &backup_path) {
                    self.play_audio(include_bytes!("../Error.wav"));
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
                        vec![0x8D, 0x44, 0x24, 0x18], 
                        vec![0xC7, 0x44, 0x24, 0x18], 
                        vec![0x50]
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
                        self.play_audio(include_bytes!("../Ding.wav"));
                        self.status_msg = format!("Success: {:?} patched!", self.selected_game);
                        self.is_error = false;
                    } else {
                        self.play_audio(include_bytes!("../Error.wav"));
                        self.status_msg = "ERROR: File is Write Protected!".to_string();
                        self.is_error = true;
                    }
                } else {
                    self.play_audio(include_bytes!("../Error.wav"));
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
                // Restore: Hapus yang diedit, pindahkan backup kembali ke asli
                let _ = fs::remove_file(&target_path);
                if let Ok(_) = fs::rename(&backup_path, &target_path) {
                    self.play_audio(include_bytes!("../Ding.wav"));
                    self.status_msg = "RESTORE SUCCESS: File is original now!".to_string();
                    self.is_error = false;
                    self.percentage = 25.0;
                } else {
                    self.play_audio(include_bytes!("../Error.wav"));
                    self.status_msg = "RESTORE ERROR: Access Denied!".to_string();
                    self.is_error = true;
                }
            } else {
                self.play_audio(include_bytes!("../Click.wav")); // Bunyi klik biasa aja, bukan error
                self.status_msg = "INFO: File is already original!".to_string();
                self.is_error = false; // Kita set false karena ini bukan error, cuma info
                self.percentage = 25.0;
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 520.0])
            .with_resizable(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "UGIE'S ANTYLAZY", 
        options, 
        Box::new(|cc| {
            // Pasang loader gambar
            egui_extras::install_image_loaders(&cc.egui_ctx); 
            
            // Bungkus pakai Ok(...) biar Rust gak marah
            Ok(Box::new(UgieApp::load())) 
        })
    )
}