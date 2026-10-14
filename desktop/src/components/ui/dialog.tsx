import * as React from "react";
import { createPortal } from "react-dom";

import { cn } from "./cn";

interface DialogContextValue {
  close: () => void;
  titleId: string;
  descriptionId: string;
}

const DialogContext = React.createContext<DialogContextValue | null>(null);

function useDialogContext(component: string): DialogContextValue {
  const ctx = React.useContext(DialogContext);
  if (!ctx) throw new Error(`${component} must be used inside <DialogContent>`);
  return ctx;
}

export interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: React.ReactNode;
}

/**
 * Accessible modal dialog, hand-rolled (no Radix). Features: Escape to
 * close, overlay click to close, body scroll lock, initial focus, and a
 * Tab focus trap while open.
 */
function Dialog({ open, onOpenChange, children }: DialogProps) {
  const close = React.useCallback(() => onOpenChange(false), [onOpenChange]);
  const titleId = React.useId();
  const descriptionId = React.useId();

  React.useEffect(() => {
    if (!open) return;
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = prevOverflow;
    };
  }, [open ]);

  if (!open) return null;
  return createPortal(
    <DialogContext.Provider value={{ close, titleId, descriptionId }}>
      {children}
    </DialogContext.Provider>,
    document.body,
  );
}

const FOCUSABLE =
  'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';

function DialogContent({
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  const { close, titleId, descriptionId } = useDialogContext("DialogContent");
  const contentRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    const el = contentRef.current;
    if (!el) return;
    const target =
      el.querySelector<HTMLElement>("[data-autofocus]") ??
      el.querySelector<HTMLElement>(FOCUSABLE) ??
      el;
    target.focus();
  }, []);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      close();
      return;
    }
    if (e.key !== "Tab") return;
    const el = contentRef.current;
    if (!el) return;
    const items = Array.from(el.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
      (i) => !i.hasAttribute("disabled") && i.getAttribute("aria-hidden") !== "true",
    );
    if (items.length === 0) {
      e.preventDefault();
      return;
    }
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  };

  return (
    <div className="ui-dialog-root">
      <div className="ui-dialog-overlay" onClick={close} aria-hidden="true" />
      <div
        ref={contentRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descriptionId}
        tabIndex={-1}
        onKeyDown={onKeyDown}
        className={cn("ui-dialog-content", className)}
        {...props}
      >
        {children}
        <button
          type="button"
          className="ui-dialog-x"
          onClick={close}
          aria-label="Close dialog"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
            <path d="M18 6 6 18M6 6l12 12" strokeLinecap="round" />
          </svg>
        </button>
      </div>
    </div>
  );
}

function DialogHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("ui-dialog-header", className)} {...props} />;
}

function DialogTitle({ className, ...props }: React.HTMLAttributes<HTMLHeadingElement>) {
  const { titleId } = useDialogContext("DialogTitle");
  return <h2 id={titleId} className={cn("ui-dialog-title", className)} {...props} />;
}

function DialogDescription({ className, ...props }: React.HTMLAttributes<HTMLParagraphElement>) {
  const { descriptionId } = useDialogContext("DialogDescription");
  return <p id={descriptionId} className={cn("ui-dialog-description", className)} {...props} />;
}

function DialogFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("ui-dialog-footer", className)} {...props} />;
}

function DialogClose({
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  const { close } = useDialogContext("DialogClose");
  return (
    <button
      type="button"
      className={cn(className)}
      onClick={(e) => {
        close();
        props.onClick?.(e);
      }}
      {...props}
    />
  );
}

export {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  DialogClose,
};
