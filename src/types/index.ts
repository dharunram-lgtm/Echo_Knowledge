export * from "./navigation";
export * from "./theme";
export * from "./documents";
export * from "./graph";
export * from "./retry";

export interface SystemStatus {
  online: boolean;
  indexedDocumentsCount: number;
  activeProvider: string | null;
  syncStatus: "idle" | "syncing" | "paused" | "error";
  lastSyncedAt?: string;
}

export interface TauriAppInfo {
  name: string;
  version: string;
  phase: string;
  platform: string;
}
