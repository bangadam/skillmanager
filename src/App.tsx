import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { MissingView } from "@/views/missing";
import { AllView, SkillToolbar, type SkillState } from "@/views/all";
import type { SkillSort } from "@/views/sort-skills";

export default function App() {
  const [skills, setSkills] = useState<SkillState[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<"all" | "missing">("all");
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SkillSort>("created-desc");

  useEffect(() => {
    let active = true;
    invoke<SkillState[]>("scan_skills").then(
      (result) => { if (active) setSkills(result); },
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
        <nav aria-label="Skill views" className="mb-6 flex gap-2"><Button variant={view === "all" ? "default" : "outline"} onClick={() => setView("all")}>All</Button><Button variant={view === "missing" ? "default" : "outline"} onClick={() => setView("missing")}>Missing</Button></nav>
        {error ? (
          <p role="alert" className="rounded-md border border-destructive p-5 text-destructive">Could not scan skills: {error}</p>
        ) : skills === null ? (
          <p role="status" className="py-12 text-muted-foreground">Scanning skill folders…</p>
        ) : view === "missing" ? (
          <MissingView skills={skills} onRefresh={setSkills} />
        ) : (
          <>
            <SkillToolbar query={query} setQuery={setQuery} sort={sort} setSort={setSort} />
            <AllView skills={skills} query={query} sort={sort} />
          </>
        )}
      </div>
    </main>
  );
}
