mod edit;
pub use edit::{edit_skill, read_skill_md, set_categories, with_category, EditResult};
mod divergence;
pub use divergence::{replace_with_link, replace_with_link_in, skill_diff, DivergentPair, SkillDiff};
mod rename;
pub use rename::rename;
mod links;
pub use links::{choose_master, choose_master_in, link, link_everywhere, missing, LinkEverywhereResult};
mod watch;
pub use watch::{watch, WatchGuard};
mod managed;
mod category;
mod duplicates;
pub use managed::{config_path, configured_roots, default_managed_roots, scan_state, ManagedKind, ManagedRoot, ManagedSource, ScanState};
pub use duplicates::{find_duplicates, DuplicatePair};
mod uninstall;
pub use uninstall::{plan_bulk_uninstall, plan_uninstall, plan_uninstall_source, uninstall, uninstall_bulk_confirmed, uninstall_bulk_confirmed_with, uninstall_with, uninstall_confirmed, uninstall_confirmed_with, BulkUninstallEntry, UninstallAction, UninstallPlan, UninstallScope};

use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn markdown(path: &std::path::Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path.join("SKILL.md")) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn find_skill(roots: &[Root], id: &str) -> io::Result<SkillState> {
    scan(roots)?.into_iter().find(|skill| skill.id == id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Skill {id:?} is not in a configured root")))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Agent {
    ClaudeCode,
    Codex,
    Cursor,
    Omp,
    Hermes,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Root {
    pub agent: Agent,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceKind {
    RealDir,
    Link,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Presence {
    pub agent: Agent,
    pub path: PathBuf,
    pub kind: PresenceKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    Created,
    Modified,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillState {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub presences: Vec<Presence>,
    pub date_millis: Option<u64>,
    pub date_source: DateSource,
    pub managed_source: Option<ManagedSource>,
    pub divergent_pairs: Vec<DivergentPair>,
    /// Parent directory of the skill's on-disk location, symlinks resolved — where the
    /// skill is actually saved. Linked presences point here from each agent's root.
    pub folder: Option<String>,
}

#[derive(Deserialize)]
struct FrontMatter {
    name: Option<String>,
    description: Option<String>,
    category: Option<String>,
}

pub fn default_roots() -> io::Result<Vec<Root>> {
    let home = std::env::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory unavailable"))?;
    Ok([
        (Agent::ClaudeCode, ".claude/skills"),
        (Agent::Codex, ".codex/skills"),
        (Agent::Cursor, ".cursor/skills"),
        (Agent::Omp, ".omp/agent/skills"),
        (Agent::Hermes, ".hermes/skills"),
    ]
    .into_iter()
    .map(|(agent, path)| Root { agent, path: home.join(path) })
    .collect())
}


/// A presence covers a root when the agent that owns the root loads it: flat copies sit directly
/// in the root, Hermes copies sit one group folder deeper. `path.file_name` is the skill's id.
pub(crate) fn covers(root: &Root, id: &str, presence: &Presence) -> bool {
    presence.path.starts_with(&root.path)
        && presence.path.file_name().is_some_and(|name| name.to_str() == Some(id))
}
fn has_skill_md(path: &std::path::Path) -> bool {
    matches!(fs::symlink_metadata(path.join("SKILL.md")), Ok(metadata) if metadata.is_file())
}

fn record(
    parser: &Matter<YAML>,
    skills: &mut BTreeMap<String, SkillState>,
    agent: Agent,
    id: String,
    path: PathBuf,
    link_metadata: &fs::Metadata,
    metadata: &fs::Metadata,
) {
    // Canonicalize the skill path itself so a symlinked presence resolves through to where the
    // skill is actually saved (`~/.claude/skills/godmode` → `~/.hermes/skills/red-teaming`).
    // A missing target (broken link) is already skipped by the caller; the parent fallback keeps
    // unreadable-but-present cases working.
    // ponytail: Windows UNC paths — switch to the `dunce` crate if that ever matters.
    let folder = path.canonicalize().ok()
        .or_else(|| path.parent().and_then(|parent| parent.canonicalize().ok()))
        .and_then(|canonical| canonical.parent().map(|parent| parent.to_string_lossy().into_owned()));
    let (name, description, category) = fs::read_to_string(path.join("SKILL.md"))
        .ok()
        .and_then(|text| parser.parse::<FrontMatter>(&text).ok()?.data)
        .map(|data| {
            let name = data.name.unwrap_or_else(|| id.clone());
            let description = data.description.unwrap_or_default();
            let category = data.category
                .map(|category| category.trim().to_lowercase())
                .filter(|category| !category.is_empty())
                .unwrap_or_else(|| category::infer(&name, &description).into());
            (name, description, category)
        })
        .unwrap_or_else(|| (id.clone(), String::new(), category::infer(&id, "").into()));
    let (date, date_source) = if let Ok(created) = metadata.created() {
        (Some(created), DateSource::Created)
    } else if let Ok(modified) = metadata.modified() {
        (Some(modified), DateSource::Modified)
    } else {
        (None, DateSource::Unavailable)
    };
    let date_millis = date.and_then(|date: SystemTime| date.duration_since(UNIX_EPOCH).ok())
        .and_then(|duration| u64::try_from(duration.as_millis()).ok());
    let kind = if link_metadata.file_type().is_symlink() { PresenceKind::Link } else { PresenceKind::RealDir };
    skills.entry(id.clone()).or_insert_with(|| SkillState {
        id,
        name,
        description,
        category,
        presences: Vec::new(),
        date_millis,
        date_source,
        managed_source: None,
        divergent_pairs: Vec::new(),
        folder,
    })
    .presences.push(Presence { agent, path, kind });
}

pub fn scan(roots: &[Root]) -> io::Result<Vec<SkillState>> {
    let parser = Matter::<YAML>::new();
    let mut skills = BTreeMap::<String, SkillState>::new();
    let mut missing_markdown = Vec::new();
    for root in roots {
        let entries = match fs::read_dir(&root.path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let link_metadata = fs::symlink_metadata(&path)?;
            if !link_metadata.is_dir() && !link_metadata.file_type().is_symlink() {
                continue;
            }
            // ponytail: broken links aren't loadable skills; a future diagnostics view can surface them.
            let metadata = match fs::metadata(&path) {
                Ok(metadata) if metadata.is_dir() => metadata,
                Ok(_) => continue,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            let id = entry.file_name().to_string_lossy().into_owned();
            if has_skill_md(&path) {
                record(&parser, &mut skills, root.agent, id, path, &link_metadata, &metadata);
                continue;
            }
            // Hermes nests skills under group folders (any depth) that carry no SKILL.md of their own.
            // A folder without SKILL.md and without skills beneath stays a plain presence — a
            // possibly broken copy of a same-named skill elsewhere — exactly as before.
            let nested = !id.starts_with('.')
                && visit(&parser, &mut skills, root.agent, &path, 0)?;
            if !nested {
                missing_markdown.push((id, Presence {
                    agent: root.agent, path,
                    kind: if link_metadata.file_type().is_symlink() { PresenceKind::Link } else { PresenceKind::RealDir },
                }));
            }
        }
    }
    for (id, presence) in missing_markdown {
        if let Some(skill) = skills.get_mut(&id) { skill.presences.push(presence); }
    }
    for skill in skills.values_mut() {
        skill.divergent_pairs = divergence::divergent_pairs(skill)?;
    }
    Ok(skills.into_values().collect())
}

/// Records every skill folder beneath `folder`, descending through group folders (dirs without
/// SKILL.md). Returns whether any skill was found. ponytail: depth 8 covers hermes' deepest
/// grouping and stops symlink cycles; raise it if a real layout needs more.
fn visit(
    parser: &Matter<YAML>,
    skills: &mut BTreeMap<String, SkillState>,
    agent: Agent,
    folder: &std::path::Path,
    depth: usize,
) -> io::Result<bool> {
    if depth > 8 { return Ok(false); }
    let mut found = false;
    for child in fs::read_dir(folder)? {
        let child = child?;
        let path = child.path();
        let id = child.file_name().to_string_lossy().into_owned();
        if id.starts_with('.') { continue; }
        let link_metadata = fs::symlink_metadata(&path)?;
        if !link_metadata.is_dir() && !link_metadata.file_type().is_symlink() {
            continue;
        }
        // ponytail: broken links aren't loadable skills; a future diagnostics view can surface them.
        let metadata = match fs::metadata(&path) {
            Ok(metadata) if metadata.is_dir() => metadata,
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if has_skill_md(&path) {
            record(parser, skills, agent, id, path, &link_metadata, &metadata);
            found = true;
        } else {
            found |= visit(parser, skills, agent, &path, depth + 1)?;
        }
    }
    Ok(found)
}
