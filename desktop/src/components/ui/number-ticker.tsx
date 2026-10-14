import * as React from "react";

import { cn } from "./cn";

export interface NumberTickerProps extends React.HTMLAttributes<HTMLSpanElement> {
  value: number;
  /** Animation duration in ms. */
  duration?: number;
  /** Delay before starting, in ms. */
  delay?: number;
  decimalPlaces?: number;
  prefix?: string;
  suffix?: string;
}

/**
 * Magic UI NumberTicker, re-implemented with requestAnimationFrame
 * (no framer-motion). Counts from 0 to `value` with ease-out cubic.
 */
function NumberTicker({
  className,
  value,
  duration = 1000,
  delay = 0,
  decimalPlaces = 0,
  prefix = "",
  suffix = "",
  ...props
}: NumberTickerProps) {
  const [display, setDisplay] = React.useState(0);

  React.useEffect(() => {
    let raf = 0;
    let start: number | null = null;
    const timeout = window.setTimeout(() => {
      const step = (t: number) => {
        if (start === null) start = t;
        const p = Math.min((t - start) / Math.max(duration, 1), 1);
        const eased = 1 - Math.pow(1 - p, 3);
        setDisplay(value * eased);
        if (p < 1) raf = requestAnimationFrame(step);
        else setDisplay(value);
      };
      raf = requestAnimationFrame(step);
    }, delay);
    return () => {
      window.clearTimeout(timeout);
      cancelAnimationFrame(raf);
    };
  }, [value, duration, delay]);

  const formatted = display.toLocaleString(undefined, {
    minimumFractionDigits: decimalPlaces,
    maximumFractionDigits: decimalPlaces,
  });

  return (
    <span className={cn("ui-number-ticker", className)} {...props}>
      {prefix}
      {formatted}
      {suffix}
    </span>
  );
}

export { NumberTicker };
