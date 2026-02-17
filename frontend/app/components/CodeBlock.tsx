import React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "~/lib/utils";

/**
 * CodeBlock - Theme-aware code display component.
 *
 * Displays code with monospace font and proper styling.
 * Supports both inline and block modes.
 * Ready for future syntax highlighting integration.
 */

const codeBlockVariants = cva(
  "font-mono bg-gray-100 dark:bg-gray-950 text-gray-900 dark:text-gray-100",
  {
    variants: {
      variant: {
        inline: "px-1.5 py-0.5 text-sm rounded-(--radius-component)",
        block: "p-4 text-sm overflow-x-auto rounded-(--radius-component-lg) border border-gray-200 dark:border-gray-800",
      },
      size: {
        xs: "text-xs",
        sm: "text-sm",
        base: "text-base",
        lg: "text-lg",
      },
    },
    defaultVariants: {
      variant: "block",
      size: "sm",
    },
  }
);

export interface CodeBlockProps
  extends React.HTMLAttributes<HTMLElement>,
    VariantProps<typeof codeBlockVariants> {
  /**
   * The code content to display
   */
  children: React.ReactNode;
  /**
   * Optional language identifier (for future syntax highlighting)
   */
  language?: string;
  /**
   * Whether to show line numbers (future feature)
   */
  showLineNumbers?: boolean;
  /**
   * Custom element to render (defaults to pre for block, code for inline)
   */
  as?: "pre" | "code" | "div";
}

const CodeBlock = React.forwardRef<HTMLElement, CodeBlockProps>(
  (
    {
      className,
      variant = "block",
      size,
      language,
      showLineNumbers = false,
      as,
      children,
      ...props
    },
    ref
  ) => {
    const Component = as || (variant === "inline" ? "code" : "pre");
    const isBlock = variant === "block";

    // For block variant, wrap children in <code> if not already
    const content = isBlock && Component === "pre" ? (
      <code className="font-mono">{children}</code>
    ) : (
      children
    );

    return React.createElement(
      Component,
      {
        ref,
        className: cn(codeBlockVariants({ variant, size, className })),
        "data-language": language,
        "data-line-numbers": showLineNumbers,
        ...props,
      },
      content
    );
  }
);
CodeBlock.displayName = "CodeBlock";

/**
 * Convenience component for inline code
 */
const InlineCode = React.forwardRef<
  HTMLElement,
  Omit<CodeBlockProps, "variant">
>((props, ref) => <CodeBlock ref={ref} variant="inline" {...props} />);
InlineCode.displayName = "InlineCode";

export { CodeBlock, InlineCode, codeBlockVariants };
