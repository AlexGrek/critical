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
import { cn } from "~/lib/utils";
import "./yaml-editor.css";

// ---------------------------------------------------------------------------
// Prism + Editor (SSR-safe lazy import)
// ---------------------------------------------------------------------------

let Editor: typeof import("react-simple-code-editor").default | null = null;
let Prism: typeof import("prismjs") | null = null;

if (typeof window !== "undefined") {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  Editor = require("react-simple-code-editor").default;
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  Prism = require("prismjs");
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  require("prismjs/components/prism-yaml");
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface YamlEditorProps {
  /** The resource object to display/edit as YAML. */
  value: Record<string, unknown>;
  /** Called with the parsed object whenever the user edits valid YAML. */
  onChange: (parsed: Record<string, unknown>) => void;
  /**
   * Top-level field names to strip from the displayed YAML.
   * These fields are server-managed and will be preserved on save
   * by the parent component merging over the original object.
   */
  readOnlyFields?: string[];
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
function highlightYaml(code: string): string {
  if (!Prism || !Prism.languages.yaml) return escapeHtml(code);
  return Prism.highlight(code, Prism.languages.yaml, "yaml");
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
  onChange,
  readOnlyFields = [],
  className,
  disabled = false,
  "data-testid": testId,
}: YamlEditorProps) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  /** Whether the user has made local edits that haven't been synced back. */
  const dirty = useRef(false);
  /** Track the external value identity to detect parent-driven updates. */
  const lastExternalRef = useRef<Record<string, unknown> | null>(null);

  // Sync external value → editor (only when the value actually changes
  // from the parent and the user hasn't made local edits).
  useEffect(() => {
    if (value === lastExternalRef.current) return;
    lastExternalRef.current = value;
    if (!dirty.current) {
      const display = stripFields(value, readOnlyFields);
      setText(toYaml(display));
      setError(null);
    }
  }, [value, readOnlyFields]);

  const handleChange = useCallback(
    (code: string) => {
      setText(code);
      dirty.current = true;

      try {
        const parsed = parse(code);
        if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
          setError("YAML must be an object (key: value pairs)");
          return;
        }
        setError(null);
        onChange(parsed as Record<string, unknown>);
      } catch (err) {
        if (err instanceof YAMLParseError) {
          setError(err.message.split("\n")[0]);
        } else {
          setError("Invalid YAML");
        }
      }
    },
    [onChange]
  );

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
          "flex-1 min-h-50 w-full overflow-auto font-mono text-xs leading-relaxed",
          "rounded-(--radius-component-lg)",
          "border bg-white text-gray-900",
          "dark:bg-gray-950 dark:text-gray-100",
          disabled && "opacity-50 cursor-not-allowed pointer-events-none",
          error
            ? "border-red-400 dark:border-red-600"
            : "border-gray-200 dark:border-gray-700"
        )}
      >
        {Editor ? (
          <Editor
            value={text}
            onValueChange={handleChange}
            highlight={highlightYaml}
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
    </div>
  );
}
