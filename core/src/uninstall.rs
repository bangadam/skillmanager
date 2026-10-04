use crate::{covers, scan, PresenceKind, Root, SkillState};
use serde::{Deserialize, Serialize};
use std::{fs, io, os::unix::fs::symlink, path::{Path, PathBuf}};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UninstallScope { Everywhere, Root(PathBuf) }

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UninstallAction {
    RemoveLink { path: PathBuf },
    Promote { path: PathBuf },
    RepointLink { path: PathBuf, target: PathBuf },
    TrashMaster { path: PathBuf },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UninstallPlan { pub actions: Vec<UninstallAction> }

// Callers pass a skills list scanned immediately before the call; the confirmed flow's
// expected-actions comparison is the drift guard between plan and execute.
fn uninstall_target(skills: &[SkillState], source: &Path) -> io::Result<SkillState> {
    skills.iter().find(|skill| skill.presences.iter().any(|copy| copy.path == source)).cloned()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Skill is no longer available; refresh and try again"))
}

pub fn plan_uninstall_source(skills: &[SkillState], roots: &[Root], source: &Path, scope: UninstallScope) -> io::Result<UninstallPlan> {
    plan_uninstall(skills, roots, &uninstall_target(skills, source)?, scope)
}

pub fn plan_uninstall(skills: &[SkillState], roots: &[Root], skill: &SkillState, scope: UninstallScope) -> io::Result<UninstallPlan> {
    if skill.managed_source.is_some() {
        return plan_managed_uninstall(skills, skill, scope);
    }
    let current = skills.iter().find(|item| item.id == skill.id && item.managed_source.is_none())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Skill is no longer in a configured root"))?;
    if skill.presences != current.presences {
        return Err(io::Error::other("Skill copies changed since the dialog opened; close and reopen it"));
    }
    let real: Vec<_> = current.presences.iter().filter(|presence| presence.kind == PresenceKind::RealDir).collect();
    // Everywhere trashes every real copy; no master is needed because nothing is kept.
    let actions = match scope {
        UninstallScope::Everywhere => {
            let mut actions: Vec<_> = current.presences.iter().filter(|presence| presence.kind == PresenceKind::Link)
                .map(|presence| UninstallAction::RemoveLink { path: presence.path.clone() }).collect();
            actions.extend(real.iter().map(|presence| UninstallAction::TrashMaster { path: presence.path.clone() }));
            actions
        }
        UninstallScope::Root(root) => {
            if real.len() != 1 {
                return Err(io::Error::other("Removing from one agent requires one master copy; choose a master for unlinked duplicates first"));
            }
            let master = &real[0].path;
            let master_target = fs::canonicalize(master)?;
            for link in current.presences.iter().filter(|presence| presence.kind == PresenceKind::Link) {
                if fs::canonicalize(&link.path)? != master_target {
                    return Err(io::Error::other(format!("{} points to another master; resolve it before uninstalling", link.path.display())));
                }
            }
            let configured = roots.iter().find(|configured| configured.path == root)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Root is not configured"))?;
            let presence = current.presences.iter().find(|presence| covers(configured, &skill.id, presence))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Skill is not installed in that root"))?;
            let selected = presence.path.clone();
            if presence.kind == PresenceKind::Link {
                vec![UninstallAction::RemoveLink { path: selected }]
            } else {
                let promoted = current.presences.iter().find(|presence| presence.path != selected)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                        "This is the only copy; select everywhere to uninstall it"))?;
                let mut actions = vec![UninstallAction::Promote { path: promoted.path.clone() }];
                actions.extend(current.presences.iter().filter(|presence| presence.path != selected && presence.path != promoted.path)
                    .map(|presence| UninstallAction::RepointLink { path: presence.path.clone(), target: promoted.path.clone() }));
                actions.push(UninstallAction::TrashMaster { path: selected });
                actions
            }
        }
    };
    Ok(UninstallPlan { actions })
}

// Managed (plugin/built-in) rows: removal means trashing the agent's cached copy.
// ponytail: the agent's plugin updater may re-sync the skill later; we do not track or block that.
fn plan_managed_uninstall(skills: &[SkillState], skill: &SkillState, scope: UninstallScope) -> io::Result<UninstallPlan> {
    let current = skills.iter().find(|item| item.id == skill.id && item.managed_source.is_some() && item.managed_source == skill.managed_source)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Skill is no longer in its managed source; refresh and try again"))?;
    let selected: Vec<_> = match scope {
        UninstallScope::Everywhere => current.presences.iter().collect(),
        UninstallScope::Root(root) => {
            let in_root: Vec<_> = current.presences.iter().filter(|presence| presence.path.parent() == Some(root.as_path())).collect();
            if in_root.is_empty() {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Skill is not installed in that root"));
            }
            in_root
        }
    };
    Ok(UninstallPlan { actions: selected.into_iter()
        .map(|presence| UninstallAction::TrashMaster { path: presence.path.clone() }).collect() })
}

pub fn uninstall(skills: &[SkillState], roots: &[Root], skill: &SkillState, scope: UninstallScope) -> io::Result<Vec<SkillState>> {
    uninstall_with(skills, roots, skill, scope, |path| trash::delete(path).map_err(io::Error::other))
}

