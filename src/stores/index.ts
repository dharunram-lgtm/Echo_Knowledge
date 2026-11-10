/**
 * ECHO State Stores
 *
 * Central export for client-side reactive state management.
 */

export interface AppState {
  isInitialized: boolean;
  activeRoute: string;
  isSidebarCollapsed: boolean;
}

export const INITIAL_APP_STATE: AppState = {
  isInitialized: true,
  activeRoute: "dashboard",
  isSidebarCollapsed: false,
};

