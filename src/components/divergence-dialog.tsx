import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { diffLines } from "diff";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import type { SkillState } from "@/views/all";

type Pair = SkillState["divergent_pairs"][number];
type Root = { path: string; agent: string };
type Contents = { left: string | null; right: string | null };
const agentName: Record<string, string> = { claude_code: "Claude Code", codex: "Codex", cursor: "Cursor", omp: "omp", hermes: "Hermes" };

export function DivergenceDialog({ skill, onClose, onRefresh }: {
  skill: SkillState | null;
  onClose: () => void;
  onRefresh: (skills: SkillState[]) => void;
}) {
  const [roots, setRoots] = useState<Root[]>([]);
  const [pair, setPair] = useState<Pair | null>(null);
  const [contents, setContents] = useState<Contents | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirm, setConfirm] = useState(false);
  const [reverse, setReverse] = useState(false);
  useEffect(() => { setPair(skill?.divergent_pairs[0] ?? null); setContents(null); setConfirm(false); setReverse(false); }, [skill]);
  useEffect(() => {
    if (!skill) return;
    let live = true;
    invoke<Root[]>("configured_roots").then((result) => { if (live) setRoots(result); }, (reason) => { if (live) setError(String(reason)); });
    return () => { live = false; };
  }, [skill]);
  const label = (path: string) => {
    const root = roots.find((root) => path === `${root.path}/${skill?.id}`);
    if (root) return `${agentName[root.agent] ?? root.agent} · ${root.path}`;
    const nested = roots.find((root) => path.startsWith(`${root.path}/`));
    return nested ? `${agentName[nested.agent] ?? nested.agent} · ${path}` : path;
  };
  useEffect(() => {
    if (!skill || !pair) return;
    let live = true;
    setContents(null);
    invoke<Contents>("skill_diff", { id: skill.id, leftRoot: pair.left, rightRoot: pair.right }).then(
      (result) => { if (live) { setContents(result); setError(null); } },
      (reason) => { if (live) setError(String(reason)); },
    );
    return () => { live = false; };
  }, [skill, pair]);
  if (!skill) return null;
  const marked = new Set(skill.divergent_pairs.flatMap(({ left, right }) => [left, right]));
  const segments = contents ? diffLines(contents.left ?? "", contents.right ?? "") : [];
  const lines = (side: "left" | "right") => segments.map((part, index) => {
    const belongs = side === "left" ? !part.added : !part.removed;
    if (!belongs) return null;
    return <span key={index} className={`block px-2 ${part.added || part.removed ? "bg-amber-200/60 text-foreground dark:bg-amber-800/50" : ""}`}>{part.value}</span>;
  });
  const winner = pair?.[reverse ? "right" : "left"];
  const loser = pair?.[reverse ? "left" : "right"];
  const replace = async () => {
    if (!pair || !confirm || !loser || !winner) return;
    setBusy(true);
    try {
      const result = await invoke<{ skills: SkillState[] }>("replace_with_link", { id: skill.id, targetRoot: loser, masterRoot: winner });
      onRefresh(result.skills);
      onClose();
    } catch (reason) { setError(String(reason)); }
    finally { setBusy(false); }
  };
  return <Dialog open onOpenChange={(open) => { if (!open && !busy) onClose(); }}>
    <DialogContent className="max-h-[90vh] max-w-[min(95vw,72rem)] overflow-y-auto">
      <DialogHeader><DialogTitle>Review copies: {skill.name}</DialogTitle>
        <DialogDescription>Different SKILL.md text needs your decision. Nothing changes until you confirm a replacement.</DialogDescription></DialogHeader>
      <div className="space-y-1 text-sm" aria-label="Copies">
        {skill.presences.map((copy) => <div key={copy.path} className="flex flex-wrap gap-2">
          <span>{label(copy.path)}</span><span className="text-muted-foreground">{copy.kind === "link" ? "link" : "real copy"}</span>
          {marked.has(copy.path) && <span className="font-medium text-amber-700 dark:text-amber-400">⚠ Divergent</span>}
        </div>)}
      </div>
      {skill.divergent_pairs.length > 0 && <label className="text-sm">Compare copies
        <select className="mt-1 block w-full rounded-md border bg-background p-2" value={pair ? skill.divergent_pairs.findIndex((item) => item.left === pair.left && item.right === pair.right) : 0} onChange={(event) => { setPair(skill.divergent_pairs[Number(event.target.value)]); setConfirm(false); setReverse(false); }}>
          {skill.divergent_pairs.map((item, index) => <option key={`${item.left}:${item.right}`} value={index}>{label(item.left)} ↔ {label(item.right)}</option>)}
        </select></label>}
      {pair && <div className="grid min-h-32 grid-cols-2 gap-2" aria-label="Side-by-side SKILL.md diff">
        {(["left", "right"] as const).map((side) => <section key={side} className="min-w-0 overflow-hidden rounded border">
          <h3 className="break-all border-b bg-muted px-2 py-1 text-xs font-medium">{label(pair[side])} · SKILL.md</h3>
          {contents?.[side] === null ? <p className="p-3 text-sm text-amber-700 dark:text-amber-400">SKILL.md missing</p> :
            <pre className="max-h-96 overflow-auto whitespace-pre-wrap break-words p-1 text-xs">{contents ? lines(side) : "Loading diff…"}</pre>}
        </section>)}
      </div>}
      {pair && <div className="rounded border border-amber-600/50 p-3 text-sm">
        <label className="mb-2 flex items-center gap-2">Keep <select className="min-w-0 max-w-full rounded border bg-background p-1" value={reverse ? "right" : "left"} onChange={(event) => { setReverse(event.target.value === "right"); setConfirm(false); }}><option value="left">{label(pair.left)}</option><option value="right">{label(pair.right)}</option></select></label>
        <p><strong>Replace:</strong> Move <code className="break-all">{loser}</code> to macOS Trash, then link it to <code className="break-all">{winner}</code>.</p>
        <p className="mt-1 text-muted-foreground">The trashed copy remains recoverable. Other roots are untouched.</p>
        <label className="mt-3 flex items-start gap-2"><input type="checkbox" checked={confirm} onChange={(event) => setConfirm(event.target.checked)} />I confirm replacing this exact target copy.</label>
      </div>}
      {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
      <DialogFooter><Button variant="outline" disabled={busy} onClick={onClose}>Cancel</Button><Button disabled={!pair || !contents || !confirm || busy || !winner || !loser} onClick={() => void replace()}>{busy ? "Replacing…" : "Move target to Trash and link"}</Button></DialogFooter>
    </DialogContent>
  </Dialog>;
}
