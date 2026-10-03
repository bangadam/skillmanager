import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { MissingView } from "@/views/missing";
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table";

type Agent = "claude_code" | "codex" | "cursor" | "omp";
type Presence = { agent: Agent; path: string; kind: "real_dir" | "link" };
type SkillState = {
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

export default function App() {
  const [skills, setSkills] = useState<SkillState[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<"all" | "missing">("all");

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
        <header className="mb-10 flex items-end justify-between gap-6 border-b border-border pb-7">
          <div>
            <p className="mb-3 text-xs font-semibold uppercase tracking-[0.25em] text-muted-foreground">Your skills, across agents</p>
            <h1 className="text-4xl font-semibold tracking-tight">Skill Manager</h1>
          </div>
          <p className="pb-1 text-sm tabular-nums text-muted-foreground" aria-live="polite">
            {skills === null ? "Scanning…" : `${skills.length} ${skills.length === 1 ? "skill" : "skills"}`}
          </p>
        </header>
        <nav aria-label="Skill views" className="mb-6 flex gap-2"><Button variant={view === "all" ? "default" : "outline"} onClick={() => setView("all")}>All</Button><Button variant={view === "missing" ? "default" : "outline"} onClick={() => setView("missing")}>Missing</Button></nav>
        {error ? (
          <p role="alert" className="rounded-md border border-destructive p-5 text-destructive">Could not scan skills: {error}</p>
        ) : skills === null ? (
          <p role="status" className="py-12 text-muted-foreground">Scanning skill folders…</p>
        ) : view === "missing" ? (
          <MissingView skills={skills} onRefresh={setSkills} />
        ) : skills.length === 0 ? (
          <p className="rounded-md border border-dashed border-border py-16 text-center text-muted-foreground">No skills found in your agent folders.</p>
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
              {skills.map((skill) => (
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
      </div>
    </main>
  );
}
