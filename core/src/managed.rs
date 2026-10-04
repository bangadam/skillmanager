use crate::{Agent, Root, SkillState};
use serde::Serialize;
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedKind { Plugin, BuiltIn }

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ManagedSource {
    pub agent: Agent,
    pub kind: ManagedKind,
    pub plugin: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ManagedRoot {
    pub agent: Agent,
    pub path: PathBuf,
    pub kind: ManagedKind,
    pub plugin: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct ScanState {
    pub skills: Vec<SkillState>,
    pub config_error: Option<String>,
    pub config_notice: Option<String>,
    pub roots: Vec<Root>,
}

pub fn config_path(home: &Path) -> PathBuf { home.join(".config/skillmanager/roots.json") }

pub fn configured_roots(roots: &[Root], config: &Path) -> (Vec<Root>, Option<String>, Option<String>) {
    let mut all_roots = roots.to_vec();
    let (config_error, config_notice) = match fs::read(config) {
        Ok(bytes) => match serde_json::from_slice::<Vec<Root>>(&bytes) {
            Ok(extra) if extra.iter().all(|root| root.path.is_absolute()) => {
                all_roots.extend(extra);
                (None, None)
            }
            Ok(_) => (Some(format!("{}: extra root paths must be absolute", config.display())), None),
            Err(error) => (Some(format!("{}: {error}", config.display())), None),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound =>
            (None, Some(format!("No extra roots configured ({}).", config.display()))),
        Err(error) => (Some(format!("{}: {error}", config.display())), None),
    };
    (all_roots, config_error, config_notice)
}

pub fn scan_state(roots: &[Root], config: &Path, managed: &[ManagedRoot]) -> io::Result<ScanState> {
    let (all_roots, config_error, config_notice) = configured_roots(roots, config);
    let mut skills = crate::scan(&all_roots)?;
    for source in managed {
        for mut skill in crate::scan(&[Root { agent: source.agent, path: source.path.clone() }])? {
            skill.managed_source = Some(ManagedSource {
                agent: source.agent, kind: source.kind, plugin: source.plugin.clone(),
            });
            skills.push(skill);
        }
    }
    Ok(ScanState { skills, config_error, config_notice, roots: all_roots })
}

pub fn default_managed_roots(home: &Path) -> io::Result<Vec<ManagedRoot>> {
    let mut roots = vec![ManagedRoot {
        agent: Agent::Codex, path: home.join(".codex/skills/.system"), kind: ManagedKind::BuiltIn, plugin: None,
    }];
    for (agent, cache) in [
        (Agent::ClaudeCode, ".claude/plugins/cache"),
        (Agent::Codex, ".codex/plugins/cache"),
        (Agent::Cursor, ".cursor/plugins/cache"),
        (Agent::Omp, ".omp/plugins/cache/plugins"),
    ] {
        let base = home.join(cache);
        collect_skill_roots(&base, &base, agent, 0, &mut roots)?;
    }
    Ok(roots)
}

fn collect_skill_roots(base: &Path, dir: &Path, agent: Agent, depth: usize, roots: &mut Vec<ManagedRoot>) -> io::Result<()> {
    // ponytail: cache layouts inspected on this Mac reach `skills` within six levels;
    // extend this bounded walk if a future agent cache nests deeper.
    if depth > 6 { return Ok(()); }
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() { continue; }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') { continue; }
        let path = entry.path();
        if name == "skills" {
            let parts: Vec<_> = dir.strip_prefix(base).unwrap_or(dir).components().collect();
            let plugin = if agent == Agent::Omp {
                parts.first().map(|p| p.as_os_str().to_string_lossy().split("___").take(2).collect::<Vec<_>>().join(" / "))
            } else {
                parts.get(1).map(|p| p.as_os_str().to_string_lossy().into_owned())
            };
            roots.push(ManagedRoot { agent, path: path.clone(), kind: ManagedKind::Plugin, plugin: plugin.clone() });
            // omp skills are grouped under skills/<group>/<skill>; other caches can use groups too.
            for group in fs::read_dir(path)? {
                let group = group?;
                if group.file_type()?.is_dir() && !group.path().join("SKILL.md").exists() {
                    roots.push(ManagedRoot { agent, path: group.path(), kind: ManagedKind::Plugin, plugin: plugin.clone() });
                }
            }
        } else {
            collect_skill_roots(base, &path, agent, depth + 1, roots)?;
        }
    }
    Ok(())
}
