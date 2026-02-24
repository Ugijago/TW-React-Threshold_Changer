// =============================================================================
//                  --- Engine Revolution Patcher (Fixed) ---
// =============================================================================
use crate::core::{GameParams, GameType};
use std::fs;
use std::path::PathBuf;

// --- 1. Scanner Otomatis (Untuk Sinkronisasi Slider) ---
pub fn detect_all_values(game: GameType, path: &PathBuf) -> Result<GameParams, String> {
    let file_target = "empire.retail.dll";
    let target_path = path.join(file_target);
    let data = fs::read(&target_path).map_err(|_| "DLL Not Found For Scanning".to_string())?;

    let current_min = scan_single_value(&data, get_pattern_min(game), true).unwrap_or(4.0);
    
    let current_ordered = scan_single_value(&data, get_pattern_ordered(game), false)
        .unwrap_or(if game == GameType::Rome2 { 5.0 } else { 15.0 });

    let mut current_ammo = 0.0;
    if game != GameType::Rome2 {
        current_ammo = scan_single_value(&data, get_pattern_ammo(game), false).unwrap_or(85.0);
    }

    let current_kakak = scan_single_value(&data, get_pattern_kakak(game), false).unwrap_or(25.0);

    Ok(GameParams {
        min_entities: current_min,
        ordered_mod: current_ordered,
        with_ammo_mod: current_ammo,
        no_ammo_mod: current_kakak,
    })
}

// --- 2. Apply Patch (Dengan Logika Backup) ---
pub fn apply_advanced(
    game: GameType,
    path: &PathBuf,
    val_min: f32,
    val_ordered: f32,
    val_with_ammo: f32,
    val_no_ammo: f32,
) -> Result<String, String> {
    let target_path = path.join("empire.retail.dll");
    let backup_path = path.join("empire.retail.dll.bak");

    // Membuat backup hanya jika belum ada
    if !backup_path.exists() {
        fs::copy(&target_path, &backup_path).map_err(|_| "Error: Failed To Create Backup!".to_string())?;
    }

    let mut data = fs::read(&target_path).map_err(|_| "Error: Empire.retail.dll Not Found!".to_string())?;

    // Tulis data ke buffer
    write_to_buffer(&mut data, game, val_min, val_ordered, val_with_ammo, val_no_ammo);

    fs::write(&target_path, data).map_err(|_| "Error: File Is In Use!".to_string())?;
    
    Ok(format!("Success: {:?} File Patched!", game))
}

// --- 3. Restore Hardcoded (Tulis Ulang Nilai Default) ---
pub fn restore_hardcoded(game: GameType, path: &PathBuf) -> Result<String, String> {
    let target_path = path.join("empire.retail.dll");
    let mut data = fs::read(&target_path).map_err(|_| "Error: Cannot Read DLL For Restore!".to_string())?;

    // Tentukan nilai default original
    let (def_min, def_ordered, def_with_ammo, def_no_ammo) = match game {
        GameType::Rome2 => (4.0, 5.0, 100.0, 25.0),
        _ => (4.0, 15.0, 85.0, 25.0),
    };

    write_to_buffer(&mut data, game, def_min, def_ordered, def_with_ammo, def_no_ammo);

    fs::write(&target_path, data).map_err(|_| "Error: Failed To Write Restored DLL!".to_string())?;
    
    Ok("Success: All Parameters Reset To Factory Default!".to_string())
}

// --- Helper: Fungsi Penulis Buffer ---
fn write_to_buffer(data: &mut Vec<u8>, game: GameType, v_min: f32, v_ord: f32, v_ammo: f32, v_no: f32) {
    patch_engine_pattern(data, get_pattern_min(game), v_min, true);
    patch_engine_pattern(data, get_pattern_ordered(game), v_ord, false);

    if game != GameType::Rome2 {
        patch_engine_pattern(data, get_pattern_ammo(game), v_ammo, false);
    }

    patch_engine_pattern(data, get_pattern_kakak(game), v_no, false);
}

