use skillmanager_core::{replace_with_link_in, scan, skill_diff, Agent, PresenceKind, Root};
use std::{fs, os::unix::fs::symlink, path::Path};

fn roots(base: &Path) -> Vec<Root> {
    [Agent::ClaudeCode, Agent::Codex, Agent::Cursor].into_iter().enumerate()
        .map(|(i, agent)| Root { agent, path: base.join(format!("root-{i}")) }).collect()
}
fn copy(root: &Root, text: Option<&str>) {
    let path = root.path.join("alpha");
    fs::create_dir_all(&path).unwrap();
    if let Some(text) = text { fs::write(path.join("SKILL.md"), text).unwrap(); }
}

#[test]
fn scan_reports_only_different_real_copies_and_includes_missing_markdown() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], Some("first word"));
    copy(&roots[1], Some("first word"));
    assert!(scan(&roots).unwrap()[0].divergent_pairs.is_empty());
    fs::write(roots[1].path.join("alpha/SKILL.md"), "second word").unwrap();
    let state = scan(&roots).unwrap();
    assert_eq!(state[0].divergent_pairs[0].left, roots[0].path.join("alpha"));
    assert_eq!(state[0].divergent_pairs[0].right, roots[1].path.join("alpha"));
    assert_eq!(skill_diff(&roots, "alpha", &roots[0].path, &roots[1].path).unwrap().right.as_deref(), Some("second word"));
    fs::remove_file(roots[1].path.join("alpha/SKILL.md")).unwrap();
    assert_eq!(scan(&roots).unwrap()[0].divergent_pairs.len(), 1);
    assert_eq!(skill_diff(&roots, "alpha", &roots[0].path, &roots[1].path).unwrap().right, None);
    copy(&roots[2], None);
    let state = scan(&roots).unwrap();
    assert_eq!(state[0].presences.len(), 3);
    assert_eq!(state[0].divergent_pairs.len(), 2); // Both missing files agree with each other.
}

#[test]
fn replacing_trashes_target_before_linking_and_preserves_other_roots() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    let trash = temp.path().join("Trash");
    copy(&roots[0], Some("master"));
    copy(&roots[1], Some("old copy"));
    copy(&roots[2], Some("third copy"));
    fs::write(roots[1].path.join("alpha/notes.txt"), "recover me").unwrap();
    let error = replace_with_link_in(&roots, "alpha", &roots[0].path, &roots[0].path, &trash).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput); // Cannot replace the master itself.
    assert!(!trash.exists());
    let result = replace_with_link_in(&roots, "alpha", &roots[1].path, &roots[0].path, &trash).unwrap();
    assert_eq!(fs::read_to_string(trash.join("alpha/SKILL.md")).unwrap(), "old copy");
    assert_eq!(fs::read_to_string(trash.join("alpha/notes.txt")).unwrap(), "recover me");
    assert_eq!(fs::read_link(roots[1].path.join("alpha")).unwrap(), roots[0].path.join("alpha"));
    assert_eq!(result[0].presences[1].kind, PresenceKind::Link);
    assert_eq!(fs::read_to_string(roots[2].path.join("alpha/SKILL.md")).unwrap(), "third copy");
    assert_eq!(result[0].divergent_pairs.len(), 2);
    assert!(replace_with_link_in(&roots, "alpha", &roots[1].path, &roots[0].path, &trash).is_err());
    assert_eq!(fs::read_to_string(roots[2].path.join("alpha/SKILL.md")).unwrap(), "third copy");
}

#[test]
fn replacement_refuses_missing_target_and_missing_master_without_trashing() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    let trash = temp.path().join("Trash");
    copy(&roots[0], Some("master"));
    assert!(replace_with_link_in(&roots, "alpha", &roots[1].path, &roots[0].path, &trash).is_err());
    assert!(!trash.exists());
    copy(&roots[1], Some("old"));
    fs::remove_dir_all(roots[0].path.join("alpha")).unwrap();
    assert!(replace_with_link_in(&roots, "alpha", &roots[1].path, &roots[0].path, &trash).is_err());
    assert!(!trash.exists());
    assert!(roots[1].path.join("alpha").is_dir());
}

#[test]
fn failed_trash_move_never_creates_link_or_removes_target() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], Some("winner"));
    copy(&roots[1], Some("loser"));
    let blocked_trash = temp.path().join("blocked-trash");
    fs::write(&blocked_trash, "not a directory").unwrap();
    assert!(replace_with_link_in(&roots, "alpha", &roots[1].path, &roots[0].path, &blocked_trash).is_err());
    assert!(fs::symlink_metadata(roots[1].path.join("alpha")).unwrap().is_dir());
    assert_eq!(fs::read_to_string(roots[1].path.join("alpha/SKILL.md")).unwrap(), "loser");
}

#[test]
fn symlinked_copy_does_not_warn_when_it_points_to_master() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], Some("same"));
    fs::create_dir_all(&roots[1].path).unwrap();
    symlink(roots[0].path.join("alpha"), roots[1].path.join("alpha")).unwrap();
    assert!(scan(&roots).unwrap()[0].divergent_pairs.is_empty());
}
