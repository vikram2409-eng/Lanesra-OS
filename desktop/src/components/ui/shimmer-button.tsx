import * as React from "react";

import { cn } from "./cn";

export interface ShimmerButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  shimmerColor?: string;
  shimmerSize?: string;
  shimmerDuration?: string;
  background?: string;
  borderRadius?: string;
}

/**
 * Magic UI ShimmerButton, re-implemented as pure CSS (no framer-motion).
 * A shimmer sweep crosses the button on a loop. Respects the "subtle"
 * motion budget via the global prefers-reduced-motion rule.
 */
const ShimmerButton = React.forwardRef<HTMLButtonElement, ShimmerButtonProps>(
  (
    {
      className,
      children,
      shimmerColor = "#ffffff",
      shimmerSize = "0.12em",
      shimmerDuration = "2.75s",
      background = "var(--primary)",
      borderRadius = "var(--radius-md)",
      style,
      ...props
    },
    ref,
  ) => (
    <button
      ref={ref}
      type="button"
      className={cn("ui-shimmer-btn", className)}
      style={
        {
          "--shimmer-color": shimmerColor,
          "--shimmer-size": shimmerSize,
          "--shimmer-duration": shimmerDuration,
          "--shimmer-bg": background,
          "--shimmer-radius": borderRadius,
          ...style,
        } as React.CSSProperties
      }
      {...props}
    >
      <span className="ui-shimmer-btn-inner">{children}</span>
    </button>
  ),
);
ShimmerButton.displayName = "ShimmerButton";

export { ShimmerButton };
