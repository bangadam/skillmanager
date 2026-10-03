import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import type { SkillState } from "@/views/all";

const agents = { claude_code: "Claude Code", codex: "Codex", cursor: "Cursor", omp: "omp" };
type ScanState = { skills: SkillState[] };

export function RenameDialog({ skill, skills, onRefresh }: { skill: SkillState; skills: SkillState[]; onRefresh: (skills: SkillState[]) => void }) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState(skill.id);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const duplicate = skill.presences.filter((presence) => presence.kind === "real_dir").length > 1;
  const invalid = !name || name.trim() !== name || name.startsWith(".") || name.includes("/") || /[\u0000-\u001f\u007f-\u009f]/u.test(name) || name === skill.id;
  const collision = skills.some((other) => !other.managed_source && other.id === name && other.id !== skill.id);
  const validation = duplicate ? "This skill has unlinked duplicate copies. Link them first, then rename." : invalid ? "Enter a different folder name without a leading dot, slash, or surrounding whitespace." : collision ? `A skill named “${name}” already exists.` : null;

  const submit = async () => {
    if (validation || busy) return;
    setBusy(true);
    setError(null);
    try {
      const state = await invoke<ScanState>("rename_skill", { id: skill.id, newName: name });
      onRefresh(state.skills);
      setOpen(false);
    } catch (reason) {
      setError(String(reason));
    } finally { setBusy(false); }
  };

  return <Dialog open={open} onOpenChange={(next) => { if (busy) return; setOpen(next); if (next) { setName(skill.id); setError(null); } }}>
    <DialogTrigger asChild><Button variant="ghost" size="sm" aria-label={`Rename ${skill.name}`}>Rename</Button></DialogTrigger>
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Rename {skill.name}</DialogTitle>
        <DialogDescription>The master folder, SKILL.md name, and linked agent folders will change together. Unlinked copies are not renamed.</DialogDescription>
      </DialogHeader>
      <label htmlFor={`rename-${skill.id}`} className="text-sm font-medium">New folder name</label>
      <Input id={`rename-${skill.id}`} value={name} onChange={(event) => { setName(event.target.value); setError(null); }} aria-invalid={!!validation || !!error} aria-describedby={`rename-feedback-${skill.id}`} autoFocus />
      <p id={`rename-feedback-${skill.id}`} role={validation || error ? "alert" : undefined} className="text-sm text-destructive">{error || validation}</p>
      <div className="space-y-1 text-sm"><p className="font-medium">Folders that will move</p>
        <ul className="max-h-44 space-y-1 overflow-auto text-muted-foreground">{skill.presences.map((presence) => <li key={presence.path} className="break-all">{agents[presence.agent]} · {presence.path} {presence.kind === "link" ? "(link)" : "(master)"} → {name || "…"}</li>)}</ul>
      </div>
      <DialogFooter><Button variant="outline" disabled={busy} onClick={() => setOpen(false)}>Cancel</Button><Button disabled={busy || !!validation} onClick={submit}>{busy ? "Renaming…" : "Confirm rename"}</Button></DialogFooter>
    </DialogContent>
  </Dialog>;
}
