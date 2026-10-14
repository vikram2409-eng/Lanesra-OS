import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "./cn";

/** shadcn-style button variants (API mirrors vikram2409-eng/ui button). */
const buttonVariants = cva("ui-btn", {
  variants: {
    variant: {
      default: "ui-btn-default",
      secondary: "ui-btn-secondary",
      outline: "ui-btn-outline",
      ghost: "ui-btn-ghost",
      destructive: "ui-btn-destructive",
    },
    size: {
      default: "ui-btn-size-default",
      sm: "ui-btn-size-sm",
      lg: "ui-btn-size-lg",
      icon: "ui-btn-size-icon",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "default",
  },
});

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, type = "button", ...props }, ref) => (
    <button
      ref={ref}
      type={type}
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  ),
);
Button.displayName = "Button";

export { Button, buttonVariants };
