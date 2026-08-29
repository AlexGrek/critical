import React from "react";
import { cn } from "~/lib/utils";

export interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  monospace?: boolean;
}

/**
 * A styled textarea component with optional monospace font, matching Input's styling.
 */
const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, monospace, ...props }, ref) => {
    return (
      <textarea
        className={cn(
          "flex min-h-24 w-full rounded-(--radius-component) border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900",
          "ring-offset-white transition-colors",
          "placeholder:text-gray-400",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus-visible:border-primary-500",
          "disabled:cursor-not-allowed disabled:opacity-50",
          "dark:border-gray-700 dark:bg-gray-900 dark:text-gray-50 dark:ring-offset-gray-950",
          "dark:placeholder:text-gray-500 dark:focus-visible:ring-primary-400",
          monospace ? "font-mono" : "font-sans",
          className
        )}
        ref={ref}
        {...props}
      />
    );
  }
);
Textarea.displayName = "Textarea";

export { Textarea };
