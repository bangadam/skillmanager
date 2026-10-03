use skillmanager_core::{choose_master_in, link, link_everywhere, missing, scan, scan_state, Agent, ManagedKind, ManagedRoot, PresenceKind, Root};
use std::{fs, os::unix::fs::symlink, path::Path};

fn roots(base: &Path) -> Vec<Root> {
    [Agent::ClaudeCode, Agent::Codex, Agent::Cursor, Agent::Omp]
        .into_iter().enumerate().map(|(n, agent)| Root { agent, path: base.join(format!("root-{n}")) }).collect()
}

fn skill(root: &Root, name: &str, content: &str) {
    fs::create_dir_all(root.path.join(name)).unwrap();
    fs::write(root.path.join(name).join("SKILL.md"), content).unwrap();
}

#[test]
fn add_to_links_to_master_and_guard_preserves_occupied_target() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    skill(&roots[0], "alpha", "one");
    let result = link(&roots, "alpha", &roots[1].path).unwrap();
    assert_eq!(result[0].presences[1].kind, PresenceKind::Link);
    assert_eq!(fs::read_link(roots[1].path.join("alpha")).unwrap(), roots[0].path.join("alpha"));
    fs::write(roots[0].path.join("alpha/SKILL.md"), "edited").unwrap();
    assert_eq!(fs::read_to_string(roots[1].path.join("alpha/SKILL.md")).unwrap(), "edited");
    let error = link(&roots, "alpha", &roots[1].path).unwrap_err().to_string();
    assert!(error.contains("already holds"), "{error}");
    assert_eq!(fs::read_link(roots[1].path.join("alpha")).unwrap(), roots[0].path.join("alpha"));
    skill(&roots[2], "alpha", "divergent");
    let error = link(&roots, "alpha", &roots[2].path).unwrap_err().to_string();
    assert!(error.contains("already holds") && error.contains("root-2"), "{error}");
    assert_eq!(fs::read_to_string(roots[2].path.join("alpha/SKILL.md")).unwrap(), "divergent");

    // A configured root can contain only a link while its real master lives outside these roots.
    let external = temp.path().join("external");
    fs::create_dir_all(external.join("remote")).unwrap();
    fs::write(external.join("remote/SKILL.md"), "outside").unwrap();
    symlink(external.join("remote"), roots[0].path.join("remote")).unwrap();
    link(&roots, "remote", &roots[3].path).unwrap();
    assert_eq!(fs::read_link(roots[3].path.join("remote")).unwrap(), fs::canonicalize(external.join("remote")).unwrap());
}

#[test]
fn selecting_master_trashes_identical_copy_then_links_it_but_refuses_divergence() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    let trash = temp.path().join("Trash");
    skill(&roots[0], "alpha", "same");
    skill(&roots[1], "alpha", "same");
    let result = choose_master_in(&roots, "alpha", &roots[1].path, &trash).unwrap();
    assert_eq!(result[0].presences[0].kind, PresenceKind::Link);
    assert_eq!(result[0].presences[1].kind, PresenceKind::RealDir);
    assert_eq!(fs::read_to_string(trash.join("alpha/SKILL.md")).unwrap(), "same");
    assert_eq!(fs::read_link(roots[0].path.join("alpha")).unwrap(), roots[1].path.join("alpha"));
    skill(&roots[2], "alpha", "different");
    let error = choose_master_in(&roots, "alpha", &roots[1].path, &trash).unwrap_err().to_string();
    assert!(error.contains("divergent") && error.contains("root-2"), "{error}");
    assert!(fs::symlink_metadata(roots[2].path.join("alpha")).unwrap().is_dir());
}

