import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import type { SkillState } from "@/views/all";

type Action = { action: "remove_link" | "promote" | "repoint_link" | "trash_master"; path: string; target?: string };
type ScanState = { skills: SkillState[] };
const labels = { remove_link: "Remove link", promote: "Promote to master", repoint_link: "Re-point link", trash_master: "Move to Trash" };
const agentLabels = { claude_code: "Claude Code", codex: "Codex", cursor: "Cursor", omp: "omp" };

export function UninstallDialog({ skill, onClose, onRefresh }: {
  skill: SkillState | null;
  onClose: () => void;
  onRefresh: (skills: SkillState[]) => void;
}) {
  const [root, setRoot] = useState("");
  const [actions, setActions] = useState<Action[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const sourcePath = skill?.presences[0]?.path;

  useEffect(() => {
    if (!sourcePath) return;
    let active = true;
    setActions(null);
    setError(null);
    void invoke<{ actions: Action[] }>("plan_uninstall", { sourcePath, root: root || null })
      .then((plan) => { if (active) setActions(plan.actions); })
      .catch((reason) => { if (active) setError(String(reason)); });
    return () => { active = false; };
  }, [sourcePath, root]);

  async function confirm() {
    if (!sourcePath || !actions || busy) return;
    setBusy(true);
    setError(null);
    try {
      const state = await invoke<ScanState>("uninstall_skill", { sourcePath, root: root || null, expectedActions: actions });
      onRefresh(state.skills);
      setRoot("");
      setActions(null);
      onClose();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={skill !== null} onOpenChange={(open) => { if (!open && !busy) { setRoot(""); setActions(null); onClose(); } }}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>Uninstall {skill?.name}</DialogTitle>
          <DialogDescription>Review every affected path before continuing. Real content moves to the macOS Trash and is recoverable there.</DialogDescription>
        </DialogHeader>
        <label className="grid gap-1 text-sm font-medium">
          Remove from
          <select className="h-9 rounded-md border border-input bg-background px-3" value={root} disabled={busy} onChange={(event) => { setActions(null); setRoot(event.target.value); }}>
            <option value="">Everywhere</option>
            {skill?.presences.map((presence) => (
              <option value={presence.path.slice(0, -(skill.id.length + 1))} key={presence.path}>
                {agentLabels[presence.agent]} · {presence.path.slice(0, -(skill.id.length + 1))}
              </option>
            ))}
          </select>
        </label>
        {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
        {actions ? <ul className="max-h-64 space-y-2 overflow-auto rounded-md border p-3 text-sm">
          {actions.map((item) => <li key={`${item.action}:${item.path}`}>
            <span className="font-medium">{labels[item.action]}:</span> <code className="break-all">{item.path}</code>
            {item.target && <> → <code className="break-all">{item.target}</code></>}
          </li>)}
        </ul> : !error && <p role="status" className="text-sm text-muted-foreground">Checking affected paths…</p>}
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={() => { setRoot(""); setActions(null); onClose(); }}>Cancel</Button>
          <Button variant="destructive" disabled={!actions || busy} onClick={() => void confirm()}>{busy ? "Uninstalling…" : "Uninstall"}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
