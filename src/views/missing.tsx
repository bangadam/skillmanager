import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "cn";
import { Button } from "@/components/ui/button";
import { AgentIcon, agentMeta } from "@/components/agent-meta";
import { covers } from "@/lib/utils";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import type { Agent } from "@/views/all";

export type Root = { agent: Agent; path: string };
export type MissingSkill = { id: string; name: string; description: string; category: string; divergent_pairs: { left: string; right: string }[]; presences: { path: string; agent: Agent; kind: "real_dir" | "link" }[]; date_millis: number | null; date_source: "created" | "modified" | "unavailable"; managed_source?: { agent: Agent; kind: "plugin" | "built_in"; plugin: string | null } | null };
type Choice = { skill: MissingSkill; everywhere: boolean; consolidate?: boolean };
const label = (root: Root) => `${agentMeta[root.agent].label} · ${root.path}`;

export function MissingView({ skills, roots, onRefresh, onDiff, consolidate, onConsolidated, dialogOnly = false, selectedId, onSelect }: { skills: MissingSkill[]; roots: Root[]; onRefresh: (skills: MissingSkill[]) => void; onDiff: (skill: MissingSkill) => void; consolidate?: MissingSkill | null; onConsolidated?: () => void; dialogOnly?: boolean; selectedId: string | null; onSelect: (id: string) => void }) {
  const [choice, setChoice] = useState<Choice | null>(null);
  const [target, setTarget] = useState("");
  const [master, setMaster] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Mirror of core::links::missing over the already-scanned skills — no extra invoke.
  const missing = useMemo(() => skills.filter((skill) => skill.managed_source == null && skill.presences.length > 0
    && roots.some((root) => !skill.presences.some((presence) => covers(root, skill.id, presence)))), [skills, roots]);

  useEffect(() => {
    if (!consolidate || !roots.length) return;
    setChoice({ skill: consolidate, everywhere: false, consolidate: true });
    setMaster(roots.find((root) => consolidate.presences.some((copy) => copy.kind === "real_dir" && covers(root, consolidate.id, copy)))?.path ?? "");
    setError(null);
  }, [consolidate, roots]);

  const open = (skill: MissingSkill, everywhere: boolean) => {
    setChoice({ skill, everywhere });
    setTarget("");
    setMaster(roots.find((root) => skill.presences.some((presence) => presence.kind === "real_dir" && covers(root, skill.id, presence)))?.path ?? "");
    setError(null);
  };
  const absent = choice ? roots.filter((root) => !choice.skill.presences.some((presence) => covers(root, choice.skill.id, presence))) : [];
  const real = choice ? roots.filter((root) => choice.skill.presences.some((presence) => presence.kind === "real_dir" && covers(root, choice.skill.id, presence))) : [];
  const act = async () => {
    if (busy || !choice || (!choice.everywhere && !choice.consolidate && !target) || (real.length > 1 && !master)) return;
    setBusy(true);
    setError(null);
    try {
      if (real.length > 1) await invoke("choose_master", { id: choice.skill.id, chosenRoot: master });
      const failures = choice.consolidate ? [] : choice.everywhere
        ? (await invoke<{ failures: string[] }>("link_everywhere", { id: choice.skill.id })).failures
        : (await invoke("link_skill", { id: choice.skill.id, targetRoot: target }), []);
      onRefresh((await invoke<{ skills: MissingSkill[] }>("scan_skills")).skills);
      if (failures.length) setError(`Some roots could not be linked: ${failures.join("; ")}`);
      else { setChoice(null); onConsolidated?.(); }
    } catch (reason) {
      // A master selection may already have linkified duplicates; refresh even when the next step fails.
      try { onRefresh((await invoke<{ skills: MissingSkill[] }>("scan_skills")).skills); } catch { /* Show the operation error first. */ }
      setError(String(reason));
      if (choice.skill.divergent_pairs?.length && String(reason).includes("divergent")) { setChoice(null); onConsolidated?.(); onDiff(choice.skill); }
    } finally { setBusy(false); }
  };

  return <section aria-label={dialogOnly ? "Consolidate copies" : "Missing skills"} className={dialogOnly ? undefined : "p-3"}>
    {!dialogOnly && <>
      <p className="px-3 pb-2 pt-1 text-xs tabular-nums text-muted-foreground" aria-live="polite">{missing.length} missing {missing.length === 1 ? "skill" : "skills"}</p>
      {error && <p role="alert" className="mx-3 mb-2 rounded-md border border-destructive p-3 text-sm text-destructive">{error}</p>}
      {missing.length === 0 ? <p className="m-3 rounded-lg border border-dashed p-12 text-center text-sm text-muted-foreground">No missing skills — every configured root has each skill.</p> :
        <ul className="space-y-1">
          {missing.map((skill) => (
            <li key={skill.presences[0]?.path ?? skill.id} className={cn(
              "flex items-center gap-3 rounded-lg px-3 py-2.5 transition-colors",
              skill.id === selectedId ? "bg-muted" : "hover:bg-muted/60",
            )}>
              <button type="button" onClick={() => onSelect(skill.id)} aria-current={skill.id === selectedId ? "true" : undefined} className="min-w-0 flex-1 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring">
                <p className="flex items-center gap-2 font-medium">
                  <span className="min-w-0 truncate">{skill.name}</span>
                  {skill.divergent_pairs?.length > 0 && <span className="rounded border border-amber-600/50 px-1.5 py-0.5 text-[0.7rem] font-normal text-amber-700 dark:text-amber-400">⚠ {skill.divergent_pairs.length}</span>}
                </p>
                <span className="mt-0.5 flex items-center justify-between gap-3">
                  <span className="min-w-0 flex-1 truncate text-sm text-muted-foreground">{skill.description || skill.id}</span>
                  <span className="flex shrink-0 items-center gap-1.5">
                    {skill.presences.map((presence) => <AgentIcon key={presence.path} agent={presence.agent} />)}
                  </span>
                </span>
              </button>
              <div className="flex shrink-0 gap-1.5">
                <Button variant="outline" size="sm" onClick={() => open(skill, false)}>Add to…</Button>
                <Button size="sm" onClick={() => open(skill, true)}>Link everywhere</Button>
              </div>
            </li>
          ))}
        </ul>}
    </>}
    <Dialog open={choice !== null} onOpenChange={(isOpen) => { if (!isOpen && !busy) { setChoice(null); setError(null); onConsolidated?.(); } }}>
      <DialogContent><DialogHeader><DialogTitle>{choice?.consolidate ? "Consolidate copies" : choice?.everywhere ? "Link everywhere" : "Add to…"}: {choice?.skill.name}</DialogTitle>
        <DialogDescription>Confirm before changing skill folders. {choice?.consolidate ? "Keep the selected master; linkify identical real copies." : choice?.everywhere ? `Link into ${absent.length} missing root${absent.length === 1 ? "" : "s"}.` : "Choose the agent root to link into."}</DialogDescription></DialogHeader>
        {!choice?.everywhere && !choice?.consolidate && <Select value={target} onValueChange={setTarget}><SelectTrigger aria-label="Target agent root"><SelectValue placeholder="Choose a target agent" /></SelectTrigger><SelectContent>{absent.map((root) => <SelectItem key={root.path} value={root.path}>{label(root)}</SelectItem>)}</SelectContent></Select>}
        {real.length > 1 && <div className="space-y-3"><p className="text-sm font-medium">Choose the master copy</p><p className="text-sm text-muted-foreground">Identical other real copies move to Trash, then become links. Divergent copies are never changed.</p>
          <RadioGroup value={master} onValueChange={setMaster}>{real.map((root) => <label key={root.path} className="flex items-center gap-2 text-sm"><RadioGroupItem value={root.path} />{label(root)}</label>)}</RadioGroup></div>}
        {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
        {choice && choice.skill.divergent_pairs?.length > 0 && <Button variant="outline" onClick={() => { setChoice(null); onConsolidated?.(); onDiff(choice.skill); }}>Review divergent copies and diff</Button>}
        <DialogFooter><Button variant="outline" disabled={busy} onClick={() => { setChoice(null); onConsolidated?.(); }}>Cancel</Button><Button disabled={busy || (!choice?.everywhere && !choice?.consolidate && !target) || (real.length > 1 && !master)} onClick={act}>{busy ? "Linking…" : choice?.consolidate ? "Consolidate copies" : "Confirm links"}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  </section>;
}
