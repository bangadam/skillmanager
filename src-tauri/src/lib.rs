#[tauri::command]
fn scan_skills() -> Result<skillmanager_core::ScanState, String> {
    let roots = skillmanager_core::default_roots().map_err(|error| error.to_string())?;
    let home = std::env::home_dir().ok_or("home directory unavailable")?;
    let managed = skillmanager_core::default_managed_roots(&home).map_err(|error| error.to_string())?;
    skillmanager_core::scan_state(&roots, &skillmanager_core::config_path(&home), &managed)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn configured_roots() -> Result<Vec<skillmanager_core::Root>, String> {
    let roots = skillmanager_core::default_roots().map_err(|error| error.to_string())?;
    let home = std::env::home_dir().ok_or("home directory unavailable")?;
    let (configured, error, _) = skillmanager_core::configured_roots(&roots, &skillmanager_core::config_path(&home));
    if let Some(error) = error { return Err(error); }
    Ok(configured)
}

#[tauri::command]
fn link_skill(id: String, target_root: String) -> Result<skillmanager_core::ScanState, String> {
    let roots = configured_roots()?;
    skillmanager_core::link(&roots, &id, std::path::Path::new(&target_root)).map_err(|error| error.to_string())?;
    scan_skills()
}

#[tauri::command]
fn choose_master(id: String, chosen_root: String) -> Result<skillmanager_core::ScanState, String> {
    let roots = configured_roots()?;
    skillmanager_core::choose_master(&roots, &id, std::path::Path::new(&chosen_root)).map_err(|error| error.to_string())?;
    scan_skills()
}

#[tauri::command]
fn link_everywhere(id: String) -> Result<skillmanager_core::LinkEverywhereResult, String> {
    let roots = configured_roots()?;
    skillmanager_core::link_everywhere(&roots, &id).map_err(|error| error.to_string())
}

#[tauri::command]
fn missing_skills() -> Result<Vec<skillmanager_core::SkillState>, String> {
    let state = scan_skills()?;
    Ok(skillmanager_core::missing(&state.skills, &state.roots).into_iter().cloned().collect())
}

#[tauri::command]
fn find_duplicates() -> Result<Vec<skillmanager_core::DuplicatePair>, String> {
    let state = scan_skills()?;
    skillmanager_core::find_duplicates(&state.skills).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_skills, find_duplicates, configured_roots, link_skill, choose_master, link_everywhere, missing_skills])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
