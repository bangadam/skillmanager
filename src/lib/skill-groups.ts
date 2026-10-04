import { agentMeta } from "@/components/agent-meta";
import type { Agent, SkillState } from "@/views/all";

export type SkillGroup = { key: string; label: string; skills: SkillState[] };

export function skillGroupKey(skill: SkillState): string {
  const source = skill.managed_source;
  if (!source) return "mine";
  if (source.plugin) return `plugin:${source.plugin}`;
  return `managed:${source.kind}:${source.agent}`;
}

export function skillGroupLabel(key: string): string {
  if (key === "mine") return "Your skills";
  if (key.startsWith("plugin:")) return key.slice("plugin:".length);
  const [, kind, agent] = key.split(":");
  return `${agentMeta[agent as Agent].label} ${kind === "built_in" ? "built-ins" : "plugins"}`;
}

export function sourceGroups(skills: SkillState[]): SkillGroup[] {
  const groups = new Map<string, SkillGroup>();
  for (const skill of skills) {
    const key = skillGroupKey(skill);
    const group = groups.get(key) ?? { key, label: skillGroupLabel(key), skills: [] };
    group.skills.push(skill);
    groups.set(key, group);
  }
  return [...groups.values()].sort((a, b) =>
    a.key === "mine" ? -1 : b.key === "mine" ? 1 : a.label.localeCompare(b.label));
}
