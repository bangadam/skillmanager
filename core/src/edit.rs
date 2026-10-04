use crate::{covers, find_skill, markdown, scan, PresenceKind, Root, SkillState};
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
        .find(|copy| copy.kind == PresenceKind::RealDir && covers(root, id, copy)))
        .map(|copy| copy.path.clone())
        .ok_or_else(|| io::Error::other(format!("{id} has no real master in configured roots")))?;
    Ok((skill, path))
}

pub fn read_skill_md(roots: &[Root], id: &str) -> io::Result<String> {
    let (_, path) = master(roots, id)?;
    Ok(markdown(&path)?.unwrap_or_default())
}

pub fn edit_skill(roots: &[Root], id: &str, new_content: &str) -> io::Result<EditResult> {
    let (skill, master_path) = master(roots, id)?;
    let original = markdown(&master_path)?;
    let files_written = write_targets(&skill, roots, &master_path, &original)?;
    // ponytail: preflight every visible copy before any write; if a disk write fails partway through, repair the filesystem before retrying.
    for path in &files_written {
        fs::write(path, new_content)?;
    }
    Ok(EditResult { skills: scan(roots)?, files_written })
}

// Resolve every SKILL.md a save must reach for one skill; divergent copies block all writes.
fn write_targets(skill: &SkillState, roots: &[Root], master_path: &std::path::Path, original: &Option<String>) -> io::Result<Vec<PathBuf>> {
    let mut files_written = Vec::new();
    let mut divergent_roots = Vec::new();
    for root in roots {
        let Some(copy) = skill.presences.iter().find(|copy| covers(root, &skill.id, copy)) else { continue };
        let content = markdown(&copy.path)?;
        if content != *original && !(original.is_none() && content.as_deref() == Some("")) {
            divergent_roots.push(copy.path.parent().unwrap().display().to_string());
        } else if copy.kind == PresenceKind::RealDir && (copy.path == master_path || content.is_some()) {
            files_written.push(copy.path.join("SKILL.md"));
        }
    }
    if !divergent_roots.is_empty() {
        return Err(io::Error::other(format!("Divergent SKILL.md in roots: {}; review the diff before saving", divergent_roots.join(", "))));
    }
    Ok(files_written)
}

/// Insert or replace the `category:` frontmatter line, creating frontmatter when absent.
/// Preserves the body byte-for-byte and the original trailing-newline state.
pub fn with_category(content: &str, category: &str) -> String {
    let category_line = format!("category: {category}");
    let lines: Vec<&str> = content.lines().collect();
    let frontmatter_end = (!lines.is_empty() && lines[0].trim() == "---")
        .then(|| (1..lines.len()).find(|&index| lines[index].trim() == "---"))
        .flatten();
    let mut result = String::new();
    if let Some(end) = frontmatter_end {
        let mut block: Vec<&str> = lines[1..end].to_vec();
        match block.iter().position(|line| line.trim_start().starts_with("category:")) {
            Some(slot) => block[slot] = &category_line,
            None => block.push(&category_line),
        }
        result.push_str("---\n");
        for line in block { result.push_str(line); result.push('\n'); }
        result.push_str("---\n");
        for line in &lines[end + 1..] { result.push_str(line); result.push('\n'); }
    } else {
        result.push_str("---\n");
        result.push_str(&category_line);
        result.push('\n');
        result.push_str("---\n\n");
        result.push_str(content);
    }
    if !content.ends_with('\n') && result.ends_with('\n') { result.pop(); }
    result
}

/// Set the `category:` frontmatter of many skills in one pass: one scan in, one write pass, one scan out.
/// Managed skills are refused; divergent copies block the whole batch (same guard as a single edit).
pub fn set_categories(skills: &[SkillState], roots: &[Root], ids: &[String], category: &str) -> io::Result<Vec<PathBuf>> {
    let category = category.trim().to_lowercase();
    if category.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Category cannot be empty"));
    }
    if category.contains([':', '"', '\'', '#', '\n', '\r']) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Category must be a single plain word"));
    }
    let mut planned = Vec::new();
    // One map instead of a linear find per id.
    let by_id: std::collections::HashMap<&str, &SkillState> = skills.iter().map(|skill| (skill.id.as_str(), skill)).collect();
    for id in ids {
        let skill = by_id.get(id.as_str())
            .copied()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Skill {id} is no longer available; refresh and try again")))?;
        if skill.managed_source.is_some() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{id} is managed by an agent; set its category there instead")));
        }
        let master_path = master(roots, id)?.1;
        let original = markdown(&master_path)?;
        let targets = write_targets(skill, roots, &master_path, &original)?;
        planned.push((targets, with_category(original.as_deref().unwrap_or(""), &category)));
    }
    let mut files_written = Vec::new();
    for (targets, updated) in planned {
        for path in targets { fs::write(&path, &updated)?; files_written.push(path); }
    }
    Ok(files_written)
}
