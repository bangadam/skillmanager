use crate::{find_skill, markdown, scan, PresenceKind, Root, SkillState};
use serde::Serialize;
use std::{fs, io, path::PathBuf};

#[derive(Debug, Serialize)]
pub struct EditResult {
    pub skills: Vec<SkillState>,
    pub files_written: Vec<PathBuf>,
}


fn master(roots: &[Root], id: &str) -> io::Result<(SkillState, PathBuf)> {
    let skill = find_skill(roots, id)?;
    // ponytail: root order picks the master even if its markdown is missing; linking still requires explicit selection.
    let path = roots.iter().find_map(|root| skill.presences.iter()
        .find(|copy| copy.path == root.path.join(id) && copy.kind == PresenceKind::RealDir))
        .ok_or_else(|| io::Error::other(format!("{id} has no real master in configured roots")))?.path.clone();
    Ok((skill, path))
}

pub fn read_skill_md(roots: &[Root], id: &str) -> io::Result<String> {
    let (_, path) = master(roots, id)?;
    Ok(markdown(&path)?.unwrap_or_default())
}

pub fn edit_skill(roots: &[Root], id: &str, new_content: &str) -> io::Result<EditResult> {
    let (skill, master_path) = master(roots, id)?;
    let original = markdown(&master_path)?;
    let mut files_written = Vec::new();
    let mut divergent_roots = Vec::new();
    for root in roots {
        let Some(copy) = skill.presences.iter().find(|copy| copy.path == root.path.join(id)) else { continue };
        let content = markdown(&copy.path)?;
        if content != original && !(original.is_none() && content.as_deref() == Some("")) {
            divergent_roots.push(copy.path.parent().unwrap().display().to_string());
        } else if copy.kind == PresenceKind::RealDir && (copy.path == master_path || content.is_some()) {
            files_written.push(copy.path.join("SKILL.md"));
        }
    }
    if !divergent_roots.is_empty() {
        return Err(io::Error::other(format!("Divergent SKILL.md in roots: {}; review the diff before saving", divergent_roots.join(", "))));
    }
    // ponytail: preflight every visible copy before any write; if a disk write fails partway through, repair the filesystem before retrying.
    for path in &files_written {
        fs::write(path, new_content)?;
    }
    Ok(EditResult { skills: scan(roots)?, files_written })
}
