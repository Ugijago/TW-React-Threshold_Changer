#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod core;
mod patch;

use crate::core::{GameParams, GameType, PatcherConfig};
use eframe::egui;
use std::fs;
use std::io::Cursor;

struct UgieApp {
    selected_game: GameType,
    config: PatcherConfig,
    val_min: f32,
    val_ordered: f32,
    val_with_ammo: f32,
    val_no_ammo: f32,
    status_msg: String,
    is_error: bool,
    show_wrong_folder_popup: bool,
}

impl UgieApp {
    fn load() -> Self {
        let config: PatcherConfig = if let Ok(data) = fs::read_to_string("itc_config.ini") {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            PatcherConfig::default()
        };

        let last_msg = if config.last_patch_info.is_empty() {
            "System Ready".to_string()
        } else {
            config.last_patch_info.clone()
        };

        let mut app = Self {
            selected_game: GameType::Attila,
            val_min: config.params_attila.min_entities,
            val_ordered: config.params_attila.ordered_mod,
            val_with_ammo: config.params_attila.with_ammo_mod,
            val_no_ammo: config.params_attila.no_ammo_mod,
            config,
            status_msg: last_msg,
            is_error: false,
            show_wrong_folder_popup: false,
        };

        app.scan_current_values();
        app
    }

