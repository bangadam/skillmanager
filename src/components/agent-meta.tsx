import { Asterisk, Pi, Send, Sparkles, SquareTerminal, type LucideIcon } from "lucide-react";
import { cn } from "cn";
import type { Agent } from "@/views/all";

export type AgentMeta = { label: string; short: string; icon: LucideIcon; iconClass: string; chipClass: string };

export const agentMeta: Record<Agent, AgentMeta> = {
  claude_code: {
    label: "Claude Code", short: "CC", icon: Asterisk,
    iconClass: "text-orange-600 dark:text-orange-400",
    chipClass: "bg-orange-500/10 text-orange-700 dark:bg-orange-500/15 dark:text-orange-300",
  },
  codex: {
    label: "Codex", short: "CDX", icon: SquareTerminal,
    iconClass: "text-cyan-600 dark:text-cyan-400",
    chipClass: "bg-cyan-500/10 text-cyan-700 dark:bg-cyan-500/15 dark:text-cyan-300",
  },
  cursor: {
    label: "Cursor", short: "CUR", icon: Sparkles,
    iconClass: "text-violet-600 dark:text-violet-400",
    chipClass: "bg-violet-500/10 text-violet-700 dark:bg-violet-500/15 dark:text-violet-300",
  },
  omp: {
    label: "omp", short: "OMP", icon: Pi,
    iconClass: "text-emerald-600 dark:text-emerald-400",
    chipClass: "bg-emerald-500/10 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300",
  },
  hermes: {
    label: "Hermes", short: "HRM", icon: Send,
    iconClass: "text-rose-600 dark:text-rose-400",
    chipClass: "bg-rose-500/10 text-rose-700 dark:bg-rose-500/15 dark:text-rose-300",
  },
};

export function AgentIcon({ agent, className }: { agent: Agent; className?: string }) {
  const meta = agentMeta[agent];
  return <meta.icon aria-hidden="true" className={cn("size-3.5 shrink-0", meta.iconClass, className)} />;
}

export function AgentChip({ agent, className }: { agent: Agent; className?: string }) {
  const meta = agentMeta[agent];
  return (
    <span className={cn("inline-flex shrink-0 items-center gap-1.5 rounded-md px-2 py-0.5 text-xs font-medium", meta.chipClass, className)}>
      <meta.icon aria-hidden="true" className="size-3" />
      {meta.label}
    </span>
  );
}