// ponytail: this fixture hook observes Trash ordering; production always calls the macOS Trash library.
pub fn uninstall_with(skills: &[SkillState], roots: &[Root], skill: &SkillState, scope: UninstallScope, mut trash: impl FnMut(&Path) -> io::Result<()>) -> io::Result<Vec<SkillState>> {
    let plan = plan_uninstall(skills, roots, skill, scope)?;
    execute_plan(&plan, &mut trash)?;
    scan(roots)
}

fn execute_plan(plan: &UninstallPlan, trash: &mut impl FnMut(&Path) -> io::Result<()>) -> io::Result<()> {
    let master = plan.actions.iter().find_map(|action| match action {
        UninstallAction::TrashMaster { path } => Some(path.as_path()), _ => None,
    });
    for action in &plan.actions {
        match action {
            UninstallAction::RemoveLink { path } => fs::remove_file(path)?,
            UninstallAction::Promote { path } => promote(master.expect("promotion needs a master"), path)?,
            UninstallAction::RepointLink { path, target } => repoint(path, target)?,
            UninstallAction::TrashMaster { path } => trash(path)?,
        }
    }
    Ok(())
}

pub fn uninstall_confirmed(skills: &[SkillState], roots: &[Root], source: &Path, scope: UninstallScope, expected: &[UninstallAction]) -> io::Result<Vec<SkillState>> {
    uninstall_confirmed_with(skills, roots, source, scope, expected, |path| trash::delete(path).map_err(io::Error::other))
}

// ponytail: the fixture hook keeps plan confirmation at the core seam while production uses system Trash.
pub fn uninstall_confirmed_with(skills: &[SkillState], roots: &[Root], source: &Path, scope: UninstallScope, expected: &[UninstallAction], trash: impl FnMut(&Path) -> io::Result<()>) -> io::Result<Vec<SkillState>> {
    let skill = uninstall_target(skills, source)?;
    let current = plan_uninstall(skills, roots, &skill, scope.clone())?;
    if current.actions != expected {
        return Err(io::Error::other("Affected paths changed; review the plan again before uninstalling"));
    }
    uninstall_with(skills, roots, &skill, scope, trash)
}

#[derive(Debug, Serialize)]
pub struct BulkUninstallEntry { pub name: String, pub plan: UninstallPlan }

pub fn plan_bulk_uninstall(skills: &[SkillState], roots: &[Root], sources: &[PathBuf]) -> io::Result<Vec<BulkUninstallEntry>> {
    sources.iter().map(|source| {
        let skill = uninstall_target(skills, source)?;
        Ok(BulkUninstallEntry { name: skill.name.clone(), plan: plan_uninstall(skills, roots, &skill, UninstallScope::Everywhere)? })
    }).collect()
}

pub fn uninstall_bulk_confirmed(skills: &[SkillState], roots: &[Root], sources: &[PathBuf], expected: &[UninstallPlan]) -> io::Result<()> {
    uninstall_bulk_confirmed_with(skills, roots, sources, expected, |path| trash::delete(path).map_err(io::Error::other))
}

// Same drift guard as the single-skill flow, checked for every skill against one fresh scan.
pub fn uninstall_bulk_confirmed_with(skills: &[SkillState], roots: &[Root], sources: &[PathBuf], expected: &[UninstallPlan], mut trash: impl FnMut(&Path) -> io::Result<()>) -> io::Result<()> {
    if sources.len() != expected.len() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "The plan does not match the requested skills"));
    }
    let entries = plan_bulk_uninstall(skills, roots, sources)?;
    for (entry, expected_plan) in entries.iter().zip(expected) {
        if entry.plan.actions != expected_plan.actions {
            return Err(io::Error::other("Affected paths changed; review the plan again before uninstalling"));
        }
    }
    for entry in &entries {
        execute_plan(&entry.plan, &mut trash)?;
    }
    Ok(())
}

fn spare_path(path: &Path) -> io::Result<PathBuf> {
    for n in 0..1000 {
        let candidate = path.with_file_name(format!(".{}.skillmanager-{n}", path.file_name().unwrap().to_string_lossy()));
        if fs::symlink_metadata(&candidate).is_err_and(|error| error.kind() == io::ErrorKind::NotFound) {
            return Ok(candidate);
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "No free temporary path for promotion"))
}

fn promote(master: &Path, destination: &Path) -> io::Result<()> {
    let staging = spare_path(destination)?;
    copy_dir(master, &staging, master, destination)?;
    fs::remove_file(destination)?;
    if let Err(error) = fs::rename(&staging, destination) {
        symlink(master, destination)?;
        return Err(error);
    }
    Ok(())
}

fn copy_dir(source: &Path, destination: &Path, old_master: &Path, new_master: &Path) -> io::Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&from)?;
        if metadata.is_dir() { copy_dir(&from, &to, old_master, new_master)?; }
        else if metadata.is_file() { fs::copy(&from, &to)?; }
        else if metadata.file_type().is_symlink() {
            let target = fs::read_link(&from)?;
            // ponytail: only absolute links within the old master move; external links stay external.
            let target = target.strip_prefix(old_master).map(|suffix| new_master.join(suffix)).unwrap_or(target);
            symlink(target, to)?;
        }
        else { return Err(io::Error::other(format!("Unsupported file in {}", from.display()))); }
    }
    fs::set_permissions(destination, fs::metadata(source)?.permissions())?;
    Ok(())
}

fn repoint(path: &Path, target: &Path) -> io::Result<()> {
    let staging = spare_path(path)?;
    symlink(target, &staging)?;
    fs::rename(staging, path)
}
