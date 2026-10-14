import * as React from "react";

import { cn } from "./cn";

const Input = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, type = "text", ...props }, ref) => (
    <input ref={ref} type={type} className={cn("ui-input", className)} {...props} />
  ),
);
Input.displayName = "Input";

export { Input };
