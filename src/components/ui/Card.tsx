import React from "react";
import { cn } from "@/utils";

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {}

export const Card = React.forwardRef<HTMLDivElement, CardProps>(
  ({ className, children, ...props }, ref) => {
    return (
      <div ref={ref} className={cn("fluent-card", className)} {...props}>
        {children}
      </div>
    );
  }
);

Card.displayName = "Card";

