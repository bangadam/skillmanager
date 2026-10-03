use skillmanager_core::{find_duplicates, scan, scan_state, Agent, ManagedKind, ManagedRoot, PresenceKind, Root};
use std::{fs, path::Path};

fn skill(root: &Path, folder: &str, markdown: &str) {
    let dir = root.join(folder);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), markdown).unwrap();
}

fn pairs(roots: &[Root]) -> Vec<skillmanager_core::DuplicatePair> {
    find_duplicates(&scan(roots).unwrap()).unwrap()
}

#[test]
fn pairs_exact_text_across_different_names_and_lists_each_copy() {
    let temp = tempfile::tempdir().unwrap();
    let claude = temp.path().join("claude");
    let codex = temp.path().join("codex");
    fs::create_dir(&claude).unwrap();
    fs::create_dir(&codex).unwrap();
    skill(&claude, "alpha", "  GUIDE\n\nRead the manual.  ");
    skill(&codex, "alpha", "  GUIDE\n\nRead the manual.  ");
    skill(&codex, "beta", "guide  read THE manual.");
    let links = temp.path().join("links");
    fs::create_dir(&links).unwrap();
    std::os::unix::fs::symlink(claude.join("alpha"), links.join("alpha")).unwrap();
    let roots = [
        Root { agent: Agent::ClaudeCode, path: claude.clone() },
        Root { agent: Agent::Codex, path: codex.clone() },
        Root { agent: Agent::Cursor, path: links.clone() },
    ];
    let pairs = pairs(&roots);
    assert_eq!(pairs.len(), 1);
    assert_eq!((&pairs[0].a.id[..], &pairs[0].b.id[..]), ("alpha", "beta"));
    assert_eq!(pairs[0].similarity, 1.0);
    assert!(pairs[0].is_exact);
    assert_eq!(pairs[0].a.presences.len(), 3);
    assert_eq!(pairs[0].a.presences[2].path, links.join("alpha"));
    assert_eq!(pairs[0].a.presences[2].kind, PresenceKind::Link);
}

#[test]
fn near_matches_pair_but_unrelated_content_does_not() {
    let temp = tempfile::tempdir().unwrap();
    let shared = "Prepare the workspace before editing the skill. Read its instructions and check existing examples. Save the file only after reviewing the expected output. Keep changes small and explain their purpose. Test the result with a local fixture. Confirm that the expected output reflects the actual files. Document the steps so another agent can reproduce the behavior.";
    let a = format!("{shared}\n\nRun a final check locally.\n\n{shared}");
    let b = format!("{shared}\n\nCollect a review sample instead.\n\n{shared}");
    skill(temp.path(), "alpha", &a);
    skill(temp.path(), "beta", &b);
    skill(temp.path(), "gamma", "Create colorful charts from sales spreadsheets.");
    let roots = [Root { agent: Agent::Omp, path: temp.path().to_path_buf() }];
    let pairs = pairs(&roots);
    assert_eq!(pairs.len(), 1);
    assert_eq!((&pairs[0].a.id[..], &pairs[0].b.id[..]), ("alpha", "beta"));
    assert!(!pairs[0].is_exact);
    assert!(pairs[0].similarity >= 0.8 && pairs[0].similarity < 1.0);
}

#[test]
fn pairing_order_is_deterministic_and_same_name_copies_are_not_pairs() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    skill(&first, "zeta", "identical content");
    skill(&first, "beta", "identical content");
    skill(&second, "beta", "identical content");
    skill(&second, "alpha", "identical content");
    let roots = [Root { agent: Agent::Codex, path: first }, Root { agent: Agent::Cursor, path: second }];
    let ids = |pairs: Vec<skillmanager_core::DuplicatePair>| pairs.into_iter().map(|pair| (pair.a.id, pair.b.id)).collect::<Vec<_>>();
    let expected = vec![("alpha".into(), "beta".into()), ("alpha".into(), "zeta".into()), ("beta".into(), "zeta".into())];
    assert_eq!(ids(pairs(&roots)), expected);
    assert_eq!(ids(pairs(&roots)), expected);
}

#[test]
fn perfect_shingle_overlap_is_not_an_exact_text_match() {
    let temp = tempfile::tempdir().unwrap();
    skill(temp.path(), "alpha", "a b a");
    skill(temp.path(), "beta", "b a b");
    let roots = [Root { agent: Agent::Omp, path: temp.path().to_path_buf() }];
    let pairs = pairs(&roots);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].similarity, 1.0);
    assert!(!pairs[0].is_exact);
}

#[test]
fn includes_extra_and_managed_skills_from_the_shared_scan() {
    let temp = tempfile::tempdir().unwrap();
    let personal = temp.path().join("personal");
    let extra = temp.path().join("extra");
    let managed = temp.path().join("plugin");
    skill(&personal, "alpha", "same content");
    skill(&extra, "beta", "same content");
    skill(&managed, "gamma", "same content");
    let config = temp.path().join("roots.json");
    fs::write(&config, serde_json::to_vec(&[Root { agent: Agent::Cursor, path: extra }]).unwrap()).unwrap();
    let state = scan_state(
        &[Root { agent: Agent::Omp, path: personal }],
        &config,
        &[ManagedRoot { agent: Agent::ClaudeCode, path: managed, kind: ManagedKind::Plugin, plugin: Some("example".into()) }],
    ).unwrap();
    let pairs = find_duplicates(&state.skills).unwrap();
    assert_eq!(pairs.len(), 3);
    assert!(pairs.iter().any(|pair| pair.a.id == "alpha" && pair.b.id == "beta"));
    assert!(pairs.iter().any(|pair| pair.b.id == "gamma" && pair.b.managed_source.is_some()));
}
