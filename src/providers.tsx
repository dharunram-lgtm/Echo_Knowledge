import type { ReactNode } from "react";
import { ThemeProvider } from "@/hooks/useTheme";

interface ProvidersProps {
  children: ReactNode;
}

export function Providers({ children }: ProvidersProps) {
  return (
    <ThemeProvider>
      {children}
    </ThemeProvider>
  );
}

export default Providers;

