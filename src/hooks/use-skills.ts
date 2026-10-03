import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SkillState } from "@/views/all";

export function useSkills() {
  const [skills, setSkills] = useState<SkillState[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    let eventArrived = false;
    let unlisten: UnlistenFn | undefined;
    async function start() {
      try {
        unlisten = await listen<SkillState[]>("skills-changed", ({ payload }) => {
          if (!active) return;
          eventArrived = true;
          setSkills(payload);
          setError(null);
        });
        if (!active) { unlisten(); return; }
      } catch (reason) {
        console.error("Could not subscribe to skill updates:", reason);
      }
      try {
        const result = await invoke<SkillState[]>("scan_skills");
        if (active && !eventArrived) { setSkills(result); setError(null); }
      } catch (reason) {
        if (active && !eventArrived) setError(String(reason));
      }
    }
    void start();
    return () => { active = false; unlisten?.(); };
  }, []);

  return { skills, error };
}
