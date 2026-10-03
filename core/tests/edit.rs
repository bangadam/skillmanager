use skillmanager_core::{edit_skill, read_skill_md, replace_with_link_in, scan, Agent, PresenceKind, Root};
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
fn saving_master_updates_links_and_reports_only_written_files() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], Some("old"));
    fs::create_dir_all(&roots[1].path).unwrap();
    symlink(roots[0].path.join("alpha"), roots[1].path.join("alpha")).unwrap();
    assert_eq!(read_skill_md(&roots, "alpha").unwrap(), "old");
    let result = edit_skill(&roots, "alpha", "new").unwrap();
    assert_eq!(result.files_written, vec![roots[0].path.join("alpha/SKILL.md")]);
    assert_eq!(fs::read_to_string(roots[1].path.join("alpha/SKILL.md")).unwrap(), "new");
    assert_eq!(result.skills[0].presences[1].kind, PresenceKind::Link);
}

#[test]
fn identical_real_copies_receive_same_save_in_root_order() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], Some("same"));
    copy(&roots[1], Some("same"));
    let result = edit_skill(&roots, "alpha", "updated").unwrap();
    assert_eq!(result.files_written, vec![roots[0].path.join("alpha/SKILL.md"), roots[1].path.join("alpha/SKILL.md")]);
    assert_eq!(read_skill_md(&roots, "alpha").unwrap(), "updated");
    assert_eq!(fs::read_to_string(roots[1].path.join("alpha/SKILL.md")).unwrap(), "updated");
    assert!(result.skills[0].divergent_pairs.is_empty());
}

#[test]
fn divergent_copy_blocks_all_writes_until_resolved_with_diff_flow() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], Some("master"));
    copy(&roots[1], Some("different"));
    let error = edit_skill(&roots, "alpha", "new").unwrap_err();
    assert!(error.to_string().contains(&roots[1].path.display().to_string()));
    assert_eq!(fs::read_to_string(roots[0].path.join("alpha/SKILL.md")).unwrap(), "master");
    assert_eq!(fs::read_to_string(roots[1].path.join("alpha/SKILL.md")).unwrap(), "different");
    replace_with_link_in(&roots, "alpha", &roots[1].path, &roots[0].path, &temp.path().join("Trash")).unwrap();
    let result = edit_skill(&roots, "alpha", "new").unwrap();
    assert_eq!(result.files_written, vec![roots[0].path.join("alpha/SKILL.md")]);
    assert_eq!(fs::read_to_string(roots[1].path.join("alpha/SKILL.md")).unwrap(), "new");
}

#[test]
fn empty_content_is_saved_and_missing_copy_is_guarded() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], Some("previous"));
    let result = edit_skill(&roots, "alpha", "").unwrap();
    assert_eq!(result.files_written, vec![roots[0].path.join("alpha/SKILL.md")]);
    assert_eq!(fs::read_to_string(roots[0].path.join("alpha/SKILL.md")).unwrap(), "");
    copy(&roots[1], None);
    assert_eq!(scan(&roots).unwrap()[0].presences.len(), 2);
    let error = edit_skill(&roots, "alpha", "next").unwrap_err();
    assert!(error.to_string().contains(&roots[1].path.display().to_string()));
    assert_eq!(fs::read_to_string(roots[0].path.join("alpha/SKILL.md")).unwrap(), "");
}

#[test]
fn visible_skill_with_missing_first_copy_reads_empty_and_guards_existing_content() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], None);
    copy(&roots[1], Some("available"));
    assert_eq!(read_skill_md(&roots, "alpha").unwrap(), "");
    let error = edit_skill(&roots, "alpha", "new").unwrap_err();
    assert!(error.to_string().contains(&roots[1].path.display().to_string()));
    assert_eq!(fs::read_to_string(roots[1].path.join("alpha/SKILL.md")).unwrap(), "available");
}

#[test]
fn missing_master_markdown_is_created_when_other_copies_are_empty() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], Some("existing"));
    copy(&roots[1], None);
    fs::remove_file(roots[0].path.join("alpha/SKILL.md")).unwrap();
    // An entirely missing skill is deliberately invisible until one copy has SKILL.md.
    assert!(scan(&roots).unwrap().is_empty());
    copy(&roots[2], Some(""));
    assert_eq!(read_skill_md(&roots, "alpha").unwrap(), "");
    let result = edit_skill(&roots, "alpha", "created").unwrap();
    assert_eq!(result.files_written, vec![roots[0].path.join("alpha/SKILL.md"), roots[2].path.join("alpha/SKILL.md")]);
    assert_eq!(fs::read_to_string(roots[0].path.join("alpha/SKILL.md")).unwrap(), "created");
    assert!(!roots[1].path.join("alpha/SKILL.md").exists());
}

#[test]
fn directory_without_any_markdown_is_invisible_not_editable() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    copy(&roots[0], None);
    assert!(scan(&roots).unwrap().is_empty());
    assert_eq!(read_skill_md(&roots, "alpha").unwrap_err().kind(), std::io::ErrorKind::NotFound);
}
