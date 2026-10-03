import { useEffect, useRef, type Dispatch, type SetStateAction } from "react";
import { Search } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { Input } from "@/components/ui/input";
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table";
import { filterAndSortSkills, type SkillSort } from "./sort-skills";

export type Agent = "claude_code" | "codex" | "cursor" | "omp";
export type Presence = { agent: Agent; path: string; kind: "real_dir" | "link" };
export type SkillState = {
  id: string;
  name: string;
  description: string;
  presences: Presence[];
  date_millis: number | null;
  date_source: "created" | "modified" | "unavailable";
};

const agentLabels: Record<Agent, { short: string; long: string }> = {
  claude_code: { short: "CC", long: "Claude Code" },
  codex: { short: "CDX", long: "Codex" },
  cursor: { short: "CUR", long: "Cursor" },
  omp: { short: "OMP", long: "omp" },
};

export function SkillToolbar({ query, setQuery, sort, setSort }: {
  query: string;
  setQuery: Dispatch<SetStateAction<string>>;
  sort: SkillSort;
  setSort: Dispatch<SetStateAction<SkillSort>>;
}) {
  const search = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const focusSearch = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
        event.preventDefault();
        search.current?.focus();
        search.current?.select();
      }
    };
    window.addEventListener("keydown", focusSearch);
    return () => window.removeEventListener("keydown", focusSearch);
  }, []);

  return (
    <div className="mb-5 flex flex-wrap items-center gap-3">
      <label className="relative min-w-52 flex-1 sm:max-w-sm">
        <span className="sr-only">Search skills</span>
        <Search aria-hidden="true" className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input ref={search} type="search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search skills…" className="pl-9" />
      </label>
      <label className="flex items-center gap-2 text-sm text-muted-foreground">
        <span>Sort by</span>
        <select className="h-9 rounded-md border border-input bg-background px-3 text-foreground shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" value={sort} onChange={(event) => setSort(event.target.value as SkillSort)}>
          <option value="created-desc">Newest created</option>
          <option value="created-asc">Oldest created</option>
          <option value="name-asc">Name A–Z</option>
          <option value="name-desc">Name Z–A</option>
        </select>
      </label>
    </div>
  );
}

export function AllView({ skills, query, sort }: { skills: SkillState[]; query: string; sort: SkillSort }) {
  const visibleSkills = filterAndSortSkills(skills, query, sort);
  return (
    <section aria-label="Skills">
      <p className="mb-4 text-sm tabular-nums text-muted-foreground" aria-live="polite">
        {visibleSkills.length} {visibleSkills.length === 1 ? "skill" : "skills"}{query.trim() && ` matching “${query.trim()}”`}
      </p>
      {visibleSkills.length === 0 ? (
        <Empty className="rounded-md border border-dashed border-border py-16">
          <EmptyHeader>
            <EmptyMedia variant="icon"><Search aria-hidden="true" /></EmptyMedia>
            <EmptyTitle>{query.trim() ? "No matching skills" : "No skills found"}</EmptyTitle>
            <EmptyDescription>{query.trim() ? "Try another name or description, or clear your search." : "Skills you add to an agent’s skills folder will appear here."}</EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[24%]">Skill</TableHead>
              <TableHead>Description</TableHead>
              <TableHead className="w-[190px]">Agents</TableHead>
              <TableHead className="w-[135px] text-right">Created</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {visibleSkills.map((skill) => (
              <TableRow key={skill.id}>
                <TableCell className="py-5 align-top font-medium">{skill.name}</TableCell>
                <TableCell className="py-5 align-top text-muted-foreground">{skill.description || "—"}</TableCell>
                <TableCell className="py-5 align-top">
                  <div className="flex flex-wrap gap-1.5">
                    {skill.presences.map((presence) => (
                      <Badge key={presence.path} variant="secondary" title={`${agentLabels[presence.agent].long} · ${presence.kind === "link" ? "link" : "real copy"}`}>
                        <span className="sr-only">{agentLabels[presence.agent].long}: </span>{agentLabels[presence.agent].short}
                      </Badge>
                    ))}
                  </div>
                </TableCell>
                <TableCell className="py-5 text-right align-top text-sm tabular-nums text-muted-foreground" title={skill.date_source === "modified" ? "Creation date unavailable; showing modified date" : undefined}>
                  {skill.date_millis === null ? "—" : new Date(skill.date_millis).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" })}
                  {skill.date_source === "modified" && <span className="sr-only"> (modified, not created)</span>}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </section>
  );
}
