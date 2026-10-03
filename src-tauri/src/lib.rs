#[tauri::command]
fn scan_skills() -> Result<Vec<skillmanager_core::SkillState>, String> {
    let roots = skillmanager_core::default_roots().map_err(|error| error.to_string())?;
    skillmanager_core::scan(&roots).map_err(|error| error.to_string())
}

#[tauri::command]
fn configured_roots() -> Result<Vec<skillmanager_core::Root>, String> {
    skillmanager_core::default_roots().map_err(|error| error.to_string())
}

#[tauri::command]
fn link_skill(id: String, target_root: String) -> Result<Vec<skillmanager_core::SkillState>, String> {
    let roots = configured_roots()?;
    skillmanager_core::link(&roots, &id, std::path::Path::new(&target_root)).map_err(|error| error.to_string())
}

#[tauri::command]
fn choose_master(id: String, chosen_root: String) -> Result<Vec<skillmanager_core::SkillState>, String> {
    let roots = configured_roots()?;
    skillmanager_core::choose_master(&roots, &id, std::path::Path::new(&chosen_root)).map_err(|error| error.to_string())
}

#[tauri::command]
fn link_everywhere(id: String) -> Result<skillmanager_core::LinkEverywhereResult, String> {
    let roots = configured_roots()?;
    skillmanager_core::link_everywhere(&roots, &id).map_err(|error| error.to_string())
}

#[tauri::command]
fn missing_skills() -> Result<Vec<skillmanager_core::SkillState>, String> {
    let roots = configured_roots()?;
    let skills = skillmanager_core::scan(&roots).map_err(|error| error.to_string())?;
    Ok(skillmanager_core::missing(&skills, &roots).into_iter().cloned().collect())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_skills, configured_roots, link_skill, choose_master, link_everywhere, missing_skills])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
