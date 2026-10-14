import * as React from "react";

import { cn } from "./cn";

export interface TooltipProps extends React.HTMLAttributes<HTMLSpanElement> {
  /** Tooltip text. Rendered via CSS attr() — keep it short plain text. */
  content: string;
  side?: "top" | "bottom" | "left" | "right";
}

/**
 * CSS-only tooltip (no JS positioning). Shows on hover and keyboard focus.
 * For rich/interactive content use Dialog or a popover instead.
 */
function Tooltip({ content, side = "top", className, children, ...props }: TooltipProps) {
  return (
    <span
      className={cn("ui-tooltip", `ui-tooltip-${side}`, className)}
      data-tip={content}
      tabIndex={0}
      {...props}
    >
      {children}
    </span>
  );
}

export { Tooltip };
