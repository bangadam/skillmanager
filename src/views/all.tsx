import { useState } from "react";
import { ChevronDown, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "cn";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { AgentIcon } from "@/components/agent-meta";
import { categoryLabel } from "@/lib/categories";
import { formatDate } from "@/lib/utils";
import { folderGroups } from "@/lib/skill-groups";
import { filterAndSortSkills, type SkillSort } from "./sort-skills";

export type Agent = "claude_code" | "codex" | "cursor" | "omp" | "hermes";
export type Presence = { agent: Agent; path: string; kind: "real_dir" | "link" };
export type SkillState = {
  divergent_pairs: { left: string; right: string }[];
  id: string;
  name: string;
  description: string;
  category: string;
  presences: Presence[];
  date_millis: number | null;
  date_source: "created" | "modified" | "unavailable";
  managed_source?: { agent: Agent; kind: "plugin" | "built_in"; plugin: string | null } | null;
  folder?: string | null;
};

export function AllView({ skills, query, sort, categoryFilter, selectedId, onSelect, onRemoveSelected, checked, checkedOwnCount, onToggleCheck, onBulkEdit, onClearChecks, onSetChecked, folderHome }: {
  skills: SkillState[];
  query: string;
  sort: SkillSort;
  categoryFilter: string | null;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onRemoveSelected: () => void;
  checked: Set<string>;
  checkedOwnCount: number;
  onToggleCheck: (id: string) => void;
  onBulkEdit: () => void;
  onClearChecks: () => void;
  onSetChecked: (ids: string[]) => void;
  folderHome: string;
}) {
  const visibleSkills = filterAndSortSkills(skills, query, sort);
  const groups = folderGroups(visibleSkills, folderHome);
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const onToggleGroup = (key: string) =>
    setCollapsed((prev) => { const next = new Set(prev); prev.has(key) ? next.delete(key) : next.add(key); return next; });
  const allChecked = visibleSkills.length > 0 && visibleSkills.every((skill) => checked.has(skill.id));
  const onToggleSelectAll = () => onSetChecked(allChecked ? [] : visibleSkills.map((skill) => skill.id));
  return (
    <section aria-label="Skills" className="flex min-h-full flex-col p-3">
      <div className="flex items-center justify-between gap-3 px-3 pb-2 pt-1">
        <p className="min-w-0 flex-1 truncate text-xs tabular-nums text-muted-foreground" aria-live="polite">
          {visibleSkills.length} {visibleSkills.length === 1 ? "skill" : "skills"}{categoryFilter && ` · ${categoryLabel(categoryFilter)}`}{query.trim() && ` matching “${query.trim()}”`}
        </p>
        {visibleSkills.length > 0 && (
          <Button variant="ghost" size="sm" onClick={onToggleSelectAll} aria-pressed={allChecked}>
            {allChecked ? "Deselect all" : "Select all"}
            <span className="sr-only"> {visibleSkills.length} skills</span>
          </Button>
        )}
      </div>
      {visibleSkills.length === 0 ? (
        <Empty className="rounded-lg border border-dashed border-border py-14">
          <EmptyHeader>
            <EmptyMedia variant="icon"><Search aria-hidden="true" /></EmptyMedia>
            <EmptyTitle>{query.trim() || categoryFilter ? "No matching skills" : "No skills found"}</EmptyTitle>
            <EmptyDescription>{query.trim() || categoryFilter ? "Try another name or description, or clear your search or category filter." : "Skills you add to an agent’s skills folder will appear here."}</EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <ul className="space-y-1">
          {groups.map((group) => {
            const isCollapsed = collapsed.has(group.key);
            return (
            <li key={group.key}>
              <button
                type="button"
                onClick={() => onToggleGroup(group.key)}
                aria-expanded={!isCollapsed}
                className="flex w-full items-center gap-1.5 rounded-md px-3 pb-0.5 pt-3 text-left text-[0.7rem] font-semibold uppercase tracking-wider text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring first:pt-1"
              >
                <ChevronDown aria-hidden="true" className={cn("size-3.5 shrink-0 transition-transform", isCollapsed && "-rotate-90")} />
                <span title={group.key} className="min-w-0 truncate normal-case tracking-normal">{group.label}</span>
                <span className="font-normal">({group.skills.length})</span>
              </button>
              {!isCollapsed && group.skills.map((skill) => {
            const selected = skill.id === selectedId;
            const isChecked = checked.has(skill.id);
            return (
              <div key={skill.presences[0]?.path ?? skill.id} className="group">
                <div className={cn(
                  "flex items-start gap-2 rounded-lg px-3 py-2.5 transition-colors",
                  selected ? "bg-muted" : "hover:bg-muted/60",
                  skill.managed_source && "text-muted-foreground",
                )}>
                  <input
                    type="checkbox"
                    checked={isChecked}
                    onChange={() => onToggleCheck(skill.id)}
                    aria-label={`Select ${skill.name} for bulk actions`}
                    className={cn(
                      "mt-0.5 size-4 shrink-0 cursor-pointer accent-foreground focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring",
                      checked.size === 0 && !isChecked && "opacity-0 group-hover:opacity-100 focus-visible:opacity-100",
                    )}
                  />
                  <button
                    type="button"
                    onClick={() => onSelect(skill.id)}
                    aria-current={selected ? "true" : undefined}
                    className="min-w-0 flex-1 rounded-md text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    <span className="flex items-baseline justify-between gap-3">
                      <span className="min-w-0 truncate font-medium">{skill.name}</span>
                      <span className="shrink-0 text-xs tabular-nums text-muted-foreground">{formatDate(skill.date_millis)}</span>
                    </span>
                    <span className="mt-0.5 flex items-center justify-between gap-3">
                      <span className="min-w-0 flex-1 truncate text-sm text-muted-foreground">{skill.description || "No description"}</span>
                      <span className="flex shrink-0 items-center gap-1.5">
                        {skill.divergent_pairs.length > 0 && (
                          <span className="rounded border border-amber-600/50 px-1.5 py-0.5 text-[0.7rem] text-amber-700 dark:text-amber-400">
                            ⚠ {skill.divergent_pairs.length}
                          </span>
                        )}
                        {skill.presences.map((presence) => (
                          <AgentIcon key={presence.path} agent={presence.agent} />
                        ))}
                      </span>
                    </span>
                  </button>
                </div>
              </div>
            );
              })}
            </li>
          )})}
        </ul>
      )}
      {checked.size > 0 && (
        <div className="sticky bottom-0 -mx-3 mt-auto flex flex-wrap items-center gap-2 border-t border-border bg-background px-3 py-2.5">
          <p className="text-sm font-medium">{checked.size} selected</p>
          <div className="flex-1" />
          <Button variant="outline" size="sm" disabled={checkedOwnCount === 0} onClick={onBulkEdit}>
            Set category<span className="sr-only"> for {checkedOwnCount} skills</span>
          </Button>
          <Button variant="destructive" size="sm" onClick={onRemoveSelected}>Remove…<span className="sr-only"> {checked.size} skills</span></Button>
          <Button variant="ghost" size="sm" onClick={onClearChecks}>Clear</Button>
        </div>
      )}
    </section>
  );
}
