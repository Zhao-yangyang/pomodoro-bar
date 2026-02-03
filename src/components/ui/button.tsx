"use client";

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center rounded-full text-sm font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-ring)] disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        primary:
          "border border-[var(--color-accent)]/60 bg-[var(--color-accent)] text-[color:var(--color-paper)] shadow-[0_12px_30px_-16px_rgba(194,106,58,0.9)] hover:-translate-y-0.5",
        secondary:
          "border border-[var(--color-paper-edge)]/80 bg-transparent text-[var(--color-paper-ink)] hover:-translate-y-0.5 hover:border-[var(--color-accent)]/60",
        ghost:
          "border border-[var(--color-paper-edge)]/60 bg-transparent text-[var(--color-muted)] hover:-translate-y-0.5 hover:text-[var(--color-paper-ink)]",
      },
      size: {
        default: "px-3 py-2",
        sm: "px-2.5 py-1.5 text-xs",
        icon: "h-9 w-9",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "default",
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => (
    <button
      ref={ref}
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  ),
);
Button.displayName = "Button";

export { Button, buttonVariants };
