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
fn find_duplicates() -> Result<Vec<skillmanager_core::DuplicatePair>, String> {
    let roots = skillmanager_core::default_roots().map_err(|error| error.to_string())?;
    skillmanager_core::find_duplicates(&roots).map_err(|error| error.to_string())
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
        .invoke_handler(tauri::generate_handler![scan_skills, find_duplicates])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
