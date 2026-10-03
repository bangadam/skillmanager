#[tauri::command]
fn scan_skills() -> Result<skillmanager_core::ScanState, String> {
    let roots = skillmanager_core::default_roots().map_err(|error| error.to_string())?;
    let home = std::env::home_dir().ok_or("home directory unavailable")?;
    let managed = skillmanager_core::default_managed_roots(&home).map_err(|error| error.to_string())?;
    skillmanager_core::scan_state(&roots, &skillmanager_core::config_path(&home), &managed)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn find_duplicates() -> Result<Vec<skillmanager_core::DuplicatePair>, String> {
    let state = scan_skills()?;
    skillmanager_core::find_duplicates(&state.skills).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_skills, find_duplicates])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
