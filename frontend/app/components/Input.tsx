import React from "react";
import { cn } from "~/lib/utils";

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  monospace?: boolean;
}

/**
 * A styled input component with optional monospace font.
 * @param {boolean} [monospace=false] - Whether to use a monospace font.
 */
const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, monospace, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          "flex h-10 w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900",
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
Input.displayName = "Input";

export { Input };
