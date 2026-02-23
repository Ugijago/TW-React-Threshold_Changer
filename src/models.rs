use serde::{Serialize, Deserialize};
use std::path::PathBuf;

// Struktur data untuk menyimpan lokasi folder game
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PatcherConfig {
    pub path_attila: Option<PathBuf>,
    pub path_rome2: Option<PathBuf>,
    pub path_britain: Option<PathBuf>,
}

// Daftar game yang didukung
#[derive(PartialEq, Clone, Copy, Debug)] 
pub enum GameType { 
    Attila, 
    Rome2, 
    Britain 
}