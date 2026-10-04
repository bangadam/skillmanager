use skillmanager_core::{plan_uninstall, scan, scan_state, uninstall_confirmed_with, uninstall_with, Agent, ManagedKind, ManagedRoot, Root, UninstallAction, UninstallScope};
use std::{fs, io, os::unix::fs::symlink, path::Path};

fn roots(base: &Path) -> Vec<Root> {
    [Agent::ClaudeCode, Agent::Codex, Agent::Cursor].into_iter().enumerate()
        .map(|(n, agent)| Root { agent, path: base.join(format!("root-{n}")) }).collect()
}

fn fixture(roots: &[Root]) {
    let master = roots[0].path.join("alpha");
    fs::create_dir_all(&master).unwrap();
    fs::write(master.join("SKILL.md"), "original").unwrap();
    for root in &roots[1..] {
        fs::create_dir_all(&root.path).unwrap();
        symlink(&master, root.path.join("alpha")).unwrap();
    }
}

fn trash(path: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    fs::rename(path, destination.join(path.file_name().unwrap()))
}

#[test]
fn everywhere_plans_exact_paths_and_unlinks_before_trashing() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    fixture(&roots);
    let skills = scan(&roots).unwrap();
    let original = skills[0].clone();
    let master = roots[0].path.join("alpha");
    let link1 = roots[1].path.join("alpha");
    let link2 = roots[2].path.join("alpha");
    let expected = vec![
        UninstallAction::RemoveLink { path: link1.clone() },
        UninstallAction::RemoveLink { path: link2.clone() },
        UninstallAction::TrashMaster { path: master.clone() },
    ];
    assert_eq!(plan_uninstall(&skills, &roots, &original, UninstallScope::Everywhere).unwrap().actions, expected);
    assert!(master.exists() && link1.exists() && link2.exists());
    let trash_dir = temp.path().join("Trash");
    let result = uninstall_with(&skills, &roots, &original, UninstallScope::Everywhere, |path| {
        assert!(!link1.exists() && !link2.exists(), "all links must be removed before Trash");
        assert!(fs::symlink_metadata(&link1).is_err() && fs::symlink_metadata(&link2).is_err());
        trash(path, &trash_dir)
    }).unwrap();
    assert!(result.is_empty());
    assert_eq!(fs::read_to_string(trash_dir.join("alpha/SKILL.md")).unwrap(), "original");
    assert!(!master.exists());
}

#[test]
fn stale_uninstall_plan_never_removes_a_copy() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    fixture(&roots);
    let skills = scan(&roots).unwrap();
    let expected = plan_uninstall(&skills, &roots, &skills[0], UninstallScope::Everywhere).unwrap().actions;
    fs::remove_file(roots[2].path.join("alpha")).unwrap();
    let fresh = scan(&roots).unwrap();
    let error = uninstall_confirmed_with(&fresh, &roots, &roots[0].path.join("alpha"), UninstallScope::Everywhere, &expected, |_| panic!("stale plan must not trash")).unwrap_err();
    assert!(error.to_string().contains("review the plan again"));
    assert!(roots[0].path.join("alpha/SKILL.md").exists());
}

#[test]
fn single_agent_unlinks_only_selected_root() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    fixture(&roots);
    let skills = scan(&roots).unwrap();
    let original = skills[0].clone();
    let selected = roots[1].path.join("alpha");
    assert_eq!(plan_uninstall(&skills, &roots, &original, UninstallScope::Root(roots[1].path.clone())).unwrap().actions,
        vec![UninstallAction::RemoveLink { path: selected.clone() }]);
    let result = uninstall_with(&skills, &roots, &original, UninstallScope::Root(roots[1].path.clone()), |_| panic!("no real content should be trashed")).unwrap();
    assert_eq!(result[0].presences.len(), 2);
    assert!(fs::symlink_metadata(selected).is_err());
    assert_eq!(fs::read_to_string(roots[2].path.join("alpha/SKILL.md")).unwrap(), "original");
}

