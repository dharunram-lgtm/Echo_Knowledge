import React, { useState } from "react";
import type { AppRoute } from "@/types/navigation";
import { Sidebar } from "./Sidebar";
import { TitleBar } from "./TitleBar";
import { StatusBar } from "./StatusBar";

interface AppShellProps {
  currentRoute: AppRoute;
  onNavigate: (route: AppRoute) => void;
  children: React.ReactNode;
}

const ROUTE_TITLES: Record<AppRoute, string> = {
  dashboard: "Dashboard",
  "my-knowledge": "My Knowledge",
  "ask-echo": "Ask ECHO",
  "knowledge-graph": "Knowledge Graph",
  collections: "Collections",
  activity: "Activity",
  settings: "Settings",
};

export const AppShell: React.FC<AppShellProps> = ({
  currentRoute,
  onNavigate,
  children,
}) => {
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[var(--bg-app)] text-[var(--text-primary)]">
      {/* Sidebar Navigation */}
      <Sidebar
        currentRoute={currentRoute}
        onNavigate={onNavigate}
        isCollapsed={isSidebarCollapsed}
        onToggleCollapse={() => setIsSidebarCollapsed((prev) => !prev)}
      />

      {/* Main App Stage */}
      <div className="flex flex-col flex-1 min-w-0 h-full overflow-hidden">
        {/* TitleBar / In-app Header */}
        <TitleBar
          currentRoute={currentRoute}
          title={ROUTE_TITLES[currentRoute] ?? "ECHO"}
        />

        {/* Scrollable Page Viewport */}
        <main className="flex-1 overflow-y-auto overflow-x-hidden bg-[var(--bg-app)]">
          {children}
        </main>

        {/* Persistent Bottom Status Bar */}
        <StatusBar />
      </div>
    </div>
  );
};

export default AppShell;

