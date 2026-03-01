/**
 * YamlEditor — a reusable textarea-based YAML editor for resource documents.
 *
 * Takes a JS object, serializes it to YAML for editing, parses it back on
 * change, and reports parse errors inline. Server-managed fields (state,
 * hash_code, deletion) can be hidden via `readOnlyFields`.
 */
import { useState, useEffect, useRef, useCallback } from "react";
import { stringify, parse, YAMLParseError } from "yaml";
import { cn } from "~/lib/utils";

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

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function YamlEditor({
  value,
  onChange,
  readOnlyFields = [],
  className,
  "data-testid": testId,
}: YamlEditorProps) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  /** Whether the user has made local edits that haven't been synced back. */
  const dirty = useRef(false);
  /** Track the external value identity to detect parent-driven updates. */
  const lastExternalRef = useRef<Record<string, unknown> | null>(null);

  // Sync external value → textarea (only when the value actually changes
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
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const raw = e.target.value;
      setText(raw);
      dirty.current = true;

      try {
        const parsed = parse(raw);
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
      <textarea
        value={text}
        onChange={handleChange}
        spellCheck={false}
        data-testid={testId}
        className={cn(
          "flex-1 min-h-[200px] w-full resize-none font-mono text-xs leading-relaxed",
          "p-3 rounded-(--radius-component-lg)",
          "border bg-white text-gray-900",
          "dark:bg-gray-950 dark:text-gray-100",
          "focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500",
          "placeholder:text-gray-400 dark:placeholder:text-gray-500",
          error
            ? "border-red-400 dark:border-red-600"
            : "border-gray-200 dark:border-gray-700"
        )}
      />
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
