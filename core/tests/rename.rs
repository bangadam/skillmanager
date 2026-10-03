use skillmanager_core::{rename, scan, Agent, PresenceKind, Root};
use std::{fs, os::unix::fs::symlink, path::Path};

fn roots(base: &Path) -> Vec<Root> {
    [Agent::ClaudeCode, Agent::Codex, Agent::Cursor, Agent::Omp]
        .into_iter().enumerate().map(|(n, agent)| Root { agent, path: base.join(format!("root-{n}")) }).collect()
}

fn skill(root: &Root, name: &str, text: &str) {
    fs::create_dir_all(root.path.join(name)).unwrap();
    fs::write(root.path.join(name).join("SKILL.md"), text).unwrap();
}

#[test]
fn rename_moves_master_repoints_all_links_and_preserves_markdown() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    let original = "---\nname: old\ndescription: Keep this exactly\ntags: [one, two]\n---\n# Body\n\nA line.\n";
    skill(&roots[0], "old", original);
    for root in &roots[1..] {
        fs::create_dir_all(&root.path).unwrap();
        symlink(roots[0].path.join("old"), root.path.join("old")).unwrap();
    }
    let journal = temp.path().join("config/renames.json");
    let result = rename(&roots, "old", "new", &journal).unwrap();
    assert_eq!(result.iter().map(|skill| skill.id.as_str()).collect::<Vec<_>>(), vec!["new"]);
    assert_eq!(result[0].presences.len(), 4);
    assert_eq!(result[0].presences.iter().filter(|presence| presence.kind == PresenceKind::Link).count(), 3);
    assert!(!roots[0].path.join("old").exists());
    assert_eq!(fs::read_to_string(roots[0].path.join("new/SKILL.md")).unwrap(), original.replace("name: old", "name: new"));
    for root in &roots[1..] {
        let link = root.path.join("new");
        assert_eq!(fs::read_link(&link).unwrap(), roots[0].path.join("new"));
        assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(roots[0].path.join("new")).unwrap());
        assert!(fs::symlink_metadata(root.path.join("old")).is_err());
    }
    let entries: serde_json::Value = serde_json::from_slice(&fs::read(&journal).unwrap()).unwrap();
    assert_eq!(entries[0]["from"], "old");
    assert_eq!(entries[0]["to"], "new");
    assert!(entries[0]["at"].as_u64().unwrap() > 0);
    assert_eq!(scan(&roots).unwrap()[0].id, "new");
}

#[test]
fn collision_and_invalid_names_leave_master_links_and_journal_untouched() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    skill(&roots[0], "old", "---\nname: old\n---\nbody");
    skill(&roots[1], "occupied", "existing");
    symlink(roots[0].path.join("old"), roots[1].path.join("old")).unwrap();
    let journal = temp.path().join("renames.json");
    for name in ["occupied", "../escape", "", ".hidden", " trailing "] {
        assert!(rename(&roots, "old", name, &journal).is_err(), "{name:?}");
        assert!(roots[0].path.join("old").is_dir());
        assert_eq!(fs::read_link(roots[1].path.join("old")).unwrap(), roots[0].path.join("old"));
        assert!(!journal.exists());
    }
}

#[test]
fn unlinked_duplicate_requires_linking_first_and_journal_appends_in_order() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    skill(&roots[0], "old", "body one");
    skill(&roots[1], "old", "body two");
    let journal = temp.path().join("renames.json");
    assert!(rename(&roots, "old", "new", &journal).unwrap_err().to_string().contains("link first"));
    assert_eq!(fs::read_to_string(roots[1].path.join("old/SKILL.md")).unwrap(), "body two");
    fs::remove_dir_all(roots[1].path.join("old")).unwrap();
    rename(&roots, "old", "new", &journal).unwrap();
    rename(&roots, "new", "final", &journal).unwrap();
    let entries: serde_json::Value = serde_json::from_slice(&fs::read(journal).unwrap()).unwrap();
    assert_eq!(entries.as_array().unwrap().len(), 2);
    assert_eq!(entries[0]["from"], "old");
    assert_eq!(entries[0]["to"], "new");
    assert_eq!(entries[1]["from"], "new");
    assert_eq!(entries[1]["to"], "final");
}

#[test]
fn rename_adds_absent_frontmatter_name_without_touching_body() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    skill(&roots[0], "old", "---\r\ndescription: still here\r\n---\r\n# Keep\r\nname: prose\r\n");
    rename(&roots, "old", "name with spaces", &temp.path().join("renames.json")).unwrap();
    assert_eq!(fs::read_to_string(roots[0].path.join("name with spaces/SKILL.md")).unwrap(),
        "---\r\ndescription: still here\r\nname: \"name with spaces\"\r\n---\r\n# Keep\r\nname: prose\r\n");
}
