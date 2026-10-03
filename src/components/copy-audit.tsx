import { Badge } from "@/components/ui/badge";
import type { SkillState } from "@/views/all";

const names = { claude_code: "Claude Code", codex: "Codex", cursor: "Cursor", omp: "omp" };

export function CopyAudit({ skill }: { skill: SkillState }) {
  return <details className="mt-2 text-sm font-normal">
    <summary className="w-fit cursor-pointer text-muted-foreground hover:text-foreground focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring">Copies ({skill.presences.length})</summary>
    <ul className="mt-3 space-y-2">{skill.presences.map((copy) =>
      <li key={copy.path} className="flex flex-wrap items-center gap-2">
        <Badge variant="secondary">{names[copy.agent]}</Badge>
        <Badge variant="outline">{copy.kind === "link" ? "Link" : "Real copy"}</Badge>
        <span className="break-all text-muted-foreground">{copy.path}</span>
      </li>)}</ul>
  </details>;
}
