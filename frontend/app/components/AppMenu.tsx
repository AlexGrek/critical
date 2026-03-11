import {
  useRef,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowLeft } from "lucide-react";
import { cn } from "~/lib/utils";
import { Button } from "~/components/Button";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface AppItem {
  id: string;
  /**
   * Icon element for the app tile.
   * Size it via className, e.g. `<Users className="w-8 h-8" />`.
   */
  icon: ReactNode;
  label: string;
  /**
   * Content rendered inside the app panel.
   * Can be a ReactNode or a render function that receives a `close` callback.
   */
  content: ReactNode | ((close: () => void) => ReactNode);
  /** Optional notification badge shown top-right of the icon tile. */
  badge?: number | string;
  /**
   * Tailwind classes for the icon tile background + text color.
   * Defaults to neutral gray (`bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-100`).
   * Example: `"bg-blue-500 text-white"`
   */
  color?: string;
}

export interface AppMenuProps {
  apps: AppItem[];
  /** Number of grid columns (default: 4). */
  columns?: 3 | 4 | 5;
  className?: string;
  "data-testid"?: string;
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

interface ActiveApp {
  app: AppItem;
  /** Icon center minus container center — used as the spring animation origin. */
  originOffset: { x: number; y: number };
}

const COL_CLASSES: Record<number, string> = {
  3: "grid-cols-3",
  4: "grid-cols-4",
  5: "grid-cols-5",
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * AppMenu — a phone-style app launcher grid that works as a tabs replacement.
 *
 * Shows a grid of icon tiles. Tapping one replaces the grid **in-place** with
 * a header (back button + app name) and scrollable content — exactly like
 * iOS/Android opening an app from the home screen, but entirely inline (no
 * portal, no modal overlay, no backdrop).
 *
 * **Prefer this over `<Tabs>` for new multi-section UIs.**
 *
 * ```tsx
 * <AppMenu
 *   apps={[
 *     {
 *       id: "settings",
 *       icon: <Settings className="w-8 h-8" />,
 *       label: "Settings",
 *       content: (close) => <SettingsPanel onClose={close} />,
 *       color: "bg-gray-700 text-white",
 *     },
 *   ]}
 * />
 * ```
 */
export function AppMenu({
  apps,
  columns = 4,
  className,
  "data-testid": testId,
}: AppMenuProps) {
  const [active, setActive] = useState<ActiveApp | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const iconRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  // Close on Escape.
  useEffect(() => {
    if (!active) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setActive(null);
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [active]);

  const openApp = useCallback((app: AppItem) => {
    const el = iconRefs.current.get(app.id);
    const container = containerRef.current;
    if (!el || !container) return;

    const iconRect = el.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();

    // Offset of icon center from container center — spring origin.
    const originOffset = {
      x: iconRect.left + iconRect.width / 2 - (containerRect.left + containerRect.width / 2),
      y: iconRect.top + iconRect.height / 2 - (containerRect.top + containerRect.height / 2),
    };

    setActive({ app, originOffset });
  }, []);

  const closeApp = useCallback(() => setActive(null), []);

  return (
    <div
      ref={containerRef}
      className={cn("relative w-full", className)}
      data-testid={testId}
    >
      {/*
       * mode="popLayout": the exiting element is removed from layout flow
       * immediately so the entering element drives the container height.
       * This lets the app panel grow as tall as its content needs.
       */}
      <AnimatePresence mode="popLayout" initial={false}>
        {!active ? (
          /* ---------------------------------------------------------------- */
          /* App grid                                                          */
          /* ---------------------------------------------------------------- */
          <motion.div
            key="grid"
            className={cn("grid gap-2 sm:gap-4 p-2 sm:p-4", COL_CLASSES[columns])}
            exit={{
              opacity: 0,
              scale: 0.95,
              transition: { duration: 0.15, ease: [0.4, 0, 0.2, 1] },
            }}
          >
            {apps.map((app) => (
              <button
                key={app.id}
                ref={(el) => {
                  if (el) iconRefs.current.set(app.id, el);
                  else iconRefs.current.delete(app.id);
                }}
                onClick={() => openApp(app)}
                data-testid={`app-item-${app.id}`}
                className={cn(
                  "flex flex-col items-center gap-1.5 sm:gap-2 py-2 px-1 relative",
                  "rounded-(--radius-component-lg)",
                  "hover:bg-gray-100 dark:hover:bg-gray-800",
                  "transition-transform duration-100 active:scale-95",
                  "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500",
                  "cursor-pointer select-none"
                )}
              >
                {/* Icon tile */}
                <div
                  className={cn(
                    "relative w-15 h-15 flex items-center justify-center",
                    "rounded-(--radius-component-lg) shadow-sm",
                    app.color ??
                      "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-100"
                  )}
                >
                  {app.icon}

                  {/* Notification badge */}
                  {app.badge != null && (
                    <span className="absolute -top-1.5 -right-1.5 min-w-5 h-5 flex items-center justify-center px-1 rounded-full bg-red-500 text-white text-[10px] font-bold leading-none">
                      {typeof app.badge === "number" && app.badge > 99
                        ? "99+"
                        : app.badge}
                    </span>
                  )}
                </div>

                {/* App name */}
                <span className="text-[11px] sm:text-xs font-medium text-center leading-tight line-clamp-2 w-full px-0.5 text-gray-700 dark:text-gray-300">
                  {app.label}
                </span>
              </button>
            ))}
          </motion.div>
        ) : (
          /* ---------------------------------------------------------------- */
          /* App panel — normal flow, height driven by content.               */
          /* ---------------------------------------------------------------- */
          <motion.div
            key={active.app.id}
            className={cn(
              "flex flex-col",
              "bg-white dark:bg-gray-900",
              "border border-gray-200 dark:border-gray-800",
              "rounded-(--radius-component-lg)",
            )}
            initial={{
              scale: 0.3,
              opacity: 0,
              x: active.originOffset.x,
              y: active.originOffset.y,
            }}
            animate={{ scale: 1, opacity: 1, x: 0, y: 0 }}
            exit={{
              scale: 0.3,
              opacity: 0,
              x: active.originOffset.x,
              y: active.originOffset.y,
              transition: { duration: 0.15, ease: [0.4, 0, 0.2, 1] },
            }}
            transition={{ type: "tween", duration: 0.2, ease: [0.4, 0, 0.2, 1] }}
          >
            {/* Header: back button + app name */}
            <div className="flex items-center gap-2 px-3 py-2 sm:px-4 sm:py-3 border-b border-gray-200 dark:border-gray-700 min-h-13">
              <Button
                variant="ghost"
                size="icon"
                onClick={closeApp}
                data-testid="app-panel-back"
                title="Back to menu"
                className="shrink-0"
              >
                <ArrowLeft className="w-5 h-5" />
              </Button>
              <span className="font-semibold text-base text-gray-900 dark:text-gray-100 truncate">
                {active.app.label}
              </span>
            </div>

            {/* Content — grows naturally, no height cap */}
            <div>
              {typeof active.app.content === "function"
                ? active.app.content(closeApp)
                : active.app.content}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
