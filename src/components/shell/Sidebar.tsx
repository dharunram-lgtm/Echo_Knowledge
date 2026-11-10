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
  return (
    <aside
      className={`h-full flex flex-col shrink-0 border-r border-[var(--border-subtle)] bg-[var(--bg-sidebar)] transition-all duration-200 select-none ${
        isCollapsed ? "w-14" : "w-60"
      }`}
    >
      {/* Brand Header */}
      <div className="flex items-center justify-between h-12 px-3 border-b border-[var(--border-subtle)]">
        <div className="flex items-center gap-2 overflow-hidden">
          <div className="flex items-center justify-center w-7 h-7 rounded-[var(--radius-md)] bg-[var(--bg-elevated)] border border-[var(--border-default)] shadow-[var(--shadow-rest)] text-[var(--accent-primary)]">
            <Cpu className="w-4 h-4" />
          </div>
          {!isCollapsed && (
            <div className="flex flex-col min-w-0">
              <span className="text-xs font-semibold tracking-wider text-[var(--text-primary)] uppercase">
                ECHO
              </span>
              <span className="text-[10px] text-[var(--text-tertiary)] truncate">
                AI Knowledge OS
              </span>
            </div>
          )}
        </div>
        <button
          type="button"
          onClick={onToggleCollapse}
          className="fluent-button-subtle p-1 rounded-[var(--radius-sm)] cursor-pointer"
          title={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
          aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {isCollapsed ? (
            <PanelLeft className="w-4 h-4" />
          ) : (
            <PanelLeftClose className="w-4 h-4" />
          )}
        </button>
      </div>

      {/* Primary Navigation List */}
      <div className="flex-1 py-3 px-2 space-y-1 overflow-y-auto">
        {!isCollapsed && (
          <div className="px-2 pb-1 text-[10px] font-semibold tracking-wider text-[var(--text-tertiary)] uppercase">
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
      </div>

      {/* Bottom Navigation / Settings */}
      <div className="p-2 border-t border-[var(--border-subtle)] space-y-1">
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

