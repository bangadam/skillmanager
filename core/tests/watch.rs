use skillmanager_core::{watch, Agent, Root, ScanState};
use std::{fs, sync::mpsc, time::{Duration, Instant}};

fn wait_for(rx: &mpsc::Receiver<ScanState>, matches: impl Fn(&ScanState) -> bool) -> ScanState {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let state = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap();
        if matches(&state) { return state; }
    }
}

#[test]
fn external_create_edit_and_delete_publish_fresh_scans() {
    let temp = tempfile::tempdir().unwrap();
    let roots = [Root { agent: Agent::Codex, path: temp.path().to_path_buf() }];
    let (tx, rx) = mpsc::channel();
    let _guard = watch(&roots, &temp.path().join("roots.json"), temp.path(), move |state| { tx.send(state).unwrap(); }).unwrap();

    let folder = temp.path().join("fresh");
    fs::create_dir(&folder).unwrap();
    let markdown = folder.join("SKILL.md");
    fs::write(&markdown, "---\nname: Fresh\ndescription: Initial\n---\n").unwrap();
    let state = wait_for(&rx, |state| state.skills.iter().any(|skill| skill.id == "fresh" && skill.description == "Initial"));
    assert_eq!(state.skills[0].presences[0].agent, Agent::Codex);

    fs::write(&markdown, "---\nname: Fresh\ndescription: Updated\n---\n").unwrap();
    wait_for(&rx, |state| state.skills.iter().any(|skill| skill.id == "fresh" && skill.description == "Updated"));

    fs::remove_dir_all(folder).unwrap();
    wait_for(&rx, |state| state.skills.iter().all(|skill| skill.id != "fresh"));
}

#[test]
fn missing_roots_do_not_prevent_watching_existing_roots() {
    let temp = tempfile::tempdir().unwrap();
    let roots = [
        Root { agent: Agent::ClaudeCode, path: temp.path().join("absent") },
        Root { agent: Agent::Omp, path: temp.path().to_path_buf() },
    ];
    let (tx, rx) = mpsc::channel();
    let _guard = watch(&roots, &temp.path().join("roots.json"), temp.path(), move |state| { tx.send(state).unwrap(); }).unwrap();
    let folder = temp.path().join("visible");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("SKILL.md"), "---\nname: Visible\n---\n").unwrap();
    wait_for(&rx, |state| state.skills.iter().any(|skill| skill.id == "visible"));
    let new_root = temp.path().join("absent");
    fs::create_dir(&new_root).unwrap();
    let new_folder = new_root.join("late");
    fs::create_dir(&new_folder).unwrap();
    fs::write(new_folder.join("SKILL.md"), "---\nname: Late arrival\n---\n").unwrap();
    wait_for(&rx, |state| state.skills.iter().any(|skill| skill.id == "late" && skill.presences[0].agent == Agent::ClaudeCode));

    let _guard = watch(&roots[..1], &temp.path().join("roots.json"), temp.path(), |_| {}).unwrap();
}

#[test]
fn deleted_root_can_be_recreated_without_restarting_watcher() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("skills");
    fs::create_dir(&root).unwrap();
    let roots = [Root { agent: Agent::Cursor, path: root.clone() }];
    let (tx, rx) = mpsc::channel();
    let _guard = watch(&roots, &temp.path().join("roots.json"), temp.path(), move |state| { tx.send(state).unwrap(); }).unwrap();
    fs::remove_dir(&root).unwrap();
    wait_for(&rx, |state| state.skills.is_empty());
    fs::create_dir(&root).unwrap();
    let folder = root.join("restored");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("SKILL.md"), "---\nname: Restored\n---\n").unwrap();
    wait_for(&rx, |state| state.skills.iter().any(|skill| skill.id == "restored"));
}

#[test]
fn config_changes_rearm_extra_roots_and_managed_sources_emit_full_state() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    let canonical = home.join("canonical");
    let extra = home.join("extra");
    let plugin = home.join(".claude/plugins/cache/vendor/skill/skills");
    fs::create_dir_all(&canonical).unwrap();
    fs::create_dir_all(&extra).unwrap();
    fs::create_dir_all(&plugin).unwrap();
    let roots = [Root { agent: Agent::ClaudeCode, path: canonical.clone() }];
    let config = home.join("roots.json");
    let (tx, rx) = mpsc::channel();
    let _guard = watch(&roots, &config, home, move |state| { tx.send(state).unwrap(); }).unwrap();
    fs::write(&config, format!(r#"[{{"agent":"codex","path":"{}"}}]"#, extra.display())).unwrap();
    wait_for(&rx, |state| state.roots.len() == 2);
    let folder = extra.join("new");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("SKILL.md"), "extra").unwrap();
    wait_for(&rx, |state| state.skills.iter().any(|skill| skill.id == "new" && skill.managed_source.is_none()));
    let folder = plugin.join("managed");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("SKILL.md"), "plugin").unwrap();
    wait_for(&rx, |state| state.skills.iter().any(|skill| skill.id == "managed" && skill.managed_source.is_some()));
}

#[test]
fn newly_created_plugin_cache_is_discovered_without_a_canonical_root_event() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    let canonical = home.join("canonical");
    fs::create_dir(&canonical).unwrap();
    let roots = [Root { agent: Agent::ClaudeCode, path: canonical }];
    let (tx, rx) = mpsc::channel();
    let _guard = watch(&roots, &home.join("roots.json"), home, move |state| { tx.send(state).unwrap(); }).unwrap();
    let plugin = home.join(".claude/plugins/cache/vendor/skill/skills/managed");
    fs::create_dir_all(&plugin).unwrap();
    fs::write(plugin.join("SKILL.md"), "new plugin").unwrap();
    wait_for(&rx, |state| state.skills.iter().any(|skill| skill.id == "managed" && skill.managed_source.is_some()));
}
