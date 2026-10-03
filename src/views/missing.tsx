import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

type Agent = "claude_code" | "codex" | "cursor" | "omp";
export type Root = { agent: Agent; path: string };
export type MissingSkill = { id: string; name: string; description: string; presences: { path: string; agent: Agent; kind: "real_dir" | "link" }[]; date_millis: number | null; date_source: "created" | "modified" | "unavailable" };
type Choice = { skill: MissingSkill; everywhere: boolean };
const agentName: Record<string, string> = { claude_code: "Claude Code", codex: "Codex", cursor: "Cursor", omp: "omp" };
const label = (root: Root) => `${agentName[root.agent] ?? root.agent} · ${root.path}`;

export function MissingView({ skills, onRefresh }: { skills: MissingSkill[]; onRefresh: (skills: MissingSkill[]) => void }) {
  const [roots, setRoots] = useState<Root[]>([]);
  const [missing, setMissing] = useState<MissingSkill[]>([]);
  const [choice, setChoice] = useState<Choice | null>(null);
  const [target, setTarget] = useState("");
  const [master, setMaster] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    Promise.all([invoke<Root[]>("configured_roots"), invoke<MissingSkill[]>("missing_skills")]).then(
      ([configured, gaps]) => { if (alive) { setRoots(configured); setMissing(gaps); } },
      (reason: unknown) => { if (alive) setError(String(reason)); },
    );
    return () => { alive = false; };
  }, [skills]);

  const open = (skill: MissingSkill, everywhere: boolean) => {
    setChoice({ skill, everywhere });
    setTarget("");
    setMaster(roots.find((root) => skill.presences.some((presence) => presence.path === `${root.path}/${skill.id}` && presence.kind === "real_dir"))?.path ?? "");
    setError(null);
  };
  const absent = choice ? roots.filter((root) => !choice.skill.presences.some((presence) => presence.path === `${root.path}/${choice.skill.id}`)) : [];
  const real = choice ? roots.filter((root) => choice.skill.presences.some((presence) => presence.path === `${root.path}/${choice.skill.id}` && presence.kind === "real_dir")) : [];
  const act = async () => {
    if (!choice || (!choice.everywhere && !target) || (real.length > 1 && !master)) return;
    setBusy(true);
    setError(null);
    try {
      if (real.length > 1) await invoke("choose_master", { id: choice.skill.id, chosenRoot: master });
      const failures = choice.everywhere
        ? (await invoke<{ failures: string[] }>("link_everywhere", { id: choice.skill.id })).failures
        : (await invoke("link_skill", { id: choice.skill.id, targetRoot: target }), []);
      onRefresh((await invoke<{ skills: MissingSkill[] }>("scan_skills")).skills);
      if (failures.length) setError(`Some roots could not be linked: ${failures.join("; ")}`);
      else setChoice(null);
    } catch (reason) {
      // A master selection may already have linkified duplicates; refresh even when the next step fails.
      try { onRefresh((await invoke<{ skills: MissingSkill[] }>("scan_skills")).skills); } catch { /* Show the operation error first. */ }
      setError(String(reason));
    } finally { setBusy(false); }
  };

  return <section aria-label="Missing skills">
    {error && <p role="alert" className="mb-4 rounded-md border border-destructive p-3 text-destructive">{error}</p>}
    {missing.length === 0 ? <p className="rounded-md border border-dashed p-12 text-center text-muted-foreground">No missing skills — every configured root has each skill.</p> :
      <ul className="divide-y divide-border">{missing.map((skill) => <li key={skill.id} className="flex flex-wrap items-center justify-between gap-4 py-5">
        <div><p className="font-medium">{skill.name}</p><p className="text-sm text-muted-foreground">{skill.description || skill.id}</p></div>
        <div className="flex gap-2"><Button variant="outline" onClick={() => open(skill, false)}>Add to…</Button><Button onClick={() => open(skill, true)}>Link everywhere</Button></div>
      </li>)}</ul>}
    <Dialog open={choice !== null} onOpenChange={(isOpen) => { if (!isOpen && !busy) { setChoice(null); setError(null); } }}>
      <DialogContent><DialogHeader><DialogTitle>{choice?.everywhere ? "Link everywhere" : "Add to…"}: {choice?.skill.name}</DialogTitle>
        <DialogDescription>Confirm before changing skill folders. {choice?.everywhere ? `Link into ${absent.length} missing root${absent.length === 1 ? "" : "s"}.` : "Choose the agent root to link into."}</DialogDescription></DialogHeader>
        {!choice?.everywhere && <Select value={target} onValueChange={setTarget}><SelectTrigger aria-label="Target agent root"><SelectValue placeholder="Choose a target agent" /></SelectTrigger><SelectContent>{absent.map((root) => <SelectItem key={root.path} value={root.path}>{label(root)}</SelectItem>)}</SelectContent></Select>}
        {real.length > 1 && <div className="space-y-3"><p className="text-sm font-medium">Choose the master copy</p><p className="text-sm text-muted-foreground">Identical other real copies move to Trash, then become links. Divergent copies are never changed.</p>
          <RadioGroup value={master} onValueChange={setMaster}>{real.map((root) => <label key={root.path} className="flex items-center gap-2 text-sm"><RadioGroupItem value={root.path} />{label(root)}</label>)}</RadioGroup></div>}
        {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
        <DialogFooter><Button variant="outline" disabled={busy} onClick={() => setChoice(null)}>Cancel</Button><Button disabled={busy || (!choice?.everywhere && !target)} onClick={act}>{busy ? "Linking…" : "Confirm links"}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  </section>;
}
