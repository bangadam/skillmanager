import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import type { CategoryOption } from "@/lib/categories";
import type { SkillState } from "@/views/all";

type EditResult = { skills: SkillState[]; files_written: string[] };

export function BulkEditDialog({ ids, categories, onClose, onRefresh }: {
  ids: string[];
  categories: CategoryOption[];
  onClose: () => void;
  onRefresh: (skills: SkillState[]) => void;
}) {
  const [category, setCategory] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const trimmed = category.trim();
  const invalid = trimmed === "" || trimmed.includes(":") || trimmed.length > 40;

  async function confirm() {
    if (invalid || busy) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<EditResult>("set_categories", { ids, category: trimmed });
      onRefresh(result.skills);
      onClose();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={ids.length > 0} onOpenChange={(open) => { if (!open && !busy) onClose(); }}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Set category for {ids.length === 1 ? "1 skill" : `${ids.length} skills`}</DialogTitle>
          <DialogDescription>
            Writes <code>category:</code> to each skill’s SKILL.md. Links follow the master copy; identical copies update together.
          </DialogDescription>
        </DialogHeader>
        <label htmlFor="bulk-category" className="text-sm font-medium">
          Category
          <Input
            id="bulk-category"
            value={category}
            onChange={(event) => { setCategory(event.target.value); setError(null); }}
            placeholder="e.g. coding"
            disabled={busy}
            className="mt-1.5"
            aria-invalid={!!error || (category.trim() !== "" && invalid)}
          />
        </label>
        {categories.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            {categories.map((option) => (
              <Button key={option.value} type="button" variant="outline" size="xs" disabled={busy}
                onClick={() => { setCategory(option.value); setError(null); }}>
                {option.label}
              </Button>
            ))}
          </div>
        )}
        {category.trim() !== "" && invalid && <p role="alert" className="text-sm text-destructive">Use one plain word, without a colon.</p>}
        {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={onClose}>Cancel</Button>
          <Button disabled={invalid || busy} onClick={() => void confirm()}>{busy ? "Saving…" : "Set category"}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
