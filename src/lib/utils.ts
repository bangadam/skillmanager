import type { Presence } from "@/views/all";
import type { Root } from "@/views/missing";

// Mirror of core::covers — a presence covers a root when it sits inside the root and is named for the skill (flat or nested group folder).
export function covers(root: Root, id: string, presence: Presence): boolean {
  return presence.path.startsWith(`${root.path}/`) && presence.path.endsWith(`/${id}`);
}


export function formatDate(millis: number | null, withYear = false): string {
  if (millis === null) return "—"
  const options: Intl.DateTimeFormatOptions = withYear
    ? { year: "numeric", month: "short", day: "numeric" }
    : { month: "short", day: "numeric" }
  return new Date(millis).toLocaleDateString(undefined, options)
}
