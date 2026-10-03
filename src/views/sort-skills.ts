export type SkillSort = "created-desc" | "created-asc" | "name-asc" | "name-desc";

type SearchableSkill = {
  id: string;
  name: string;
  description: string;
  date_millis: number | null;
};

export function filterAndSortSkills<T extends SearchableSkill>(skills: T[], query: string, sort: SkillSort): T[] {
  const needle = query.trim().toLocaleLowerCase();
  const matches = skills.filter((skill) =>
    skill.name.toLocaleLowerCase().includes(needle) || skill.description.toLocaleLowerCase().includes(needle),
  );
  return matches.sort((left, right) => {
    if (sort === "name-asc" || sort === "name-desc") {
      const order = left.name.localeCompare(right.name, undefined, { sensitivity: "base" });
      if (order !== 0) return order * (sort === "name-asc" ? 1 : -1);
    } else {
      // ponytail: a skill without a filesystem date goes last; show an explicit date-source sort only if users need it.
      if (left.date_millis === null || right.date_millis === null) {
        if (left.date_millis !== right.date_millis) return left.date_millis === null ? 1 : -1;
      } else if (left.date_millis !== right.date_millis) {
        return (left.date_millis - right.date_millis) * (sort === "created-asc" ? 1 : -1);
      }
    }
    return left.id.localeCompare(right.id);
  });
}
