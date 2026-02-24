use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum GameType {
    Attila,
    Rome2,
    Britain,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct GameParams {
    pub min_entities: f32,
    pub ordered_mod: f32,
    pub with_ammo_mod: f32,
    pub no_ammo_mod: f32,
}

// Default ini sekarang kita jadikan standar umum (biasanya ikut Attila/Britain)
impl Default for GameParams {
    fn default() -> Self {
        Self {
            min_entities: 4.0,
            ordered_mod: 15.0,   // Standar Attila
            with_ammo_mod: 85.0,
            no_ammo_mod: 25.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PatcherConfig {
    pub path_attila: Option<PathBuf>,
    pub path_rome2: Option<PathBuf>,
    pub path_britain: Option<PathBuf>,
    
    pub params_attila: GameParams,
    pub params_rome2: GameParams,
    pub params_britain: GameParams,

    pub last_patch_info: String,
}

impl Default for PatcherConfig {
    fn default() -> Self {
        Self {
            path_attila: None,
            path_rome2: None,
            path_britain: None,
            // Attila & Britain pakai default (15%)
            params_attila: GameParams::default(),
            params_britain: GameParams::default(),
            
            // KHUSUS ROME 2: Kita override ordered_mod jadi 5.0 (Data Debugger: 3D4CCCCD)
            params_rome2: GameParams {
                min_entities: 4.0,
                ordered_mod: 5.0,     // <--- FIX: Sesuai data 0.05
                with_ammo_mod: 0.0,   // Rome 2 biasanya tidak pakai offset ini
                no_ammo_mod: 25.0,
            },
            
            last_patch_info: "SYSTEM READY".to_string(),
        }
    }
}