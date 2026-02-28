/**
 * ResourcePicker — async search-as-you-type dropdown backed by
 * /api/v1/global/{kind}/search?startwith={prefix}
 *
 * The picker is "fire-and-forget": selecting an item calls onSelect then
 * immediately clears the input so the user can search again.
 *
 * Prefix logic
 * ------------
 * If the user's input already contains an underscore (e.g. "u_al", "g_eng")
 * it is forwarded as-is. Otherwise the kind's well-known prefix is prepended:
 *   kind="users"  "alice"  → search "u_alice"
 *   kind="groups" "eng"    → search "g_eng"
 */
import React, {
  useState,
  useEffect,
  useRef,
  useCallback,
  useId,
} from "react";
import {
  useFloating,
  autoUpdate,
  offset,
  flip,
  size,
  FloatingPortal,
} from "@floating-ui/react";
import { Search, Loader2 } from "lucide-react";
import { cn } from "~/lib/utils";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface PickerItem {
  id: string;
  name?: string;
  [key: string]: unknown;
}

export interface ResourcePickerProps {
  /** ArangoDB collection kind, e.g. "users" or "groups". */
  kind: string;
  /**
   * ID prefix for this kind. If the user's input has no underscore it is
   * automatically prepended before searching.
   * Defaults to the well-known prefix for the kind (u_, g_, sa_, pa_).
   */
  prefix?: string;
  placeholder?: string;
  onSelect: (id: string, item: PickerItem) => void;
  disabled?: boolean;
  className?: string;
  "data-testid"?: string;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const KIND_PREFIX: Record<string, string> = {
  users: "u_",
  groups: "g_",
  service_accounts: "sa_",
  pipeline_accounts: "pa_",
};

function buildStartwith(input: string, prefix: string): string {
  const t = input.trim();
  if (!t) return prefix;
  if (t.includes("_")) return t; // already has explicit prefix
  return prefix + t;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function ResourcePicker({
  kind,
  prefix,
  placeholder = "Search…",
  onSelect,
  disabled,
  className,
  "data-testid": testId,
}: ResourcePickerProps) {
  const resolvedPrefix = prefix ?? KIND_PREFIX[kind] ?? "";
  const listboxId = useId();

  const [inputValue, setInputValue] = useState("");
  const [items, setItems] = useState<PickerItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  const [activeIdx, setActiveIdx] = useState(-1);
  const [hasFetchedOnce, setHasFetchedOnce] = useState(false);

  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);

  // ---- floating-ui positioning ----
  const { refs, floatingStyles } = useFloating({
    placement: "bottom-start",
    whileElementsMounted: autoUpdate,
    middleware: [
      offset(4),
      flip({ padding: 8 }),
      size({
        padding: 8,
        apply({ rects, elements }) {
          Object.assign(elements.floating.style, {
            minWidth: `${rects.reference.width}px`,
          });
        },
      }),
    ],
  });

  // Merge our ref + floating ref onto the wrapper div
  const setWrapperRef = useCallback(
    (el: HTMLDivElement | null) => {
      refs.setReference(el);
    },
    [refs]
  );

  // ---- search ----
  const doSearch = useCallback(
    async (raw: string) => {
      setLoading(true);
      try {
        const startwith = buildStartwith(raw, resolvedPrefix);
        const res = await fetch(
          `/api/v1/global/${kind}/search?startwith=${encodeURIComponent(startwith)}`
        );
        if (!res.ok) throw new Error();
        const data: { items: PickerItem[] } = await res.json();
        setItems(data.items);
        setHasFetchedOnce(true);
        setOpen(true);
        setActiveIdx(-1);
      } catch {
        setItems([]);
        setOpen(true);
      } finally {
        setLoading(false);
      }
    },
    [kind, resolvedPrefix]
  );

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    setInputValue(v);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (!v.trim()) {
      setOpen(false);
      setItems([]);
      setHasFetchedOnce(false);
      return;
    }
    debounceRef.current = setTimeout(() => doSearch(v), 250);
  };

