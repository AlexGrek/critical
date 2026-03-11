import {
  useRef,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";
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
  originRect: DOMRect;
  targetRect: {
    top: number;
    left: number;
    width: number;
    height: number;
    fullscreen: boolean;
  };
}

/** Compute the panel's destination rect at the moment the app is opened. */
function computeTargetRect(): ActiveApp["targetRect"] {
  const isMobile = window.innerWidth < 640;
  if (isMobile) {
    return {
      top: 0,
      left: 0,
      width: window.innerWidth,
      height: window.innerHeight,
      fullscreen: true,
    };
  }
  const w = Math.min(720, window.innerWidth - 64);
  const h = Math.min(560, window.innerHeight - 80);
  return {
    top: Math.round((window.innerHeight - h) / 2),
    left: Math.round((window.innerWidth - w) / 2),
    width: w,
    height: h,
    fullscreen: false,
  };
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
 * AppMenu — a phone-style app launcher grid.
 *
 * Each app has an icon tile and a label.  Tapping an app opens its content
 * in a panel that morphs from the icon's position — exactly like
 * iOS/Android home screen behaviour.
 *
 * The open panel shows a back-button + app-name header (instead of a tab
 * strip), making it easier to use on mobile while remaining usable on
 * desktop.
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
  const [mounted, setMounted] = useState(false);
  const [active, setActive] = useState<ActiveApp | null>(null);
  const iconRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  // Only render the portal after client-side hydration.
  useEffect(() => {
    setMounted(typeof document !== "undefined");
  }, []);

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
    if (!el) return;
    // Snapshot both rects at open time so resize never causes re-animation.
    const originRect = el.getBoundingClientRect();
    const targetRect = computeTargetRect();
    setActive({ app, originRect, targetRect });
  }, []);

  const closeApp = useCallback(() => setActive(null), []);

  return (
    <div className={cn("relative", className)} data-testid={testId}>
      {/* ------------------------------------------------------------------ */}
      {/* App grid                                                            */}
      {/* ------------------------------------------------------------------ */}
      <motion.div
        className={cn("grid gap-2 sm:gap-4 p-2 sm:p-4", COL_CLASSES[columns])}
        animate={{
          opacity: active ? 0.3 : 1,
          scale: active ? 0.97 : 1,
        }}
        transition={{ duration: 0.25, ease: [0.4, 0, 0.2, 1] }}
      >
        {apps.map((app) => (
          <button
            key={app.id}
            ref={(el) => {
              if (el) iconRefs.current.set(app.id, el);
              else iconRefs.current.delete(app.id);
            }}
            onClick={() => openApp(app)}
            disabled={!!active}
            data-testid={`app-item-${app.id}`}
            className={cn(
              "flex flex-col items-center gap-1.5 sm:gap-2 py-2 px-1 relative",
              "rounded-(--radius-component-lg)",
              "hover:bg-gray-100 dark:hover:bg-gray-800",
              "transition-transform duration-100 active:scale-95",
              "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500",
              "cursor-pointer select-none disabled:cursor-default"
            )}
          >
            {/* Icon tile */}
            <div
              className={cn(
                "relative w-[60px] h-[60px] flex items-center justify-center",
                "rounded-(--radius-component-lg) shadow-sm",
                app.color ??
                  "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-100"
              )}
            >
              {app.icon}

              {/* Notification badge */}
              {app.badge != null && (
                <span className="absolute -top-1.5 -right-1.5 min-w-[20px] h-5 flex items-center justify-center px-1 rounded-full bg-red-500 text-white text-[10px] font-bold leading-none">
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

      {/* ------------------------------------------------------------------ */}
      {/* Portal: morphing app panel                                          */}
      {/* ------------------------------------------------------------------ */}
      {mounted &&
        createPortal(
          <AnimatePresence>
            {active && (
              <>
                {/* Backdrop */}
                <motion.div
                  className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  onClick={closeApp}
                />

                {/* App panel — morphs from icon rect to full/modal size */}
                <motion.div
                  className="fixed z-50 overflow-hidden flex flex-col bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 shadow-2xl"
                  initial={{
                    top: active.originRect.top,
                    left: active.originRect.left,
                    width: active.originRect.width,
                    height: active.originRect.height,
                    borderRadius: "var(--radius-component-lg, 8px)",
                  }}
                  animate={{
                    top: active.targetRect.top,
                    left: active.targetRect.left,
                    width: active.targetRect.width,
                    height: active.targetRect.height,
                    borderRadius: active.targetRect.fullscreen
                      ? 0
                      : "var(--radius-component-xl, 16px)",
                  }}
                  exit={{
                    top: active.originRect.top,
                    left: active.originRect.left,
                    width: active.originRect.width,
                    height: active.originRect.height,
                    opacity: 0,
                    borderRadius: "var(--radius-component-lg, 8px)",
                  }}
                  transition={{
                    type: "tween",
                    duration: 0.3,
                    ease: [0.4, 0, 0.2, 1],
                  }}
                  onClick={(e) => e.stopPropagation()}
                >
                  {/* Header: back button + app name */}
                  <motion.div
                    className="flex items-center gap-2 px-3 py-2 sm:px-4 sm:py-3 border-b border-gray-200 dark:border-gray-700 shrink-0 min-h-[52px] bg-white dark:bg-gray-900"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ delay: 0.18, duration: 0.15 }}
                  >
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
                  </motion.div>

                  {/* Scrollable content */}
                  <motion.div
                    className="flex-1 overflow-y-auto min-h-0"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ delay: 0.22, duration: 0.18 }}
                  >
                    {typeof active.app.content === "function"
                      ? active.app.content(closeApp)
                      : active.app.content}
                  </motion.div>
                </motion.div>
              </>
            )}
          </AnimatePresence>,
          document.body
        )}
    </div>
  );
}
