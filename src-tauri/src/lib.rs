use tauri::Emitter;

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
fn skill_diff(id: String, left_root: String, right_root: String) -> Result<skillmanager_core::SkillDiff, String> {
    let roots = configured_roots()?;
    skillmanager_core::skill_diff(&roots, &id, std::path::Path::new(&left_root), std::path::Path::new(&right_root)).map_err(|error| error.to_string())
}

#[tauri::command]
fn replace_with_link(id: String, target_root: String, master_root: String) -> Result<skillmanager_core::ScanState, String> {
    let roots = configured_roots()?;
    skillmanager_core::replace_with_link(&roots, &id, std::path::Path::new(&target_root), std::path::Path::new(&master_root)).map_err(|error| error.to_string())?;
    scan_skills()
}

#[tauri::command]
fn link_everywhere(id: String) -> Result<skillmanager_core::LinkEverywhereResult, String> {
    let roots = configured_roots()?;
    skillmanager_core::link_everywhere(&roots, &id).map_err(|error| error.to_string())
}

#[tauri::command]
fn rename_skill(id: String, new_name: String) -> Result<skillmanager_core::ScanState, String> {
    let roots = configured_roots()?;
    let home = std::env::home_dir().ok_or("home directory unavailable")?;
    let journal = skillmanager_core::config_path(&home).with_file_name("renames.json");
    skillmanager_core::rename(&roots, &id, &new_name, &journal).map_err(|error| error.to_string())?;
    scan_skills()
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
        .setup(|app| {
            let handle = app.handle().clone();
            std::thread::Builder::new().name("skill-watcher".into()).spawn(move || {
                let roots = match skillmanager_core::default_roots() {
                    Ok(roots) => roots,
                    Err(error) => { eprintln!("skill watcher roots: {error}"); return; }
                };
                let home = match std::env::home_dir() {
                    Some(home) => home,
                    None => { eprintln!("skill watcher: home directory unavailable"); return; }
                };
                let config = skillmanager_core::config_path(&home);
                let (watch_roots, config_error, _) = skillmanager_core::configured_roots(&roots, &config);
                if let Some(error) = config_error { eprintln!("skill watcher config: {error}"); }
                // ponytail: managed/plugin caches are read-only display and aren't watched;
                // watch them if external managed-cache updates must appear without another event.
                match skillmanager_core::watch(&watch_roots, move |_| {
                    let managed = match skillmanager_core::default_managed_roots(&home) {
                        Ok(managed) => managed,
                        Err(error) => { eprintln!("skill watcher managed roots: {error}"); return; }
                    };
                    match skillmanager_core::scan_state(&roots, &config, &managed) {
                        Ok(state) => {
                            if let Err(error) = handle.emit("skills-changed", state) {
                                eprintln!("skill watcher emit: {error}");
                            }
                        }
                        Err(error) => eprintln!("skill watcher full rescan: {error}"),
                    }
                }) {
                    Ok(_guard) => loop { std::thread::park(); },
                    Err(error) => eprintln!("skill watcher startup: {error}"),
                }
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![scan_skills, find_duplicates, configured_roots, link_skill, choose_master, link_everywhere, missing_skills, rename_skill, skill_diff, replace_with_link])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