    fn scan_current_values(&mut self) {
        // Ambil params default buat game yang baru dipilih
        let default_params = match self.selected_game {
            GameType::Attila => &self.config.params_attila,
            GameType::Rome2 => &self.config.params_rome2,
            GameType::Britain => &self.config.params_britain,
        };

        // Pasang dulu angka dari config/default (biar gak sisa data game sebelah)
        self.val_min = default_params.min_entities;
        self.val_ordered = default_params.ordered_mod;
        self.val_with_ammo = default_params.with_ammo_mod;
        self.val_no_ammo = default_params.no_ammo_mod;

        // Baru cek path-nya, kalau ada foldernya, timpa pakai hasil scan asli dari DLL
        let current_path = match self.selected_game {
            GameType::Attila => &self.config.path_attila,
            GameType::Rome2 => &self.config.path_rome2,
            GameType::Britain => &self.config.path_britain,
        };

        if let Some(path) = current_path {
            if let Ok(values) = patch::detect_all_values(self.selected_game, path) {
                self.val_min = values.min_entities;
                self.val_ordered = values.ordered_mod;
                self.val_with_ammo = values.with_ammo_mod;
                self.val_no_ammo = values.no_ammo_mod;
            }
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

    fn save_config(&mut self) {
        let current_params = GameParams {
            min_entities: self.val_min,
            ordered_mod: self.val_ordered,
            with_ammo_mod: self.val_with_ammo,
            no_ammo_mod: self.val_no_ammo,
        };

        match self.selected_game {
            GameType::Attila => self.config.params_attila = current_params,
            GameType::Rome2 => self.config.params_rome2 = current_params,
            GameType::Britain => self.config.params_britain = current_params,
        }

        self.config.last_patch_info = self.status_msg.clone();

        if let Ok(data) = serde_json::to_string_pretty(&self.config) {
            let _ = fs::write("itc_config.ini", data);
        }
    }
}

impl eframe::App for UgieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.show_wrong_folder_popup {
            egui::Window::new("Directory Error")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(format!("The selected folder is not valid for {:?}.", self.selected_game));
                        ui.add_space(10.0);
                        if ui.button("   OK   ").clicked() {
                            self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                            self.show_wrong_folder_popup = false;
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            ui.vertical_centered(|ui| {
                ui.add(egui::Image::new(egui::include_image!("../assets/Icon/Logo.png")).fit_to_exact_size(egui::vec2(110.0, 110.0)));
                ui.add_space(5.0);
                ui.heading(egui::RichText::new("Total War Idle Threshold Changer").strong().size(18.0));
                ui.label(egui::RichText::new("By Ugie").italics().color(egui::Color32::GRAY));
            });

            ui.add_space(10.0);
            ui.separator();

            ui.add_space(5.0);
            ui.label("1. Select Game :");
            ui.horizontal(|ui| {
                let total_width = (80.0 * 3.0) + (15.0 * 2.0);
                let centering_space = (ui.available_width() - total_width) / 2.0;
                ui.add_space(centering_space);
                ui.spacing_mut().item_spacing = egui::vec2(15.0, 0.0);

                let games = [
                    (GameType::Attila, egui::include_image!("../assets/Icon/Attila.png")),
                    (GameType::Rome2, egui::include_image!("../assets/Icon/Rome2.png")),
                    (GameType::Britain, egui::include_image!("../assets/Icon/Britain.png")),
                ];

                for (g_type, img) in games {
                    let is_sel = self.selected_game == g_type;
                    let btn_img = egui::Image::new(img)
                        .fit_to_exact_size(egui::vec2(80.0, 80.0))
                        .tint(if is_sel { egui::Color32::WHITE } else { egui::Color32::from_gray(100) });

                    let response = ui.add(egui::Button::image(btn_img).frame(is_sel)).on_hover_text(format!("Select {:?}", g_type));

                    if response.clicked() {
                        self.selected_game = g_type;
                        self.scan_current_values();
                        self.play_audio(include_bytes!("../assets/Sound/Click.wav"));
                        self.status_msg = format!("Selected Game {:?}", g_type);
                        self.is_error = false;
                    }
                }
            });

            ui.add_space(20.0);
            ui.label("2. Install Location :");
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
                            self.scan_current_values();
                            self.status_msg = "Directory Verified & Data Loaded".to_string();
                            self.is_error = false;
                            self.save_config();
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
            ui.label("3. Engine Parameters :");
            ui.add_space(5.0);

            // Tanda tanya putih, agak gede, posisi kanan angka
            let help_icon = |ui: &mut egui::Ui, text: &str| {
                ui.add(egui::Label::new(
                    egui::RichText::new(" (?)")
                        .size(15.0) 
                        .strong()
                        .color(egui::Color32::WHITE)
                )).on_hover_text(egui::RichText::new(text).size(13.0));
            };

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                egui::Grid::new("slider_grid").num_columns(2).spacing([15.0, 12.0]).show(ui, |ui| {
                    
                    ui.label("Minimum Threshold:");
                    ui.horizontal(|ui| {
                        if ui.add(egui::Slider::new(&mut self.val_min, 1.0..=15.0).step_by(1.0)).changed() {
                            self.save_config();
                        }
                        help_icon(ui, "Minimum Threshold For Weight Modifier, Any Modifier Less Than This Value Will Be Clamped To This Value.");
                    });
                    ui.end_row();

                    ui.label("Modifier Ordered:");
                    ui.horizontal(|ui| {
                        if ui.add(egui::Slider::new(&mut self.val_ordered, 0.0..=100.0).suffix("%")).changed() {
                            self.save_config();
                        }
                        help_icon(ui, "Weight Modifier For Unit To React When Get Attacked While Being Ordered.");
                    });
                    ui.end_row();

                    if self.selected_game != GameType::Rome2 {
                        ui.label("Modifier Idle With Ammo:");
                        ui.horizontal(|ui| {
                            if ui.add(egui::Slider::new(&mut self.val_with_ammo, 0.0..=100.0).suffix("%")).changed() {
                                self.save_config();
                            }
                            help_icon(ui, "Weight Modifier For Unit To React When Get Attacked While Being In Idle And Still Have Ammo.");
                        });
                        ui.end_row();
                    }

                    ui.label("Modifier Idle:");
                    ui.horizontal(|ui| {
                        if ui.add(egui::Slider::new(&mut self.val_no_ammo, 0.0..=100.0).suffix("%")).changed() {
                            self.save_config();
                        }
                        help_icon(ui, "Weight Modifier For Unit To React When Get Attacked While Being In Idle State.");
                    });
                    ui.end_row();
                });
            });

            ui.add_space(20.0);

            let current_path_clone = match self.selected_game {
                GameType::Attila => self.config.path_attila.clone(),
                GameType::Rome2 => self.config.path_rome2.clone(),
                GameType::Britain => self.config.path_britain.clone(),
            };
            let has_path = current_path_clone.is_some();

            ui.vertical_centered_justified(|ui| {
                ui.add_enabled_ui(has_path, |ui| {
                    let apply_btn = ui.add_sized(
                        [ui.available_width(), 45.0], 
                        egui::Button::new(egui::RichText::new("🚀 APPLY PATCH").strong().size(16.0))
                    ).on_disabled_hover_text("Please select game folder first!");

                    if apply_btn.clicked() {
                        if let Some(p) = &current_path_clone {
                            match patch::apply_advanced(self.selected_game, p, self.val_min, self.val_ordered, self.val_with_ammo, self.val_no_ammo) {
                                Ok(m) => {
                                    self.status_msg = m;
                                    self.is_error = false;
                                    self.play_audio(include_bytes!("../assets/Sound/Ding.wav"));
                                    self.scan_current_values();
                                    self.save_config();
                                }
                                Err(e) => {
                                    self.status_msg = e;
                                    self.is_error = true;
                                    self.play_audio(include_bytes!("../assets/Sound/Error.wav"));
                                }
                            }
                        }
                    }
                    ui.add_space(8.0);

                    if ui.add_sized([ui.available_width(), 35.0], egui::Button::new("🔄 Restore To Original")).clicked() {
                        if let Some(p) = &current_path_clone {
                            match patch::restore_hardcoded(self.selected_game, p) {
                                Ok(m) => {
                                    self.status_msg = m;
                                    self.is_error = false;
                                    self.play_audio(include_bytes!("../assets/Sound/Ding.wav"));
                                    self.scan_current_values();
                                    self.save_config();
                                }
                                Err(e) => {
                                    self.status_msg = e;
                                    self.is_error = true;
                                    self.play_audio(include_bytes!("../assets/Sound/Error.wav"));
                                }
                            }
                        }
                    }
                });
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(5.0);
            ui.vertical_centered(|ui| {
                let color = if self.is_error { egui::Color32::LIGHT_RED } else { egui::Color32::LIGHT_GREEN };
                ui.label(egui::RichText::new(&self.status_msg).color(color).size(18.0).strong());
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.add_space(5.0);
                if ui.button("☕ Tip").on_hover_text("Support Me Via Ko-Fi").clicked() {
                    let _ = webbrowser::open("https://ko-fi.com/ugiejago");
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    // Logic untuk load Logo.png ke window icon (Pojok kiri atas)
    let icon = image::load_from_memory(include_bytes!("../assets/Icon/Logo.png"))
        .map(|img| {
            let img = img.to_rgba8();
            let (width, height) = img.dimensions();
            egui::IconData {
                rgba: img.into_raw(),
                width,
                height,
            }
        })
        .unwrap_or_default();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 720.0])
            .with_resizable(false)
            .with_icon(std::sync::Arc::new(icon)) // Icon terpasang disini
            .with_position(egui::pos2(500.0, 150.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Total War Idle Threshold Changer v1.0",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(UgieApp::load()))
        }),
    )
}