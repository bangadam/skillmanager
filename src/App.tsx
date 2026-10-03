import { useState } from "react";
import { Button } from "@/components/ui/button";
import { MissingView } from "@/views/missing";
import { useSkills } from "@/hooks/use-skills";
import { filterSkills, SkillFilter, type SkillCategory } from "@/components/skill-filter";
import { AllView, SkillToolbar } from "@/views/all";
import type { SkillSort } from "@/views/sort-skills";
import DuplicatesView from "@/views/duplicates";
import { DivergenceDialog } from "@/components/divergence-dialog";
import type { SkillState } from "@/views/all";

export default function App() {
  const { skills, setSkills, error, configError, configNotice } = useSkills();
  const [category, setCategory] = useState<SkillCategory>("all");
  const [view, setView] = useState<"all" | "missing" | "duplicates">("all");
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SkillSort>("created-desc");
  const [diffSkill, setDiffSkill] = useState<SkillState | null>(null);


  return (
    <main className="min-h-screen bg-background px-8 pb-16 pt-12 text-foreground sm:px-12">
      <div className="mx-auto max-w-6xl">
        <header className="mb-10 border-b border-border pb-7">
          <p className="mb-3 text-xs font-semibold uppercase tracking-[0.25em] text-muted-foreground">Your skills, across agents</p>
          <h1 className="text-4xl font-semibold tracking-tight">Skill Manager</h1>
        </header>
        {configError && <p role="alert" className="mb-5 rounded-md border border-destructive p-4 text-destructive">Extra roots config: {configError}</p>}
        {configNotice && <p role="status" className="mb-5 text-sm text-muted-foreground">{configNotice}</p>}
        <nav aria-label="Skill views" className="mb-6 flex gap-2 border-b border-border pb-3">
          <Button variant={view === "all" ? "default" : "outline"} onClick={() => setView("all")}>All</Button>
          <Button variant={view === "missing" ? "default" : "outline"} onClick={() => setView("missing")}>Missing</Button>
          <Button variant={view === "duplicates" ? "default" : "outline"} onClick={() => setView("duplicates")}>Duplicates</Button>
        </nav>
        {view === "duplicates" ? <DuplicatesView skills={skills} onGoToMissing={() => setView("missing")} /> : error ? (
          <p role="alert" className="rounded-md border border-destructive p-5 text-destructive">Could not scan skills: {error}</p>
        ) : skills === null ? (
          <p role="status" className="py-12 text-muted-foreground">Scanning skill folders…</p>
        ) : view === "missing" ? (
          <MissingView skills={skills} onRefresh={setSkills} onDiff={setDiffSkill} />
        ) : (
          <>
            <div className="mb-5 flex flex-wrap items-center gap-3">
              <SkillToolbar query={query} setQuery={setQuery} sort={sort} setSort={setSort} />
              <SkillFilter skills={skills} value={category} onChange={setCategory} />
            </div>
            <AllView skills={filterSkills(skills, category)} query={query} sort={sort} onDiff={setDiffSkill} />
          </>
        )}
        <DivergenceDialog skill={diffSkill && skills?.find((skill) => skill.id === diffSkill.id && !skill.managed_source) || null} onClose={() => setDiffSkill(null)} onRefresh={setSkills} />
      </div>
    </main>
  );
}