  const handleSelect = useCallback(
    (item: PickerItem) => {
      onSelect(item.id, item);
      setInputValue("");
      setItems([]);
      setOpen(false);
      setActiveIdx(-1);
      setHasFetchedOnce(false);
      inputRef.current?.focus();
    },
    [onSelect]
  );

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    switch (e.key) {
      case "Escape":
        setOpen(false);
        setActiveIdx(-1);
        break;
      case "ArrowDown":
        if (!open) break;
        e.preventDefault();
        setActiveIdx((i) => Math.min(i + 1, items.length - 1));
        break;
      case "ArrowUp":
        if (!open) break;
        e.preventDefault();
        setActiveIdx((i) => Math.max(i - 1, 0));
        break;
      case "Enter":
        if (!open) break;
        e.preventDefault();
        if (activeIdx >= 0 && items[activeIdx]) {
          handleSelect(items[activeIdx]);
        }
        break;
    }
  };

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      const ref = refs.reference.current as Element | null;
      const floating = refs.floating.current as Element | null;
      if (
        ref &&
        !ref.contains(e.target as Node) &&
        floating &&
        !floating.contains(e.target as Node)
      ) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open, refs]);

  // Cleanup debounce on unmount
  useEffect(
    () => () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    },
    []
  );

  return (
    <div ref={setWrapperRef} className={cn("relative", className)}>
      {/* Search icon */}
      <Search className="pointer-events-none absolute left-3 top-1/2 z-10 h-3.5 w-3.5 -translate-y-1/2 text-gray-400" />

      {/* Controlled input */}
      <input
        ref={inputRef}
        type="text"
        value={inputValue}
        onChange={handleInputChange}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        data-testid={testId}
        autoComplete="off"
        role="combobox"
        aria-autocomplete="list"
        aria-controls={listboxId}
        aria-expanded={open}
        aria-activedescendant={
          activeIdx >= 0 ? `${listboxId}-opt-${activeIdx}` : undefined
        }
        className={cn(
          "w-full rounded-(--radius-component) border border-gray-200 bg-white",
          "py-2 pl-8 pr-8 text-sm font-mono",
          "text-gray-900 placeholder:text-gray-400",
          "dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 dark:placeholder:text-gray-500",
          "focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500",
          "disabled:cursor-not-allowed disabled:opacity-50",
          "transition-colors"
        )}
      />

      {/* Loading spinner */}
      {loading && (
        <Loader2 className="pointer-events-none absolute right-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 animate-spin text-gray-400" />
      )}

      {/* Dropdown */}
      {open && (
        <FloatingPortal>
          <div
            ref={refs.setFloating}
            style={floatingStyles}
            id={listboxId}
            role="listbox"
            className={cn(
              "z-50 overflow-hidden py-1",
              "rounded-(--radius-component-lg)",
              "border border-gray-200 dark:border-gray-700",
              "bg-white dark:bg-gray-900",
              "shadow-2xl"
            )}
          >
            {items.length === 0 ? (
              <p className="px-3 py-2.5 text-sm text-gray-400 dark:text-gray-500">
                {loading ? "Searching…" : "No results"}
              </p>
            ) : (
              items.map((item, idx) => (
                <button
                  key={item.id}
                  id={`${listboxId}-opt-${idx}`}
                  type="button"
                  role="option"
                  aria-selected={idx === activeIdx}
                  onMouseDown={(e) => {
                    e.preventDefault(); // keep input focus
                    handleSelect(item);
                  }}
                  onMouseEnter={() => setActiveIdx(idx)}
                  data-testid={`picker-result-${item.id}`}
                  className={cn(
                    "flex w-full cursor-pointer items-center gap-3 px-3 py-2 text-left text-sm transition-colors",
                    idx === activeIdx
                      ? "bg-primary-50 dark:bg-primary-900/30 text-primary-800 dark:text-primary-200"
                      : "text-gray-900 dark:text-gray-100 hover:bg-gray-50 dark:hover:bg-gray-800"
                  )}
                >
                  <span className="shrink-0 font-mono text-xs text-gray-500 dark:text-gray-400">
                    {item.id}
                  </span>
                  {item.name && (
                    <span className="truncate text-gray-600 dark:text-gray-300">
                      {item.name}
                    </span>
                  )}
                </button>
              ))
            )}
          </div>
        </FloatingPortal>
      )}
    </div>
  );
}
