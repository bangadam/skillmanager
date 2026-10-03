use crate::{scan, PresenceKind, Root, SkillState};
use serde::Serialize;
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Debug, Serialize)]
pub struct EditResult {
    pub skills: Vec<SkillState>,
    pub files_written: Vec<PathBuf>,
}

fn markdown(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path.join("SKILL.md")) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn master(roots: &[Root], id: &str) -> io::Result<(SkillState, PathBuf)> {
    let skill = scan(roots)?.into_iter().find(|skill| skill.id == id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("No skill {id:?} in configured roots")))?;
    // ponytail: scan only exposes skills with SKILL.md somewhere; choose the first real copy with text so a visible skill never opens a missing copy as blank.
    let path = skill.presences.iter().filter(|copy| copy.kind == PresenceKind::RealDir)
        .find(|copy| copy.path.join("SKILL.md").is_file())
        .ok_or_else(|| io::Error::other(format!("{id} has no real master SKILL.md in configured roots")))?.path.clone();
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
    for copy in &skill.presences {
        if markdown(&copy.path)? != original {
            divergent_roots.push(copy.path.parent().unwrap().display().to_string());
        } else if copy.kind == PresenceKind::RealDir {
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
