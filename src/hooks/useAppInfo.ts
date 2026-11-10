import { useState, useEffect } from "react";
import type { TauriAppInfo } from "@/types";

export function useAppInfo(): TauriAppInfo {
  const [info, setInfo] = useState<TauriAppInfo>({
    name: "ECHO",
    version: "0.1.0",
    phase: "Phase 1: Project Foundation",
    platform: "Windows",
  });

  useEffect(() => {
    async function fetchInfo() {
      try {
        // Dynamic import to prevent errors in non-Tauri preview environments
        const { invoke } = await import("@tauri-apps/api/core");
        const backendInfo = await invoke<TauriAppInfo>("get_app_info");
        if (backendInfo) {
          setInfo(backendInfo);
        }
      } catch {
        // Fallback to default in web preview
      }
    }

    fetchInfo();
  }, []);

  return info;
}

