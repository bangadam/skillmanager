#[tauri::command]
fn scan_skills() -> Result<Vec<skillmanager_core::SkillState>, String> {
    let roots = skillmanager_core::default_roots().map_err(|error| error.to_string())?;
    skillmanager_core::scan(&roots).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_skills])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
