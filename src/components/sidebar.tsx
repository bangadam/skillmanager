import { useMemo, useState, type ReactNode } from "react";
import { Bot, ChevronDown, Code2, Database, Layers, Microscope, Package, Palette, PenLine, Server, ShieldCheck, Shapes, PanelLeft, TriangleAlert, User } from "lucide-react";
import { cn } from "cn";
import { AgentIcon, agentMeta } from "@/components/agent-meta";
import type { Agent, Presence, SkillState } from "@/views/all";
import { covers } from "@/lib/utils";
import { sourceGroups } from "@/lib/skill-groups";
import type { Root } from "@/views/missing";
import type { CategoryOption } from "@/lib/categories";
export type View = "all" | "missing" | "duplicates";

function NavItem({ active, onClick, icon, count, label }: {
  active: boolean;
  onClick: () => void;
  icon?: ReactNode;
  count?: number;
  label: string;
}) {
  return (
    <button
      type="button"
      title={label}
      onClick={onClick}
      aria-current={active ? "page" : undefined}
      className={cn(
        "flex w-full items-center justify-center gap-2.5 rounded-lg py-1.5 text-left text-sm transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring md:justify-start md:px-2.5",
        active ? "bg-muted font-medium text-foreground" : "text-muted-foreground hover:bg-muted/60 hover:text-foreground",
      )}
    >
      {icon}
      <span className="hidden min-w-0 flex-1 truncate md:inline">{label}</span>
      {count !== undefined && <span className="hidden text-xs tabular-nums text-muted-foreground md:inline">{count}</span>}
    </button>
  );
}

function SectionHeading({ children }: { children: ReactNode }) {
  return <h2 className="hidden px-2.5 pb-1 pt-5 text-[0.7rem] font-semibold uppercase tracking-wider text-muted-foreground md:block">{children}</h2>;
}

// Section whose items can be collapsed away; the heading becomes its toggle.
function CollapsibleSection({ title, children, defaultOpen = true }: { title: string; children: ReactNode; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <>
      <button
        type="button"
        onClick={() => setOpen((previous) => !previous)}
        aria-expanded={open}
        className="hidden w-full items-center gap-1 px-2.5 pb-1 pt-5 text-[0.7rem] font-semibold uppercase tracking-wider text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring md:flex"
      >
        <ChevronDown aria-hidden="true" className={`size-3 shrink-0 transition-transform ${open ? "" : "-rotate-90"}`} />
        {title}
      </button>
      {open && children}
    </>
  );
}

// Mirror of core::links::missing — a user skill absent from at least one configured root.
function isMissing(skill: SkillState, roots: Root[]): boolean {
  return skill.managed_source == null && skill.presences.length > 0
    && roots.some((root) => !skill.presences.some((presence: Presence) => covers(root, skill.id, presence)));
}

const categoryIcons: Record<string, ReactNode> = {
  ai: <Bot aria-hidden="true" className="size-4 shrink-0" />,
  coding: <Code2 aria-hidden="true" className="size-4 shrink-0" />,
  data: <Database aria-hidden="true" className="size-4 shrink-0" />,
  design: <Palette aria-hidden="true" className="size-4 shrink-0" />,
  devops: <Server aria-hidden="true" className="size-4 shrink-0" />,
  research: <Microscope aria-hidden="true" className="size-4 shrink-0" />,
  security: <ShieldCheck aria-hidden="true" className="size-4 shrink-0" />,
  writing: <PenLine aria-hidden="true" className="size-4 shrink-0" />,
};

