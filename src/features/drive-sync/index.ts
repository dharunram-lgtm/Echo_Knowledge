/**
 * Google Drive Sync Feature Module
 *
 * Coordinates optional encrypted sync between local ECHO repository
 * and user's Google Drive storage.
 */

export interface DriveSyncState {
  isEnabled: boolean;
  isAuthenticated: boolean;
  userEmail?: string;
  lastSyncTimestamp?: string;
  remoteFolderId?: string;
  syncStatus: "idle" | "syncing" | "paused" | "error";
  errorMessage?: string;
}

export const INITIAL_DRIVE_SYNC_STATE: DriveSyncState = {
  isEnabled: false,
  isAuthenticated: false,
  syncStatus: "idle",
};

