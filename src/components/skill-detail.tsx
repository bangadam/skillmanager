import { MousePointerClick, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { RenameDialog } from "@/components/rename-dialog";
import { AgentChip, agentMeta } from "@/components/agent-meta";
import { categoryLabel } from "@/lib/categories";
import { formatDate } from "@/lib/utils";
import type { SkillState } from "@/views/all";

function SectionHeading({ children }: { children: string }) {
  return <h2 className="pb-2 pt-6 text-[0.7rem] font-semibold uppercase tracking-wider text-muted-foreground">{children}</h2>;
}

function loadedFrom(path: string, id: string): string {
  return path.endsWith(`/${id}`) ? path.slice(0, -id.length - 1) : path;
}

export function SkillDetail({ skill, allSkills, onRefresh, width, onWidthChange, onEdit, onUninstall, onConsolidate }: {
  skill: SkillState | null;
  allSkills: SkillState[];
  onRefresh: (skills: SkillState[]) => void;
  width: number;
  onWidthChange: (width: number) => void;
  onEdit: (skill: SkillState) => void;
  onUninstall: (skill: SkillState) => void;
  onConsolidate: (skill: SkillState) => void;
}) {
  const startResize = (down: React.PointerEvent) => {
    const startX = down.clientX;
    const startWidth = width;
    down.currentTarget.setPointerCapture(down.pointerId);
    const move = (move: PointerEvent) => {
      onWidthChange(Math.min(Math.max(startWidth + (startX - move.clientX), 288), 720));
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  if (skill === null) {
    return (
      <aside aria-label="Skill details" className="relative hidden shrink-0 overflow-y-auto border-l border-border p-6 xl:block" style={{ width }}>
        <div role="separator" aria-orientation="vertical" aria-label="Resize details panel" tabIndex={0}
          onPointerDown={startResize}
          onKeyDown={(event) => {
            if (event.key === "ArrowLeft") onWidthChange(Math.max(width - 24, 288));
            if (event.key === "ArrowRight") onWidthChange(Math.min(width + 24, 720));
          }}
          className="absolute inset-y-0 left-0 z-10 w-1.5 -translate-x-px cursor-col-resize bg-transparent transition-colors hover:bg-ring/40 focus-visible:outline-2 focus-visible:outline-ring" />
        <Empty className="border-none py-16">
          <EmptyHeader>
            <EmptyMedia variant="icon"><MousePointerClick aria-hidden="true" /></EmptyMedia>
            <EmptyTitle>Select a skill</EmptyTitle>
            <EmptyDescription>Pick a skill to see where it lives, its copies, and what you can do with it.</EmptyDescription>
          </EmptyHeader>
        </Empty>
      </aside>
    );
  }

  const realCopies = skill.presences.filter((presence) => presence.kind === "real_dir");
  const source = skill.managed_source;

  return (
    <aside aria-label={`Details for ${skill.name}`} className="relative hidden shrink-0 overflow-y-auto border-l border-border p-6 xl:block" style={{ width }}>
      <div role="separator" aria-orientation="vertical" aria-label="Resize details panel" tabIndex={0}
        onPointerDown={startResize}
        onKeyDown={(event) => {
          if (event.key === "ArrowLeft") onWidthChange(Math.max(width - 24, 288));
          if (event.key === "ArrowRight") onWidthChange(Math.min(width + 24, 720));
        }}
        className="absolute inset-y-0 left-0 z-10 w-1.5 -translate-x-px cursor-col-resize bg-transparent transition-colors hover:bg-ring/40 focus-visible:outline-2 focus-visible:outline-ring" />
      <h1 className="break-words text-xl font-semibold tracking-tight">{skill.name}</h1>
      {source && (
        <p className="mt-1.5 text-xs text-muted-foreground">
          {agentMeta[source.agent].label} {source.kind === "built_in" ? "built-in" : "plugin"}{source.plugin ? `: ${source.plugin}` : ""} — read-only
        </p>
      )}
      <p className="mt-2.5 text-sm leading-relaxed break-words text-muted-foreground">{skill.description || "No description in SKILL.md."}</p>
      <p className="mt-3 text-xs text-muted-foreground">
        {categoryLabel(skill.category)} · {skill.date_millis === null ? "Date unavailable" : `${skill.date_source === "modified" ? "Modified" : "Created"} ${formatDate(skill.date_millis, true)}`}
        {skill.date_source === "modified" && <span className="sr-only"> (modified, not created)</span>}
      </p>

      <SectionHeading>Available in</SectionHeading>
      <ul className="space-y-2.5">
        {skill.presences.map((presence) => (
          <li key={presence.path} className="flex items-center gap-2">
            <AgentChip agent={presence.agent} />
            <span className="min-w-0 flex-1 truncate text-xs text-muted-foreground" title={presence.path}>
              {loadedFrom(presence.path, skill.id)}
            </span>
            <span className="shrink-0 text-[0.7rem] text-muted-foreground/80">{presence.kind === "link" ? "link" : "copy"}</span>
          </li>
        ))}
      </ul>

      <SectionHeading>Actions</SectionHeading>
      {source ? (
        <p className="text-sm text-muted-foreground">Managed by {agentMeta[source.agent].label}. Edit or uninstall it in {agentMeta[source.agent].label} itself.</p>
      ) : (
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" onClick={() => onEdit(skill)}><Pencil aria-hidden="true" />Edit</Button>
          <RenameDialog skill={skill} skills={allSkills} onRefresh={onRefresh} />
          {realCopies.length > 1 && <Button variant="outline" size="sm" onClick={() => onConsolidate(skill)}>Consolidate copies</Button>}
          <Button variant="destructive" size="sm" onClick={() => onUninstall(skill)}><Trash2 aria-hidden="true" />Uninstall</Button>
        </div>
      )}

      {skill.divergent_pairs.length > 0 && (
        <div className="mt-4 rounded-lg border border-amber-600/40 bg-amber-500/10 p-3.5">
          <p className="text-sm font-medium text-amber-700 dark:text-amber-300">
            {skill.divergent_pairs.length} divergent {skill.divergent_pairs.length === 1 ? "copy" : "copies"}
          </p>
          <p className="mt-0.5 text-xs text-muted-foreground">Linked folders that no longer match the master copy.</p>
        </div>
      )}
    </aside>
  );
}
