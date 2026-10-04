import { useEffect, useRef, type Dispatch, type SetStateAction } from "react";
import { Pencil, RefreshCw, Search, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { CategoryOption } from "@/lib/categories";
import type { SkillSort } from "@/views/sort-skills";
import type { SkillState } from "@/views/all";

const sortLabels: Record<SkillSort, string> = {
  "created-desc": "Newest created",
  "created-asc": "Oldest created",
  "name-asc": "Name A–Z",
  "name-desc": "Name Z–A",
};

export function ListToolbar({ query, setQuery, sort, setSort, category, setCategory, categories, selected, onEdit, onUninstall, onRefresh }: {
  query: string;
  setQuery: Dispatch<SetStateAction<string>>;
  sort: SkillSort;
  setSort: Dispatch<SetStateAction<SkillSort>>;
  category: string | null;
  setCategory: Dispatch<SetStateAction<string | null>>;
  categories: CategoryOption[];
  selected: SkillState | null;
  onEdit: (skill: SkillState) => void;
  onUninstall: (skill: SkillState) => void;
  onRefresh: () => void;
}) {
  const search = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const focusSearch = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
        event.preventDefault();
        search.current?.focus();
        search.current?.select();
      }
    };
    window.addEventListener("keydown", focusSearch);
    return () => window.removeEventListener("keydown", focusSearch);
  }, []);

  const actionable = selected !== null && !selected.managed_source;

  return (
    <header className="flex h-14 shrink-0 items-center gap-2 border-b border-border px-3">
      <Button variant="ghost" size="icon" aria-label={actionable ? `Edit ${selected?.name}` : "Select a skill to edit"} disabled={!actionable} onClick={() => selected && onEdit(selected)}>
        <Pencil aria-hidden="true" />
      </Button>
      <Button variant="ghost" size="icon" aria-label={actionable ? `Uninstall ${selected?.name}` : "Select a skill to uninstall"} disabled={!actionable} onClick={() => selected && onUninstall(selected)}>
        <Trash2 aria-hidden="true" />
      </Button>
      <Button variant="ghost" size="icon" aria-label="Refresh skill list" onClick={onRefresh}>
        <RefreshCw aria-hidden="true" />
      </Button>
      <span aria-hidden="true" className="mx-1 h-5 w-px bg-border" />
      <div className="hidden gap-2 md:flex">
        <Select value={sort} onValueChange={(value) => setSort(value as SkillSort)}>
          <SelectTrigger aria-label="Sort skills" className="w-40"><SelectValue /></SelectTrigger>
          <SelectContent>
            {(Object.keys(sortLabels) as SkillSort[]).map((key) => (
              <SelectItem key={key} value={key}>{sortLabels[key]}</SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={category ?? "all"} onValueChange={(value) => setCategory(value === "all" ? null : value)}>
          <SelectTrigger aria-label="Filter by category" className="w-44"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All categories</SelectItem>
            {categories.map((option) => (
              <SelectItem key={option.value} value={option.value}>{option.label} · {option.count}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <label className="relative min-w-0 flex-1 md:ml-2 md:max-w-64">
        <span className="sr-only">Search skills</span>
        <Search aria-hidden="true" className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input ref={search} type="search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search skills" className="pl-9" />
      </label>
    </header>
  );
}
