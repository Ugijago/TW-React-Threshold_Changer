use crate::core::{GameParams, GameType};
use std::fs;
use std::path::PathBuf;

struct PeInfo {
    image_base: u32,
    code_start: usize,
    code_end: usize,
    rdata_start: usize,
    rdata_end: usize,
    sec_table: usize,
    num_sections: usize,
}

fn parse_pe(data: &[u8]) -> Option<PeInfo> {
    if data.len() < 0x40 {
        return None;
    }
    let e_lfanew = u32::from_le_bytes(data[0x3C..0x40].try_into().ok()?) as usize;
    if data.len() < e_lfanew + 56 {
        return None;
    }
    let num_sections = u16::from_le_bytes(data[e_lfanew + 6..e_lfanew + 8].try_into().ok()?) as usize;
    let opt_hdr_size = u16::from_le_bytes(data[e_lfanew + 20..e_lfanew + 22].try_into().ok()?) as usize;
    let image_base = u32::from_le_bytes(data[e_lfanew + 52..e_lfanew + 56].try_into().ok()?);

    let sec_table = e_lfanew + 24 + opt_hdr_size;
    let mut code_start = 0;
    let mut code_end = data.len();
    let mut rdata_start = 0;
    let mut rdata_end = data.len();

    for i in 0..num_sections {
        let sec = sec_table + i * 40;
        if data.len() < sec + 24 {
            break;
        }
        let name = &data[sec..sec + 8];
        let r_size = u32::from_le_bytes(data[sec + 16..sec + 20].try_into().ok()?) as usize;
        let r_offset = u32::from_le_bytes(data[sec + 20..sec + 24].try_into().ok()?) as usize;
        if name.starts_with(b".text") {
            code_start = r_offset;
            code_end = r_offset + r_size;
        } else if name.starts_with(b".rdata") {
            rdata_start = r_offset;
            rdata_end = r_offset + r_size;
        }
    }

    Some(PeInfo {
        image_base,
        code_start,
        code_end,
        rdata_start,
        rdata_end,
        sec_table,
        num_sections,
    })
}

fn offset_to_va(pe: &PeInfo, data: &[u8], file_offset: usize) -> Option<u32> {
    for i in 0..pe.num_sections {
        let sec = pe.sec_table + i * 40;
        let v_addr = u32::from_le_bytes(data[sec + 12..sec + 16].try_into().ok()?);
        let r_size = u32::from_le_bytes(data[sec + 16..sec + 20].try_into().ok()?);
        let r_offset = u32::from_le_bytes(data[sec + 20..sec + 24].try_into().ok()?) as usize;
        if file_offset >= r_offset && file_offset < r_offset + r_size as usize {
            return Some(pe.image_base + v_addr + (file_offset - r_offset) as u32);
        }
    }
    None
}

