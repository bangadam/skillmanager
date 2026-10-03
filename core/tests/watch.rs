use skillmanager_core::{watch, Agent, Root, SkillState};
use std::{fs, sync::mpsc, time::{Duration, Instant}};

fn wait_for(rx: &mpsc::Receiver<Vec<SkillState>>, matches: impl Fn(&[SkillState]) -> bool) -> Vec<SkillState> {
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
    let _guard = watch(&roots, move |state| { tx.send(state).unwrap(); }).unwrap();

    let folder = temp.path().join("fresh");
    fs::create_dir(&folder).unwrap();
    let markdown = folder.join("SKILL.md");
    fs::write(&markdown, "---\nname: Fresh\ndescription: Initial\n---\n").unwrap();
    let state = wait_for(&rx, |state| state.iter().any(|skill| skill.id == "fresh" && skill.description == "Initial"));
    assert_eq!(state[0].presences[0].agent, Agent::Codex);

    fs::write(&markdown, "---\nname: Fresh\ndescription: Updated\n---\n").unwrap();
    wait_for(&rx, |state| state.iter().any(|skill| skill.id == "fresh" && skill.description == "Updated"));

    fs::remove_dir_all(folder).unwrap();
    wait_for(&rx, |state| state.iter().all(|skill| skill.id != "fresh"));
}

#[test]
fn missing_roots_do_not_prevent_watching_existing_roots() {
    let temp = tempfile::tempdir().unwrap();
    let roots = [
        Root { agent: Agent::ClaudeCode, path: temp.path().join("absent") },
        Root { agent: Agent::Omp, path: temp.path().to_path_buf() },
    ];
    let (tx, rx) = mpsc::channel();
    let _guard = watch(&roots, move |state| { tx.send(state).unwrap(); }).unwrap();
    let folder = temp.path().join("visible");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("SKILL.md"), "---\nname: Visible\n---\n").unwrap();
    wait_for(&rx, |state| state.iter().any(|skill| skill.id == "visible"));

    let _guard = watch(&roots[..1], |_| {}).unwrap();
}

#[test]
fn deleted_root_can_be_recreated_without_restarting_watcher() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("skills");
    fs::create_dir(&root).unwrap();
    let roots = [Root { agent: Agent::Cursor, path: root.clone() }];
    let (tx, rx) = mpsc::channel();
    let _guard = watch(&roots, move |state| { tx.send(state).unwrap(); }).unwrap();
    fs::remove_dir(&root).unwrap();
    wait_for(&rx, |state| state.is_empty());
    fs::create_dir(&root).unwrap();
    let folder = root.join("restored");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("SKILL.md"), "---\nname: Restored\n---\n").unwrap();
    wait_for(&rx, |state| state.iter().any(|skill| skill.id == "restored"));
}
