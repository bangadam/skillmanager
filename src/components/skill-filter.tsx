import { Button } from "@/components/ui/button";

type ManagedKind = "plugin" | "built_in";
export type CategorizedSkill = { managed_source?: { kind: ManagedKind; agent: string; plugin: string | null } | null; presences?: { agent: string }[] };
export type SkillCategory = "all" | "mine" | "plugin" | "built_in";
export type AgentKey = "any" | "claude_code" | "codex" | "cursor" | "omp";

export function filterSkills<T extends CategorizedSkill>(skills: T[], category: SkillCategory): T[] {
  return category === "all" ? skills : skills.filter((skill) =>
    category === "mine" ? !skill.managed_source : skill.managed_source?.kind === category,
  );
}

export function filterByAgent<T extends { presences?: { agent: string }[] }>(skills: T[], agent: AgentKey): T[] {
  return agent === "any" ? skills : skills.filter((skill) => skill.presences?.some((presence) => presence.agent === agent));
}

const categories: { key: SkillCategory; label: string }[] = [
  { key: "all", label: "All" },
  { key: "mine", label: "Mine" },
  { key: "plugin", label: "Plugin" },
  { key: "built_in", label: "Built-in" },
];

export function SkillFilter({ skills, value, onChange }: {
  skills: CategorizedSkill[];
  value: SkillCategory;
  onChange: (category: SkillCategory) => void;
}) {
  return (
    <div role="group" aria-label="Skill source" className="flex flex-wrap gap-1">
      {categories.map(({ key, label }) => (
        <Button key={key} type="button" size="sm" variant={value === key ? "default" : "outline"}
          aria-pressed={value === key} onClick={() => onChange(key)}>
          {label} <span className="ml-1 tabular-nums">{filterSkills(skills, key).length}</span>
        </Button>
      ))}
    </div>
  );
}

const agents: { key: AgentKey; label: string }[] = [
  { key: "any", label: "Any agent" },
  { key: "claude_code", label: "Claude Code" },
  { key: "codex", label: "Codex" },
  { key: "cursor", label: "Cursor" },
  { key: "omp", label: "omp" },
];

export function AgentFilter({ skills, value, onChange }: {
  skills: { presences?: { agent: string }[] }[];
  value: AgentKey;
  onChange: (agent: AgentKey) => void;
}) {
  return (
    <label className="flex items-center gap-2 text-sm text-muted-foreground">
      <span>Agent</span>
      <select className="h-9 rounded-md border border-input bg-background px-3 text-foreground shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" value={value} onChange={(event) => onChange(event.target.value as AgentKey)} aria-label="Filter by agent">
        {agents.map(({ key, label }) => (
          <option key={key} value={key}>{key === "any" ? label : `${label} (${filterByAgent(skills, key).length})`}</option>
        ))}
      </select>
    </label>
  );
}
