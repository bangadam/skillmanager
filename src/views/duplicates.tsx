import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Badge } from "@/components/ui/badge";

type Agent = "claude_code" | "codex" | "cursor" | "omp";
type Copy = { agent: Agent; path: string; kind: "real_dir" | "link" };
type Skill = { id: string; name: string; presences: Copy[] };
type Pair = { a: Skill; b: Skill; similarity: number; is_exact: boolean };

const agentNames: Record<Agent, string> = {
  claude_code: "Claude Code", codex: "Codex", cursor: "Cursor", omp: "omp",
};

function Copies({ skill }: { skill: Skill }) {
  return (
    <section className="min-w-0">
      <h3 className="mb-2 font-medium">{skill.name} <span className="text-sm font-normal text-muted-foreground">({skill.id})</span></h3>
      <ul className="space-y-2">
        {skill.presences.map((copy) => (
          <li key={copy.path} className="flex flex-wrap items-center gap-2 text-sm">
            <Badge variant="secondary">{agentNames[copy.agent]}</Badge>
            <Badge variant="outline">{copy.kind === "link" ? "Link" : "Real copy"}</Badge>
            <span className="min-w-0 break-all text-muted-foreground">{copy.path}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

export default function DuplicatesView({ skills, onGoToMissing }: { skills: unknown[] | null; onGoToMissing?: () => void }) {
  const [pairs, setPairs] = useState<Pair[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!skills) return;
    let active = true;
    invoke<Pair[]>("find_duplicates").then(
      (result) => { if (active) { setPairs(result); setError(null); } },
      (reason: unknown) => { if (active) setError(String(reason)); },
    );
    return () => { active = false; };
  }, [skills]);

  if (error) return <p role="alert" className="rounded-md border border-destructive p-5 text-destructive">Could not find duplicates: {error}</p>;
  if (!pairs) return <p role="status" className="py-12 text-muted-foreground">Comparing skill files…</p>;

  return (
    <section aria-label="Duplicate skills">
      <p className="mb-5 text-sm tabular-nums text-muted-foreground" aria-live="polite">{pairs.length} {pairs.length === 1 ? "duplicate pair" : "duplicate pairs"}{pairs.length > 0 && ". Select a pair to see every copy."}</p>
      {pairs.length === 0 && <p className="rounded-md border border-dashed border-border py-16 text-center text-muted-foreground">No similar skills found. Nothing to reconcile.</p>}
      <div className="space-y-3">
        {pairs.map((pair) => (
          <details key={`${pair.a.id}:${pair.b.id}`} className="rounded-md border border-border">
            <summary className="cursor-pointer px-5 py-4 font-medium focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring">
              {pair.a.name} <span aria-hidden="true">↔</span> {pair.b.name}
              <span className="ml-3 text-sm font-normal text-muted-foreground">{pair.is_exact ? "Exact match" : `${Math.round(pair.similarity * 100)}% similar`}</span>
            </summary>
            <div className="grid gap-6 border-t border-border px-5 py-5 sm:grid-cols-2">
              <Copies skill={pair.a} />
              <Copies skill={pair.b} />
              {onGoToMissing ? (
                <p className="text-sm text-muted-foreground sm:col-span-2">To manage copies, <button type="button" className="underline underline-offset-2 hover:text-foreground" onClick={onGoToMissing}>open the Missing view</button> for absent roots, or use Consolidate copies in All for unlinked duplicates. This comparison never changes files.</p>
              ) : (
                <p className="text-sm text-muted-foreground sm:col-span-2">Manage absent roots in Missing or unlinked duplicates in All. This comparison never changes files.</p>
              )}
            </div>
          </details>
        ))}
      </div>
    </section>
  );
}
