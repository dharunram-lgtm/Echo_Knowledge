import React from "react";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/utils";

export interface PlaceholderViewProps {
  icon: LucideIcon;
  title: string;
  description: string;
  phaseNote?: string;
  className?: string;
}

export const PlaceholderView: React.FC<PlaceholderViewProps> = ({
  icon: Icon,
  title,
  description,
  phaseNote,
  className,
}) => {
  return (
    <div
      className={cn(
        "fluent-card p-10 flex flex-col items-center justify-center text-center space-y-3",
        className
      )}
    >
      <div className="w-12 h-12 rounded-[var(--radius-xl)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] flex items-center justify-center text-[var(--accent-primary)]">
        <Icon className="w-6 h-6" />
      </div>
      <h3 className="text-sm font-semibold text-[var(--text-primary)]">{title}</h3>
      <p className="text-xs text-[var(--text-secondary)] max-w-md">{description}</p>
      {phaseNote && (
        <span className="fluent-badge text-[10px] mt-2 font-mono">{phaseNote}</span>
      )}
    </div>
  );
};

export default PlaceholderView;

