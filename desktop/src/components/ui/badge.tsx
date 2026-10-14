import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "./cn";

/** shadcn-style badge. Adds success/warning to the standard set. */
const badgeVariants = cva("ui-badge", {
  variants: {
    variant: {
      default: "ui-badge-default",
      secondary: "ui-badge-secondary",
      outline: "ui-badge-outline",
      success: "ui-badge-success",
      warning: "ui-badge-warning",
      destructive: "ui-badge-destructive",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

export interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return <span className={cn(badgeVariants({ variant }), className)} {...props} />;
}

export { Badge, badgeVariants };
