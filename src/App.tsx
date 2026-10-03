import { useState } from "react";
import { useSkills } from "@/hooks/use-skills";
import { AllView, SkillToolbar } from "@/views/all";
import type { SkillSort } from "@/views/sort-skills";

export default function App() {
  const { skills, error } = useSkills();
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SkillSort>("created-desc");

  return (
    <main className="min-h-screen bg-background px-8 pb-16 pt-12 text-foreground sm:px-12">
      <div className="mx-auto max-w-6xl">
        <header className="mb-10 border-b border-border pb-7">
          <p className="mb-3 text-xs font-semibold uppercase tracking-[0.25em] text-muted-foreground">Your skills, across agents</p>
          <h1 className="text-4xl font-semibold tracking-tight">Skill Manager</h1>
        </header>
        {error ? (
          <p role="alert" className="rounded-md border border-destructive p-5 text-destructive">Could not scan skills: {error}</p>
        ) : skills === null ? (
          <p role="status" className="py-12 text-muted-foreground">Scanning skill folders…</p>
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
