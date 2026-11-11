import React from "react";
import {
  LayoutDashboard,
  Files,
  Sparkles,
  Network,
  FolderTree,
  Activity as ActivityIcon,
  Settings as SettingsIcon,
  PanelLeftClose,
  PanelLeft,
  Cpu,
} from "lucide-react";
import type { AppRoute, NavigationItemConfig } from "@/types/navigation";
import { NavigationItem } from "./NavigationItem";

interface SidebarProps {
  currentRoute: AppRoute;
  onNavigate: (route: AppRoute) => void;
  isCollapsed: boolean;
  onToggleCollapse: () => void;
}

const PRIMARY_NAV_ITEMS: NavigationItemConfig[] = [
  {
    id: "dashboard",
    label: "Dashboard",
    path: "/",
    icon: LayoutDashboard,
  },
  {
    id: "my-knowledge",
    label: "My Knowledge",
    path: "/my-knowledge",
    icon: Files,
  },
  {
    id: "ask-echo",
    label: "Ask ECHO",
    path: "/ask-echo",
    icon: Sparkles,
  },
  {
    id: "knowledge-graph",
    label: "Knowledge Graph",
    path: "/knowledge-graph",
    icon: Network,
  },
  {
    id: "collections",
    label: "Collections",
    path: "/collections",
    icon: FolderTree,
  },
  {
    id: "activity",
    label: "Activity",
    path: "/activity",
    icon: ActivityIcon,
  },
];

const SETTINGS_NAV_ITEM: NavigationItemConfig = {
  id: "settings",
  label: "Settings",
  path: "/settings",
  icon: SettingsIcon,
};

export const Sidebar: React.FC<SidebarProps> = ({
  currentRoute,
  onNavigate,
  isCollapsed,
  onToggleCollapse,
}) => {
  const collapseButton = (
    <button
      type="button"
      onClick={onToggleCollapse}
      className="fluent-button-subtle inline-flex shrink-0 cursor-pointer items-center justify-center rounded-[var(--radius-sm)] p-1"
      title={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
      aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
      aria-expanded={!isCollapsed}
      aria-controls="app-sidebar"
    >
      {isCollapsed ? (
        <PanelLeft className="h-4 w-4" />
      ) : (
        <PanelLeftClose className="h-4 w-4" />
      )}
    </button>
  );

  return (
    <aside
      id="app-sidebar"
      className={`flex h-full shrink-0 select-none flex-col overflow-hidden border-r border-[var(--border-subtle)] bg-[var(--bg-sidebar)] transition-[width] duration-200 ease-in-out ${
        isCollapsed ? "w-16" : "w-60"
      }`}
    >
      {/* Brand Header */}
      <div
        className={`shrink-0 border-b border-[var(--border-subtle)] ${
          isCollapsed
            ? "flex flex-col items-center gap-1.5 py-2"
            : "flex h-12 items-center px-3"
        }`}
      >
        {isCollapsed ? (
          <>
            <div
              className="flex h-7 w-7 items-center justify-center rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-elevated)] text-[var(--accent-primary)] shadow-[var(--shadow-rest)]"
              title="ECHO - AI Knowledge OS"
              aria-hidden="true"
            >
              <Cpu className="h-4 w-4" />
            </div>
            {collapseButton}
          </>
        ) : (
          <>
            <div className="flex min-w-0 items-center gap-2">
              <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-elevated)] text-[var(--accent-primary)] shadow-[var(--shadow-rest)]">
                <Cpu className="h-4 w-4" />
              </div>
              <div className="flex min-w-0 flex-col">
                <span className="text-xs font-semibold uppercase tracking-wider text-[var(--text-primary)]">
                  ECHO
                </span>
                <span className="truncate text-[10px] text-[var(--text-tertiary)]">
                  AI Knowledge OS
                </span>
              </div>
            </div>
            <div className="ml-auto pl-2">{collapseButton}</div>
          </>
        )}
      </div>

      {/* Primary Navigation List */}
      <nav
        aria-label="Primary navigation"
        className="min-h-0 flex-1 space-y-1 overflow-x-hidden overflow-y-auto px-2 py-3"
      >
        {!isCollapsed && (
          <div className="px-2 pb-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]">
            Navigation
          </div>
        )}
        {PRIMARY_NAV_ITEMS.map((item) => (
          <NavigationItem
            key={item.id}
            item={item}
            isActive={currentRoute === item.id}
            isCollapsed={isCollapsed}
            onClick={() => onNavigate(item.id)}
          />
        ))}
      </nav>

      {/* Bottom Navigation / Settings */}
      <div className="shrink-0 space-y-1 border-t border-[var(--border-subtle)] p-2">
        <NavigationItem
          item={SETTINGS_NAV_ITEM}
          isActive={currentRoute === "settings"}
          isCollapsed={isCollapsed}
          onClick={() => onNavigate("settings")}
        />
      </div>
    </aside>
  );
};

export default Sidebar;
