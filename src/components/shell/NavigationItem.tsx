import React from "react";
import type { NavigationItemConfig } from "@/types/navigation";

interface NavigationItemProps {
  item: NavigationItemConfig;
  isActive: boolean;
  isCollapsed?: boolean;
  onClick: () => void;
}

export const NavigationItem: React.FC<NavigationItemProps> = ({
  item,
  isActive,
  isCollapsed = false,
  onClick,
}) => {
  const Icon = item.icon;

  return (
    <button
      type="button"
      onClick={onClick}
      className={`fluent-nav-item w-full text-left transition-all ${
        isActive ? "active font-medium" : "text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
      }`}
      title={isCollapsed ? item.label : undefined}
      aria-label={item.label}
    >
      <Icon className="h-4 w-4 shrink-0" strokeWidth={2} />
      {!isCollapsed && (
        <span className="flex-1 truncate text-xs tracking-normal">
          {item.label}
        </span>
      )}
      {!isCollapsed && item.badge !== undefined && (
        <span className="fluent-badge ml-auto text-[10px] py-0 px-1.5">
          {item.badge}
        </span>
      )}
    </button>
  );
};

export default NavigationItem;

