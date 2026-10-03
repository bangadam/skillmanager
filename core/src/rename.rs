use crate::{find_skill, scan, PresenceKind, Root, SkillState};
use serde::{Deserialize, Serialize};
use std::{fs, io, os::unix::fs::symlink, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

#[derive(Deserialize, Serialize)]
struct RenameEntry { from: String, to: String, at: u64 }

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.trim() == name && name != "." && name != ".."
        && !name.starts_with('.') && !name.contains('/') && !name.chars().any(char::is_control)
}

fn markdown_with_name(bytes: &[u8], name: &str) -> io::Result<Vec<u8>> {
    let text = std::str::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let value = if name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_') {
        name.to_owned()
    } else { serde_json::to_string(name)? };
    let newline = if text.starts_with("---\r\n") { "\r\n" } else { "\n" };
    let Some(open) = text.strip_prefix(&format!("---{newline}")) else {
        return Ok(format!("---{newline}name: {value}{newline}---{newline}{text}").into_bytes());
    };
    let mut pos = text.len() - open.len();
    let mut name_range = None;
    let mut end = None;
    for line in open.split_inclusive(newline) {
        let bare = line.strip_suffix(newline).unwrap_or(line);
        if bare == "---" || bare == "..." { end = Some(pos); break; }
        if bare.starts_with("name:") {
            name_range = Some(pos..pos + bare.len());
        }
        pos += line.len();
    }
    let end = end.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "SKILL.md has unclosed frontmatter"))?;
    let mut updated = Vec::with_capacity(bytes.len() + name.len() + 10);
    if let Some(range) = name_range {
        updated.extend_from_slice(&bytes[..range.start]);
        updated.extend_from_slice(format!("name: {value}").as_bytes());
        updated.extend_from_slice(&bytes[range.end..]);
    } else {
        updated.extend_from_slice(&bytes[..end]);
        updated.extend_from_slice(format!("name: {value}{newline}").as_bytes());
        updated.extend_from_slice(&bytes[end..]);
    }
    Ok(updated)
}

/// Rename the only master and its links, returning the freshly scanned skill list.
/// `journal` is normally ~/.config/skillmanager/renames.json; fixtures pass an isolated path.
pub fn rename(roots: &[Root], id: &str, new_name: &str, journal: &Path) -> io::Result<Vec<SkillState>> {
    if !valid_name(new_name) || id == new_name {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Enter a different valid directory name (no leading dot, slash, or surrounding whitespace)"));
    }
    let skill = find_skill(roots, id)?;
    let mut real = skill.presences.iter().filter(|presence| presence.kind == PresenceKind::RealDir);
    let Some(master) = real.next() else {
        return Err(io::Error::other("Skill has no master copy in a configured root"));
    };
    if real.next().is_some() {
        return Err(io::Error::other("Skill has unlinked duplicate copies; link first, then rename"));
    }
    let source = &master.path;
    let destination = source.with_file_name(new_name);
    let canonical = fs::canonicalize(source)?;
    let links: Vec<(PathBuf, PathBuf)> = skill.presences.iter().filter(|presence| presence.kind == PresenceKind::Link)
        .map(|presence| Ok((presence.path.clone(), presence.path.with_file_name(new_name))))
        .collect::<io::Result<_>>()?;
    for (old, _) in &links {
        if fs::canonicalize(old)? != canonical {
            return Err(io::Error::other(format!("{} points to a different master; resolve it before renaming", old.display())));
        }
    }
    // ponytail: refuse *all* occupied destinations, even broken links and non-skills; never overwrite a user's file.
    for path in roots.iter().map(|root| root.path.join(new_name)) {
        match fs::symlink_metadata(&path) {
            Ok(_) => return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("{} already exists; choose another name", path.display()))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(error),
        }
    }
    let markdown = fs::read(source.join("SKILL.md"))?;
    let updated = markdown_with_name(&markdown, new_name)?;
    let mut history: Vec<RenameEntry> = match fs::read(journal) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error),
    };
    let at = SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_secs();
    history.push(RenameEntry { from: id.to_owned(), to: new_name.to_owned(), at });
    let history = serde_json::to_vec_pretty(&history)?;
    fs::create_dir_all(journal.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Journal needs a parent directory"))?)?;
    // The journal is staged before touching skills, so write/permission failures cannot leave an unjournaled rename.
    let journal_temp = journal.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&journal_temp, history)?;
    let result = (|| {
        fs::rename(source, &destination)?;
        fs::write(destination.join("SKILL.md"), updated)?;
        for (old, new) in &links {
            symlink(&destination, new)?;
            fs::remove_file(old)?;
        }
        fs::rename(&journal_temp, journal)?;
        scan(roots)
    })();
    if result.is_err() { let _ = fs::remove_file(&journal_temp); }
    result
}
