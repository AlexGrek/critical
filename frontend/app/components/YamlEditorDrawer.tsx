import React, { useState, useCallback, useMemo, useRef } from "react";
import { Drawer, Button, YamlEditor } from "~/components";
import type { YamlEditorProps, YamlEditorHandle } from "./YamlEditor";
import { Code2, AlertCircle, CheckCircle2 } from "lucide-react";
import { cn } from "~/lib/utils";

export interface YamlEditorDrawerProps
  extends Omit<YamlEditorProps, "hideFooter" | "data-testid"> {
  /** Label for the trigger button. Default: "Edit YAML" */
  triggerLabel?: string;
  /** CSS class for the trigger button. */
  triggerClassName?: string;
  /** Drawer title. Default: "Edit Resource" */
  drawerTitle?: string;
  /** Whether the drawer is open (controlled component). */
  open?: boolean;
  /** Called when the drawer's open state changes. */
  onOpenChange?: (open: boolean) => void;
  /** Data-testid for the trigger button. */
  "data-testid"?: string;
  /** Data-testid for the YAML editor inside the drawer. */
  editorTestId?: string;
  /** Called when user clicks Cancel. Drawer closes automatically. */
  onCancel?: () => void;
  /** Called after successful save. Drawer closes automatically. */
  onSaveSuccess?: () => void;
  /** Show a success message after saving (in addition to onSaveSuccess callback). */
  showSaveSuccess?: boolean;
}

/**
 * A reusable drawer-based YAML editor with Save/Cancel buttons in the footer.
 * Opens a right-sliding drawer with the YAML editor and footer action buttons.
 *
 * @example
 * <YamlEditorDrawer
 *   value={group}
 *   onSave={handleSave}
 *   readOnlyFields={["state", "hash_code", "deletion"]}
 *   triggerLabel="Edit as YAML"
 *   drawerTitle="Edit Group"
 * />
 *
 * @example
 * // Readonly mode (no Save button, just close)
 * <YamlEditorDrawer
 *   value={group}
 *   triggerLabel="View YAML"
 *   drawerTitle="Group YAML"
 * />
 */
export function YamlEditorDrawer({
  value,
  onSave,
  readOnlyFields,
  allowedTopLevelKeys,
  triggerLabel = "Edit YAML",
  triggerClassName,
  drawerTitle = "Edit Resource",
  open: controlledOpen,
  onOpenChange,
  "data-testid": testId,
  editorTestId,
  disabled = false,
  onCancel,
  onSaveSuccess,
  showSaveSuccess = false,
}: YamlEditorDrawerProps) {
  const [internalOpen, setInternalOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [successMessage, setSuccessMessage] = useState("");
  const [saveError, setSaveError] = useState<string | null>(null);
  const editorRef = useRef<YamlEditorHandle>(null);

  // Use controlled open state if provided, otherwise internal state
  const isOpen = controlledOpen !== undefined ? controlledOpen : internalOpen;

  const handleOpenChange = useCallback(
    (newOpen: boolean) => {
      if (controlledOpen === undefined) {
        setInternalOpen(newOpen);
      }
      onOpenChange?.(newOpen);
      // Clear messages when drawer closes
      if (!newOpen) {
        setSuccessMessage("");
        setSaveError(null);
      }
    },
    [controlledOpen, onOpenChange]
  );

  // Memoize the YAML value to avoid re-serialization on every render
  const yamlValue = useMemo<Record<string, unknown>>(
    () => value as unknown as Record<string, unknown>,
    [value]
  );

  const handleSave = useCallback(async () => {
    if (!editorRef.current) return;

    try {
      setIsSaving(true);
      setSaveError(null);
      await editorRef.current.save();

      if (showSaveSuccess) {
        setSuccessMessage("Saved successfully");
        // Auto-hide success message after 3 seconds
        setTimeout(() => setSuccessMessage(""), 3000);
      }
      onSaveSuccess?.();
      handleOpenChange(false);
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : "Save failed");
    } finally {
      setIsSaving(false);
    }
  }, [showSaveSuccess, onSaveSuccess, handleOpenChange]);

  const handleCancel = useCallback(() => {
    onCancel?.();
    handleOpenChange(false);
  }, [onCancel, handleOpenChange]);

  const isReadOnly = !onSave || disabled;

  return (
    <Drawer.Root open={isOpen} onOpenChange={handleOpenChange}>
      <Drawer.Trigger asChild>
        <Button
          variant="outline"
          size="sm"
          disabled={disabled}
          className={triggerClassName}
          data-testid={testId}
        >
          <Code2 className="w-4 h-4 mr-2" />
          {triggerLabel}
        </Button>
      </Drawer.Trigger>

      <Drawer.Content direction="right">
        <Drawer.Header>
          <Drawer.Title>{drawerTitle}</Drawer.Title>
        </Drawer.Header>

        <Drawer.Body className="flex flex-col flex-1 gap-2">
          {/* Save error message */}
          {saveError && (
            <div className="flex items-start gap-2 p-3 rounded-(--radius-component) bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-800">
              <AlertCircle className="w-4 h-4 text-red-600 dark:text-red-400 shrink-0 mt-0.5" />
              <p className="text-xs text-red-700 dark:text-red-300">{saveError}</p>
            </div>
          )}

          {/* Success message */}
          {successMessage && (
            <div className="flex items-start gap-2 p-3 rounded-(--radius-component) bg-green-50 dark:bg-green-950 border border-green-200 dark:border-green-800">
              <CheckCircle2 className="w-4 h-4 text-green-600 dark:text-green-400 shrink-0 mt-0.5" />
              <p className="text-xs text-green-700 dark:text-green-300">
                {successMessage}
              </p>
            </div>
          )}

          {/* YAML Editor (hidden footer, save via drawer button) */}
          <div className="flex-1 min-h-0">
            <YamlEditor
              ref={editorRef}
              value={yamlValue}
              onSave={isReadOnly ? undefined : onSave}
              readOnlyFields={readOnlyFields}
              allowedTopLevelKeys={allowedTopLevelKeys}
              disabled={isSaving || disabled}
              hideFooter={true}
              data-testid={editorTestId}
            />
          </div>
        </Drawer.Body>

        {/* Footer with Save/Cancel buttons */}
        <Drawer.Footer>
          <Button
            variant="outline"
            onClick={handleCancel}
            disabled={isSaving}
            data-testid="yaml-drawer-cancel"
          >
            Cancel
          </Button>
          {!isReadOnly && (
            <Button
              variant="primary"
              onClick={handleSave}
              disabled={isSaving}
              data-testid="yaml-drawer-save"
            >
              {isSaving ? "Saving..." : "Save"}
            </Button>
          )}
        </Drawer.Footer>
      </Drawer.Content>
    </Drawer.Root>
  );
}
