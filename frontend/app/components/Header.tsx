import React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "~/lib/utils";

/**
 * Header - Theme-aware heading component with semantic levels (H1-H6).
 *
 * Automatically adapts to all 5 themes with appropriate font sizing and weight.
 * Supports all semantic heading levels with consistent styling.
 */

const headerVariants = cva(
  "font-mono text-gray-900 dark:text-gray-50 tracking-normal",
  {
    variants: {
      level: {
        h1: "text-xl sm:text-2xl lg:text-2xl leading-tight",
        h2: "text-lg sm:text-xl lg:text-xl leading-tight",
        h3: "text-base sm:text-lg lg:text-lg leading-snug",
        h4: "text-sm sm:text-base lg:text-base leading-snug",
        h5: "text-sm sm:text-sm lg:text-base leading-normal",
        h6: "text-xs sm:text-sm lg:text-sm leading-normal",
      },
      weight: {
        thin: "font-thin",
        light: "font-light",
        normal: "font-normal",
        medium: "font-medium",
        semibold: "font-semibold",
        bold: "font-bold",
        extrabold: "font-extrabold",
      },
      align: {
        left: "text-left",
        center: "text-center",
        right: "text-right",
      },
    },
    defaultVariants: {
      level: "h1",
      weight: "light",
      align: "left",
    },
  }
);

export interface HeaderProps
  extends React.HTMLAttributes<HTMLHeadingElement>,
    VariantProps<typeof headerVariants> {
  /**
   * Semantic heading level (h1-h6)
   */
  level?: "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
  /**
   * Custom element to render (defaults to the level prop)
   */
  as?: "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
}

const Header = React.forwardRef<HTMLHeadingElement, HeaderProps>(
  ({ className, level = "h1", as, weight, align, ...props }, ref) => {
    const Component = as || level;

    return React.createElement(Component, {
      ref,
      className: cn(headerVariants({ level, weight, align, className })),
      ...props,
    });
  }
);
Header.displayName = "Header";

/**
 * Convenience components for each heading level
 */
const H1 = React.forwardRef<HTMLHeadingElement, Omit<HeaderProps, "level">>(
  (props, ref) => <Header ref={ref} level="h1" {...props} />
);
H1.displayName = "H1";

const H2 = React.forwardRef<HTMLHeadingElement, Omit<HeaderProps, "level">>(
  (props, ref) => <Header ref={ref} level="h2" {...props} />
);
H2.displayName = "H2";

const H3 = React.forwardRef<HTMLHeadingElement, Omit<HeaderProps, "level">>(
  (props, ref) => <Header ref={ref} level="h3" {...props} />
);
H3.displayName = "H3";

const H4 = React.forwardRef<HTMLHeadingElement, Omit<HeaderProps, "level">>(
  (props, ref) => <Header ref={ref} level="h4" {...props} />
);
H4.displayName = "H4";

const H5 = React.forwardRef<HTMLHeadingElement, Omit<HeaderProps, "level">>(
  (props, ref) => <Header ref={ref} level="h5" {...props} />
);
H5.displayName = "H5";

const H6 = React.forwardRef<HTMLHeadingElement, Omit<HeaderProps, "level">>(
  (props, ref) => <Header ref={ref} level="h6" {...props} />
);
H6.displayName = "H6";

export { Header, headerVariants, H1, H2, H3, H4, H5, H6 };
