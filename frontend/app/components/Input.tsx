import React, { useState, useEffect } from "react";
import { Copy, Check } from "lucide-react";
import { cn } from "~/lib/utils";

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  monospace?: boolean;
  copyable?: boolean;
}

/**
 * A styled input component with optional monospace font and copy button.
 * @param {boolean} [monospace=false] - Whether to use a monospace font.
 * @param {boolean} [copyable=false] - Whether to show a copy button.
 */
const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, monospace, copyable, type, ...props }, ref) => {
    const [flashKey, setFlashKey] = useState(0);
    const [isCopied, setIsCopied] = useState(false);
    const internalRef = React.useRef<HTMLInputElement>(null);

    // Use the passed ref if available, otherwise use internal ref
    const resolvedRef = (node: HTMLInputElement | null) => {
      internalRef.current = node;
      if (typeof ref === "function") {
        ref(node);
      } else if (ref && typeof ref === "object") {
        ref.current = node;
      }
    };

    const handleCopy = async () => {
      const inputElement = internalRef.current;
      if (!inputElement) {
        console.error("Input element not found");
        return;
      }

      const text = inputElement.value || "";
      if (!text) {
        console.error("Input has no value to copy");
        return;
      }

      try {
        await navigator.clipboard.writeText(text);
        setFlashKey((prev) => prev + 1);
        setIsCopied(true);
      } catch (err) {
        console.error("Failed to copy to clipboard:", err);
      }
    };

    const handleAnimationEnd = () => {
      setFlashKey(0);
    };

    useEffect(() => {
      if (!isCopied) return;
      const timer = setTimeout(() => {
        setIsCopied(false);
      }, 1000);
      return () => clearTimeout(timer);
    }, [isCopied]);

    if (copyable) {
      return (
        <div className="relative flex items-center">
          <input
            type={type}
            className={cn(
              "flex h-10 w-full rounded-(--radius-component) border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900",
              "ring-offset-white",
              "placeholder:text-gray-400",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus-visible:border-primary-500",
              "disabled:cursor-not-allowed disabled:opacity-50",
              "dark:border-gray-700 dark:bg-gray-900 dark:text-gray-50 dark:ring-offset-gray-950",
              "dark:placeholder:text-gray-500 dark:focus-visible:ring-primary-400",
              monospace ? "font-mono" : "font-sans",
              "pr-10",
              className
            )}
            style={
              flashKey > 0
                ? {
                    animation: "textFlash 0.6s ease-out",
                  }
                : undefined
            }
            ref={resolvedRef}
            onAnimationEnd={handleAnimationEnd}
            {...props}
          />
          <button
            type="button"
            onClick={handleCopy}
            className={cn(
              "absolute right-2 p-2 rounded-(--radius-component) transition-colors",
              isCopied
                ? "bg-primary-100 dark:bg-primary-900"
                : "hover:bg-gray-100 dark:hover:bg-gray-800"
            )}
            title={isCopied ? "Copied!" : "Copy to clipboard"}
          >
            {isCopied ? (
              <Check className="w-4 h-4 text-primary-600 dark:text-primary-400" />
            ) : (
              <Copy className="w-4 h-4 text-gray-600 dark:text-gray-400" />
            )}
          </button>
        </div>
      );
    }

    return (
      <input
        type={type}
        className={cn(
          "flex h-10 w-full rounded-(--radius-component) border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900",
          "ring-offset-white transition-colors",
          "placeholder:text-gray-400",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus-visible:border-primary-500",
          "disabled:cursor-not-allowed disabled:opacity-50",
          "dark:border-gray-700 dark:bg-gray-900 dark:text-gray-50 dark:ring-offset-gray-950",
          "dark:placeholder:text-gray-500 dark:focus-visible:ring-primary-400",
          monospace ? "font-mono" : "font-sans",
          className
        )}
        ref={resolvedRef}
        {...props}
      />
    );
  }
);
Input.displayName = "Input";

export { Input };
