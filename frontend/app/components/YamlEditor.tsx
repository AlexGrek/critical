/**
 * YamlEditor — a code editor with YAML syntax highlighting.
 *
 * Uses react-simple-code-editor + Prism for lightweight highlighting.
 * Takes a JS object, serializes it to YAML for editing, parses it back on
 * change, and reports parse errors inline. Server-managed fields (state,
 * hash_code, deletion) can be hidden via `readOnlyFields`.
 */
import { useState, useEffect, useRef, useCallback } from "react";
import { stringify, parse, YAMLParseError } from "yaml";
import { Copy, Check, Lock } from "lucide-react";
import { cn } from "~/lib/utils";
import "./yaml-editor.css";

// ---------------------------------------------------------------------------
// Prism + Editor (SSR-safe lazy import via dynamic import)
// ---------------------------------------------------------------------------

type EditorComponent = typeof import("react-simple-code-editor").default;
type PrismType = typeof import("prismjs");

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface YamlEditorProps {
  /** The resource object to display/edit as YAML. */
  value: Record<string, unknown>;
  /**
   * Called with the parsed object when the user clicks Save.
   * Should throw on error; the editor will display the thrown message.
   * Omit for read-only mode (no Save button rendered).
   */
  onSave?: (parsed: Record<string, unknown>) => Promise<void>;
  /**
   * Top-level field names to strip from the displayed YAML.
   * These fields are server-managed and will be preserved on save
   * by the parent component merging over the original object.
   */
  readOnlyFields?: string[];
  /**
   * If provided, the editor will show a validation error when the parsed
   * YAML contains top-level keys that are not in this set, and block saving.
   * Should list all valid schema fields for the resource kind.
   */
  allowedTopLevelKeys?: readonly string[];
  className?: string;
  "data-testid"?: string;
  disabled?: boolean;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Remove read-only keys from an object before displaying. */
function stripFields(
  obj: Record<string, unknown>,
  fields: string[]
): Record<string, unknown> {
  const copy = { ...obj };
  for (const f of fields) delete copy[f];
  return copy;
}

/** Serialize a JS object to a YAML string with sensible defaults. */
function toYaml(obj: Record<string, unknown>): string {
  return stringify(obj, { lineWidth: 0, defaultKeyType: "PLAIN" });
}

/** Highlight YAML using Prism (client-side only). */
function highlightYaml(code: string, prism: PrismType | null): string {
  if (!prism || !prism.languages.yaml) return escapeHtml(code);
  return prism.highlight(code, prism.languages.yaml, "yaml");
}

/** Simple HTML escape for SSR fallback. */
function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function YamlEditor({
  value,
  onSave,
  readOnlyFields = [],
  allowedTopLevelKeys,
  className,
  disabled = false,
  "data-testid": testId,
}: YamlEditorProps) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isCopied, setIsCopied] = useState(false);
  const [EditorComp, setEditorComp] = useState<EditorComponent | null>(null);
  const prismRef = useRef<PrismType | null>(null);
  /** Parsed object from the current editor text (null if parse error). */
  const parsedRef = useRef<Record<string, unknown> | null>(null);
  /** Whether the user has made local edits that haven't been synced back. */
  const dirty = useRef(false);
  /** Track the external value identity to detect parent-driven updates. */
  const lastExternalRef = useRef<Record<string, unknown> | null>(null);

  // Lazy-load editor + prism on client only
  useEffect(() => {
    let cancelled = false;
    Promise.all([
      import("react-simple-code-editor"),
      import("prismjs"),
    ]).then(async ([editorMod, prismMod]) => {
      // @ts-expect-error prism-yaml has no type declarations
      await import("prismjs/components/prism-yaml");
      if (!cancelled) {
        prismRef.current = prismMod.default ?? prismMod;
        setEditorComp(() => editorMod.default);
      }
    });
    return () => { cancelled = true; };
  }, []);

  // Sync external value → editor (only when the value actually changes
  // from the parent and the user hasn't made local edits).
  useEffect(() => {
    if (value === lastExternalRef.current) return;
    lastExternalRef.current = value;
    if (!dirty.current) {
      const display = stripFields(value, readOnlyFields);
      const yaml = toYaml(display);
      setText(yaml);
      setError(null);
      // Pre-populate parsedRef so Save is enabled without requiring a keystroke first.
      try {
        const parsed = parse(yaml);
        parsedRef.current =
          parsed !== null && typeof parsed === "object" && !Array.isArray(parsed)
            ? (parsed as Record<string, unknown>)
            : null;
      } catch {
        parsedRef.current = null;
      }
    }
  }, [value, readOnlyFields]);

  const handleChange = useCallback(
    (code: string) => {
      setText(code);
      dirty.current = true;
      setSaveError(null);

      try {
        const parsed = parse(code);
        if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
          setError("YAML must be an object (key: value pairs)");
          parsedRef.current = null;
          return;
        }
        // Validate top-level keys against schema if allowedTopLevelKeys provided
        if (allowedTopLevelKeys) {
          const allowed = new Set(allowedTopLevelKeys);
          const unknown = Object.keys(parsed).filter((k) => !allowed.has(k));
          if (unknown.length > 0) {
            setError(`Unknown field${unknown.length > 1 ? "s" : ""}: ${unknown.join(", ")}`);
            parsedRef.current = null;
            return;
          }
        }
        setError(null);
        parsedRef.current = parsed as Record<string, unknown>;
      } catch (err) {
        if (err instanceof YAMLParseError) {
          setError(err.message.split("\n")[0]);
        } else {
          setError("Invalid YAML");
        }
        parsedRef.current = null;
      }
    },
    [allowedTopLevelKeys]
  );

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(text);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 1500);
    } catch {
      // clipboard API not available
    }
  }, [text]);

  const handleSave = useCallback(async () => {
    if (!onSave || !parsedRef.current) return;
    setIsSaving(true);
    setSaveError(null);
    try {
      await onSave(parsedRef.current);
      dirty.current = false;
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : "Save failed");
    } finally {
      setIsSaving(false);
    }
  }, [onSave]);

  /** Reset dirty flag when the user explicitly syncs (e.g. parent re-renders
   *  after a save). We detect this via value identity change. */
  useEffect(() => {
    dirty.current = false;
  }, [value]);

  return (
    <div className={cn("flex flex-col gap-2 flex-1 min-h-0", className)}>
      <div
        data-testid={testId}
        className={cn(
          "flex flex-col flex-1 min-h-50 w-full overflow-hidden font-mono text-xs leading-relaxed",
          "rounded-(--radius-component-lg)",
          "border bg-white text-gray-900",
          "dark:bg-gray-950 dark:text-gray-100",
          error
            ? "border-red-400 dark:border-red-600"
            : "border-gray-200 dark:border-gray-700"
        )}
      >
        {/* Read-only top bar (only when disabled) */}
        {disabled && (
          <div className={cn(
            "group flex items-center justify-between px-2 py-0.5 shrink-0",
            "border-b border-gray-100 dark:border-gray-800",
            "bg-gray-50/60 dark:bg-gray-900/40"
          )}>
            <div className="flex items-center gap-1">
              <Lock className="w-2.5 h-2.5 text-gray-400 dark:text-gray-500" />
              <span
                className="text-gray-400 dark:text-gray-500 tracking-wide"
                style={{ fontSize: "0.6rem", fontVariant: "small-caps" }}
              >
                Read only
              </span>
            </div>
            <button
              type="button"
              onClick={handleCopy}
              className={cn(
                "pointer-events-auto p-1 cursor-pointer",
                "rounded-(--radius-component)",
                "opacity-30 group-hover:opacity-100 transition-opacity duration-150",
                "hover:bg-gray-100 dark:hover:bg-gray-800",
                "text-gray-600 dark:text-gray-400"
              )}
              title={isCopied ? "Copied!" : "Copy to clipboard"}
              data-testid="yaml-copy-button"
            >
              {isCopied ? (
                <Check className="w-3 h-3 text-primary-600 dark:text-primary-400" />
              ) : (
                <Copy className="w-3 h-3" />
              )}
            </button>
          </div>
        )}

        {/* Scrollable editor content — relative so the floating copy button can be positioned */}
        <div className={cn("relative flex-1 overflow-auto", disabled && "pointer-events-none")}>
          {/* Floating copy button for editable editors */}
          {!disabled && (
            <button
              type="button"
              onClick={handleCopy}
              className={cn(
                "absolute top-2 right-2 z-10 pointer-events-auto p-1.5 cursor-pointer",
                "rounded-(--radius-component)",
                "opacity-30 hover:opacity-100 transition-opacity duration-150",
                "hover:bg-gray-100/80 dark:hover:bg-gray-800/80",
                "text-gray-600 dark:text-gray-400"
              )}
              title={isCopied ? "Copied!" : "Copy to clipboard"}
              data-testid="yaml-copy-button"
            >
              {isCopied ? (
                <Check className="w-3.5 h-3.5 text-primary-600 dark:text-primary-400" />
              ) : (
                <Copy className="w-3.5 h-3.5" />
              )}
            </button>
          )}
          {EditorComp ? (
            <EditorComp
              value={text}
              onValueChange={handleChange}
              highlight={(code: string) => highlightYaml(code, prismRef.current)}
              disabled={disabled}
              padding={12}
              style={{
                fontFamily: "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace",
                fontSize: "0.75rem",
                lineHeight: "1.625",
                minHeight: "100%",
              }}
              className="yaml-editor-inner"
            />
          ) : (
            /* SSR fallback: plain textarea */
            <textarea
              value={text}
              onChange={(e) => handleChange(e.target.value)}
              spellCheck={false}
              disabled={disabled}
              className={cn(
                "w-full h-full resize-none font-mono text-xs leading-relaxed p-3",
                "bg-transparent focus:outline-none",
              )}
            />
          )}
        </div>
      </div>
      {error && (
        <div
          className={cn(
            "px-3 py-2 text-xs font-mono rounded-(--radius-component)",
            "bg-red-50 dark:bg-red-950/40 text-red-600 dark:text-red-400",
            "border border-red-200 dark:border-red-800"
          )}
          data-testid="yaml-parse-error"
        >
          {error}
        </div>
      )}
      {saveError && (
        <div
          className={cn(
            "px-3 py-2 text-xs font-mono rounded-(--radius-component)",
            "bg-red-50 dark:bg-red-950/40 text-red-600 dark:text-red-400",
            "border border-red-200 dark:border-red-800"
          )}
          data-testid="yaml-save-error"
        >
          {saveError}
        </div>
      )}
      {onSave && (
        <div className="flex justify-end shrink-0">
          <button
            type="button"
            onClick={handleSave}
            disabled={disabled || isSaving || !!error || !parsedRef.current}
            data-testid="yaml-save-button"
            className={cn(
              "px-3 py-1.5 text-xs font-medium rounded-(--radius-component)",
              "bg-indigo-600 text-white hover:bg-indigo-700 dark:bg-indigo-500 dark:hover:bg-indigo-600",
              "disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none",
              "transition-colors"
            )}
          >
            {isSaving ? "Saving…" : "Save"}
          </button>
        </div>
      )}
    </div>
  );
}
