import React from "react";
import { cn } from "@/utils";

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "primary" | "subtle";
  size?: "sm" | "md" | "lg";
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "default", size = "md", children, ...props }, ref) => {
    const variantClass =
      variant === "primary"
        ? "fluent-button-primary"
        : variant === "subtle"
        ? "fluent-button-subtle"
        : "";

    const sizeClass =
      size === "sm"
        ? "px-2 py-1 text-xs"
        : size === "lg"
        ? "px-4 py-2 text-base"
        : "px-3 py-1.5 text-xs";

    return (
      <button
        ref={ref}
        className={cn("fluent-button", variantClass, sizeClass, className)}
        {...props}
      >
        {children}
      </button>
    );
  }
);

Button.displayName = "Button";

