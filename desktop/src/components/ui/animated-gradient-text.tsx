import * as React from "react";

import { cn } from "./cn";

export interface AnimatedGradientTextProps extends React.HTMLAttributes<HTMLSpanElement> {
  colorFrom?: string;
  colorTo?: string;
  /** Full sweep duration, e.g. "6s". */
  speed?: string;
}

/**
 * Magic UI AnimatedGradientText, re-implemented as pure CSS
 * (no framer-motion): an animated gradient clipped to the text.
 */
function AnimatedGradientText({
  className,
  children,
  colorFrom = "#8b5cf6",
  colorTo = "#635bff",
  speed = "6s",
  style,
  ...props
}: AnimatedGradientTextProps) {
  return (
    <span
      className={cn("ui-gradient-text", className)}
      style={
        {
          "--gradient-from": colorFrom,
          "--gradient-to": colorTo,
          "--gradient-speed": speed,
          ...style,
        } as React.CSSProperties
      }
      {...props}
    >
      {children}
    </span>
  );
}

export { AnimatedGradientText };
