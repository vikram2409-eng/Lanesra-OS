import * as React from "react";

import { cn } from "./cn";

export interface CheckboxProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type"> {
  /** Controlled checked state. */
  checked?: boolean;
  /** Fires with the new boolean state (hand-rolled replacement for Radix onCheckedChange). */
  onCheckedChange?: (checked: boolean) => void;
  /** Optional visible label text rendered next to the box. */
  label?: React.ReactNode;
}

/**
 * Accessible checkbox built on a native input (no Radix). The input stays
 * in the tab order and drives the styled box via sibling selectors.
 */
const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, checked, defaultChecked, onCheckedChange, disabled, label, id, onChange, ...props }, ref) => {
    const autoId = React.useId();
    const inputId = id ?? autoId;
    return (
      <span className={cn("ui-checkbox-wrap", className)}>
        <input
          ref={ref}
          id={inputId}
          type="checkbox"
          className="ui-checkbox-input"
          checked={checked}
          defaultChecked={defaultChecked}
          disabled={disabled}
          onChange={(e) => {
            onCheckedChange?.(e.target.checked);
            onChange?.(e);
          }}
          {...props}
        />
        <span className="ui-checkbox-box" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3.5">
            <path d="M20 6 9 17l-5-5" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </span>
        {label != null && (
          <label htmlFor={inputId} className="ui-checkbox-label">
            {label}
          </label>
        )}
      </span>
    );
  },
);
Checkbox.displayName = "Checkbox";

export { Checkbox };
