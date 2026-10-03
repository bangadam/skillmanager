use tauri::Emitter;

#[tauri::command]
fn scan_skills() -> Result<Vec<skillmanager_core::SkillState>, String> {
    let roots = skillmanager_core::default_roots().map_err(|error| error.to_string())?;
    skillmanager_core::scan(&roots).map_err(|error| error.to_string())
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
                match skillmanager_core::watch(&roots, move |state| {
                    if let Err(error) = handle.emit("skills-changed", state) {
                        eprintln!("skill watcher emit: {error}");
                    }
                }) {
                    Ok(_guard) => loop { std::thread::park(); },
                    Err(error) => eprintln!("skill watcher startup: {error}"),
                }
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![scan_skills])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
