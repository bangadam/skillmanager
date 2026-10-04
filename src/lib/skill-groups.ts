import type { SkillState } from "@/views/all";

export type SkillGroup = { key: string; label: string; skills: SkillState[] };

export function folderGroupKey(skill: SkillState): string {
  return skill.folder ?? "(unknown)";
}

// Longest common directory-boundary prefix across all folders — in practice $HOME.
export function homePrefix(skills: SkillState[]): string {
  const folders = skills.map((skill) => skill.folder).filter((folder): folder is string => folder != null);
  return folders.length === 0 ? "" : folders.reduce((prefix, folder) => {
    let i = 0;
    while (i < prefix.length && i < folder.length && prefix[i] === folder[i]) i++;
    // Snap back to a directory boundary so the prefix never cuts mid-segment.
    return prefix.slice(0, prefix.lastIndexOf("/", i - 1) + 1);
  });
}

// Groups by the folder the skill is saved in, symlinks resolved; labels trim `home` to `~`.
// `home` defaults to the passed list's own prefix; pass the app-wide one to keep labels
// stable when the list is filtered down to a single folder.
export function folderGroups(skills: SkillState[], home = homePrefix(skills)): SkillGroup[] {
  // Trim only when it shortens: home === folder (everything in one dir) reads better untrimmed.
  const trim = (folder: string) => home && folder.length > home.length && folder.startsWith(home)
    ? `~${folder.slice(home.length)}` : folder;
  const groups = new Map<string, SkillGroup>();
  for (const skill of skills) {
    const key = folderGroupKey(skill);
    const group = groups.get(key) ?? { key, label: key === "(unknown)" ? "Unknown location" : trim(key), skills: [] };
    group.skills.push(skill);
    groups.set(key, group);
  }
  return [...groups.values()].sort((a, b) => a.key.localeCompare(b.key));
}
