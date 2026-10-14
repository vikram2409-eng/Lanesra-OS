import * as React from "react";

import { cn } from "./cn";

function Skeleton({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div aria-hidden="true" className={cn("ui-skeleton", className)} {...props} />;
}

export { Skeleton };
