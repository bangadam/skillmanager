import type { SkillState } from "@/views/all";

export type CategoryOption = { value: string; label: string; count: number };

const categoryLabels: Record<string, string> = {
  ai: "AI",
  coding: "Coding",
  data: "Data",
  design: "Design",
  devops: "DevOps",
  other: "Other",
  research: "Research",
  security: "Security",
  writing: "Writing",
};

export function categoryLabel(category: string): string {
  return categoryLabels[category] ?? category.charAt(0).toUpperCase() + category.slice(1);
}

export function categoryOptions(skills: SkillState[]): CategoryOption[] {
  const counts = new Map<string, number>();
  for (const skill of skills) counts.set(skill.category, (counts.get(skill.category) ?? 0) + 1);
  return [...counts.entries()]
    .sort((left, right) => (left[0] === "other" ? 1 : 0) - (right[0] === "other" ? 1 : 0) || categoryLabel(left[0]).localeCompare(categoryLabel(right[0])))
    .map(([value, count]) => ({ value, label: categoryLabel(value), count }));
}
