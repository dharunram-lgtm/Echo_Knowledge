/**
 * ECHO Application Lifecycle & Environment
 */

export interface AppConfig {
  appName: string;
  version: string;
  isDesktop: boolean;
  defaultTheme: "light" | "dark" | "system";
}

export const APP_CONFIG: AppConfig = {
  appName: "ECHO Knowledge",
  version: "1.1.0",
  isDesktop: typeof window !== "undefined" && "__TAURI_INTERNALS__" in window,
  defaultTheme: "dark",
};

