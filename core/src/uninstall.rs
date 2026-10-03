use crate::{scan, PresenceKind, Root, SkillState};
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

#[derive(Debug, Serialize)]
pub struct UninstallPlan { pub actions: Vec<UninstallAction> }

pub fn plan_uninstall(roots: &[Root], skill: &SkillState, scope: UninstallScope) -> io::Result<UninstallPlan> {
    if skill.managed_source.is_some() {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Cannot uninstall a managed plugin or built-in skill"));
    }
    let current = scan(roots)?.into_iter().find(|item| item.id == skill.id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Skill is no longer in a configured root"))?;
    if skill.presences.len() != current.presences.len() ||
        skill.presences.iter().any(|presence| !current.presences.iter().any(|now| now == presence)) {
        return Err(io::Error::other("Skill copies changed since the dialog opened; close and reopen it"));
    }
    let real: Vec<_> = current.presences.iter().filter(|presence| presence.kind == PresenceKind::RealDir).collect();
    if real.len() != 1 {
        return Err(io::Error::other("Uninstall requires one master copy; choose a master for unlinked duplicates first"));
    }
    let master = &real[0].path;
    for link in current.presences.iter().filter(|presence| presence.kind == PresenceKind::Link) {
        if fs::canonicalize(&link.path)? != fs::canonicalize(master)? {
            return Err(io::Error::other(format!("{} points to another master; resolve it before uninstalling", link.path.display())));
        }
    }
    let actions = match scope {
        UninstallScope::Everywhere => {
            let mut actions: Vec<_> = current.presences.iter().filter(|presence| presence.kind == PresenceKind::Link)
                .map(|presence| UninstallAction::RemoveLink { path: presence.path.clone() }).collect();
            actions.push(UninstallAction::TrashMaster { path: master.clone() });
            actions
        }
        UninstallScope::Root(root) => {
            if !roots.iter().any(|configured| configured.path == root) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Root is not configured"));
            }
            let selected = root.join(&skill.id);
            let presence = current.presences.iter().find(|presence| presence.path == selected)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Skill is not installed in that root"))?;
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

pub fn uninstall(roots: &[Root], skill: &SkillState, scope: UninstallScope) -> io::Result<Vec<SkillState>> {
    uninstall_with(roots, skill, scope, |path| trash::delete(path).map_err(io::Error::other))
}

// ponytail: this fixture hook observes Trash ordering; production always calls the macOS Trash library.
pub fn uninstall_with(roots: &[Root], skill: &SkillState, scope: UninstallScope, mut trash: impl FnMut(&Path) -> io::Result<()>) -> io::Result<Vec<SkillState>> {
    let plan = plan_uninstall(roots, skill, scope)?;
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
    scan(roots)
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
