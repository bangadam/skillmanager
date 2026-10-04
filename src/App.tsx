import { useEffect, useMemo, useState } from "react";
import { Sidebar, type View } from "@/components/sidebar";
import { ListToolbar } from "@/components/list-toolbar";
import { SkillDetail } from "@/components/skill-detail";
import { folderGroupKey, homePrefix } from "@/lib/skill-groups";
import { categoryOptions } from "@/lib/categories";
import { useSkills } from "@/hooks/use-skills";
import { AllView, type Agent, type SkillState } from "@/views/all";
import type { SkillSort } from "@/views/sort-skills";
import { MissingView } from "@/views/missing";
import DuplicatesView from "@/views/duplicates";
import { DivergenceDialog } from "@/components/divergence-dialog";
import { UninstallDialog } from "@/components/uninstall-dialog";
import { EditDialog } from "@/components/edit-dialog";
import { BulkUninstallDialog, type BulkGroup } from "@/components/bulk-uninstall-dialog";
import { BulkEditDialog } from "@/components/bulk-edit-dialog";

export default function App() {
  const { skills, setSkills, roots, error, configError, refresh } = useSkills();
  const [view, setView] = useState<View>("all");
  const [groupFilter, setGroupFilter] = useState<string | null>(null);
  const folderHome = useMemo(() => skills === null ? "" : homePrefix(skills), [skills]);
  const [bulkGroup, setBulkGroup] = useState<BulkGroup | null>(null);
  const [agentFilter, setAgentFilter] = useState<Agent | null>(null);
  const [categoryFilter, setCategoryFilter] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SkillSort>("created-desc");
  const [selected, setSelected] = useState<string | null>(null);
  const [checked, setChecked] = useState<Set<string>>(new Set());
  const [bulkEditIds, setBulkEditIds] = useState<string[]>([]);
  const [diffSkill, setDiffSkill] = useState<SkillState | null>(null);
  const [uninstallSkill, setUninstallSkill] = useState<SkillState | null>(null);
  const [editSkill, setEditSkill] = useState<SkillState | null>(null);
  const [consolidateSkill, setConsolidateSkill] = useState<SkillState | null>(null);
  const [detailWidth, setDetailWidth] = useState(480);
  const [sidebarWidth, setSidebarWidth] = useState(240);

  useEffect(() => {
    if (skills === null) return;
    if (selected !== null && !skills.some((skill) => skill.id === selected)) setSelected(null);
    else if (selected === null && skills.length > 0) setSelected(skills[0].id);
  }, [skills, selected]);

  useEffect(() => {
    if (groupFilter !== null && skills !== null && !skills.some((skill) => folderGroupKey(skill) === groupFilter)) setGroupFilter(null);
  }, [skills, groupFilter]);

  useEffect(() => {
    if (categoryFilter !== null && skills !== null && !skills.some((skill) => skill.category === categoryFilter)) setCategoryFilter(null);
  }, [skills, categoryFilter]);

  useEffect(() => {
    setChecked((previous) => {
      if (skills === null) return previous;
      const kept = [...previous].filter((id) => skills.some((skill) => skill.id === id));
      return kept.length === previous.size ? previous : new Set(kept);
    });
  }, [skills]);

  const visible = useMemo(() => skills === null ? [] : skills.filter((skill) =>
    (groupFilter === null || folderGroupKey(skill) === groupFilter)
    && (agentFilter === null || skill.presences.some((presence) => presence.agent === agentFilter))
    && (categoryFilter === null || skill.category === categoryFilter)), [skills, groupFilter, agentFilter, categoryFilter]);
  const selectedSkill = skills?.find((skill) => skill.id === selected) ?? null;
  const showListChrome = !error && skills !== null && view !== "duplicates";
  const categories = useMemo(() => skills === null ? [] : categoryOptions(skills), [skills]);
  const checkedSkills = useMemo(() => skills === null ? [] : skills.filter((skill) => checked.has(skill.id)), [skills, checked]);
  const checkedOwn = checkedSkills.filter((skill) => skill.managed_source == null);
  const removeSelected = () => setBulkGroup({
    label: null,
    sources: checkedSkills.map((skill) => skill.presences[0]?.path).filter((path): path is string => path !== undefined),
    managed: checkedSkills.some((skill) => skill.managed_source != null),
  });
  const clearChecks = () => setChecked(new Set());
  const setChecks = (ids: string[]) => setChecked(new Set(ids));
  const toggleCheck = (id: string) => setChecked((previous) => {
    const next = new Set(previous);
    if (next.has(id)) next.delete(id); else next.add(id);
    return next;
  });

  return (
    <div className="flex h-dvh overflow-hidden bg-background text-foreground">
      <Sidebar skills={skills ?? []} roots={roots} view={view} onView={setView} agentFilter={agentFilter} onAgentFilter={setAgentFilter} groupFilter={groupFilter} onGroupFilter={setGroupFilter}
        categories={categories} categoryFilter={categoryFilter} onCategoryFilter={setCategoryFilter} width={sidebarWidth} onWidthChange={setSidebarWidth} />
      <div className="flex min-w-0 flex-1 flex-col">
        {configError && (
          <p role="alert" className="border-b border-destructive/30 px-4 py-2 text-sm text-destructive">
            Extra roots config: {configError}
          </p>
        )}
        {view === "all" && (
          <ListToolbar query={query} setQuery={setQuery} sort={sort} setSort={setSort} category={categoryFilter} setCategory={setCategoryFilter}
            categories={categories}
            selected={selectedSkill} onEdit={setEditSkill} onUninstall={setUninstallSkill} onRefresh={refresh} />
        )}
        <div className="flex min-h-0 flex-1">
          <section aria-label={view === "missing" ? "Missing skills" : view === "duplicates" ? "Duplicate skills" : "Skills"} className="min-w-0 flex-1 overflow-y-auto">
            {error ? (
              <p role="alert" className="m-3 rounded-md border border-destructive p-5 text-destructive">Could not scan skills: {error}</p>
            ) : skills === null ? (
              <p role="status" className="p-6 text-sm text-muted-foreground">Scanning skill folders…</p>
            ) : view === "duplicates" ? (
              <DuplicatesView skills={skills} onGoToMissing={() => setView("missing")} />
            ) : view === "missing" ? (
              <MissingView skills={skills} roots={roots ?? []} selectedId={selected} onSelect={setSelected} onRefresh={setSkills} onDiff={setDiffSkill} />
            ) : (
              <AllView skills={visible} query={query} sort={sort} categoryFilter={categoryFilter} selectedId={selected} onSelect={setSelected} folderHome={folderHome}
                onRemoveSelected={removeSelected} checked={checked} checkedOwnCount={checkedOwn.length}
                onToggleCheck={toggleCheck} onBulkEdit={() => setBulkEditIds(checkedOwn.map((skill) => skill.id))} onClearChecks={clearChecks} onSetChecked={setChecks} />
            )}
          </section>
          {showListChrome && (
            <SkillDetail skill={selectedSkill} allSkills={skills ?? []} onRefresh={setSkills} width={detailWidth} onWidthChange={setDetailWidth}
              onEdit={setEditSkill} onUninstall={setUninstallSkill} onConsolidate={setConsolidateSkill} />
          )}
        </div>
      </div>
      {consolidateSkill && skills && (
        <MissingView skills={skills} roots={roots ?? []} selectedId={selected} onSelect={setSelected} onRefresh={setSkills} onDiff={setDiffSkill}
          consolidate={consolidateSkill} onConsolidated={() => setConsolidateSkill(null)} dialogOnly />
      )}
      <DivergenceDialog skill={diffSkill && skills?.find((skill) => skill.id === diffSkill.id && !skill.managed_source) || null} onClose={() => setDiffSkill(null)} onRefresh={setSkills} />
      <EditDialog skill={editSkill && skills?.find((skill) => skill.id === editSkill.id && !skill.managed_source) || null} resolving={!!diffSkill} onClose={() => setEditSkill(null)} onDiff={setDiffSkill} onRefresh={setSkills} />
      <UninstallDialog skill={uninstallSkill} onClose={() => setUninstallSkill(null)} onRefresh={setSkills} />
      <BulkUninstallDialog group={bulkGroup} onClose={() => { setBulkGroup(null); clearChecks(); }} onRefresh={setSkills} />
      <BulkEditDialog ids={bulkEditIds} categories={categories}
        onClose={() => setBulkEditIds([])} onRefresh={(next) => { setSkills(next); clearChecks(); }} />
    </div>
  );
}
