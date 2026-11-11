import React, { useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { NavigationItemConfig } from "@/types/navigation";

interface NavigationItemProps {
  item: NavigationItemConfig;
  isActive: boolean;
  isCollapsed?: boolean;
  onClick: () => void;
}

interface TooltipPosition {
  top: number;
  left: number;
}

export const NavigationItem: React.FC<NavigationItemProps> = ({
  item,
  isActive,
  isCollapsed = false,
  onClick,
}) => {
  const Icon = item.icon;
  const buttonRef = useRef<HTMLButtonElement>(null);
  const [tooltipPosition, setTooltipPosition] = useState<TooltipPosition | null>(
    null,
  );
  const tooltipId = useId();

  const hideTooltip = () => setTooltipPosition(null);

  const showTooltip = () => {
    if (!isCollapsed || !buttonRef.current) return;
    const rect = buttonRef.current.getBoundingClientRect();
    setTooltipPosition({
      top: rect.top + rect.height / 2,
      left: rect.right + 10,
    });
  };

  useEffect(() => {
    if (!isCollapsed) {
      hideTooltip();
    }
  }, [isCollapsed]);

  useEffect(() => {
    if (!tooltipPosition) return;

    const hide = () => hideTooltip();
    window.addEventListener("resize", hide);
    window.addEventListener("scroll", hide, true);

    return () => {
      window.removeEventListener("resize", hide);
      window.removeEventListener("scroll", hide, true);
    };
  }, [tooltipPosition]);

  return (
    <>
      <button
        ref={buttonRef}
        type="button"
        onClick={() => {
          hideTooltip();
          onClick();
        }}
        onMouseEnter={showTooltip}
        onMouseLeave={hideTooltip}
        onFocus={showTooltip}
        onBlur={hideTooltip}
        aria-label={item.label}
        aria-current={isActive ? "page" : undefined}
        aria-describedby={
          isCollapsed && tooltipPosition ? tooltipId : undefined
        }
        className={`fluent-nav-item w-full transition-all ${
          isCollapsed ? "fluent-nav-item--collapsed" : "text-left"
        } ${
          isActive
            ? "active font-medium"
            : "text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
        }`}
      >
        <Icon className="h-4 w-4 shrink-0" strokeWidth={2} />
        {!isCollapsed && (
          <span className="flex-1 truncate text-xs tracking-normal">
            {item.label}
          </span>
        )}
        {!isCollapsed && item.badge !== undefined && (
          <span className="fluent-badge ml-auto px-1.5 py-0 text-[10px]">
            {item.badge}
          </span>
        )}
      </button>

      {isCollapsed &&
        tooltipPosition &&
        createPortal(
          <div
            id={tooltipId}
            role="tooltip"
            className="pointer-events-none fixed z-[100] -translate-y-1/2 whitespace-nowrap rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-elevated)] px-2.5 py-1.5 text-[11px] font-medium text-[var(--text-primary)] shadow-[var(--shadow-flyout)]"
            style={{
              top: tooltipPosition.top,
              left: tooltipPosition.left,
            }}
          >
            {item.label}
          </div>,
          document.body,
        )}
    </>
  );
};

export default NavigationItem;
