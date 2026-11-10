import React from "react";
import { cn } from "@/utils";

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: "neutral" | "info" | "success" | "warning";
}

export const Badge: React.FC<BadgeProps> = ({
  className,
  children,
  ...props
}) => {
  return (
    <span className={cn("fluent-badge", className)} {...props}>
      {children}
    </span>
  );
};

