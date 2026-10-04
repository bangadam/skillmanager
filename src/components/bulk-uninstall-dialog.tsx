import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import type { SkillState } from "@/views/all";
export type BulkGroup = { label: string | null; sources: string[]; managed: boolean };
type Action = { action: "remove_link" | "promote" | "repoint_link" | "trash_master"; path: string; target?: string };
type BulkEntry = { name: string; plan: { actions: Action[] } };


export function BulkUninstallDialog({ group, onClose, onRefresh }: {
  group: BulkGroup | null;
  onClose: () => void;
  onRefresh: (skills: SkillState[]) => void;
}) {
  const [entries, setEntries] = useState<BulkEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!group) return;
    let active = true;
    setEntries(null);
    setError(null);
    void invoke<BulkEntry[]>("plan_bulk_uninstall", { sourcePaths: group.sources })
      .then((plans) => { if (active) setEntries(plans); })
      .catch((reason: unknown) => { if (active) setError(String(reason)); });
    return () => { active = false; };
  }, [group]);

  const trashCount = entries?.flatMap((entry) => entry.plan.actions).filter((action) => action.action === "trash_master").length ?? 0;
  const linkCount = entries?.flatMap((entry) => entry.plan.actions).filter((action) => action.action === "remove_link").length ?? 0;

  async function confirm() {
    if (!group || !entries || busy) return;
    setBusy(true);
    setError(null);
    try {
      const state = await invoke<{ skills: SkillState[] }>("uninstall_skills", {
        sourcePaths: group.sources,
        expectedPlans: entries.map((entry) => entry.plan),
      });
      onRefresh(state.skills);
      onClose();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={group !== null} onOpenChange={(open) => { if (!open && !busy) { setEntries(null); setError(null); onClose(); } }}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>Remove {group?.sources.length === 1 ? "1 skill" : `${group?.sources.length ?? 0} skills`}</DialogTitle>
          <DialogDescription>
            {group?.label ? `Every skill in ${group.label}.` : "Each skill you selected."} Real content moves to the macOS Trash and is recoverable there.
          </DialogDescription>
        </DialogHeader>
        {group?.managed && (
          <p role="note" className="rounded-md border border-amber-600/50 bg-amber-50 p-3 text-sm text-amber-800 dark:bg-amber-950 dark:text-amber-300">
            Managed skills — this removes them from the plugin cache. The agent may re-install them next time it updates or re-syncs its plugins.
          </p>
        )}
        {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
        {entries ? (
          <>
            <ul className="max-h-48 space-y-1 overflow-auto rounded-md border p-3 text-sm">
              {entries.map((entry) => <li key={entry.name} className="truncate">{entry.name}</li>)}
            </ul>
            <p className="text-sm text-muted-foreground">
              {trashCount} {trashCount === 1 ? "folder" : "folders"} to Trash{linkCount > 0 && `, ${linkCount} ${linkCount === 1 ? "link" : "links"} removed`}
            </p>
          </>
        ) : !error && <p role="status" className="text-sm text-muted-foreground">Checking affected paths…</p>}
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={onClose}>Cancel</Button>
          <Button variant="destructive" disabled={!entries || busy} onClick={() => void confirm()}>
            {busy ? "Removing…" : `Remove ${group?.sources.length ?? 0}`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