export function Sidebar({ skills, roots, view, onView, agentFilter, onAgentFilter, groupFilter, onGroupFilter, categories, categoryFilter, onCategoryFilter }: {
  skills: SkillState[];
  roots: Root[] | null;
  view: View;
  onView: (view: View) => void;
  agentFilter: Agent | null;
  onAgentFilter: (agent: Agent | null) => void;
  groupFilter: string | null;
  onGroupFilter: (key: string | null) => void;
  categories: CategoryOption[];
  categoryFilter: string | null;
  onCategoryFilter: (category: string | null) => void;
}) {
  const [collapsed, setCollapsed] = useState(false);
  const agents = roots === null ? [] : [...new Set(roots.map((root) => root.agent))];
  const missingCount = useMemo(() => skills.filter((skill) => roots !== null && isMissing(skill, roots)).length, [skills, roots]);
  const groups = useMemo(() => sourceGroups(skills), [skills]);

  return (
    <nav aria-label="Skill views" className={`flex shrink-0 flex-col border-r border-border bg-sidebar transition-[width] ${collapsed ? "w-14" : "w-14 md:w-60"}`}>
      <div className="flex items-center justify-center gap-2 px-0 pb-1 pt-4 md:justify-start md:px-4">
        <button
          type="button"
          onClick={() => setCollapsed((previous) => !previous)}
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          aria-pressed={collapsed}
          title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          className="shrink-0 rounded-md text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        >
          <PanelLeft aria-hidden="true" className="size-4" />
        </button>
        {!collapsed && <p className="hidden text-sm font-semibold tracking-tight md:inline">Skill Manager</p>}
      </div>
      <div className="flex-1 overflow-y-auto px-1.5 pb-4 md:px-2">
        {!collapsed && <SectionHeading>Skills</SectionHeading>}
        <div className="space-y-0.5">
          <NavItem label="All skills" active={view === "all" && agentFilter === null && groupFilter === null && categoryFilter === null} onClick={() => { onView("all"); onAgentFilter(null); onGroupFilter(null); onCategoryFilter(null); }} icon={<Layers aria-hidden="true" className="size-4 shrink-0" />} count={skills.length} />
          <NavItem label="Missing somewhere" active={view === "missing"} onClick={() => onView("missing")} icon={<TriangleAlert aria-hidden="true" className="size-4 shrink-0" />} count={missingCount} />
        </div>
        {agents.length > 0 && !collapsed && <CollapsibleSection title="Available in">
          <div className="mt-3 space-y-0.5 md:mt-0">
            {agents.map((agent) => (
              <NavItem key={agent} label={agentMeta[agent].label} active={view === "all" && agentFilter === agent}
                onClick={() => { onView("all"); onAgentFilter(agent); onGroupFilter(null); }}
                icon={<AgentIcon agent={agent} className="size-4" />}
                count={skills.filter((skill) => skill.presences.some((presence) => presence.agent === agent)).length} />
            ))}
          </div>
        </CollapsibleSection>}
        {!collapsed && <SectionHeading>Collections</SectionHeading>}
        <div className="mt-3 space-y-0.5 md:mt-0">
          {groups.map((group) => (
            <NavItem key={group.key} label={group.label} active={view === "all" && groupFilter === group.key}
              onClick={() => { onView("all"); onGroupFilter(groupFilter === group.key ? null : group.key); onAgentFilter(null); }}
              icon={group.key === "mine" ? <User aria-hidden="true" className="size-4 shrink-0" />
                : group.key.startsWith("plugin:") ? <Package aria-hidden="true" className="size-4 shrink-0" />
                : <AgentIcon agent={group.key.split(":")[2] as Agent} className="size-4" />}
              count={group.skills.length} />
          ))}
        </div>
        {categories.length > 0 && !collapsed && <>
          <SectionHeading>Categories</SectionHeading>
          <div className="mt-3 space-y-0.5 md:mt-0">
            {categories.map((category) => (
              <NavItem key={category.value} label={category.label} active={view === "all" && categoryFilter === category.value}
                onClick={() => { onView("all"); onCategoryFilter(categoryFilter === category.value ? null : category.value); onAgentFilter(null); onGroupFilter(null); }}
                icon={categoryIcons[category.value] ?? <Shapes aria-hidden="true" className="size-4 shrink-0" />}
                count={category.count} />
            ))}
          </div>
        </>}
      </div>
    </nav>
  );
}
