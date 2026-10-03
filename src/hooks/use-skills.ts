import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SkillState } from "@/views/all";

type ScanState = { skills: SkillState[]; config_error: string | null; config_notice: string | null };

export function useSkills() {
  const [skills, setSkills] = useState<SkillState[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [configError, setConfigError] = useState<string | null>(null);
  const [configNotice, setConfigNotice] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    let eventArrived = false;
    let unlisten: UnlistenFn | undefined;
    async function start() {
      try {
        unlisten = await listen<ScanState>("skills-changed", ({ payload }) => {
          if (!active) return;
          eventArrived = true;
          setSkills(payload.skills);
          setConfigError(payload.config_error);
          setConfigNotice(payload.config_notice);
          setError(null);
        });
        if (!active) { unlisten(); return; }
      } catch (reason) {
        console.error("Could not subscribe to skill updates:", reason);
      }
      try {
        const result = await invoke<ScanState>("scan_skills");
        if (active && !eventArrived) {
          setSkills(result.skills);
          setConfigError(result.config_error);
          setConfigNotice(result.config_notice);
          setError(null);
        }
      } catch (reason) {
        if (active && !eventArrived) setError(String(reason));
      }
    }
    void start();
    return () => { active = false; unlisten?.(); };
  }, []);

  return { skills, setSkills, error, configError, configNotice };
}
