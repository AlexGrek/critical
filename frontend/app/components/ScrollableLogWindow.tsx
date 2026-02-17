import React, { useEffect, useRef, useState } from "react";
import { cn } from "~/lib/utils";

/**
 * ScrollableLogWindow - Terminal-like log display component.
 *
 * Features:
 * - Always white text on black background (terminal aesthetic)
 * - Auto-scrolls to bottom when new content appears (only if already at bottom)
 * - Monospace font for consistent log formatting
 * - Theme-aware border radius
 * - Supports both controlled and uncontrolled content
 */

export interface ScrollableLogWindowProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "children"> {
  /**
   * Log lines to display (can be string[] or string with newlines)
   */
  logs?: string[] | string;
  /**
   * Custom content (alternative to logs prop)
   */
  children?: React.ReactNode;
  /**
   * Maximum height of the log window
   */
  maxHeight?: string | number;
  /**
   * Whether to show a header with title
   */
  title?: string;
  /**
   * Scroll threshold in pixels - considers "at bottom" if within this distance
   */
  scrollThreshold?: number;
}

const ScrollableLogWindow = React.forwardRef<
  HTMLDivElement,
  ScrollableLogWindowProps
>(
  (
    {
      className,
      logs,
      children,
      maxHeight = "400px",
      title,
      scrollThreshold = 10,
      ...props
    },
    ref
  ) => {
    const scrollContainerRef = useRef<HTMLDivElement>(null);
    const [isAtBottom, setIsAtBottom] = useState(true);
    const prevLogsLengthRef = useRef(0);

    // Normalize logs to array of strings
    const logLines = React.useMemo(() => {
      if (!logs) return [];
      if (Array.isArray(logs)) return logs;
      return logs.split("\n");
    }, [logs]);

    // Check if user is at bottom
    const checkIfAtBottom = () => {
      const container = scrollContainerRef.current;
      if (!container) return true;

      const { scrollTop, scrollHeight, clientHeight } = container;
      const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
      return distanceFromBottom <= scrollThreshold;
    };

    // Handle scroll events
    const handleScroll = () => {
      setIsAtBottom(checkIfAtBottom());
    };

    // Auto-scroll to bottom when new logs appear (only if already at bottom)
    useEffect(() => {
      const container = scrollContainerRef.current;
      if (!container) return;

      // If new content was added and we were at the bottom, scroll to bottom
      const logsIncreased = logLines.length > prevLogsLengthRef.current;
      prevLogsLengthRef.current = logLines.length;

      if (logsIncreased && isAtBottom) {
        // Use requestAnimationFrame to ensure DOM has updated
        requestAnimationFrame(() => {
          container.scrollTop = container.scrollHeight;
        });
      }
    }, [logLines, isAtBottom]);

    // Combine external ref with internal ref
    const combinedRef = React.useCallback(
      (node: HTMLDivElement | null) => {
        scrollContainerRef.current = node;
        if (typeof ref === "function") {
          ref(node);
        } else if (ref) {
          ref.current = node;
        }
      },
      [ref]
    );

    return (
      <div className={cn("flex flex-col relative", className)} {...props}>
        {title && (
          <div
            className="bg-gray-900 text-white px-4 py-2 font-mono text-sm font-semibold border-b border-gray-700"
            style={{
              borderTopLeftRadius: 'var(--radius-component-lg)',
              borderTopRightRadius: 'var(--radius-component-lg)',
            }}
          >
            {title}
          </div>
        )}
        <div
          ref={combinedRef}
          onScroll={handleScroll}
          className={cn(
            "overflow-y-auto overflow-x-auto",
            "bg-black text-white font-mono text-sm",
            "p-4 leading-relaxed",
            "border border-gray-800",
            // Custom scrollbar styling
            "scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-900"
          )}
          style={{
            maxHeight,
            borderRadius: title ? undefined : 'var(--radius-component-lg)',
            borderBottomLeftRadius: title ? 'var(--radius-component-lg)' : undefined,
            borderBottomRightRadius: title ? 'var(--radius-component-lg)' : undefined,
          }}
        >
          {children ||
            logLines.map((line, index) => (
              <div key={index} className="whitespace-pre-wrap break-words">
                {line || "\u00A0"}
              </div>
            ))}
        </div>
        {/* Optional scroll indicator */}
        {!isAtBottom && (
          <div
            className="absolute bottom-4 right-4 bg-primary-600 text-white px-3 py-1 text-xs font-medium shadow-lg animate-pulse"
            style={{ borderRadius: 'var(--radius-component)' }}
          >
            New logs ↓
          </div>
        )}
      </div>
    );
  }
);
ScrollableLogWindow.displayName = "ScrollableLogWindow";

export { ScrollableLogWindow };
