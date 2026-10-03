import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import type { SkillState } from "@/views/all";

type EditResult = { skills: SkillState[]; files_written: string[] };

export function EditDialog({ skill, resolving, onClose, onDiff, onRefresh }: {
  skill: SkillState | null;
  resolving: boolean;
  onClose: () => void;
  onDiff: (skill: SkillState) => void;
  onRefresh: (skills: SkillState[]) => void;
}) {
  const [content, setContent] = useState("");
  const [saved, setSaved] = useState("");
  const [written, setWritten] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);
  useEffect(() => {
    if (!skill) return;
    let live = true;
    setLoaded(false);
    setWritten([]);
    setError(null);
    invoke<string>("read_skill_md", { id: skill.id }).then(
      (text) => { if (live) { setContent(text); setSaved(text); setLoaded(true); } },
      (reason) => { if (live) setError(String(reason)); },
    );
    return () => { live = false; };
  }, [skill?.id]);
  if (!skill) return null;
  const close = () => {
    if (busy) return;
    if (loaded && content !== saved && !window.confirm("Discard unsaved SKILL.md changes?")) return;
    onClose();
  };
  const save = async () => {
    if (!loaded || busy) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<EditResult>("edit_skill", { id: skill.id, newContent: content });
      setSaved(content);
      setWritten(result.files_written);
      onRefresh(result.skills);
    } catch (reason) {
      setError(String(reason));
      // ponytail: ask the existing diff flow to resolve conflicts; retry remains an explicit Save click.
      try {
        const state = await invoke<{ skills: SkillState[] }>("scan_skills");
        onRefresh(state.skills);
        const current = state.skills.find((item) => item.id === skill.id && !item.managed_source);
        if (current?.divergent_pairs.length) onDiff(current);
      } catch { /* Original save error remains visible. */ }
    } finally { setBusy(false); }
  };
  return <Dialog open={!resolving} onOpenChange={(open) => { if (!open) close(); }}>
    <DialogContent className="max-h-[90vh] max-w-[min(95vw,52rem)] overflow-y-auto">
      <DialogHeader><DialogTitle>Edit {skill.name}</DialogTitle>
        <DialogDescription>Edit the full SKILL.md text. Saving updates the master and identical unlinked copies; links follow the master.</DialogDescription></DialogHeader>
      <label htmlFor="skill-md-editor" className="text-sm font-medium">SKILL.md</label>
      <textarea id="skill-md-editor" className="min-h-72 w-full resize-y rounded-md border border-input bg-background p-3 font-mono text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" value={content} onChange={(event) => { setContent(event.target.value); setWritten([]); }} disabled={!loaded || busy} spellCheck={false} />
      {written.length > 0 && <div role="status" className="text-sm"><p>Saved to:</p><ul className="list-inside list-disc break-all">{written.map((path) => <li key={path}>{path}</li>)}</ul></div>}
      {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
      <DialogFooter><Button variant="outline" disabled={busy} onClick={close}>Close</Button><Button disabled={!loaded || busy} onClick={() => void save()}>{busy ? "Saving…" : "Save"}</Button></DialogFooter>
    </DialogContent>
  </Dialog>;
}