#[inline(always)]
fn fast_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let first = needle[0];
    let max = haystack.len() - needle.len();
    let mut i = 0;
    while i <= max {
        if haystack[i] == first && &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_value_offset(pe: &PeInfo, data: &[u8], string_name: &str) -> Option<usize> {
    let needle = string_name.as_bytes();
    let rdata_slice = data.get(pe.rdata_start..pe.rdata_end).unwrap_or(data);
    let rel_str_pos = fast_find(rdata_slice, needle)?;
    let string_file_offset = pe.rdata_start + rel_str_pos;

    let string_va = offset_to_va(pe, data, string_file_offset)?;
    let va_bytes = string_va.to_le_bytes();

    let code_slice = data.get(pe.code_start..pe.code_end).unwrap_or(data);
    let push_pattern = [0x50, 0x68, va_bytes[0], va_bytes[1], va_bytes[2], va_bytes[3]];
    let rel_push_pos = fast_find(code_slice, &push_pattern)?;
    let push_pos = pe.code_start + rel_push_pos;

    if push_pos >= 4 {
        Some(push_pos - 4)
    } else {
        None
    }
}

fn read_value(pe: &PeInfo, data: &[u8], string_name: &str, is_int: bool) -> Option<f32> {
    let offset = find_value_offset(pe, data, string_name)?;
    let b = data.get(offset..offset + 4)?;
    let val_bytes: [u8; 4] = b.try_into().ok()?;
    if is_int {
        Some(u32::from_le_bytes(val_bytes) as f32)
    } else {
        Some(f32::from_le_bytes(val_bytes) * 100.0)
    }
}

fn write_value(pe: &PeInfo, data: &mut [u8], string_name: &str, value: f32, is_int: bool) -> Result<(), String> {
    let offset = find_value_offset(pe, data, string_name)
        .ok_or_else(|| format!("Parameter '{}' not found in DLL!", string_name))?;
    let new_bytes = if is_int {
        (value as u32).to_le_bytes()
    } else {
        (value / 100.0).to_le_bytes()
    };
    data[offset..offset + 4].copy_from_slice(&new_bytes);
    Ok(())
}

fn get_string_keys(game: GameType) -> (&'static str, &'static str, Option<&'static str>, &'static str) {
    match game {
        GameType::Attila | GameType::Britain => (
            "MELEE_ATTACK_THRESHOLD_MINIMUM",
            "MELEE_ATTACK_THRESHOLD_MODIFIER_ORDERED",
            Some("MELEE_ATTACK_THRESHOLD_MODIFIER_IDLE_AMMO_REMAINING"),
            "MELEE_ATTACK_THRESHOLD_MODIFIER_IDLE_NO_AMMO",
        ),
        GameType::Rome2 => (
            "MELEE_ATTACK_THRESHOLD_MINIMUM",
            "MELEE_ATTACK_THRESHOLD_MODIFIER_ORDERED",
            None,
            "MELEE_ATTACK_THRESHOLD_MODIFIER_IDLE",
        ),
    }
}

pub fn detect_all_values(game: GameType, path: &PathBuf) -> Result<GameParams, String> {
    let target_path = path.join("empire.retail.dll");
    let data = fs::read(&target_path).map_err(|_| "DLL Not Found For Scanning".to_string())?;
    let pe = parse_pe(&data).ok_or_else(|| "Failed to parse PE header".to_string())?;

    let (s_min, s_ord, s_ammo, s_idle) = get_string_keys(game);

    let current_min = read_value(&pe, &data, s_min, true).unwrap_or(4.0);
    let current_ordered = read_value(&pe, &data, s_ord, false)
        .unwrap_or(if game == GameType::Rome2 { 5.0 } else { 15.0 });

    let current_ammo = if let Some(key) = s_ammo {
        read_value(&pe, &data, key, false).unwrap_or(85.0)
    } else {
        0.0
    };

    let current_idle = read_value(&pe, &data, s_idle, false).unwrap_or(25.0);

    Ok(GameParams {
        min_entities: current_min,
        ordered_mod: current_ordered,
        with_ammo_mod: current_ammo,
        no_ammo_mod: current_idle,
    })
}

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

    if !backup_path.exists() {
        fs::copy(&target_path, &backup_path)
            .map_err(|_| "Error: Failed To Create Backup!".to_string())?;
    }

    let mut data = fs::read(&target_path)
        .map_err(|_| "Error: Empire.retail.dll Not Found!".to_string())?;
    let pe = parse_pe(&data).ok_or_else(|| "Failed to parse PE header".to_string())?;

    let (s_min, s_ord, s_ammo, s_idle) = get_string_keys(game);

    write_value(&pe, &mut data, s_min, val_min, true)?;
    write_value(&pe, &mut data, s_ord, val_ordered, false)?;
    if let Some(key) = s_ammo {
        write_value(&pe, &mut data, key, val_with_ammo, false)?;
    }
    write_value(&pe, &mut data, s_idle, val_no_ammo, false)?;

    fs::write(&target_path, data).map_err(|_| "Error: File Is In Use!".to_string())?;

    Ok(format!("Success: {:?} File Patched!", game))
}

pub fn restore_hardcoded(game: GameType, path: &PathBuf) -> Result<String, String> {
    let target_path = path.join("empire.retail.dll");
    let mut data = fs::read(&target_path)
        .map_err(|_| "Error: Cannot Read DLL For Restore!".to_string())?;
    let pe = parse_pe(&data).ok_or_else(|| "Failed to parse PE header".to_string())?;

    let (def_min, def_ordered, def_with_ammo, def_no_ammo) = match game {
        GameType::Rome2 => (4.0, 5.0, 0.0, 25.0),
        _ => (4.0, 15.0, 85.0, 25.0),
    };

    let (s_min, s_ord, s_ammo, s_idle) = get_string_keys(game);

    write_value(&pe, &mut data, s_min, def_min, true)?;
    write_value(&pe, &mut data, s_ord, def_ordered, false)?;
    if let Some(key) = s_ammo {
        write_value(&pe, &mut data, key, def_with_ammo, false)?;
    }
    write_value(&pe, &mut data, s_idle, def_no_ammo, false)?;

    fs::write(&target_path, data).map_err(|_| "Error: Failed To Write Restored DLL!".to_string())?;

    Ok("Success: All Parameters Reset To Factory Default!".to_string())
}