use skillmanager_core::{default_managed_roots, scan_state, Agent, ManagedKind, ManagedRoot, Root};
use std::{fs, path::Path};

fn skill(root: &Path, name: &str) {
    let folder = root.join(name);
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("SKILL.md"), format!("---\nname: {name}\n---\nBody")).unwrap();
}

#[test]
fn configured_extra_roots_join_personal_scan_and_missing_config_is_not_an_error() {
    let temp = tempfile::tempdir().unwrap();
    let canonical = temp.path().join("canonical");
    let extra = temp.path().join("extra");
    skill(&canonical, "one");
    skill(&extra, "two");
    let config = temp.path().join("roots.json");
    let roots = [Root { agent: Agent::Codex, path: canonical }];
    let before = scan_state(&roots, &config, &[]).unwrap();
    assert_eq!(before.skills.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["one"]);
    assert!(before.config_error.is_none());
    fs::write(&config, format!(r#"[{{"agent":"cursor","path":"{}"}}]"#, extra.display())).unwrap();
    let after = scan_state(&roots, &config, &[]).unwrap();
    assert_eq!(after.skills.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["one", "two"]);
    assert_eq!(after.skills[1].presences[0].agent, Agent::Cursor);
    assert!(after.config_error.is_none());
}

#[test]
fn invalid_config_surfaces_error_without_hiding_canonical_skills() {
    let temp = tempfile::tempdir().unwrap();
    skill(temp.path(), "owned");
    let config = temp.path().join("roots.json");
    fs::write(&config, "{not json").unwrap();
    let state = scan_state(&[Root { agent: Agent::Omp, path: temp.path().into() }], &config, &[]).unwrap();
    assert_eq!(state.skills[0].id, "owned");
    assert!(state.config_error.as_ref().unwrap().contains("roots.json"));
}

#[test]
fn managed_skills_are_separate_read_only_rows_and_never_missing() {
    let temp = tempfile::tempdir().unwrap();
    let mine = temp.path().join("mine");
    let plugins = temp.path().join("plugins");
    let builtin = temp.path().join("system");
    skill(&mine, "shared");
    skill(&plugins, "shared");
    skill(&builtin, "builtin");
    let roots = [Root { agent: Agent::Codex, path: mine }];
    let state = scan_state(
        &roots,
        &temp.path().join("missing.json"),
        &[
            ManagedRoot { agent: Agent::ClaudeCode, path: plugins, kind: ManagedKind::Plugin, plugin: Some("builder-skills".into()) },
            ManagedRoot { agent: Agent::Codex, path: builtin, kind: ManagedKind::BuiltIn, plugin: None },
        ],
    ).unwrap();
    assert_eq!(state.skills.len(), 3);
    let plugin = state.skills.iter().find(|s| s.managed_source.as_ref().and_then(|m| m.plugin.as_deref()) == Some("builder-skills")).unwrap();
    assert_eq!(plugin.id, "shared");
    assert_eq!(plugin.presences[0].agent, Agent::ClaudeCode);
    assert!(state.skills.iter().any(|s| s.id == "shared" && s.managed_source.is_none()));
}

#[test]
fn discovers_four_agents_cache_layouts_and_skips_absent_sources() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    skill(&home.join(".codex/skills/.system"), "built-in");
    skill(&home.join(".claude/plugins/cache/vendor/plugin/1.0/skills"), "claude-plugin");
    skill(&home.join(".codex/plugins/cache/marketplace/publisher/2.0/skills"), "codex-plugin");
    skill(&home.join(".cursor/plugins/cache/public/atlassian/hash/skills"), "cursor-plugin");
    skill(&home.join(".omp/plugins/cache/plugins/vendor___plugin___3/skills/group"), "omp-plugin");
    let managed = default_managed_roots(home).unwrap();
    let state = scan_state(&[], &home.join("missing.json"), &managed).unwrap();
    assert_eq!(state.skills.len(), 5);
    for (id, agent, plugin) in [
        ("claude-plugin", Agent::ClaudeCode, "plugin"),
        ("codex-plugin", Agent::Codex, "publisher"),
        ("cursor-plugin", Agent::Cursor, "atlassian"),
        ("omp-plugin", Agent::Omp, "vendor / plugin"),
    ] {
        let skill = state.skills.iter().find(|skill| skill.id == id).unwrap();
        assert_eq!(skill.managed_source.as_ref().unwrap().agent, agent);
        assert_eq!(skill.managed_source.as_ref().unwrap().plugin.as_deref(), Some(plugin));
    }
    assert_eq!(state.skills.iter().find(|skill| skill.id == "built-in").unwrap().managed_source.as_ref().unwrap().kind, ManagedKind::BuiltIn);
}
