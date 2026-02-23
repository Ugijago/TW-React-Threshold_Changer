use std::fs;
use std::path::PathBuf;
use crate::models::GameType;

/// Function to execute the patching process
pub fn execute_patch(game: GameType, folder: &PathBuf, percentage: f32) -> Result<String, String> {
    let file_target = "empire.retail.dll";
    let file_backup = "empire.retail.dll.bak";
    let target_path = folder.join(file_target);
    let backup_path = folder.join(file_backup);

    // 1. Check & Create Backup
    if !backup_path.exists() {
        fs::copy(&target_path, &backup_path).map_err(|_| "Failed to create backup!")?;
    }

    // 2. Read DLL file into memory
    let mut data = fs::read(&target_path).map_err(|_| "DLL file not found!")?;

    // 3. Prepare Byte Pattern based on selected game
    let (head, body_prefix, tail) = match game {
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

    // 4. Search for the pattern in the file
    let mut found = false;
    let new_val_bytes = (percentage / 100.0).to_le_bytes(); // Convert % to Float Bytes
    let full_pattern_len = head.len() + body_prefix.len() + 4 + tail.len();

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

    if found {
        fs::write(&target_path, data).map_err(|_| "Failed to write file (Write Protected)!")?;
        Ok(format!("SUCCESS: {:?} Patch Applied!", game))
    } else {
        Err("ERROR: Pattern Not Found! Is the file already patched?".to_string())
    }
}

/// Function to restore the file to its original state
pub fn execute_restore(folder: &PathBuf) -> Result<String, String> {
    let file_target = "empire.retail.dll";
    let file_backup = "empire.retail.dll.bak";
    let target_path = folder.join(file_target);
    let backup_path = folder.join(file_backup);

    if backup_path.exists() {
        let _ = fs::remove_file(&target_path);
        fs::rename(&backup_path, &target_path).map_err(|_| "Failed to restore file!")?;
        Ok("RESTORE SUCCESS: File is back to original!".to_string())
    } else {
        Ok("INFO: No backup found, file is already original.".to_string())
    }
}