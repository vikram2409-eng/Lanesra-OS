import * as React from "react";

import { cn } from "./cn";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps
  extends Omit<React.SelectHTMLAttributes<HTMLSelectElement>, "children"> {
  options: SelectOption[];
  /** Fires with the new value (mirrors the Radix Select onValueChange API). */
  onValueChange?: (value: string) => void;
  placeholder?: string;
}

/** Styled native select (no Radix). Keeps full keyboard/screen-reader behavior. */
const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, options, onValueChange, placeholder, value, defaultValue, onChange, ...props }, ref) => {
    return (
      <span className={cn("ui-select", className)}>
        <select
          ref={ref}
          className="ui-select-input"
          value={value}
          defaultValue={defaultValue}
          onChange={(e) => {
            onValueChange?.(e.target.value);
            onChange?.(e);
          }}
          {...props}
        >
          {placeholder != null && (
            <option value="" disabled hidden={value !== ""}>
              {placeholder}
            </option>
          )}
          {options.map((o) => (
            <option key={o.value} value={o.value} disabled={o.disabled}>
              {o.label}
            </option>
          ))}
        </select>
        <svg className="ui-select-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
          <path d="m6 9 6 6 6-6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </span>
    );
  },
);
Select.displayName = "Select";

export { Select };