#[test]
fn master_selection_preserves_non_markdown_assets_and_empty_directories() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    let trash = temp.path().join("Trash");
    skill(&roots[0], "alpha", "same");
    skill(&roots[1], "alpha", "same");
    fs::write(roots[1].path.join("alpha/attachment.png"), b"asset").unwrap();
    let error = choose_master_in(&roots, "alpha", &roots[0].path, &trash).unwrap_err().to_string();
    assert!(error.contains("divergent"), "{error}");
    assert_eq!(fs::read(roots[1].path.join("alpha/attachment.png")).unwrap(), b"asset");
    assert!(!trash.exists());
    fs::remove_file(roots[1].path.join("alpha/attachment.png")).unwrap();
    fs::create_dir(roots[1].path.join("alpha/empty")).unwrap();
    assert!(choose_master_in(&roots, "alpha", &roots[0].path, &trash).unwrap_err().to_string().contains("divergent"));
    assert!(roots[1].path.join("alpha/empty").is_dir());
}

#[test]
fn later_divergence_refuses_before_trashing_earlier_identical_copy() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    let trash = temp.path().join("Trash");
    skill(&roots[0], "alpha", "same");
    skill(&roots[1], "alpha", "same");
    skill(&roots[2], "alpha", "other");
    let error = choose_master_in(&roots, "alpha", &roots[0].path, &trash).unwrap_err().to_string();
    assert!(error.contains("root-2"), "{error}");
    assert!(fs::symlink_metadata(roots[1].path.join("alpha")).unwrap().is_dir());
    assert!(!trash.exists());
}

#[test]
fn missing_counts_all_configured_roots_and_link_everywhere_reports_partial_failure() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    skill(&roots[0], "alpha", "body");
    skill(&roots[2], "alpha", "body");
    skill(&roots[0], "complete", "body");
    for root in &roots[1..] {
        fs::create_dir_all(&root.path).unwrap();
        symlink(roots[0].path.join("complete"), root.path.join("complete")).unwrap();
    }
    assert_eq!(missing(&scan(&roots).unwrap(), &roots).iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), vec!["alpha"]);
    let error = link_everywhere(&roots, "alpha").unwrap_err().to_string();
    assert!(error.contains("master"), "{error}");
    let trash = temp.path().join("Trash");
    choose_master_in(&roots, "alpha", &roots[0].path, &trash).unwrap();
    fs::create_dir_all(roots[3].path.join("alpha")).unwrap(); // Occupied after scan, but not a loadable skill.
    let result = link_everywhere(&roots, "alpha").unwrap();
    assert_eq!(result.failures.len(), 1);
    assert!(result.failures[0].contains("root-3"));
    assert_eq!(fs::read_link(roots[1].path.join("alpha")).unwrap(), roots[0].path.join("alpha"));
    assert_eq!(result.skills[0].presences.len(), 3);
    assert!(fs::symlink_metadata(roots[3].path.join("alpha")).unwrap().is_dir());
}

#[test]
fn missing_uses_extra_roots_and_excludes_managed_skills() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    let extra = temp.path().join("extra");
    let plugin = temp.path().join("plugin");
    skill(&roots[0], "alpha", "owned");
    skill(&roots[1], "alpha", "owned");
    skill(&Root { agent: Agent::Omp, path: plugin.clone() }, "plugin-only", "plugin");
    let config = temp.path().join("roots.json");
    fs::write(&config, format!(r#"[{{"agent":"codex","path":"{}"}}]"#, extra.display())).unwrap();
    let state = scan_state(&roots, &config, &[ManagedRoot { agent: Agent::Omp, path: plugin, kind: ManagedKind::Plugin, plugin: None }]).unwrap();
    assert_eq!(state.roots.len(), 5);
    let gaps = missing(&state.skills, &state.roots);
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].id, "alpha");
    assert!(!gaps.iter().any(|skill| skill.managed_source.is_some()));
}

#[test]
fn linking_everywhere_uses_extra_root_from_config() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    skill(&roots[0], "alpha", "body");
    let extra = temp.path().join("extra");
    let config = temp.path().join("roots.json");
    fs::write(&config, format!(r#"[{{"agent":"cursor","path":"{}"}}]"#, extra.display())).unwrap();
    let state = scan_state(&roots, &config, &[]).unwrap();
    let result = link_everywhere(&state.roots, "alpha").unwrap();
    assert!(result.failures.is_empty());
    assert_eq!(result.skills[0].presences.len(), 5);
    assert_eq!(fs::read_link(extra.join("alpha")).unwrap(), roots[0].path.join("alpha"));
}