// --- Helper: Pattern Locators ---
fn get_pattern_min(game: GameType) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    match game {
        GameType::Attila => (vec![0x68, 0xA8, 0x34, 0xAC, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0x64, 0x36, 0xAC, 0x11]),
        GameType::Rome2 => (vec![0x68, 0x68, 0x68, 0x74, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0x54, 0x69, 0x74, 0x11]),
        GameType::Britain => (vec![0x68, 0x88, 0xA5, 0xC1, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0x4C, 0xA7, 0xC1, 0x11]),
    }
}

fn get_pattern_ordered(game: GameType) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    match game {
        GameType::Attila => (vec![0x68, 0xA8, 0x34, 0xAC, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0xF4, 0x35, 0xAC, 0x11]),
        GameType::Rome2 => (vec![0x68, 0x68, 0x68, 0x74, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0xE8, 0x68, 0x74, 0x11]),
        GameType::Britain => (vec![0x68, 0x88, 0xA5, 0xC1, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0xDC, 0xA6, 0xC1, 0x11]),
    }
}

fn get_pattern_ammo(game: GameType) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    match game {
        GameType::Attila => (vec![0x68, 0xA8, 0x34, 0xAC, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0x04, 0x35, 0xAC, 0x11]),
        GameType::Britain => (vec![0x68, 0x88, 0xA5, 0xC1, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0xEC, 0xA5, 0xC1, 0x11]),
        _ => (vec![], vec![], vec![]),
    }
}

fn get_pattern_kakak(game: GameType) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    match game {
        GameType::Attila => (vec![0x68, 0xA8, 0x34, 0xAC, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0xC4, 0x35, 0xAC, 0x11]),
        GameType::Rome2 => (vec![0x68, 0x68, 0x68, 0x74, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0xC0, 0x68, 0x74, 0x11]),
        GameType::Britain => (vec![0x68, 0x88, 0xA5, 0xC1, 0x11, 0x8D, 0x44, 0x24, 0x18], vec![0xC7, 0x44, 0x24, 0x18], vec![0x50, 0x68, 0xAC, 0xA6, 0xC1, 0x11]),
    }
}

// --- Core Utils: Scan & Patch ---
fn scan_single_value(data: &[u8], pattern: (Vec<u8>, Vec<u8>, Vec<u8>), is_int: bool) -> Option<f32> {
    let (head, body, tail) = pattern;
    if head.is_empty() { return None; }
    
    let full_len = head.len() + body.len() + 4 + tail.len();
    for i in 0..(data.len() - full_len) {
        if data[i..i + head.len()] == head {
            let b_pos = i + head.len();
            if data[b_pos..b_pos + body.len()] == body {
                let v_pos = b_pos + body.len();
                if data[v_pos + 4..v_pos + 4 + tail.len()] == tail {
                    let b = [data[v_pos], data[v_pos + 1], data[v_pos + 2], data[v_pos + 3]];
                    return if is_int {
                        Some(u32::from_le_bytes(b) as f32)
                    } else {
                        Some(f32::from_le_bytes(b) * 100.0)
                    };
                }
            }
        }
    }
    None
}

fn patch_engine_pattern(data: &mut Vec<u8>, pattern: (Vec<u8>, Vec<u8>, Vec<u8>), value: f32, is_int: bool) {
    let (head, body, tail) = pattern;
    if head.is_empty() { return; }
    
    let new_bytes = if is_int {
        (value as u32).to_le_bytes()
    } else {
        (value / 100.0).to_le_bytes()
    };
    
    let full_len = head.len() + body.len() + 4 + tail.len();
    for i in 0..(data.len() - full_len) {
        if data[i..i + head.len()] == head {
            let b_pos = i + head.len();
            if data[b_pos..b_pos + body.len()] == body {
                let v_pos = b_pos + body.len();
                if data[v_pos + 4..v_pos + 4 + tail.len()] == tail {
                    data[v_pos..v_pos + 4].copy_from_slice(&new_bytes);
                    break;
                }
            }
        }
    }
}