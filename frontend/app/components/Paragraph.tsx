import React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "~/lib/utils";

/**
 * Paragraph - Theme-aware paragraph component with size and color variants.
 *
 * Provides consistent typography across all themes with support for
 * different sizes, colors, and alignment options.
 */

const paragraphVariants = cva(
  "font-sans text-gray-700 dark:text-gray-300 leading-relaxed",
  {
    variants: {
      size: {
        xs: "text-xs",
        sm: "text-sm",
        base: "text-base",
        lg: "text-lg",
        xl: "text-xl",
      },
      variant: {
        default: "text-gray-700 dark:text-gray-300",
        muted: "text-gray-600 dark:text-gray-400",
        subtle: "text-gray-500 dark:text-gray-500",
        primary: "text-primary-600 dark:text-primary-400",
        success: "text-green-600 dark:text-green-400",
        warning: "text-yellow-600 dark:text-yellow-400",
        danger: "text-red-600 dark:text-red-400",
      },
      align: {
        left: "text-left",
        center: "text-center",
        right: "text-right",
        justify: "text-justify",
      },
      weight: {
        normal: "font-normal",
        medium: "font-medium",
        semibold: "font-semibold",
        bold: "font-bold",
      },
    },
    defaultVariants: {
      size: "base",
      variant: "default",
      align: "left",
      weight: "normal",
    },
  }
);

export interface ParagraphProps
  extends React.HTMLAttributes<HTMLParagraphElement>,
    VariantProps<typeof paragraphVariants> {
  /**
   * Custom element to render (defaults to p)
   */
  as?: "p" | "span" | "div";
}

const Paragraph = React.forwardRef<HTMLParagraphElement, ParagraphProps>(
  ({ className, size, variant, align, weight, as, ...props }, ref) => {
    const Component = as || "p";

    return React.createElement(Component, {
      ref,
      className: cn(paragraphVariants({ size, variant, align, weight, className })),
      ...props,
    });
  }
);
Paragraph.displayName = "Paragraph";

export { Paragraph, paragraphVariants };
