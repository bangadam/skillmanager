use skillmanager_core::{scan, Agent, DateSource, PresenceKind, Root};
use std::{fs, path::Path};

fn skill(root: &Path, folder: &str, markdown: &str) {
    let dir = root.join(folder);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), markdown).unwrap();
}

#[test]
fn groups_by_folder_and_reports_display_metadata_and_each_agent_copy() {
    let temp = tempfile::tempdir().unwrap();
    let claude = temp.path().join("claude");
    let codex = temp.path().join("codex");
    fs::create_dir_all(&claude).unwrap();
    fs::create_dir_all(&codex).unwrap();
    skill(&claude, "alpha", "---\nname: Friendly Alpha\ndescription: First copy\n---\nBody");
    skill(&codex, "alpha", "---\nname: Different name\ndescription: Second copy\n---\nBody");
    skill(&codex, "beta", "---\nname: Beta\ndescription: Another skill\n---\nBody");
    let result = scan(&[
        Root { agent: Agent::ClaudeCode, path: claude },
        Root { agent: Agent::Codex, path: codex },
    ]).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "alpha");
    assert_eq!(result[0].name, "Friendly Alpha");
    assert_eq!(result[0].description, "First copy");
    assert_eq!(result[0].presences.len(), 2);
    assert_eq!(result[0].presences[0].agent, Agent::ClaudeCode);
    assert_eq!(result[0].presences[0].kind, PresenceKind::RealDir);
    assert_eq!(result[0].presences[1].agent, Agent::Codex);
    assert_eq!(result[1].id, "beta");
}

#[test]
fn ignores_missing_roots_non_skills_and_nested_subfolders() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("skills");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("not-a-skill")).unwrap();
    fs::write(root.join("loose.md"), "---\nname: loose\n---").unwrap();
    skill(&root.join("not-a-skill"), "nested", "---\nname: nested\n---");
    assert!(scan(&[Root { agent: Agent::Cursor, path: root.clone() }]).unwrap().is_empty());
    assert!(scan(&[Root { agent: Agent::Omp, path: root.join("absent") }]).unwrap().is_empty());
    skill(&root, "visible", "Text without frontmatter");
    let result = scan(&[Root { agent: Agent::Cursor, path: root }]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "visible");
    assert_eq!(result[0].name, "visible");
    assert_eq!(result[0].description, "");
}

#[test]
fn malformed_and_partial_frontmatter_degrades_per_field() {
    let temp = tempfile::tempdir().unwrap();
    skill(temp.path(), "invalid", "---\nname: [not closed\n---\nbody");
    skill(temp.path(), "no-name", "---\ndescription: Still useful\n---\nbody");
    skill(temp.path(), "no-description", "---\nname: Named\n---\nbody");
    skill(temp.path(), "wrong-types", "---\nname: [array]\ndescription: {unexpected: object}\n---\nbody");
    let result = scan(&[Root { agent: Agent::Omp, path: temp.path().to_path_buf() }]).unwrap();
    let find = |id| result.iter().find(|s| s.id == id).unwrap();
    assert_eq!((find("invalid").name.as_str(), find("invalid").description.as_str()), ("invalid", ""));
    assert_eq!((find("no-name").name.as_str(), find("no-name").description.as_str()), ("no-name", "Still useful"));
    assert_eq!((find("no-description").name.as_str(), find("no-description").description.as_str()), ("Named", ""));
    assert_eq!((find("wrong-types").name.as_str(), find("wrong-types").description.as_str()), ("wrong-types", ""));
}

#[cfg(unix)]
#[test]
fn symlink_presence_uses_target_content_but_reports_link() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let linked = temp.path().join("linked");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&linked).unwrap();
    skill(&source, "one", "---\nname: Linked name\ndescription: Linked content\n---\nbody");
    std::os::unix::fs::symlink(source.join("one"), linked.join("one")).unwrap();
    let result = scan(&[Root { agent: Agent::Codex, path: linked }]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Linked name");
    assert_eq!(result[0].description, "Linked content");
    assert_eq!(result[0].presences[0].kind, PresenceKind::Link);
}

#[test]
fn date_source_honestly_matches_filesystem_birth_or_fallback() {
    let temp = tempfile::tempdir().unwrap();
    skill(temp.path(), "dated", "---\nname: Dated\n---");
    let metadata = fs::metadata(temp.path().join("dated")).unwrap();
    let result = scan(&[Root { agent: Agent::ClaudeCode, path: temp.path().to_path_buf() }]).unwrap();
    let expected = if metadata.created().is_ok() { DateSource::Created }
        else if metadata.modified().is_ok() { DateSource::Modified }
        else { DateSource::Unavailable };
    assert_eq!(result[0].date_source, expected);
    assert_eq!(result[0].date_millis.is_some(), expected != DateSource::Unavailable);
}
