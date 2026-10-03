use crate::{scan, PresenceKind, Root, SkillState};
use std::{collections::BTreeMap, fs, io, os::unix::fs::symlink, path::{Path, PathBuf}};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LinkEverywhereResult {
    pub skills: Vec<SkillState>,
    pub failures: Vec<String>,
}

pub fn missing<'a>(skills: &'a [SkillState], roots: &[Root]) -> Vec<&'a SkillState> {
    skills.iter().filter(|skill| {
        skill.managed_source.is_none() && !skill.presences.is_empty()
            && roots.iter().any(|root| !skill.presences.iter().any(|presence| presence.path == root.path.join(&skill.id)))
    }).collect()
}

fn find_skill(roots: &[Root], id: &str) -> io::Result<SkillState> {
    scan(roots)?.into_iter().find(|skill| skill.id == id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Skill {id:?} is not in a configured root")))
}

fn root<'a>(roots: &'a [Root], path: &Path) -> io::Result<&'a Root> {
    roots.iter().find(|root| root.path == path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("{} is not a configured root", path.display())))
}

fn master(skill: &SkillState) -> io::Result<PathBuf> {
    let mut real = skill.presences.iter().filter(|presence| presence.kind == PresenceKind::RealDir);
    let first = real.next();
    if real.next().is_some() {
        return Err(io::Error::other(format!("{} has multiple real copies; choose a master first", skill.id)));
    }
    match first {
        Some(copy) => Ok(copy.path.clone()),
        None => skill.presences.first()
            .ok_or_else(|| io::Error::other(format!("{} has no master copy", skill.id)))
            .and_then(|copy| fs::canonicalize(&copy.path)),
    }
}

pub fn link(roots: &[Root], id: &str, target_root: &Path) -> io::Result<Vec<SkillState>> {
    let target_root = root(roots, target_root)?;
    let skill = find_skill(roots, id)?;
    let destination = target_root.path.join(id);
    match fs::symlink_metadata(&destination) {
        Ok(_) => return Err(io::Error::new(io::ErrorKind::AlreadyExists,
            format!("{} already holds a copy of {id}; resolve or remove that copy before linking", destination.display()))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    let source = master(&skill)?;
    fs::create_dir_all(&target_root.path)?;
    symlink(&source, &destination)?;
    scan(roots)
}

pub fn link_everywhere(roots: &[Root], id: &str) -> io::Result<LinkEverywhereResult> {
    let skill = find_skill(roots, id)?;
    master(&skill)?;
    let mut failures = Vec::new();
    for root in roots {
        if skill.presences.iter().any(|presence| presence.path == root.path.join(id)) { continue; }
        if let Err(error) = link(roots, id, &root.path) {
            failures.push(format!("{}: {error}", root.path.display()));
        }
    }
    Ok(LinkEverywhereResult { skills: scan(roots)?, failures })
}

// Compare full trees, including empty folders, file types and links, before trashing any copy.
fn contents(path: &Path) -> io::Result<BTreeMap<PathBuf, (u8, Vec<u8>)>> {
    fn visit(base: &Path, dir: &Path, entries: &mut BTreeMap<PathBuf, (u8, Vec<u8>)>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let key = path.strip_prefix(base).unwrap().to_path_buf();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.is_dir() {
                entries.insert(key, (0, Vec::new()));
                visit(base, &path, entries)?;
            } else if metadata.is_file() { entries.insert(key, (1, fs::read(&path)?)); }
            else if metadata.file_type().is_symlink() {
                entries.insert(key, (2, fs::read_link(&path)?.as_os_str().as_encoded_bytes().to_vec()));
            } else { return Err(io::Error::other(format!("Unsupported file in {}", path.display()))); }
        }
        Ok(())
    }
    let mut entries = BTreeMap::new();
    visit(path, path, &mut entries)?;
    Ok(entries)
}

fn select_master(roots: &[Root], id: &str, chosen_root: &Path, mut trash: impl FnMut(&Path) -> io::Result<()>) -> io::Result<Vec<SkillState>> {
    root(roots, chosen_root)?;
    let skill = find_skill(roots, id)?;
    let chosen = chosen_root.join(id);
    if !skill.presences.iter().any(|presence| presence.path == chosen && presence.kind == PresenceKind::RealDir) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{} is not a real copy of {id}", chosen.display())));
    }
    let original = contents(&chosen)?;
    let copies: Vec<_> = skill.presences.iter().filter(|presence| presence.kind == PresenceKind::RealDir && presence.path != chosen).collect();
    // Validate every copy before any move: divergent content must not be touched.
    for copy in &copies {
        if contents(&copy.path)? != original {
            return Err(io::Error::other(format!("{} contains a divergent copy of {id}; resolve its diff before linking", copy.path.display())));
        }
    }
    for copy in copies {
        trash(&copy.path)?;
        symlink(&chosen, &copy.path).map_err(|error| io::Error::new(error.kind(),
            format!("{} was moved to Trash but linking it failed: {error}", copy.path.display())))?;
    }
    scan(roots)
}

pub fn choose_master(roots: &[Root], id: &str, chosen_root: &Path) -> io::Result<Vec<SkillState>> {
    select_master(roots, id, chosen_root, |path| trash::delete(path).map_err(io::Error::other))
}

// ponytail: fixture Trash lives on the same filesystem; production uses the macOS Trash API.
// When cross-volume fixture moves are needed, supply a dedicated Trash adapter instead.
pub fn choose_master_in(roots: &[Root], id: &str, chosen_root: &Path, trash_dir: &Path) -> io::Result<Vec<SkillState>> {
    select_master(roots, id, chosen_root, |path| {
        fs::create_dir_all(trash_dir)?;
        let mut target = trash_dir.join(path.file_name().unwrap());
        let mut suffix = 1;
        while target.exists() { target = trash_dir.join(format!("{}-{suffix}", path.file_name().unwrap().to_string_lossy())); suffix += 1; }
        fs::rename(path, target)
    })
}