#[test]
fn removing_master_root_promotes_a_copy_and_repoints_every_remaining_link_before_trash() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    fixture(&roots);
    fs::create_dir_all(roots[0].path.join("alpha/assets")).unwrap();
    fs::write(roots[0].path.join("alpha/assets/icon"), b"image").unwrap();
    symlink(roots[0].path.join("alpha/assets/icon"), roots[0].path.join("alpha/icon-link")).unwrap();
    let skills = scan(&roots).unwrap();
    let original = skills[0].clone();
    let old = roots[0].path.join("alpha");
    let promoted = roots[1].path.join("alpha");
    let remaining = roots[2].path.join("alpha");
    let plan = plan_uninstall(&skills, &roots, &original, UninstallScope::Root(roots[0].path.clone())).unwrap();
    assert_eq!(plan.actions, vec![
        UninstallAction::Promote { path: promoted.clone() },
        UninstallAction::RepointLink { path: remaining.clone(), target: promoted.clone() },
        UninstallAction::TrashMaster { path: old.clone() },
    ]);
    assert!(fs::symlink_metadata(&promoted).unwrap().file_type().is_symlink());
    let trash_dir = temp.path().join("Trash");
    let result = uninstall_with(&skills, &roots, &original, UninstallScope::Root(roots[0].path.clone()), |path| {
        assert_eq!(fs::read_link(&remaining).unwrap(), promoted);
        assert!(fs::symlink_metadata(&promoted).unwrap().is_dir());
        assert_eq!(fs::read(promoted.join("assets/icon")).unwrap(), b"image");
        assert_eq!(fs::read(promoted.join("icon-link")).unwrap(), b"image");
        trash(path, &trash_dir)
    }).unwrap();
    assert_eq!(result[0].presences.len(), 2);
    assert_eq!(fs::read_link(remaining).unwrap(), promoted);
    assert!(fs::symlink_metadata(&promoted).unwrap().is_dir());
    assert_eq!(fs::read(promoted.join("icon-link")).unwrap(), b"image");
    assert_eq!(fs::read_to_string(trash_dir.join("alpha/SKILL.md")).unwrap(), "original");
}

#[test]
fn only_copy_refuses_single_root_but_everywhere_works() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    fixture(&roots);
    fs::remove_file(roots[1].path.join("alpha")).unwrap();
    fs::remove_file(roots[2].path.join("alpha")).unwrap();
    let skills = scan(&roots).unwrap();
    let original = skills[0].clone();
    let error = plan_uninstall(&skills, &roots, &original, UninstallScope::Root(roots[0].path.clone())).unwrap_err().to_string();
    assert!(error.contains("everywhere"), "{error}");
    assert!(uninstall_with(&skills, &roots, &original, UninstallScope::Root(roots[0].path.clone()), |_| panic!("must refuse" )).is_err());
    assert!(roots[0].path.join("alpha/SKILL.md").is_file());
    let trash_dir = temp.path().join("Trash");
    assert!(uninstall_with(&skills, &roots, &original, UninstallScope::Everywhere, |path| trash(path, &trash_dir)).unwrap().is_empty());
    assert_eq!(fs::read_to_string(trash_dir.join("alpha/SKILL.md")).unwrap(), "original");
}

#[test]
fn uninstalling_a_managed_row_trashes_only_the_cached_copy() {
    let temp = tempfile::tempdir().unwrap();
    let roots = roots(temp.path());
    fixture(&roots);
    let plugin = temp.path().join("plugin");
    fs::create_dir_all(plugin.join("alpha")).unwrap();
    fs::write(plugin.join("alpha/SKILL.md"), "managed").unwrap();
    let state = scan_state(&roots, &temp.path().join("missing-config"), &[ManagedRoot {
        agent: Agent::Omp, path: plugin.clone(), kind: ManagedKind::Plugin, plugin: Some("vendor / plugin".into()),
    }]).unwrap();
    let managed = state.skills.iter().find(|skill| skill.managed_source.is_some()).unwrap().clone();
    assert_eq!(plan_uninstall(&state.skills, &roots, &managed, UninstallScope::Everywhere).unwrap().actions,
        vec![UninstallAction::TrashMaster { path: plugin.join("alpha") }]);
    assert_eq!(plan_uninstall(&state.skills, &roots, &managed, UninstallScope::Root(plugin.clone())).unwrap().actions,
        vec![UninstallAction::TrashMaster { path: plugin.join("alpha") }]);
    let missing = plan_uninstall(&state.skills, &roots, &managed, UninstallScope::Root(roots[0].path.clone())).unwrap_err().to_string();
    assert!(missing.contains("not installed in that root"), "{missing}");
    let trash_dir = temp.path().join("Trash");
    let result = uninstall_with(&state.skills, &roots, &managed, UninstallScope::Everywhere, |path| trash(path, &trash_dir)).unwrap();
    assert!(!plugin.join("alpha").exists());
    assert_eq!(fs::read_to_string(trash_dir.join("alpha/SKILL.md")).unwrap(), "managed");
    assert!(roots[0].path.join("alpha/SKILL.md").exists());
    assert_eq!(result[0].presences.len(), 3);
}
