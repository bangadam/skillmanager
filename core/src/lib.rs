use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Agent {
    ClaudeCode,
    Codex,
    Cursor,
    Omp,
}

#[derive(Clone, Debug)]
pub struct Root {
    pub agent: Agent,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceKind {
    RealDir,
    Link,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Presence {
    pub agent: Agent,
    pub path: PathBuf,
    pub kind: PresenceKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    Created,
    Modified,
    Unavailable,
}

#[derive(Clone, Debug, Serialize)]
pub struct SkillState {
    pub id: String,
    pub name: String,
    pub description: String,
    pub presences: Vec<Presence>,
    pub date_millis: Option<u64>,
    pub date_source: DateSource,
}

#[derive(Deserialize)]
struct FrontMatter {
    name: Option<String>,
    description: Option<String>,
}

pub fn default_roots() -> io::Result<Vec<Root>> {
    let home = std::env::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory unavailable"))?;
    Ok([
        (Agent::ClaudeCode, ".claude/skills"),
        (Agent::Codex, ".codex/skills"),
        (Agent::Cursor, ".cursor/skills"),
        (Agent::Omp, ".omp/agent/skills"),
    ]
    .into_iter()
    .map(|(agent, path)| Root { agent, path: home.join(path) })
    .collect())
}

pub fn scan(roots: &[Root]) -> io::Result<Vec<SkillState>> {
    let parser = Matter::<YAML>::new();
    let mut skills = BTreeMap::<String, SkillState>::new();
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
            let markdown = path.join("SKILL.md");
            if !matches!(fs::symlink_metadata(&markdown), Ok(metadata) if metadata.is_file()) {
                continue;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            let (name, description) = fs::read_to_string(markdown)
                .ok()
                .and_then(|text| parser.parse::<FrontMatter>(&text).ok()?.data)
                .map(|data| (data.name.unwrap_or_else(|| id.clone()), data.description.unwrap_or_default()))
                .unwrap_or_else(|| (id.clone(), String::new()));
            let (date, date_source) = if let Ok(created) = metadata.created() {
                (Some(created), DateSource::Created)
            } else if let Ok(modified) = metadata.modified() {
                (Some(modified), DateSource::Modified)
            } else {
                (None, DateSource::Unavailable)
            };
            let date_millis = date.and_then(|date: SystemTime| date.duration_since(UNIX_EPOCH).ok())
                .and_then(|duration| u64::try_from(duration.as_millis()).ok());
            let skill = skills.entry(id.clone()).or_insert_with(|| SkillState {
                id,
                name,
                description,
                presences: Vec::new(),
                date_millis,
                date_source,
            });
            skill.presences.push(Presence {
                agent: root.agent,
                path,
                kind: if link_metadata.file_type().is_symlink() {
                    PresenceKind::Link
                } else {
                    PresenceKind::RealDir
                },
            });
        }
    }
    Ok(skills.into_values().collect())
}
