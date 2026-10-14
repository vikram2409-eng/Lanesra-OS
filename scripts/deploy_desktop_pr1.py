#!/usr/bin/env python3
"""Deploy Desktop UI Foundation (PR 1)."""

import os, shutil

REPO = "."

def write_file(path, content):
    full = os.path.join(REPO, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    open(full, "w", encoding="utf-8").write(content)
    print(f"Wrote {path}")

# File: desktop/package.json
write_file("desktop/package.json", """{
  "name": "lanesra-os-desktop",
  "private": true,
  "version": "0.14.0",
  "license": "MIT",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tanstack/react-query": "^5",
    "@tauri-apps/api": "^2",
    "class-variance-authority": "^0.7.1",
    "clsx": "^2.1.1",
    "react": "^18",
    "react-dom": "^18",
    "tailwind-merge": "^3.7.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@testing-library/jest-dom": "^7.0.1",
    "@testing-library/react": "^16.3.3",
    "@testing-library/user-event": "^14.6.8",
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "@vitejs/plugin-react": "^4",
    "jsdom": "^30.1.2",
    "typescript": "^5",
    "vite": "^5",
    "vitest": "^2"
  }
}
""")

# File: desktop/src/components/AppShell.tsx
write_file("desktop/src/components/AppShell.tsx", """import { useQuery } from "@tanstack/react-query";

import { GlobalSearch } from "./GlobalSearch";
import { AdminCommandPalette } from "./AdminCommandPalette";
import { NotificationBell } from "./NotificationBell";
import { VoiceModeButton } from "../features/voice/VoiceModeButton";
import { api } from "../lib/api";
import type { CustomObjectDefinition, User } from "../lib/types";
import logoIcon from "../assets/lanesraos-logo-icon.png";

// Custom-object sections are dynamic (admin-defined at runtime), so their
// key is a template literal - `custom:<objectKey>` - rather than one more
// fixed union member. App.tsx parses this prefix to know when to render
// CustomObjectRecords instead of a hardcoded screen.
export type Section =
  | "dashboard"
  | "companies"
  | "contacts"
  | "products"
  | "opportunities"
  | "quotes"
  | "orders"
  | "invoices"
  | "contracts"
  | "tasks"
  | "reports"
  | "assistant"
  | "admin"
  | "account"
  | `custom:${string}`;

export const customObjectSection = (key: string): Section => `custom:${key}`;

// Core entities each have their own fixed nav section; anything else is a
// custom object, addressed by its `key` via customObjectSection. Lives
// here (rather than in GlobalSearch, its original home) so the App
// Switcher below can resolve an AppDefinition.object_keys entry to a nav
// Section without GlobalSearch.tsx importing back from this module -
// GlobalSearch already imports Section/customObjectSection from here, and
// a two-way import would be a circular dependency. GlobalSearch and
// Dashboard both now import `sectionFor` from here instead of defining/
// re-exporting their own copy.
const CORE_ENTITY_SECTION: Record<string, Section> = {
  Company: "companies",
  Contact: "contacts",
  Opportunity: "opportunities",
  Quote: "quotes",
  Order: "orders",
  Invoice: "invoices",
  Contract: "contracts",
  Task: "tasks",
  Product: "products",
};

export function sectionFor(entityType: string): Section {
  return CORE_ENTITY_SECTION[entityType] ?? customObjectSection(entityType);
}

// Nav sections an App Builder app can scope down to - every entity-type
// section above, i.e. everything a Custom Object or a built-in object
// could resolve to via sectionFor. Structural sections (dashboard, admin,
// reports, account) are never filtered by an active app - see the
// App Switcher's own doc comment below for why.
const SCOPABLE_SECTIONS = new Set<Section>(Object.values(CORE_ENTITY_SECTION));

/** Addendum Phase 5 (Customer 360 / Contact 360): a "+ New X" button on a
 * detail view's related-record tab navigates to that record type's own
 * section and pre-fills its create form with the relationship it was
 * launched from, instead of starting from a blank form the user has to
 * re-select the company/contact on. One-shot: the target section reads
 * it once on mount (it fully unmounts/remounts on every section change,
 * so "on mount" reliably means "just navigated here") and immediately
 * clears it via `onPrefillConsumed`, so a later plain sidebar click into
 * the same section starts from a blank list as normal.
 *
 * Record-detail-page round: `openId` reuses this exact one-shot mechanism
 * for the opposite direction - a related-record link on a detail page
 * (e.g. a Quote's "Company" link, an Order's "Source quote" link) jumps to
 * that record's own section and opens its detail view directly instead of
 * landing on a plain list the user has to search. The target section reads
 * it once on mount to seed its initial view as `{mode:"detail", id:
 * openId}`, then clears it the same way companyId/contactId already do. */
export interface Prefill {
  companyId?: string;
  contactId?: string;
  openId?: string;
}

const NAV_ITEMS: { section: Section; label: string; adminOnly?: boolean }[] = [
  { section: "dashboard", label: "Dashboard" },
  { section: "companies", label: "Companies" },
  { section: "contacts", label: "Contacts" },
  { section: "opportunities", label: "Sales Pipeline" },
  { section: "products", label: "Products" },
  { section: "quotes", label: "Quotes" },
  { section: "orders", label: "Orders" },
  { section: "invoices", label: "Invoices" },
  { section: "contracts", label: "Contracts" },
  { section: "tasks", label: "Tasks" },
  { section: "reports", label: "Reports" },
  // AI & Agentic Layer, Phase 5: any signed-in user can chat with the
  // records-mode assistant (look up and act on records). The separate
  // admin-mode assistant lives inside the Admin panel instead, alongside
  // every other admin-only capability - see ADMIN_CATEGORIES in Settings.tsx.
  { section: "assistant", label: "Assistant" },
  // Users lives inside the Admin panel now, alongside branding, custom
  // fields, business rules, workflow automation, numbering and Dashboard
  // KPIs - one nav item for every administrator-facing capability.
  { section: "admin", label: "Admin", adminOnly: true },
];

export function AppShell({
  active,
  onNavigate,
  onOpenSearchResult,
  onOpenAdminTab,
  user,
  onLogout,
  customObjects,
  activeAppId,
  onSwitchApp,
  children,
}: {
  active: Section;
  onNavigate: (section: Section) => void;
  /** Global search "jump to a record" - reuses the same one-shot openId
   * prefill mechanism every list screen's ID-hyperlinks already use. */
  onOpenSearchResult: (section: Section, id: string) => void;
  /** Admin Control Center Modernization (issue #197): the Command
   * Palette's own "Admin" result picks a specific AdminTab (a string here
   * to avoid this module depending on Settings.tsx's own AdminTab type) -
   * App.tsx resolves it into AdminPanel's `openAdminTab` prop. */
  onOpenAdminTab: (adminTab: string) => void;
  user: User;
  onLogout: () => void;
  customObjects: CustomObjectDefinition[];
  /** App Builder: which accessible app (if any) is currently selected in
   * the switcher below - `null` is "All", the pre-App-Builder sidebar with
   * every section visible. Lifted to App.tsx (not local state here)
   * because Dashboard also needs to know it, to render that app's own
   * dashboard instead of the role-resolved default. */
  activeAppId: string | null;
  onSwitchApp: (appId: string | null) => void;
  children: React.ReactNode;
}) {
  const isAdmin = user.roles.includes("Administrator");
  // Every published app the signed-in user has a grant on (or all of them,
  // if they're an Administrator) - see app_service::list_accessible. Fetched
  // here rather than threaded down as a prop, the same self-contained-query
  // pattern GlobalSearch/NotificationBell already use; App.tsx runs the same
  // query (React Query dedupes it) to resolve the active app's dashboard_id.
  const apps = useQuery({ queryKey: ["accessibleApps"], queryFn: () => api.listAccessibleApps() });
  const accessibleApps = apps.data ?? [];
  const activeApp = accessibleApps.find((a) => a.app.id === activeAppId)?.app ?? null;

  // With an app selected, only its own object_keys' sections (and any
  // structural section - Dashboard/Admin/Reports/Account) show in the
  // sidebar. `null` (no app selected, "All") shows everything, exactly the
  // pre-App-Builder sidebar.
  const allowedSections = activeApp ? new Set(activeApp.object_keys.map(sectionFor)) : null;
  function sectionVisible(section: Section): boolean {
    if (!allowedSections || !SCOPABLE_SECTIONS.has(section)) return true;
    return allowedSections.has(section);
  }

  return (
    <div className="app-shell">
      <AdminCommandPalette onNavigate={onNavigate} onOpenRecord={onOpenSearchResult} onOpenAdminTab={onOpenAdminTab} isAdmin={isAdmin} />
      <nav className="sidebar">
        <div className="sidebar-brand">
          <img src={logoIcon} alt="LanesraOS" className="brand-logo" />
          <span>Lanesra OS</span>
        </div>
        {accessibleApps.length > 0 && (
          <select
            className="app-switcher"
            value={activeAppId ?? ""}
            onChange={(e) => onSwitchApp(e.target.value || null)}
            aria-label="Switch app"
            style={{ width: "100%", marginBottom: 8 }}
          >
            <option value="">All</option>
            {accessibleApps.map((a) => (
              <option key={a.app.id} value={a.app.id}>
                {a.app.icon} {a.app.name}
              </option>
            ))}
          </select>
        )}
        {NAV_ITEMS.filter((item) => (!item.adminOnly || isAdmin) && sectionVisible(item.section)).map((item) => (
          <button
            key={item.section}
            className={`nav-item${active === item.section ? " active" : ""}`}
            onClick={() => onNavigate(item.section)}
          >
            {item.label}
          </button>
        ))}
        {customObjects.filter((o) => sectionVisible(customObjectSection(o.key))).length > 0 && (
          <div className="sidebar-divider" />
        )}
        {customObjects
          .filter((o) => sectionVisible(customObjectSection(o.key)))
          .map((o) => (
            <button
              key={o.key}
              className={`nav-item${active === customObjectSection(o.key) ? " active" : ""}`}
              onClick={() => onNavigate(customObjectSection(o.key))}
            >
              {o.icon} {o.plural_label}
            </button>
          ))}
      </nav>
      <main className="main">
        <div className="topbar">
          <GlobalSearch customObjects={customObjects} onOpenResult={onOpenSearchResult} />
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <VoiceModeButton />
            <NotificationBell />
            <button
              className="link-button"
              style={{ fontSize: 13, color: "var(--text-muted)" }}
              onClick={() => onNavigate("account")}
            >
              {user.display_name} · {user.roles.join(", ")}
            </button>
            <button className="btn" onClick={onLogout}>
              Sign out
            </button>
          </div>
        </div>
        {children}
      </main>
    </div>
  );
}
""")

# File: desktop/src/components/ui/animated-gradient-text.tsx
write_file("desktop/src/components/ui/animated-gradient-text.tsx", """import * as React from "react";

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
""")

# File: desktop/src/components/ui/badge.tsx
write_file("desktop/src/components/ui/badge.tsx", """import * as React from "react";
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
""")

# File: desktop/src/components/ui/button.tsx
write_file("desktop/src/components/ui/button.tsx", """import * as React from "react";
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
""")

# File: desktop/src/components/ui/card.tsx
write_file("desktop/src/components/ui/card.tsx", """import * as React from "react";

import { cn } from "./cn";

const Card = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("ui-card", className)} {...props} />
  ),
);
Card.displayName = "Card";

const CardHeader = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("ui-card-header", className)} {...props} />
  ),
);
CardHeader.displayName = "CardHeader";

const CardTitle = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("ui-card-title", className)} {...props} />
  ),
);
CardTitle.displayName = "CardTitle";

const CardDescription = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("ui-card-description", className)} {...props} />
  ),
);
CardDescription.displayName = "CardDescription";

const CardContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("ui-card-content", className)} {...props} />
  ),
);
CardContent.displayName = "CardContent";

const CardFooter = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("ui-card-footer", className)} {...props} />
  ),
);
CardFooter.displayName = "CardFooter";

export { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter };
""")

# File: desktop/src/components/ui/checkbox.tsx
write_file("desktop/src/components/ui/checkbox.tsx", """import * as React from "react";

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
""")

# File: desktop/src/components/ui/cn.ts
write_file("desktop/src/components/ui/cn.ts", """import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** shadcn-style class merger: conditional classes via clsx, deduped via tailwind-merge. */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
""")

# File: desktop/src/components/ui/dialog.tsx
write_file("desktop/src/components/ui/dialog.tsx", """import * as React from "react";
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
""")

# File: desktop/src/components/ui/index.ts
write_file("desktop/src/components/ui/index.ts", """/** shadcn/Magic-UI-flavored component library (hand-rolled, no Radix, no framer-motion). */
export { cn } from "./cn";
export { Button, buttonVariants, type ButtonProps } from "./button";
export {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "./card";
export { Badge, badgeVariants, type BadgeProps } from "./badge";
export { Input } from "./input";
export { Label } from "./label";
export { Checkbox, type CheckboxProps } from "./checkbox";
export { Select, type SelectProps, type SelectOption } from "./select";
export { Separator, type SeparatorProps } from "./separator";
export { Skeleton } from "./skeleton";
export {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  DialogClose,
  type DialogProps,
} from "./dialog";
export { Tabs, TabsList, TabsTrigger, TabsContent, type TabsProps } from "./tabs";
export { Tooltip, type TooltipProps } from "./tooltip";
export { ShimmerButton, type ShimmerButtonProps } from "./shimmer-button";
export { AnimatedGradientText, type AnimatedGradientTextProps } from "./animated-gradient-text";
export { NumberTicker, type NumberTickerProps } from "./number-ticker";
""")

# File: desktop/src/components/ui/input.tsx
write_file("desktop/src/components/ui/input.tsx", """import * as React from "react";

import { cn } from "./cn";

const Input = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, type = "text", ...props }, ref) => (
    <input ref={ref} type={type} className={cn("ui-input", className)} {...props} />
  ),
);
Input.displayName = "Input";

export { Input };
""")

# File: desktop/src/components/ui/label.tsx
write_file("desktop/src/components/ui/label.tsx", """import * as React from "react";

import { cn } from "./cn";

const Label = React.forwardRef<HTMLLabelElement, React.LabelHTMLAttributes<HTMLLabelElement>>(
  ({ className, ...props }, ref) => (
    <label ref={ref} className={cn("ui-label", className)} {...props} />
  ),
);
Label.displayName = "Label";

export { Label };
""")

# File: desktop/src/components/ui/number-ticker.tsx
write_file("desktop/src/components/ui/number-ticker.tsx", """import * as React from "react";

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
""")

# File: desktop/src/components/ui/select.tsx
write_file("desktop/src/components/ui/select.tsx", """import * as React from "react";

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
""")

# File: desktop/src/components/ui/separator.tsx
write_file("desktop/src/components/ui/separator.tsx", """import * as React from "react";

import { cn } from "./cn";

export interface SeparatorProps extends React.HTMLAttributes<HTMLDivElement> {
  orientation?: "horizontal" | "vertical";
  decorative?: boolean;
}

function Separator({
  className,
  orientation = "horizontal",
  decorative = true,
  ...props
}: SeparatorProps) {
  return (
    <div
      role={decorative ? "none" : "separator"}
      aria-orientation={decorative ? undefined : orientation}
      className={cn("ui-separator", orientation === "vertical" && "ui-separator-vertical", className)}
      {...props}
    />
  );
}

export { Separator };
""")

# File: desktop/src/components/ui/shimmer-button.tsx
write_file("desktop/src/components/ui/shimmer-button.tsx", """import * as React from "react";

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
""")

# File: desktop/src/components/ui/skeleton.tsx
write_file("desktop/src/components/ui/skeleton.tsx", """import * as React from "react";

import { cn } from "./cn";

function Skeleton({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div aria-hidden="true" className={cn("ui-skeleton", className)} {...props} />;
}

export { Skeleton };
""")

# File: desktop/src/components/ui/tabs.tsx
write_file("desktop/src/components/ui/tabs.tsx", """import * as React from "react";

import { cn } from "./cn";

interface TabsContextValue {
  value: string;
  setValue: (v: string) => void;
}

const TabsContext = React.createContext<TabsContextValue | null>(null);

function useTabsContext(component: string): TabsContextValue {
  const ctx = React.useContext(TabsContext);
  if (!ctx) throw new Error(`${component} must be used inside <Tabs>`);
  return ctx;
}

export interface TabsProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
}

function Tabs({ value, defaultValue, onValueChange, children, className, ...props }: TabsProps) {
  const [internal, setInternal] = React.useState(defaultValue ?? "");
  const current = value ?? internal;
  const setValue = React.useCallback(
    (v: string) => {
      setInternal(v);
      onValueChange?.(v);
    },
    [onValueChange],
  );
  return (
    <div className={cn("ui-tabs", className)} {...props}>
      <TabsContext.Provider value={{ value: current, setValue }}>
        {children}
      </TabsContext.Provider>
    </div>
  );
}

function TabsList({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div role="tablist" className={cn("ui-tabs-list", className)} {...props} />;
}

export interface TabsTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  value: string;
}

function TabsTrigger({ value, className, disabled, ...props }: TabsTriggerProps) {
  const { value: current, setValue } = useTabsContext("TabsTrigger");
  const selected = current === value;
  return (
    <button
      type="button"
      role="tab"
      aria-selected={selected}
      disabled={disabled}
      data-state={selected ? "active" : "inactive"}
      className={cn("ui-tabs-trigger", className)}
      onClick={() => setValue(value)}
      {...props}
    />
  );
}

export interface TabsContentProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string;
}

function TabsContent({ value, className, children, ...props }: TabsContentProps) {
  const { value: current } = useTabsContext("TabsContent");
  if (current !== value) return null;
  return (
    <div role="tabpanel" className={cn("ui-tabs-content", className)} {...props}>
      {children}
    </div>
  );
}

export { Tabs, TabsList, TabsTrigger, TabsContent };
""")

# File: desktop/src/components/ui/tooltip.tsx
write_file("desktop/src/components/ui/tooltip.tsx", """import * as React from "react";

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
""")

# File: desktop/src/components/ui/ui.css
write_file("desktop/src/components/ui/ui.css", """/* Desktop UI Foundation (PR 1) — styles for desktop/src/components/ui/.
   Token-only: every color/spacing/radius/shadow below resolves to a custom
   property from styles/tokens.css. No hardcoded palette values.

   Namespaced tint tokens: the desktop's own --accent is the brand blue
   (used in 50+ places), so shadcn's "accent" (a tinted hover surface) lives
   under --ui-accent / --ui-accent-foreground to avoid the collision. */

/* ------------------------------------------------------------------ */
/* Namespaced shadcn "accent" tint (see note above)                    */
/* ------------------------------------------------------------------ */
:root {
  --ui-accent: #eef2ff;
  --ui-accent-foreground: #4338ca;
}
html[data-theme="dark"] {
  --ui-accent: #2a2148;
  --ui-accent-foreground: #c4b5fd;
}
@media (prefers-color-scheme: dark) {
  html:not([data-theme]) {
    --ui-accent: #2a2148;
    --ui-accent-foreground: #c4b5fd;
  }
}

/* ------------------------------------------------------------------ */
/* Button                                                              */
/* ------------------------------------------------------------------ */
.ui-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  white-space: nowrap;
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  font-weight: 600;
  line-height: 1.25;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  user-select: none;
  transition:
    background-color var(--motion-fast) ease,
    border-color var(--motion-fast) ease,
    color var(--motion-fast) ease,
    box-shadow var(--motion-fast) ease,
    opacity var(--motion-fast) ease;
}
.ui-btn:focus-visible {
  outline: none;
  box-shadow:
    0 0 0 2px var(--background),
    0 0 0 4px var(--ring);
}
.ui-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}
.ui-btn-default {
  background: var(--primary);
  border-color: var(--primary);
  color: var(--primary-foreground);
  box-shadow: var(--shadow-sm);
}
.ui-btn-default:hover {
  filter: brightness(0.94);
}
.ui-btn-destructive {
  background: var(--destructive);
  border-color: var(--destructive);
  color: var(--destructive-foreground);
  box-shadow: var(--shadow-sm);
}
.ui-btn-destructive:hover {
  filter: brightness(0.94);
}
.ui-btn-outline {
  background: var(--card);
  border-color: var(--border);
  color: var(--foreground);
}
.ui-btn-outline:hover {
  background: var(--muted);
}
.ui-btn-secondary {
  background: var(--secondary);
  border-color: var(--secondary);
  color: var(--secondary-foreground);
}
.ui-btn-secondary:hover {
  filter: brightness(0.96);
}
.ui-btn-ghost {
  background: transparent;
  border-color: transparent;
  color: var(--foreground);
}
.ui-btn-ghost:hover {
  background: var(--ui-accent);
  color: var(--ui-accent-foreground);
}
.ui-btn-size-default {
  height: 40px;
  padding: 0 var(--space-4);
}
.ui-btn-size-sm {
  height: 32px;
  padding: 0 var(--space-3);
  font-size: var(--text-xs);
}
.ui-btn-size-lg {
  height: 48px;
  padding: 0 var(--space-6);
  font-size: var(--text-base);
}
.ui-btn-size-icon {
  height: 40px;
  width: 40px;
  padding: 0;
}

/* ------------------------------------------------------------------ */
/* Card                                                                */
/* ------------------------------------------------------------------ */
.ui-card {
  background: var(--card);
  color: var(--card-foreground);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
}
.ui-card-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding: var(--space-6) var(--space-6) 0;
}
.ui-card-title {
  font-size: var(--text-lg);
  font-weight: 700;
  letter-spacing: -0.01em;
  line-height: 1.3;
}
.ui-card-description {
  font-size: var(--text-sm);
  color: var(--muted-foreground);
  line-height: 1.5;
}
.ui-card-content {
  padding: var(--space-6);
}
.ui-card-header + .ui-card-content {
  padding-top: var(--space-4);
}
.ui-card-footer {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-6) var(--space-6);
}

/* ------------------------------------------------------------------ */
/* Badge                                                               */
/* ------------------------------------------------------------------ */
.ui-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px solid transparent;
  border-radius: var(--radius-full);
  padding: 2px 10px;
  font-size: var(--text-xs);
  font-weight: 600;
  line-height: 1.5;
  white-space: nowrap;
  transition: background-color var(--motion-fast) ease, color var(--motion-fast) ease;
}
.ui-badge-default {
  background: var(--primary);
  color: var(--primary-foreground);
}
.ui-badge-secondary {
  background: var(--secondary);
  color: var(--secondary-foreground);
}
.ui-badge-outline {
  background: transparent;
  border-color: var(--border);
  color: var(--foreground);
}
.ui-badge-success {
  background: var(--success);
  color: #ffffff;
}
.ui-badge-warning {
  background: var(--warning);
  color: #ffffff;
}
.ui-badge-destructive {
  background: var(--destructive);
  color: var(--destructive-foreground);
}

/* ------------------------------------------------------------------ */
/* Input / Label                                                       */
/* ------------------------------------------------------------------ */
.ui-input {
  display: block;
  width: 100%;
  height: 40px;
  padding: 0 var(--space-3);
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  color: var(--foreground);
  background: var(--card);
  border: 1px solid var(--input);
  border-radius: var(--radius-md);
  transition: border-color var(--motion-fast) ease, box-shadow var(--motion-fast) ease;
}
.ui-input::placeholder {
  color: var(--muted-foreground);
}
.ui-input:focus {
  outline: none;
  border-color: var(--ring);
  box-shadow: 0 0 0 3px var(--ring);
}
.ui-input:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.ui-label {
  display: inline-block;
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--foreground);
  margin-bottom: var(--space-1);
}

/* ------------------------------------------------------------------ */
/* Checkbox                                                            */
/* ------------------------------------------------------------------ */
.ui-checkbox-wrap {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
}
.ui-checkbox-input {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}
.ui-checkbox-box {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex: none;
  border: 1.5px solid var(--input);
  border-radius: var(--radius-sm);
  background: var(--card);
  color: var(--primary-foreground);
  transition:
    background-color var(--motion-fast) ease,
    border-color var(--motion-fast) ease,
    box-shadow var(--motion-fast) ease;
}
.ui-checkbox-box svg {
  width: 12px;
  height: 12px;
  opacity: 0;
  transform: scale(0.6);
  transition: opacity var(--motion-fast) ease, transform var(--motion-fast) ease;
}
.ui-checkbox-input:checked + .ui-checkbox-box {
  background: var(--primary);
  border-color: var(--primary);
}
.ui-checkbox-input:checked + .ui-checkbox-box svg {
  opacity: 1;
  transform: scale(1);
}
.ui-checkbox-input:focus-visible + .ui-checkbox-box {
  box-shadow: 0 0 0 3px var(--ring);
}
.ui-checkbox-input:disabled ~ .ui-checkbox-box,
.ui-checkbox-input:disabled ~ .ui-checkbox-label {
  opacity: 0.5;
  cursor: not-allowed;
}
.ui-checkbox-label {
  font-size: var(--text-sm);
  color: var(--foreground);
  cursor: pointer;
}

/* ------------------------------------------------------------------ */
/* Select (native, styled)                                             */
/* ------------------------------------------------------------------ */
.ui-select {
  position: relative;
  display: inline-block;
}
.ui-select-input {
  appearance: none;
  -webkit-appearance: none;
  display: block;
  width: 100%;
  height: 40px;
  padding: 0 36px 0 var(--space-3);
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  color: var(--foreground);
  background: var(--card);
  border: 1px solid var(--input);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: border-color var(--motion-fast) ease, box-shadow var(--motion-fast) ease;
}
.ui-select-input:focus {
  outline: none;
  border-color: var(--ring);
  box-shadow: 0 0 0 3px var(--ring);
}
.ui-select-input:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.ui-select-chevron {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: var(--muted-foreground);
  pointer-events: none;
}

/* ------------------------------------------------------------------ */
/* Separator / Skeleton                                                */
/* ------------------------------------------------------------------ */
.ui-separator {
  background: var(--border);
  flex: none;
}
.ui-separator:not(.ui-separator-vertical) {
  height: 1px;
  width: 100%;
}
.ui-separator-vertical {
  width: 1px;
  align-self: stretch;
}
.ui-skeleton {
  border-radius: var(--radius-md);
  background: var(--muted);
  animation: ui-skeleton-pulse 1.6s ease-in-out infinite;
}
@keyframes ui-skeleton-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.45;
  }
}

/* ------------------------------------------------------------------ */
/* Dialog                                                              */
/* ------------------------------------------------------------------ */
.ui-dialog-root {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-6);
}
.ui-dialog-overlay {
  position: absolute;
  inset: 0;
  background: rgba(9, 9, 11, 0.55);
  animation: ui-dialog-fade-in var(--motion-base) ease;
}
.ui-dialog-content {
  position: relative;
  width: 100%;
  max-width: 480px;
  max-height: calc(100vh - 96px);
  overflow: auto;
  background: var(--popover);
  color: var(--popover-foreground);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  padding: var(--space-6);
  animation: ui-dialog-pop-in var(--motion-base) cubic-bezier(0.16, 1, 0.3, 1);
}
.ui-dialog-content:focus {
  outline: none;
}
.ui-dialog-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  margin-bottom: var(--space-4);
  padding-right: var(--space-8);
}
.ui-dialog-title {
  margin: 0;
  font-size: var(--text-lg);
  font-weight: 700;
  letter-spacing: -0.01em;
}
.ui-dialog-description {
  margin: 0;
  font-size: var(--text-sm);
  color: var(--muted-foreground);
  line-height: 1.5;
}
.ui-dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  margin-top: var(--space-6);
}
.ui-dialog-x {
  position: absolute;
  top: var(--space-4);
  right: var(--space-4);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
}
.ui-dialog-x:hover {
  background: var(--muted);
  color: var(--foreground);
}
.ui-dialog-x svg {
  width: 16px;
  height: 16px;
}
@keyframes ui-dialog-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
@keyframes ui-dialog-pop-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

/* ------------------------------------------------------------------ */
/* Tabs                                                                */
/* ------------------------------------------------------------------ */
.ui-tabs {
  display: block;
}
.ui-tabs-list {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 4px;
  background: var(--muted);
  border-radius: var(--radius-md);
}
.ui-tabs-trigger {
  border: 0;
  background: transparent;
  color: var(--muted-foreground);
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  font-weight: 600;
  padding: 6px var(--space-3);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    background-color var(--motion-fast) ease,
    color var(--motion-fast) ease,
    box-shadow var(--motion-fast) ease;
}
.ui-tabs-trigger:hover {
  color: var(--foreground);
}
.ui-tabs-trigger[data-state="active"] {
  background: var(--card);
  color: var(--foreground);
  box-shadow: var(--shadow-sm);
}
.ui-tabs-trigger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.ui-tabs-content {
  margin-top: var(--space-4);
}
.ui-tabs-content:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: 2px;
}

/* ------------------------------------------------------------------ */
/* Tooltip (CSS-only)                                                  */
/* ------------------------------------------------------------------ */
.ui-tooltip {
  position: relative;
  display: inline-flex;
}
.ui-tooltip::after {
  content: attr(data-tip);
  position: absolute;
  z-index: 90;
  max-width: 240px;
  padding: 6px 10px;
  font-size: var(--text-xs);
  font-weight: 500;
  line-height: 1.4;
  white-space: normal;
  color: var(--popover-foreground);
  background: var(--popover);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-md);
  opacity: 0;
  pointer-events: none;
  transform: translateY(2px);
  transition: opacity var(--motion-fast) ease, transform var(--motion-fast) ease;
}
.ui-tooltip:hover::after,
.ui-tooltip:focus-visible::after {
  opacity: 1;
  transform: none;
}
.ui-tooltip:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: 2px;
}
.ui-tooltip-top::after {
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%) translateY(2px);
}
.ui-tooltip-top:hover::after,
.ui-tooltip-top:focus-visible::after {
  transform: translateX(-50%);
}
.ui-tooltip-bottom::after {
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%) translateY(-2px);
}
.ui-tooltip-bottom:hover::after,
.ui-tooltip-bottom:focus-visible::after {
  transform: translateX(-50%);
}
.ui-tooltip-left::after {
  right: calc(100% + 8px);
  top: 50%;
  transform: translateY(-50%) translateX(2px);
}
.ui-tooltip-left:hover::after,
.ui-tooltip-left:focus-visible::after {
  transform: translateY(-50%);
}
.ui-tooltip-right::after {
  left: calc(100% + 8px);
  top: 50%;
  transform: translateY(-50%) translateX(-2px);
}
.ui-tooltip-right:hover::after,
.ui-tooltip-right:focus-visible::after {
  transform: translateY(-50%);
}

/* ------------------------------------------------------------------ */
/* Magic UI — ShimmerButton (pure CSS)                                  */
/* ------------------------------------------------------------------ */
.ui-shimmer-btn {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border: 1px solid transparent;
  border-radius: var(--shimmer-radius, var(--radius-md));
  background: var(--shimmer-bg, var(--primary));
  color: var(--primary-foreground);
  font-family: var(--font-sans);
  font-size: var(--text-sm);
  font-weight: 600;
  padding: 10px var(--space-5);
  cursor: pointer;
  isolation: isolate;
}
.ui-shimmer-btn::before {
  content: "";
  position: absolute;
  inset: 0;
  z-index: 1;
  background: linear-gradient(
    110deg,
    transparent 30%,
    var(--shimmer-color, #ffffff) 50%,
    transparent 70%
  );
  background-size: 250% 100%;
  background-repeat: no-repeat;
  opacity: 0.55;
  mix-blend-mode: overlay;
  animation: ui-shimmer-sweep var(--shimmer-duration, 2.75s) linear infinite;
  pointer-events: none;
}
.ui-shimmer-btn-inner {
  position: relative;
  z-index: 2;
}
.ui-shimmer-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.ui-shimmer-btn:disabled::before {
  animation: none;
}
@keyframes ui-shimmer-sweep {
  from {
    background-position: 200% 0;
  }
  to {
    background-position: -200% 0;
  }
}

/* ------------------------------------------------------------------ */
/* Magic UI — AnimatedGradientText (pure CSS)                           */
/* ------------------------------------------------------------------ */
.ui-gradient-text {
  background: linear-gradient(
    90deg,
    var(--gradient-from, #8b5cf6),
    var(--gradient-to, #635bff),
    var(--gradient-from, #8b5cf6)
  );
  background-size: 200% auto;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation: ui-gradient-pan var(--gradient-speed, 6s) linear infinite;
}
@keyframes ui-gradient-pan {
  to {
    background-position: 200% center;
  }
}

/* ------------------------------------------------------------------ */
/* Magic UI — NumberTicker (rAF-driven; no animation CSS needed)        */
/* ------------------------------------------------------------------ */
.ui-number-ticker {
  font-variant-numeric: tabular-nums;
  font-feature-settings: "tnum";
}
""")

# File: desktop/src/features/auth/Login.tsx
write_file("desktop/src/features/auth/Login.tsx", """import { useState } from "react";

import { api, ApiError } from "../../lib/api";
import type { User } from "../../lib/types";
import logoIcon from "../../assets/lanesraos-logo-icon.png";

export function Login({ businessName, onLogin }: { businessName: string; onLogin: (user: User) => void }) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const user = await api.login({ username, password });
      onLogin(user);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Could not sign in");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="centered-screen">
      <div className="card auth-card">
        <img src={logoIcon} alt="LanesraOS" className="auth-logo" />
        <h1>{businessName}</h1>
        <p>Sign in to your local Lanesra OS workspace</p>
        {error && <div className="error-banner">{error}</div>}
        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <div className="form-field">
            <label>Username</label>
            <input value={username} onChange={(e) => setUsername(e.target.value)} required autoFocus />
          </div>
          <div className="form-field">
            <label>Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </div>
          <button className="btn btn-primary" type="submit" disabled={submitting}>
            {submitting ? "Signing in..." : "Sign in"}
          </button>
        </form>
      </div>
    </div>
  );
}
""")

# File: desktop/src/features/firstRun/FirstRun.tsx
write_file("desktop/src/features/firstRun/FirstRun.tsx", """import { useState } from "react";

import { api, ApiError } from "../../lib/api";
import type { User, Workspace } from "../../lib/types";
import logoIcon from "../../assets/lanesraos-logo-icon.png";

export function FirstRun({ onComplete }: { onComplete: (workspace: Workspace, user: User) => void }) {
  const [businessName, setBusinessName] = useState("");
  const [currencyCode, setCurrencyCode] = useState("USD");
  const [adminUsername, setAdminUsername] = useState("admin");
  const [adminDisplayName, setAdminDisplayName] = useState("");
  const [adminPassword, setAdminPassword] = useState("");
  const [loadSampleData, setLoadSampleData] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const [workspace, user] = await api.firstRunSetup({
        business_name: businessName,
        legal_name: null,
        currency_code: currencyCode,
        locale: "en-US",
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone ?? "UTC",
        default_tax_rate_bp: 0,
        admin_username: adminUsername,
        admin_display_name: adminDisplayName || adminUsername,
        admin_password: adminPassword,
        load_sample_data: loadSampleData,
      });
      onComplete(workspace, user);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Could not set up the workspace");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="centered-screen">
      <div className="card auth-card">
        <img src={logoIcon} alt="LanesraOS" className="auth-logo" />
        <h1>Welcome to Lanesra OS</h1>
        <p>Set up your local workspace. Everything stays on this computer.</p>
        {error && <div className="error-banner">{error}</div>}
        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <div className="form-field">
            <label>Business name</label>
            <input value={businessName} onChange={(e) => setBusinessName(e.target.value)} required />
          </div>
          <div className="form-field">
            <label>Currency</label>
            <input
              value={currencyCode}
              onChange={(e) => setCurrencyCode(e.target.value.toUpperCase())}
              maxLength={3}
              required
            />
          </div>
          <div className="form-field">
            <label>Administrator username</label>
            <input value={adminUsername} onChange={(e) => setAdminUsername(e.target.value)} required />
          </div>
          <div className="form-field">
            <label>Administrator display name</label>
            <input value={adminDisplayName} onChange={(e) => setAdminDisplayName(e.target.value)} />
          </div>
          <div className="form-field">
            <label>Administrator password (min 8 characters)</label>
            <input
              type="password"
              value={adminPassword}
              onChange={(e) => setAdminPassword(e.target.value)}
              minLength={8}
              required
            />
          </div>
          <label style={{ fontSize: 13, display: "flex", gap: 8, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={loadSampleData}
              onChange={(e) => setLoadSampleData(e.target.checked)}
            />
            Start with sample data
          </label>
          <button className="btn btn-primary" type="submit" disabled={submitting}>
            {submitting ? "Setting up..." : "Create workspace"}
          </button>
        </form>
      </div>
    </div>
  );
}
""")

# File: desktop/src/styles.css
write_file("desktop/src/styles.css", """@import "./styles/tokens.css";
@import "./components/ui/ui.css";

/* Desktop UI Foundation (PR 1): the app now defaults to the Light theme
   (user decision). An explicit [data-theme] on <html> always wins;
   otherwise the OS prefers-color-scheme preference applies when no choice
   is set. Theme Studio's applyThemeTokens() keeps working — it sets inline
   custom properties, which outrank every rule below. */
:root {
  color-scheme: light dark;
  --bg: #f5f7fb;
  --bg-elevated: #ffffff;
  --surface-sidebar: var(--bg-elevated);
  --border: #dde3ef;
  --text: #131826;
  --text-muted: #5b6572;
  --accent: #4f7cff;
  --accent-secondary: #4f7cff;
  --accent-contrast: #ffffff;
  --danger: #ef4444;
  --success: #22c55e;
  --warning: #f59e0b;
  --info: #3b82f6;
  /* Runtime UX Modernization (issue #198): the default categorical chart
     palette, applied before any workspace publishes a theme - see
     applyTheme.ts for how a Published theme's own palette overrides this. */
  --chart-1: #635bff;
  --chart-2: #0f9d76;
  --chart-3: #f59e0b;
  --chart-4: #ef4444;
  --chart-5: #3b82f6;
  --chart-6: #8b5cf6;
  --font-family: -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  --base-font-size: 14px;
  font-family: var(--font-family);
  font-size: var(--base-font-size);
}

@media (prefers-color-scheme: dark) {
  html:not([data-theme]) {
    --bg: #0f1420;
    --bg-elevated: #161d2e;
    --surface-sidebar: var(--bg-elevated);
    --border: #2a3550;
    --text: #e6ebf5;
    --text-muted: #94a3b8;
  }
}

/* Explicit theme overrides — html-qualified to outrank :root. */
html[data-theme="dark"] {
  --bg: #0f1420;
  --bg-elevated: #161d2e;
  --surface-sidebar: var(--bg-elevated);
  --border: #2a3550;
  --text: #e6ebf5;
  --text-muted: #94a3b8;
}

html[data-theme="light"] {
  --bg: #f5f7fb;
  --bg-elevated: #ffffff;
  --surface-sidebar: var(--bg-elevated);
  --border: #dde3ef;
  --text: #131826;
  --text-muted: #5b6572;
}

/* Ontology Explorer visual language (issue #339, extended for Object 360
   issue #352): a warm, organic canvas tone distinct from the cooler
   dot-grid `.graph-canvas-wrap` uses for every other builder (Agent Team
   Builder/Workflow Studio/Business Rule Board) - its own tokens so it
   stays theme-adaptive (light by default, dark override, same as every
   other token above) instead of a hardcoded cream that would look wrong
   in dark mode. */
:root {
  --ontology-canvas-bg: #faf6ee;
  --ontology-canvas-grid: #ece0c4;
}
html[data-theme="dark"] {
  --ontology-canvas-bg: #1c170f;
  --ontology-canvas-grid: #342b1b;
}
@media (prefers-color-scheme: dark) {
  html:not([data-theme]) {
    --ontology-canvas-bg: #1c170f;
    --ontology-canvas-grid: #342b1b;
  }
}

/* UX/UI Modernization, Phase A (issue #191): Theme Studio overrides every
   var above except --accent-contrast (no modeled token for it yet) via
   inline styles on :root - see lib/applyTheme.ts. A workspace with no
   Published theme never has those inline styles set, so it keeps exactly
   this static palette. Shape (radius_scale) and density are stored and
   previewed in Theme Studio but not yet retrofitted across this
   stylesheet's ~15 hardcoded border-radius/spacing rules - a named
   follow-up, not silently claimed here. */

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: var(--bg);
  color: var(--text);
  min-width: 1024px;
}

button {
  font-family: inherit;
  cursor: pointer;
}

input,
select,
textarea {
  font-family: inherit;
  background: var(--bg-elevated);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 8px;
}

.app-shell {
  display: grid;
  grid-template-columns: 220px 1fr;
  min-height: 100vh;
}

.sidebar {
  background: var(--surface-sidebar);
  border-right: 1px solid var(--border);
  padding: 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 700;
  font-size: 18px;
  padding: 8px;
  margin-bottom: 12px;
}

/* Desktop UI Foundation (PR 1): brand logo. */
.brand-logo {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  flex: none;
  display: block;
}
.auth-logo {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  display: block;
  margin: 0 auto 16px;
}

.nav-item {
  padding: 8px 12px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text);
  text-align: left;
  font-size: 14px;
}

.nav-item:hover {
  background: rgba(127, 127, 127, 0.1);
}

.nav-item.active {
  background: var(--accent);
  color: var(--accent-contrast);
}

.sidebar-divider {
  height: 1px;
  background: var(--border);
  margin: 8px 4px;
}

.main {
  padding: 24px 32px;
  overflow-y: auto;
}

.topbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.btn {
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  color: var(--text);
  padding: 7px 14px;
  border-radius: 6px;
  font-size: 14px;
}

.btn-primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-contrast);
}

.link-button {
  background: none;
  border: none;
  padding: 0;
  font: inherit;
  cursor: pointer;
}

.link-button:hover {
  text-decoration: underline;
}

/* Record-detail-page round: every list's ID column is a visible hyperlink
   into that record's detail page, not just an invisible whole-row click
   target - the same "click the ID to open the record" affordance every
   list already gave you for the row itself, made explicit. */
.id-link {
  color: var(--accent);
  font-weight: 600;
}

.id-link:hover {
  text-decoration: underline;
}

/* Admin landing page redesign round: a categorized home in place of the
   old flat tab row, plus the breadcrumb shown once inside a tool. */
.admin-landing-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
  margin-top: 16px;
}

.admin-cat-card {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 18px;
}

.admin-cat-head {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 12px;
}

.admin-cat-icon {
  font-size: 20px;
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: var(--bg);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.admin-cat-head h3 {
  margin: 0 0 2px;
  font-size: 15px;
}

.admin-cat-head p {
  margin: 0;
  font-size: 12.5px;
  color: var(--text-muted);
}

.admin-cat-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.admin-cat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  text-align: left;
  background: none;
  border: 0;
  border-radius: 8px;
  padding: 8px 9px;
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text);
  cursor: pointer;
}

.admin-cat-item:hover {
  background: var(--bg);
  color: var(--accent);
}

.admin-cat-item .admin-cat-arrow {
  color: var(--text-muted);
  font-weight: 400;
}

.admin-cat-item:hover .admin-cat-arrow {
  color: var(--accent);
}

.admin-breadcrumb {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.btn-danger {
  border-color: var(--danger);
  color: var(--danger);
  background: transparent;
}

.card {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
}

/* A generic centered dialog - first used by Deployment Management's
   update-with-diff review step. Deliberately minimal (no other screen in
   the app has needed a true modal before this - version history/duplicate
   review etc. all use an inline panel instead). */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
}

.modal {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 20px;
  width: 100%;
  max-height: 85vh;
  overflow-y: auto;
}

.kpi-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}

.kpi-tile {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 14px 16px;
  cursor: pointer;
}

.kpi-tile:hover {
  border-color: var(--accent);
}

.kpi-tile .value {
  font-size: 22px;
  font-weight: 700;
}

.kpi-tile .label {
  font-size: 12px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

th,
td {
  text-align: left;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
}

th {
  color: var(--text-muted);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
}

tr:hover td {
  background: rgba(127, 127, 127, 0.06);
}

.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  background: rgba(148, 163, 184, 0.2);
}

.badge-success {
  background: rgba(34, 197, 94, 0.18);
  color: var(--success);
}

.badge-warning {
  background: rgba(245, 158, 11, 0.18);
  color: var(--warning);
}

.badge-danger {
  background: rgba(239, 68, 68, 0.18);
  color: var(--danger);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  max-width: 640px;
}

.form-grid.full,
.form-field.full {
  grid-column: 1 / -1;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}

.form-field label {
  color: var(--text-muted);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.error-banner {
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid var(--danger);
  color: var(--danger);
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 12px;
  font-size: 14px;
}

.success-banner {
  background: rgba(34, 197, 94, 0.12);
  border: 1px solid var(--success);
  color: var(--success);
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 12px;
  font-size: 14px;
}

.empty-state {
  color: var(--text-muted);
  padding: 24px;
  text-align: center;
}

.centered-screen {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
}

.auth-card {
  width: 380px;
}

.auth-card h1 {
  font-size: 20px;
  margin-bottom: 4px;
}

.auth-card p {
  color: var(--text-muted);
  font-size: 13px;
  margin-top: 0;
}

.line-items-table input {
  width: 100%;
}

.tab-row {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.tab {
  padding: 8px 4px;
  border: none;
  background: none;
  color: var(--text-muted);
  border-bottom: 2px solid transparent;
  font-size: 14px;
}

.tab.active {
  color: var(--text);
  border-bottom-color: var(--accent);
}

/* Printable documents (quotes/orders/invoices) - see PrintOverlay and
   PrintableDocument. On screen this is a full-page preview overlay; the
   @media print block below hides everything else so window.print() /
   "Save as PDF" captures only the document itself, in both the Tauri
   webview and a Team Workspace browser tab. Colors here are deliberately
   plain black-on-white rather than the app's --text/--bg tokens, since a
   printed business document shouldn't follow dark mode. */
.print-overlay {
  position: fixed;
  inset: 0;
  background: #fff;
  color: #111;
  z-index: 1000;
  overflow: auto;
}

.print-overlay-bar {
  position: sticky;
  top: 0;
  display: flex;
  gap: 8px;
  padding: 12px 24px;
  background: #f2f2f2;
  border-bottom: 1px solid #ddd;
}

.print-overlay-page {
  max-width: 800px;
  margin: 0 auto;
  padding: 40px 32px;
}

.print-doc-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 28px;
}

.print-doc-business {
  font-weight: 700;
  font-size: 18px;
}

.print-doc-logo {
  max-width: 64px;
  max-height: 64px;
  object-fit: contain;
}

.print-doc-muted {
  color: #666;
  font-size: 13px;
}

.print-doc-heading {
  text-align: right;
}

.print-doc-heading h1 {
  margin: 0 0 4px 0;
  font-size: 20px;
}

.print-doc-parties {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 28px;
}

.print-doc-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #888;
}

.print-doc-strong {
  font-weight: 600;
}

.print-doc-dates {
  text-align: right;
  font-size: 13px;
}

.print-doc-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 16px;
}

.print-doc-table th,
.print-doc-table td {
  text-align: left;
  padding: 6px 8px;
  border-bottom: 1px solid #ddd;
  font-size: 13px;
}

.print-doc-table th {
  font-size: 11px;
  text-transform: uppercase;
  color: #888;
}

.print-doc-totals {
  margin-left: auto;
  max-width: 280px;
  font-size: 13px;
}

.print-doc-totals > div {
  display: flex;
  justify-content: space-between;
  padding: 2px 0;
}

.print-doc-total-line {
  font-weight: 700;
  border-top: 1px solid #333;
  margin-top: 4px;
  padding-top: 4px !important;
}

.print-doc-notes {
  margin-top: 28px;
  font-size: 13px;
}

/* Business Rules / Workflow Automation visual redesign - shared chrome
   both builders use: a breadcrumb+title header with action buttons, and
   numbered sections. Kept generic (not rule- or workflow- specific) so
   both builders share one set of classes. */
.builder-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
  gap: 16px;
}

.builder-breadcrumb {
  color: var(--text-muted);
  font-size: 12px;
  margin-bottom: 2px;
}

.builder-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.builder-title-row h2 {
  margin: 0;
}

.builder-subtitle {
  color: var(--text-muted);
  font-size: 13px;
  margin: 4px 0 0 0;
  max-width: 640px;
}

.builder-header-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.builder-layout {
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: 16px;
  align-items: start;
}

.builder-section {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
  margin-bottom: 16px;
}

.builder-section-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
  margin-bottom: 12px;
}

.step-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 999px;
  background: var(--accent);
  color: var(--accent-contrast);
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.builder-row-card {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px 12px;
  margin-bottom: 8px;
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
  background: var(--bg);
}

.builder-row-remove {
  margin-left: auto;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 16px;
  line-height: 1;
  padding: 4px;
  cursor: pointer;
}

.builder-row-remove:hover {
  color: var(--danger);
}

.builder-and-divider {
  text-align: center;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.05em;
  margin: -2px 0 6px 0;
}

/* One level of nested OR-grouping (migration 0020) - a dashed box around
   the conditions that get OR'd together into one sub-unit before that
   unit participates in the rule/workflow's top-level match_type. */
.builder-or-group {
  border: 1px dashed var(--accent);
  border-radius: 8px;
  padding: 10px 12px 4px 12px;
  margin-bottom: 8px;
  background: var(--bg-elevated);
}

.builder-or-group-label {
  color: var(--accent);
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 8px;
}

.builder-or-group .builder-row-card {
  background: var(--bg);
}

.builder-summary-panel {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
  position: sticky;
  top: 0;
}

.builder-summary-panel h4 {
  margin: 0 0 12px 0;
  font-size: 13px;
}

.builder-summary-panel h4:not(:first-child) {
  margin-top: 18px;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;
  padding: 5px 0;
  border-bottom: 1px solid var(--border);
}

.summary-row:last-child {
  border-bottom: none;
}

.summary-row .label {
  color: var(--text-muted);
}

.summary-row .value {
  text-align: right;
  font-weight: 600;
}

.legend-group {
  margin-bottom: 10px;
}

.legend-group-title {
  color: var(--text-muted);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 4px;
}

/* Business Rule Board 2.0 (issue #194): the IF -> THEN lane board that
   replaces the old numbered-steps form. One .rule-lane-set per rule in a
   chain (the "if" rule, plus any linked "else_if"/"else" siblings), laid
   out left to right so branching reads the same way a flowchart would.
   Drag-and-drop reorders/groups cards within a lane - there's no existing
   list-drag primitive in this codebase (the Shared Visual Builder
   Framework's own drag hook is for free-form x/y canvas nodes, a different
   shape entirely), so this is a small, self-contained native HTML5
   drag-and-drop implementation scoped to this one screen. */
.rule-board {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.rule-chain {
  display: flex;
  gap: 16px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.rule-lane-set {
  flex: 0 0 auto;
  width: 420px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.rule-lane-set-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.rule-branch-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.rule-branch-badge.if {
  background: var(--accent);
  color: var(--accent-contrast);
}

.rule-branch-badge.else_if {
  background: var(--warning, #b8860b);
  color: #fff;
}

.rule-branch-badge.else {
  background: var(--text-muted);
  color: var(--bg);
}

.rule-lane {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  padding: 12px;
  min-height: 80px;
}

.rule-lane-title {
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.rule-lane-card {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  margin-bottom: 8px;
}

.rule-lane-card.dragging {
  opacity: 0.4;
}

.rule-lane-card .rule-drag-handle {
  cursor: grab;
  color: var(--text-muted);
  padding: 10px 2px;
  user-select: none;
  flex-shrink: 0;
}

.rule-lane-card .builder-row-card,
.rule-lane-card .builder-or-group {
  flex: 1;
  margin-bottom: 0;
}

.rule-lane-dropzone {
  height: 10px;
  margin: -4px 0;
  border-radius: 4px;
}

.rule-lane-dropzone.drop-over {
  background: var(--accent);
  opacity: 0.35;
}

.rule-conflict-banner {
  border: 1px solid var(--danger);
  border-radius: 10px;
  padding: 12px 14px;
  background: color-mix(in srgb, var(--danger) 10%, transparent);
  font-size: 13px;
}

.rule-conflict-banner ul {
  margin: 6px 0 0 0;
  padding-left: 18px;
}

.rule-template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
  margin: 12px 0;
}

.rule-template-card {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 14px;
  background: var(--bg-elevated);
  cursor: pointer;
  text-align: left;
}

.rule-template-card:hover {
  border-color: var(--accent);
}

.rule-template-card h4 {
  margin: 0 0 6px 0;
  font-size: 14px;
}

.rule-template-card p {
  margin: 0;
  font-size: 12px;
  color: var(--text-muted);
}

.rule-ifthen-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  align-items: start;
}

.rule-ifthen-row > .rule-lane {
  flex: 1 1 280px;
}

/* Screen Builder 2.0 (issue #195, 5a): palette | canvas | inspector/layers
   three-pane layout, same drag/drop visual language as the business rule
   lane board above (a drag handle, a dashed drop-zone that lights up on
   hover) applied to a nested 12-column grid instead of a flat lane list. */
.page-builder-layout {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  flex-wrap: wrap;
}

.page-builder-palette {
  flex: 0 0 200px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-height: 640px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  background: var(--bg-elevated);
}

.page-builder-palette-group-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.page-builder-palette-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  cursor: grab;
  font-size: 13px;
  margin-bottom: 6px;
  user-select: none;
}

.page-builder-palette-item.dragging {
  opacity: 0.4;
}

.page-builder-palette-icon {
  flex-shrink: 0;
}

.page-builder-canvas-wrap {
  flex: 1 1 420px;
  min-width: 320px;
  overflow-x: auto;
  padding: 8px 0;
}

.page-builder-canvas-frame {
  border: 1px dashed var(--border);
  border-radius: 12px;
  padding: 16px;
  margin: 0 auto;
  background: var(--bg);
  transition: width 0.15s ease;
}

.page-builder-dropzone {
  height: 10px;
  margin: 4px 0;
  border-radius: 4px;
  border: 1px dashed transparent;
}

.page-builder-dropzone.drop-over {
  background: var(--accent);
  opacity: 0.35;
  border-color: var(--accent);
}

.page-builder-node {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-elevated);
  margin-bottom: 4px;
}

.page-builder-node.selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}

.page-builder-node.dragging {
  opacity: 0.4;
}

.page-builder-node-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  font-size: 12px;
  cursor: pointer;
}

.page-builder-drag-handle {
  cursor: grab;
  color: var(--text-muted);
  flex-shrink: 0;
}

.page-builder-node-icon {
  flex-shrink: 0;
}

.page-builder-node-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.page-builder-node-delete {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px 4px;
  flex-shrink: 0;
}

.page-builder-node-delete:hover {
  color: var(--danger);
}

.page-builder-node-body {
  padding: 8px 8px 2px;
  border-top: 1px dashed var(--border);
}

.page-builder-empty-hint {
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
  padding: 32px 0;
}

.page-builder-side {
  flex: 0 0 260px;
  min-width: 220px;
}

.page-builder-inspector,
.page-builder-layers {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  background: var(--bg-elevated);
  margin-bottom: 12px;
}

.page-builder-inspector-head {
  font-weight: 600;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
}

.page-builder-inspector-divider {
  border-top: 1px solid var(--border);
  margin: 14px 0;
}

.page-builder-layer-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
}

.page-builder-layer-row:hover {
  background: var(--bg);
}

.page-builder-layer-row.selected {
  background: var(--accent);
  color: var(--accent-contrast);
}

.page-builder-layer-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 900px) {
  .page-builder-palette,
  .page-builder-side {
    flex: 1 1 100%;
  }
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  padding: 3px 0;
}

.char-counter {
  text-align: right;
  color: var(--text-muted);
  font-size: 11px;
  margin-top: 2px;
}

/* Workflow Automation's visual flow canvas - a static, live rendering of
   the trigger/conditions/actions state (editing happens in the left
   panel's numbered sections, not by dragging nodes here). */
.workflow-layout {
  display: grid;
  grid-template-columns: 380px 1fr;
  gap: 16px;
  align-items: start;
}

.workflow-canvas-wrap {
  position: relative;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: auto;
  min-height: 480px;
  background-image: radial-gradient(var(--border) 1px, transparent 1px);
  background-size: 20px 20px;
}

.workflow-canvas {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px;
  transform-origin: top center;
  width: fit-content;
  margin: 0 auto;
}

.workflow-node {
  width: 320px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  overflow: hidden;
}

.workflow-node-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.workflow-node-trigger .workflow-node-head {
  background: rgba(34, 197, 94, 0.15);
  color: var(--success);
}

.workflow-node-conditions .workflow-node-head {
  background: rgba(79, 124, 255, 0.15);
  color: var(--accent);
}

.workflow-node-actions .workflow-node-head {
  background: rgba(245, 158, 11, 0.15);
  color: var(--warning);
}

.workflow-node-body {
  padding: 12px;
}

.workflow-trigger-body {
  display: flex;
  align-items: center;
  gap: 10px;
}

.workflow-trigger-icon {
  font-size: 22px;
}

.workflow-trigger-title {
  font-weight: 600;
  font-size: 14px;
}

.workflow-trigger-detail {
  color: var(--text-muted);
  font-size: 12px;
}

.workflow-condition-chip {
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 8px;
  font-size: 12px;
  margin-bottom: 6px;
}

.workflow-condition-chip:last-child {
  margin-bottom: 0;
}

.workflow-match-chip {
  display: inline-block;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
  text-align: center;
  width: 100%;
  margin: -2px 0 6px 0;
}

/* Canvas-node counterpart to .builder-or-group - the same one level of
   nested OR-grouping, drawn small inside the Conditions node. */
.workflow-or-group {
  border: 1px dashed var(--accent);
  border-radius: 6px;
  padding: 6px 6px 0 6px;
  margin-bottom: 6px;
}

.workflow-or-group-label {
  color: var(--accent);
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 4px;
}

.workflow-action-card {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 10px;
  margin-bottom: 8px;
  background: var(--bg);
}

.workflow-action-card:last-child {
  margin-bottom: 0;
}

.workflow-action-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.workflow-action-title {
  font-weight: 600;
  font-size: 13px;
}

.workflow-action-subtitle {
  color: var(--text-muted);
  font-size: 12px;
}

.workflow-connector {
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 28px;
  color: var(--text-muted);
}

.workflow-connector-line {
  width: 2px;
  flex: 1;
  background: var(--border);
}

.workflow-connector-arrow {
  font-size: 12px;
  line-height: 1;
  margin-top: -2px;
}

.workflow-end-node {
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 6px 20px;
  font-size: 12px;
  font-weight: 700;
  color: var(--text-muted);
  background: var(--bg-elevated);
}

.workflow-zoom-controls {
  position: sticky;
  float: right;
  bottom: 12px;
  margin-right: 12px;
  margin-top: -48px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: fit-content;
}

.workflow-zoom-controls .btn {
  width: 30px;
  height: 30px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* AI Agent Platform v2, Phase 5b: the Agent Team Builder's free-form
   canvas - a genuine node-drag/SVG-connector graph editor, unlike
   Workflow Automation's own fixed linear canvas above. Reuses that
   canvas's dotted-grid background and sticky zoom-controls placement,
   the only two things worth reusing from a fixed layout. */
.graph-editor-layout {
  display: grid;
  grid-template-columns: 1fr 320px;
  gap: 16px;
  align-items: start;
}

.graph-palette {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}

.graph-connect-banner {
  background: rgba(79, 124, 255, 0.12);
  border: 1px solid var(--accent);
  color: var(--accent);
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 12px;
  margin-bottom: 8px;
}

/* Shown until a freshly-created graph has its first edge - the only other
   cue that nodes need to be manually connected is the small per-node "→"
   button itself, easy to miss on first use (confirmed by a real user
   confused that a canvas full of just-added nodes looked "all unrelated"). */
.graph-hint-banner {
  background: var(--bg-elevated);
  border: 1px dashed var(--border);
  color: var(--text-muted);
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 12px;
  margin-bottom: 8px;
}

.graph-canvas-wrap {
  position: relative;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: auto;
  max-height: 640px;
  background-image: radial-gradient(var(--border) 1px, transparent 1px);
  background-size: 20px 20px;
}

.graph-canvas {
  position: relative;
  transform-origin: top left;
}

.graph-edges-svg {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
}

.graph-edges-svg g {
  pointer-events: all;
}

.graph-node {
  position: absolute;
  width: 220px;
  border: 2px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  overflow: hidden;
  cursor: grab;
  user-select: none;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
}

.graph-node:active {
  cursor: grabbing;
}

.graph-node-selected {
  box-shadow: 0 0 0 2px var(--accent);
}

.graph-node-head {
  padding: 6px 10px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.graph-node-body {
  padding: 8px 10px;
}

.graph-node-body strong {
  display: block;
  font-size: 13px;
}

.graph-node-body small {
  display: block;
  color: var(--text-muted);
  font-size: 11px;
  margin-top: 2px;
}

.graph-node-connect {
  position: absolute;
  right: -12px;
  bottom: -12px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  color: var(--accent);
  cursor: pointer;
  font-size: 12px;
  line-height: 1;
}

.graph-zoom-controls {
  position: sticky;
  float: right;
  bottom: 12px;
  margin-right: 12px;
  margin-top: -48px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: fit-content;
}

.graph-zoom-controls .btn {
  width: 30px;
  height: 30px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Ontology Explorer visual language (issue #339, extended for Object 360
   issue #352): the Dependency Explorer's and Object 360's own node shape
   (a small colored dot, not a boxed card), so these are scoped new
   classes rather than edits to .graph-node/.graph-canvas above - those
   stay exactly as every other VisualBuilderCanvas consumer (Agent Team
   Builder, Workflow Studio, Business Rule Board) already depends on. */
.ontology-canvas-wrap {
  background-color: var(--ontology-canvas-bg);
  background-image: radial-gradient(var(--ontology-canvas-grid) 1.5px, transparent 1.5px);
  background-size: 24px 24px;
}

.ontology-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 14px;
  padding: 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 999px;
  width: fit-content;
  max-width: 100%;
}

.ontology-legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  white-space: nowrap;
  border: 0;
  background: transparent;
  border-radius: 999px;
  padding: 5px 11px;
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease, opacity 0.15s ease;
}

.ontology-legend-item:hover {
  background: rgba(127, 127, 127, 0.1);
  color: var(--text);
}

.ontology-legend-item-off {
  opacity: 0.4;
}

@keyframes ontology-dot-in {
  from {
    opacity: 0;
    transform: scale(0.3);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.ontology-node-dot {
  position: absolute;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
  animation: ontology-dot-in 0.3s cubic-bezier(0.34, 1.3, 0.64, 1) both;
  transition: left 0.4s cubic-bezier(0.34, 1.2, 0.64, 1), top 0.4s cubic-bezier(0.34, 1.2, 0.64, 1), transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.15s ease,
    opacity 0.2s ease;
}

.ontology-node-dot::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: radial-gradient(circle at 32% 28%, rgba(255, 255, 255, 0.55), rgba(255, 255, 255, 0) 65%);
  pointer-events: none;
}

.ontology-node-dot:hover {
  transform: scale(1.35);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.3);
  z-index: 3;
}

.ontology-node-dimmed {
  opacity: 0.18;
}

.ontology-node-dimmed:hover {
  opacity: 0.5;
}

@keyframes ontology-root-pulse {
  0%,
  100% {
    box-shadow: 0 0 0 3px var(--accent), 0 2px 6px rgba(0, 0, 0, 0.3);
  }
  50% {
    box-shadow: 0 0 0 5px var(--accent), 0 2px 10px rgba(0, 0, 0, 0.3);
  }
}

.ontology-node-root {
  cursor: default;
  animation: ontology-dot-in 0.3s cubic-bezier(0.34, 1.3, 0.64, 1) both, ontology-root-pulse 2.4s ease-in-out 0.3s infinite;
}

.ontology-node-root:hover {
  transform: none;
}

.ontology-node-inspecting:not(.ontology-node-root) {
  box-shadow: 0 0 0 2px var(--text), 0 2px 8px rgba(0, 0, 0, 0.25);
  transform: scale(1.15);
}

@keyframes ontology-card-in {
  from {
    opacity: 0;
    transform: translateY(6px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.ontology-inspector-card {
  position: absolute;
  top: 16px;
  right: 16px;
  width: 290px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 16px;
  box-shadow: 0 20px 44px rgba(0, 0, 0, 0.22), 0 2px 8px rgba(0, 0, 0, 0.1);
  z-index: 5;
  animation: ontology-card-in 0.2s cubic-bezier(0.2, 0.8, 0.2, 1) both;
}

.ontology-inspector-head {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-bottom: 10px;
}

.ontology-inspector-head strong {
  font-size: 15px;
}

.ontology-inspector-subtitle {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-top: 2px;
}

.ontology-inspector-definition {
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--text-muted);
  margin: 0 0 12px;
}

.ontology-inspector-actions {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
}

.ontology-inspector-actions .btn {
  flex: 1;
  padding: 7px 8px;
  font-size: 12px;
  border-radius: 8px;
}

.ontology-inspector-groups {
  max-height: 260px;
  overflow-y: auto;
}

.ontology-inspector-group {
  padding-top: 8px;
  margin-top: 8px;
  border-top: 1px solid var(--border);
}

.ontology-inspector-group:first-child {
  padding-top: 0;
  margin-top: 0;
  border-top: 0;
}

.ontology-inspector-group-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-bottom: 5px;
}

.ontology-inspector-bullet {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  border: 0;
  border-radius: 6px;
  background: none;
  padding: 4px 6px;
  margin: 0 -6px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.ontology-inspector-bullet:hover {
  background: rgba(127, 127, 127, 0.1);
  color: var(--accent);
}

.graph-property-panel {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  padding: 12px;
  min-height: 200px;
}

.graph-condition-row {
  display: flex;
  gap: 6px;
  margin-bottom: 6px;
  align-items: center;
}

.graph-condition-row input,
.graph-condition-row select {
  flex: 1;
  min-width: 0;
}

.graph-router-branch {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px;
  margin-bottom: 8px;
}

.graph-run-row {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px;
  margin-bottom: 8px;
}

@media print {
  body > *:not(#print-root) {
    display: none !important;
  }
  .no-print {
    display: none !important;
  }
  .print-overlay {
    position: static;
    inset: auto;
  }
  .print-overlay-page {
    max-width: none;
    padding: 0;
  }
}

/* Help tab - a category/topic list on the left, article content on the
   right, matching the search-then-drill-in shape of every other admin
   list+detail screen rather than inventing a new layout pattern. */
.help-admin-layout {
  display: grid;
  grid-template-columns: 260px 1fr;
  gap: 20px;
  align-items: start;
}
.help-admin-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.help-admin-cat-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  margin: 14px 0 6px;
}
.help-admin-cat-label:first-child {
  margin-top: 0;
}
.help-admin-topic-btn {
  display: block;
  width: 100%;
  text-align: left;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  padding: 9px 10px;
  font-size: 13.5px;
  color: var(--text);
  cursor: pointer;
}
.help-admin-topic-btn:hover {
  background: var(--bg-elevated);
}
.help-admin-topic-btn.active {
  background: var(--bg-elevated);
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 600;
}
.help-admin-toc {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 18px 0 0;
  padding: 14px 16px;
}
.help-admin-toc div {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  margin-bottom: 2px;
}
.help-admin-toc a {
  font-size: 13px;
  color: var(--text-muted);
}
.help-admin-toc a:hover {
  color: var(--accent);
}
.help-admin-article h4 {
  font-size: 16px;
  margin: 26px 0 10px;
  scroll-margin-top: 16px;
}
.help-admin-article h4:first-child {
  margin-top: 0;
}
.help-admin-article p,
.help-admin-article ul,
.help-admin-article ol {
  font-size: 14px;
  line-height: 1.75;
  color: var(--text);
  margin: 0 0 14px;
}
.help-admin-article ul,
.help-admin-article ol {
  padding-left: 20px;
}
.help-admin-article li {
  margin-bottom: 5px;
}
.help-admin-article code {
  font-family: var(--font-mono, ui-monospace, monospace);
  background: rgba(127, 127, 127, 0.15);
  border-radius: 4px;
  padding: 1px 5px;
  font-size: 12.5px;
}
.help-admin-article pre {
  background: #0d0f13;
  color: #d7f7ff;
  border-radius: 10px;
  padding: 14px 16px;
  overflow-x: auto;
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 12.5px;
  line-height: 1.7;
  margin: 0 0 14px;
}
.help-admin-article pre code {
  background: transparent;
  padding: 0;
  color: inherit;
}
.help-admin-article .help-note {
  background: rgba(99, 102, 241, 0.12);
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 13px;
  margin: 0 0 14px;
}
.help-admin-related {
  margin-top: 32px;
  padding-top: 18px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
@media (max-width: 800px) {
  .help-admin-layout {
    grid-template-columns: 1fr;
  }
}

/* ListTable (Runtime UX Modernization, issue #198): the shared
   dynamic-column table - density, column resize/reorder/picker chrome,
   and the table<->card mobile collapse. See components/ListTable.tsx. */
.list-table-wrap {
  width: 100%;
}

.list-table-toolbar {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.segmented {
  display: inline-flex;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}
.segmented-btn {
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
  padding: 5px 10px;
  cursor: pointer;
}
.segmented-btn + .segmented-btn {
  border-left: 1px solid var(--border);
}
.segmented-btn.active {
  background: var(--accent);
  color: #fff;
}

.list-table-column-picker {
  position: absolute;
  right: 0;
  top: calc(100% + 4px);
  z-index: 10;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px;
  min-width: 180px;
  max-height: 320px;
  overflow-y: auto;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}
.list-table-column-picker-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}
.list-table-column-picker-row:hover {
  background: rgba(127, 127, 127, 0.08);
  border-radius: 4px;
}

.list-table-table th {
  position: relative;
  user-select: none;
}
.list-table-resize-handle {
  position: absolute;
  /* Stays fully inside its own <th> box (not straddling the boundary)
     so it never sits under the next column's header, which would
     otherwise win hit-testing since later DOM siblings paint on top. */
  right: 0;
  top: 0;
  bottom: 0;
  width: 6px;
  cursor: col-resize;
  touch-action: none;
}
.list-table-resize-handle:hover {
  background: var(--accent);
  opacity: 0.4;
}

/* Density: default row padding already matches "comfortable" (see the
   base th/td rule); compact/dense tighten it. */
.list-table-compact .list-table-table th,
.list-table-compact .list-table-table td {
  padding: 5px 10px;
}
.list-table-dense .list-table-table th,
.list-table-dense .list-table-table td {
  padding: 2px 8px;
  font-size: 13px;
}

/* Mobile card collapse: both markups stay in the DOM; only one shows at
   a time, so no resize listener is needed. */
.list-table-cards {
  display: none;
}
@media (max-width: 720px) {
  .list-table-table {
    display: none;
  }
  .list-table-cards {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
}
.list-table-card {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  cursor: pointer;
  background: var(--bg-elevated);
}
.list-table-card-field {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}
.list-table-card-label {
  color: var(--text-muted);
}
.list-table-card-value {
  text-align: right;
}

/* QuickPreviewDrawer: a right-side slide-in, not a centered .modal - it
   sits alongside the list rather than blocking it. */
.quick-preview-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
}
.quick-preview-drawer {
  width: 380px;
  max-width: 90vw;
  height: 100%;
  background: var(--bg-elevated);
  border-left: 1px solid var(--border);
  padding: 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: quick-preview-slide-in 0.15s ease-out;
}
@keyframes quick-preview-slide-in {
  from {
    transform: translateX(16px);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
.quick-preview-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}
.quick-preview-fields {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
}
.quick-preview-field {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border);
}
.quick-preview-label {
  color: var(--text-muted);
}
.quick-preview-value {
  text-align: right;
}
.quick-preview-footer {
  display: flex;
  justify-content: flex-end;
}

/* RecordHeader (Runtime UX Modernization, issue #198): the one
   title/subtitle/status/owner/record-number/avatar + up to 4 key
   attributes header every detail/edit view uses now. */
.record-header {
  margin-bottom: 16px;
}
.record-header-top {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}
.record-header-avatar {
  width: 40px;
  height: 40px;
  border-radius: 999px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-weight: 700;
}
.record-header-attributes {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  margin-top: 10px;
}
.record-header-attribute {
  display: flex;
  flex-direction: column;
  font-size: 13px;
}
.record-header-attribute-label {
  color: var(--text-muted);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

/* Sticky command bar: the Back/Edit toolbar atop a long record detail
   page - a variant of .toolbar that stays visible while scrolling,
   without changing every other .toolbar in the app. */
.record-command-bar {
  position: sticky;
  top: 0;
  z-index: 3;
  background: var(--bg);
  padding: 8px 0;
  margin-bottom: 8px;
}

/* InlineEditField (Runtime UX Modernization, issue #198): a read-only
   detail field that's click-to-edit when the viewer has write access. */
.inline-edit-value {
  cursor: pointer;
  border-radius: 4px;
  padding: 2px 4px;
  margin: -2px -4px;
}
.inline-edit-value:hover,
.inline-edit-value:focus-visible {
  background: rgba(127, 127, 127, 0.1);
  outline: 1px dashed var(--border);
}
.inline-edit-active select,
.inline-edit-active input,
.inline-edit-active textarea {
  max-width: 220px;
}
.inline-edit-error {
  color: var(--danger);
  font-size: 12px;
  margin-bottom: 4px;
}

/* DashboardGrid (Runtime UX Modernization, issue #198): the shared
   widget grid - one widget-positioning implementation for both the live
   Dashboard (view mode) and the dashboard-layout editor's canvas (edit
   mode, with drag/resize handles). See components/DashboardGrid.tsx. */
.dashboard-grid-item {
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}
.dashboard-grid-item.editable {
  border: 1px solid transparent;
  border-radius: 8px;
}
.dashboard-grid-item.editable:hover {
  border-color: var(--border);
}
.dashboard-grid-item.editable.selected {
  border-color: var(--accent);
}
.dashboard-grid-item-handle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-muted);
  background: var(--bg-elevated);
  border-radius: 8px 8px 0 0;
  cursor: move;
  touch-action: none;
  user-select: none;
}
.dashboard-grid-item-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.dashboard-grid-item-resize {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 14px;
  height: 14px;
  cursor: nwse-resize;
  touch-action: none;
}
.dashboard-grid-item-resize::after {
  content: "";
  position: absolute;
  right: 3px;
  bottom: 3px;
  width: 6px;
  height: 6px;
  border-right: 2px solid var(--border);
  border-bottom: 2px solid var(--border);
}

/* Lookup Custom Field Type: search-as-you-type combobox, used wherever a
   field_type === "lookup" field renders on a create/edit form (see
   CustomFieldsSection.tsx's useCustomFieldElements). */
.lookup-picker {
  position: relative;
}
.lookup-picker-input-row {
  display: flex;
  align-items: center;
  gap: 4px;
}
.lookup-picker-input-row input {
  flex: 1;
}
.lookup-picker-clear {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 2px 6px;
}
.lookup-picker-clear:hover {
  color: var(--text);
}
.lookup-picker-menu {
  position: absolute;
  z-index: 20;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  max-height: 240px;
  overflow-y: auto;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.25);
  list-style: none;
  margin: 0;
  padding: 4px 0;
}
.lookup-picker-option {
  padding: 6px 10px;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.lookup-picker-option.active {
  background: var(--accent);
  color: var(--accent-contrast);
}
.lookup-picker-option-empty {
  padding: 6px 10px;
  color: var(--text-muted);
  cursor: default;
}
""")

# File: desktop/src/styles/tokens.css
write_file("desktop/src/styles/tokens.css", """/* Desktop UI Foundation (PR 1) — design tokens ported from the online demo's
   Modern UI token system (root styles.css, "Modern UI PR-A" block).
   Light is the default theme (user decision); dark via [data-theme="dark"]
   on <html> or the OS prefers-color-scheme preference when no explicit
   choice is set. Token names match the demo exactly so future
   demo<->desktop ports are mechanical.

   Deliberate deviations from the demo block (documented, not drift):
   - The demo's legacy aliases (--bg/--panel/--text/--line/--brand/--brand2/
     --danger mapped to the new tokens, and --muted mapped to
     --muted-foreground) are NOT ported: the desktop's own styles.css keeps
     its --bg/--text/--border/--text-muted/--accent names with desktop
     values (declared after this import, so they win), and the --muted
     alias would break shadcn --muted-as-surface semantics used by ui.css.
     Only --warn -> --warning is kept as a harmless convenience.
   - Dark selectors are html-qualified (html[data-theme="dark"]) so they
     beat :root on equal specificity regardless of source order. */

:root {
  --background: #ffffff;
  --foreground: #09090b;
  --card: #ffffff;
  --card-foreground: #09090b;
  --popover: #ffffff;
  --popover-foreground: #09090b;
  --primary: #635bff;
  --primary-foreground: #ffffff;
  --secondary: #f4f4f5;
  --secondary-foreground: #18181b;
  --muted: #f4f4f5;
  --muted-foreground: #71717a;
  --destructive: #ef4444;
  --destructive-foreground: #ffffff;
  --success: #16a34a;
  --warning: #d97706;
  --info: #3b82f6;
  --border: #e4e4e7;
  --input: #e4e4e7;
  --ring: #635bff66;
  --sidebar: #111827;
  --sidebar-foreground: #f4f4f5;
  --chart-1: #635bff;
  --chart-2: #0f9d76;
  --chart-3: #f59e0b;
  --chart-4: #ef4444;
  --chart-5: #3b82f6;
  --chart-6: #8b5cf6;
  --radius: 0.625rem;
  --radius-sm: calc(var(--radius) - 4px);
  --radius-md: calc(var(--radius) - 2px);
  --radius-lg: calc(var(--radius) + 2px);
  --radius-xl: calc(var(--radius) + 4px);
  --radius-full: 999px;
  --shadow-sm: 0 1px 2px rgba(9, 9, 11, 0.05);
  --shadow: 0 1px 3px rgba(9, 9, 11, 0.08);
  --shadow-md: 0 4px 12px rgba(9, 9, 11, 0.08);
  --shadow-lg: 0 16px 40px rgba(9, 9, 11, 0.12);
  --font-sans: Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-mono: ui-monospace, SFMono-Regular, Menlo, monospace;
  --text-xs: 0.75rem;
  --text-sm: 0.875rem;
  --text-base: 1rem;
  --text-lg: 1.125rem;
  --text-xl: 1.25rem;
  --text-2xl: 1.5rem;
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --motion-fast: 120ms;
  --motion-base: 200ms;
  --motion-slow: 320ms;
  --warn: var(--warning);
}

/* Explicit dark theme — html-qualified to outrank :root. */
html[data-theme="dark"] {
  --background: #0a0f1e;
  --foreground: #f4f4f5;
  --card: #111a30;
  --card-foreground: #f4f4f5;
  --popover: #16203a;
  --popover-foreground: #f4f4f5;
  --primary: #8b5cf6;
  --primary-foreground: #ffffff;
  --secondary: #1e2a4a;
  --secondary-foreground: #f4f4f5;
  --muted: #1a2440;
  --muted-foreground: #9aa4bd;
  --destructive: #f87171;
  --destructive-foreground: #ffffff;
  --success: #4ade80;
  --warning: #fbbf24;
  --info: #60a5fa;
  --border: #26314f;
  --input: #26314f;
  --ring: #8b5cf680;
  --sidebar: #0a0f1e;
  --sidebar-foreground: #f4f4f5;
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4);
  --shadow: 0 1px 3px rgba(0, 0, 0, 0.45);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.45);
  --shadow-lg: 0 16px 40px rgba(0, 0, 0, 0.5);
}

/* OS dark preference applies only when the user hasn't made an explicit
   [data-theme] choice. */
@media (prefers-color-scheme: dark) {
  html:not([data-theme]) {
    --background: #0a0f1e;
    --foreground: #f4f4f5;
    --card: #111a30;
    --card-foreground: #f4f4f5;
    --popover: #16203a;
    --popover-foreground: #f4f4f5;
    --primary: #8b5cf6;
    --primary-foreground: #ffffff;
    --secondary: #1e2a4a;
    --secondary-foreground: #f4f4f5;
    --muted: #1a2440;
    --muted-foreground: #9aa4bd;
    --destructive: #f87171;
    --destructive-foreground: #ffffff;
    --success: #4ade80;
    --warning: #fbbf24;
    --info: #60a5fa;
    --border: #26314f;
    --input: #26314f;
    --ring: #8b5cf680;
    --sidebar: #0a0f1e;
    --sidebar-foreground: #f4f4f5;
    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4);
    --shadow: 0 1px 3px rgba(0, 0, 0, 0.45);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.45);
    --shadow-lg: 0 16px 40px rgba(0, 0, 0, 0.5);
  }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
""")

# File: desktop/src/vite-env.d.ts
write_file("desktop/src/vite-env.d.ts", """/// <reference types="vite/client" />
""")

# Copy logos
for name in ["lanesraos-logo-light.png", "lanesraos-logo-dark.png", "lanesraos-logo-icon.png"]:
    src = os.path.join(REPO, "media", name)
    dst = os.path.join(REPO, "desktop/src/assets", name)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if os.path.exists(src):
        shutil.copy2(src, dst)
        print(f"Copied {name}")

print("Done")

# Regenerate Tauri icons from logo (requires PIL)
try:
    from PIL import Image
    logo_path = os.path.join(REPO, "media/lanesraos-logo-icon.png")
    if os.path.exists(logo_path):
        logo = Image.open(logo_path)
        icons_dir = os.path.join(REPO, "desktop/src-tauri/icons")
        for size, name in [(32, "32x32.png"), (128, "128x128.png"), (256, "128x128@2x.png")]:
            resized = logo.resize((size, size), Image.LANCZOS)
            resized.save(os.path.join(icons_dir, name))
            print(f"Generated {name}")
except Exception as e:
    print(f"Icon generation skipped: {e}")
