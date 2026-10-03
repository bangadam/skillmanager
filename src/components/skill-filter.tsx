import { Button } from "@/components/ui/button";

type ManagedKind = "plugin" | "built_in";
export type CategorizedSkill = { managed_source?: { kind: ManagedKind; agent: string; plugin: string | null } | null };
export type SkillCategory = "all" | "mine" | "plugin" | "built_in";

export function filterSkills<T extends CategorizedSkill>(skills: T[], category: SkillCategory): T[] {
  return category === "all" ? skills : skills.filter((skill) =>
    category === "mine" ? !skill.managed_source : skill.managed_source?.kind === category,
  );
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
