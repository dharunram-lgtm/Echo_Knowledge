import React, { useState } from "react";
import type { AppRoute } from "@/types/navigation";
import { AppShell } from "@/components/shell/AppShell";
import { Dashboard } from "@/pages/Dashboard";
import { MyKnowledge } from "@/pages/MyKnowledge";
import { AskEcho } from "@/pages/AskEcho";
import { KnowledgeGraph } from "@/pages/KnowledgeGraph";
import { Collections } from "@/pages/Collections";
import { Activity } from "@/pages/Activity";
import { Settings } from "@/pages/Settings";

export interface NavigationOptions {
  previewDocumentId?: string;
}

export interface RouterContextType {
  currentRoute: AppRoute;
  navigate: (route: AppRoute, options?: NavigationOptions) => void;
  previewDocumentId: string | null;
  clearPreviewDocumentId: () => void;
}

export const RouterContext = React.createContext<RouterContextType | null>(null);

export function useRouter(): RouterContextType {
  const ctx = React.useContext(RouterContext);
  if (!ctx) {
    throw new Error("useRouter must be used within a RouterProvider");
  }
  return ctx;
}

export const AppRouter: React.FC = () => {
  const [currentRoute, setCurrentRoute] = useState<AppRoute>("dashboard");
  const [previewDocumentId, setPreviewDocumentId] = useState<string | null>(null);

  const navigate = React.useCallback((route: AppRoute, options?: NavigationOptions) => {
    setCurrentRoute(route);
    if (options?.previewDocumentId) {
      setPreviewDocumentId(options.previewDocumentId);
    }
  }, []);

  const clearPreviewDocumentId = React.useCallback(() => {
    setPreviewDocumentId(null);
  }, []);

  const renderPage = () => {
    switch (currentRoute) {
      case "dashboard":
        return <Dashboard />;
      case "my-knowledge":
        return <MyKnowledge />;
      case "ask-echo":
        return <AskEcho />;
      case "knowledge-graph":
        return <KnowledgeGraph />;
      case "collections":
        return <Collections />;
      case "activity":
        return <Activity />;
      case "settings":
        return <Settings />;
      default:
        return <Dashboard />;
    }
  };

  return (
        <RouterContext.Provider
          value={{
            currentRoute,
            navigate,
            previewDocumentId,
            clearPreviewDocumentId,
          }}
        >
          <AppShell currentRoute={currentRoute} onNavigate={navigate}>
            {renderPage()}
          </AppShell>
        </RouterContext.Provider>
        );
};

        export default AppRouter;

