import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import type { ThemeMode, ThemeContextValue } from "@/types/theme";

const ThemeContext = createContext<ThemeContextValue | null>(null);

const STORAGE_KEY = "echo_ui_theme";

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<ThemeMode>(() => {
    const saved = localStorage.getItem(STORAGE_KEY) as ThemeMode | null;
    return saved ?? "dark";
  });

  const [resolvedTheme, setResolvedTheme] = useState<"light" | "dark">("dark");

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");

    const computeResolved = (mode: ThemeMode): "light" | "dark" => {
      if (mode === "system") {
        return media.matches ? "dark" : "light";
      }
      return mode;
    };

    const nextResolved = computeResolved(theme);
    setResolvedTheme(nextResolved);

    const root = document.documentElement;
    root.setAttribute("data-theme", nextResolved);
    if (nextResolved === "dark") {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }

    const listener = () => {
      if (theme === "system") {
        const auto = media.matches ? "dark" : "light";
        setResolvedTheme(auto);
        root.setAttribute("data-theme", auto);
        if (auto === "dark") {
          root.classList.add("dark");
        } else {
          root.classList.remove("dark");
        }
      }
    };

    media.addEventListener("change", listener);
    return () => media.removeEventListener("change", listener);
  }, [theme]);

  const setTheme = (next: ThemeMode) => {
    setThemeState(next);
    localStorage.setItem(STORAGE_KEY, next);
  };

  const toggleTheme = () => {
    setTheme(resolvedTheme === "dark" ? "light" : "dark");
  };

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme, setTheme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return context;
}
