import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { filterSkills, SkillFilter, type SkillCategory } from "@/components/skill-filter";
import { AllView, SkillToolbar, type SkillState } from "@/views/all";
import type { SkillSort } from "@/views/sort-skills";
import DuplicatesView from "@/views/duplicates";

export default function App() {
  const [skills, setSkills] = useState<SkillState[] | null>(null);
  const [category, setCategory] = useState<SkillCategory>("all");
  const [configError, setConfigError] = useState<string | null>(null);
  const [configNotice, setConfigNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<"all" | "duplicates">("all");
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SkillSort>("created-desc");

  useEffect(() => {
    let active = true;
    invoke<{ skills: SkillState[]; config_error: string | null; config_notice: string | null }>("scan_skills").then(
      (result) => { if (active) { setSkills(result.skills); setConfigError(result.config_error); setConfigNotice(result.config_notice); } },
      (reason: unknown) => { if (active) setError(String(reason)); },
    );
    return () => { active = false; };
  }, []);

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
          <button type="button" aria-current={view === "all" ? "page" : undefined} onClick={() => setView("all")} className={`rounded-md px-4 py-2 text-sm ${view === "all" ? "bg-secondary font-medium" : "text-muted-foreground hover:text-foreground"}`}>All</button>
          <button type="button" aria-current={view === "duplicates" ? "page" : undefined} onClick={() => setView("duplicates")} className={`rounded-md px-4 py-2 text-sm ${view === "duplicates" ? "bg-secondary font-medium" : "text-muted-foreground hover:text-foreground"}`}>Duplicates</button>
        </nav>
        {view === "duplicates" ? <DuplicatesView /> : error ? (
          <p role="alert" className="rounded-md border border-destructive p-5 text-destructive">Could not scan skills: {error}</p>
        ) : skills === null ? (
          <p role="status" className="py-12 text-muted-foreground">Scanning skill folders…</p>
        ) : (
          <>
            <div className="mb-5 flex flex-wrap items-center gap-3">
              <SkillToolbar query={query} setQuery={setQuery} sort={sort} setSort={setSort} />
              <SkillFilter skills={skills} value={category} onChange={setCategory} />
            </div>
            <AllView skills={filterSkills(skills, category)} query={query} sort={sort} />
          </>
        )}
      </div>
    </main>
  );
}
