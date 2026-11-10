import type { LucideIcon } from "lucide-react";

export type AppRoute =
  | "dashboard"
  | "my-knowledge"
  | "ask-echo"
  | "knowledge-graph"
  | "collections"
  | "activity"
  | "settings";

export interface NavigationItemConfig {
  id: AppRoute;
  label: string;
  path: string;
  icon: LucideIcon;
  badge?: string | number;
  description?: string;
}

