use crate::{find_skill, markdown, scan, PresenceKind, Root, SkillState};
use serde::{Deserialize, Serialize};
use std::{fs, io, os::unix::fs::symlink, path::{Path, PathBuf}};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DivergentPair {
    pub left: PathBuf,
    pub right: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct SkillDiff {
    pub left: Option<String>,
    pub right: Option<String>,
}


pub(crate) fn divergent_pairs(skill: &SkillState) -> io::Result<Vec<DivergentPair>> {
    if skill.presences.len() < 2 { return Ok(Vec::new()); }
    let copies = skill.presences.iter().map(|copy| Ok((&copy.path, markdown(&copy.path)?))).collect::<io::Result<Vec<_>>>()?;
    let mut pairs = Vec::new();
    for (index, (left, left_text)) in copies.iter().enumerate() {
        for (right, right_text) in copies.iter().skip(index + 1) {
            if left_text != right_text {
                pairs.push(DivergentPair { left: (*left).clone(), right: (*right).clone() });
            }
        }
    }
    Ok(pairs)
}

fn copy<'a>(skill: &'a SkillState, roots: &[Root], path: &Path) -> io::Result<&'a crate::Presence> {
    if !roots.iter().any(|configured| configured.path == path)
        && !skill.presences.iter().any(|presence| presence.path == path) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{} is not a configured root or a copy of {}", path.display(), skill.id)));
    }
    skill.presences.iter().find(|copy| copy.path == path)
        .or_else(|| roots.iter().find(|configured| configured.path == path)
            .and_then(|configured| skill.presences.iter().find(|copy| copy.path == configured.path.join(&skill.id))))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("{} has no copy of {}", path.display(), skill.id)))
}


pub fn skill_diff(roots: &[Root], id: &str, left_root: &Path, right_root: &Path) -> io::Result<SkillDiff> {
    let skill = find_skill(roots, id)?;
    let left = copy(&skill, roots, left_root)?;
    let right = copy(&skill, roots, right_root)?;
    Ok(SkillDiff { left: markdown(&left.path)?, right: markdown(&right.path)? })
}

fn replace(roots: &[Root], id: &str, target_root: &Path, master_root: &Path, trash: impl FnOnce(&Path) -> io::Result<()>) -> io::Result<Vec<SkillState>> {
    let skill = find_skill(roots, id)?;
    let target = copy(&skill, roots, target_root)?;
    let master = copy(&skill, roots, master_root)?;
    if target.path == master.path || target.kind != PresenceKind::RealDir || master.kind != PresenceKind::RealDir {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Replacement requires distinct real target and master copies"));
    }
    // ponytail: the move must succeed before link creation; on a link failure the old folder remains recoverable in Trash.
    trash(&target.path)?;
    symlink(&master.path, &target.path).map_err(|error| io::Error::new(error.kind(),
        format!("{} is in Trash but its replacement link failed: {error}", target.path.display())))?;
    scan(roots)
}

pub fn replace_with_link(roots: &[Root], id: &str, target_root: &Path, master_root: &Path) -> io::Result<Vec<SkillState>> {
    replace(roots, id, target_root, master_root, |path| trash::delete(path).map_err(io::Error::other))
}

// ponytail: a fixture Trash emulates the same move-before-link contract; production uses macOS Trash.
pub fn replace_with_link_in(roots: &[Root], id: &str, target_root: &Path, master_root: &Path, trash_dir: &Path) -> io::Result<Vec<SkillState>> {
    replace(roots, id, target_root, master_root, |path| {
        fs::create_dir_all(trash_dir)?;
        let name = path.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Target has no folder name"))?;
        let mut destination = trash_dir.join(name);
        let mut suffix = 1;
        while destination.exists() {
            destination = trash_dir.join(format!("{}-{suffix}", name.to_string_lossy()));
            suffix += 1;
        }
        fs::rename(path, destination)
    })
}
