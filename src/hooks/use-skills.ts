import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SkillState } from "@/views/all";
import type { Root } from "@/views/missing";

type ScanState = { skills: SkillState[]; roots: Root[]; config_error: string | null; config_notice: string | null };

export function useSkills() {
  const [skills, setSkills] = useState<SkillState[] | null>(null);
  const [roots, setRoots] = useState<Root[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [configError, setConfigError] = useState<string | null>(null);
  const [configNotice, setConfigNotice] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState<number | null>(null);

  const apply = (state: ScanState) => {
    setSkills(state.skills);
    setRoots(state.roots);
    setConfigError(state.config_error);
    setConfigNotice(state.config_notice);
    setUpdatedAt(Date.now());
  };

  useEffect(() => {
    let active = true;
    let eventArrived = false;
    let unlisten: UnlistenFn | undefined;
    async function start() {
      try {
        unlisten = await listen<ScanState>("skills-changed", ({ payload }) => {
          if (!active) return;
          eventArrived = true;
          apply(payload);
          setError(null);
        });
        if (!active) { unlisten(); return; }
      } catch (reason) {
        console.error("Could not subscribe to skill updates:", reason);
      }
      try {
        const result = await invoke<ScanState>("scan_skills");
        if (active && !eventArrived) {
          apply(result);
          setError(null);
        }
      } catch (reason) {
        if (active && !eventArrived) setError(String(reason));
      }
    }
    void start();
    return () => { active = false; unlisten?.(); };
  }, []);

  const refresh = useCallback(async () => {
    try {
      apply(await invoke<ScanState>("scan_skills"));
      setError(null);
    } catch (reason) {
      setError(String(reason));
    }
  }, []);

  return { skills, setSkills, roots, error, configError, configNotice, updatedAt, refresh };
}
