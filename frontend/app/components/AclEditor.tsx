/**
 * AclEditor — reusable modal for editing AccessControlStore (ACL) on any resource.
 *
 * Permission presets:
 *   Read  (7)  = FETCH + LIST + NOTIFY
 *   Write (31) = READ + CREATE + MODIFY
 *   Root  (127)= WRITE + CUSTOM1 + CUSTOM2
 *
 * Each ACL entry (AccessControlList) holds a list of principals + a permission bitmask.
 * The editor flattens this to a per-(principal, permission) row for easier editing,
 * then groups back by permission level on save.
 */

import React, { useState } from "react";
import { Shield, X } from "lucide-react";
import { cn } from "~/lib/utils";

import { Modal } from "./Modal";
import { Button } from "./Button";
import { Paragraph } from "./Paragraph";
import { ResourcePicker } from "./ResourcePicker";
import { Table } from "./Table";

// ---------------------------------------------------------------------------
// Exported ACL types
// ---------------------------------------------------------------------------

export interface AccessControlList {
  permissions: number;
  principals: string[];
  scope?: string;
}

export interface AccessControlStore {
  list: AccessControlList[];
  last_mod_date: string;
}

// ---------------------------------------------------------------------------
// Permission helpers
// ---------------------------------------------------------------------------

const PERMISSION_PRESETS = [
  { label: "Read", value: 7 },
  { label: "Write", value: 31 },
  { label: "Root", value: 127 },
] as const;

function permissionLabel(p: number): string {
  const preset = PERMISSION_PRESETS.find((pr) => pr.value === p);
  return preset ? preset.label : `Custom (${p})`;
}

/**
 * A small colored badge showing the permission level.
 * Exported so parent components can display ACL entries without re-implementing.
 */
export function PermissionBadge({ permissions }: { permissions: number }) {
  const label = permissionLabel(permissions);
  const colorClass =
    permissions === 127
      ? "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400"
      : permissions === 31
        ? "bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400"
        : permissions === 7
          ? "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400"
          : "bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-400";
  return (
    <span
      className={cn(
        "inline-flex items-center px-1.5 py-0.5 rounded-(--radius-component) text-xs font-medium",
        colorClass
      )}
    >
      {label}
    </span>
  );
}

// ---------------------------------------------------------------------------
// Internal flat representation and converters
// ---------------------------------------------------------------------------

interface AclEntry {
  principal: string;
  permissions: number;
  /** Stable local key for React rendering */
  _id: string;
}

function flattenAcl(acl: AccessControlStore): AclEntry[] {
  return acl.list.flatMap((entry) =>
    entry.principals.map((principal) => ({
      principal,
      permissions: entry.permissions,
      _id: `${principal}::${entry.permissions}::${Math.random()}`,
    }))
  );
}

function buildAcl(entries: AclEntry[]): AccessControlStore {
  const byPerms = new Map<number, Set<string>>();
  for (const entry of entries) {
    const set = byPerms.get(entry.permissions) ?? new Set<string>();
    set.add(entry.principal);
    byPerms.set(entry.permissions, set);
  }
  return {
    list: Array.from(byPerms.entries()).map(([permissions, principals]) => ({
      permissions,
      principals: Array.from(principals),
    })),
    last_mod_date: new Date().toISOString(),
  };
}

// ---------------------------------------------------------------------------
// PermissionToggle — inline button group for selecting a preset
// ---------------------------------------------------------------------------

function PermissionToggle({
  value,
  onChange,
}: {
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <div className="flex rounded-(--radius-component) overflow-hidden border border-gray-200 dark:border-gray-700">
      {PERMISSION_PRESETS.map((preset, idx) => (
        <button
          key={preset.value}
          type="button"
          onClick={() => onChange(preset.value)}
          data-testid={`permission-preset-${preset.label.toLowerCase()}`}
          className={cn(
            "px-3 py-1 text-xs font-medium transition-colors",
            idx > 0 && "border-l border-gray-200 dark:border-gray-700",
            value === preset.value
              ? "bg-primary-600 text-white dark:bg-primary-500"
              : "bg-white dark:bg-gray-900 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800"
          )}
        >
          {preset.label}
        </button>
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// KindToggle — Users / Groups selector
// ---------------------------------------------------------------------------

type PrincipalKind = "users" | "groups";

function KindToggle({
  value,
  onChange,
}: {
  value: PrincipalKind;
  onChange: (v: PrincipalKind) => void;
}) {
  return (
    <div className="flex rounded-(--radius-component) overflow-hidden border border-gray-200 dark:border-gray-700 shrink-0">
      {(["users", "groups"] as const).map((kind, idx) => (
        <button
          key={kind}
          type="button"
          onClick={() => onChange(kind)}
          data-testid={`add-kind-${kind}`}
          className={cn(
            "px-2.5 py-1 text-xs font-medium capitalize transition-colors",
            idx > 0 && "border-l border-gray-200 dark:border-gray-700",
            value === kind
              ? "bg-gray-900 text-white dark:bg-gray-100 dark:text-gray-900"
              : "bg-white dark:bg-gray-900 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800"
          )}
        >
          {kind}
        </button>
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// AclEditor
// ---------------------------------------------------------------------------

export interface AclEditorProps {
  /** Current ACL to edit (read on modal open). */
  acl: AccessControlStore;
  /** Called with the updated ACL when the user clicks "Save Changes". */
  onSave: (newAcl: AccessControlStore) => void;
  /**
   * The element that opens the editor. Must be a single React element so it
   * can be wrapped by Modal.Trigger with asChild.
   */
  trigger: React.ReactElement;
}

export function AclEditor({ acl, onSave, trigger }: AclEditorProps) {
  const [open, setOpen] = useState(false);
  const [entries, setEntries] = useState<AclEntry[]>([]);
  const [addKind, setAddKind] = useState<PrincipalKind>("users");
  const [addPermissions, setAddPermissions] = useState<number>(7);

  const handleOpenChange = (isOpen: boolean) => {
    if (isOpen) {
      setEntries(flattenAcl(acl));
      setAddPermissions(7);
      setAddKind("users");
    }
    setOpen(isOpen);
  };

  const handleSave = () => {
    onSave(buildAcl(entries));
    setOpen(false);
  };

  const handleRemove = (id: string) => {
    setEntries((prev) => prev.filter((e) => e._id !== id));
  };

  /** Called when a principal is selected in the ResourcePicker. */
  const handleSelect = (principalId: string) => {
    // Skip if this exact (principal, permission) pair already exists
    if (
      entries.some(
        (e) => e.principal === principalId && e.permissions === addPermissions
      )
    )
      return;
    setEntries((prev) => [
      ...prev,
      {
        principal: principalId,
        permissions: addPermissions,
        _id: `${principalId}::${addPermissions}::${Math.random()}`,
      },
    ]);
  };

  return (
    <Modal.Root open={open} onOpenChange={handleOpenChange}>
      <Modal.Trigger asChild>{trigger}</Modal.Trigger>

      <Modal.Content className="max-w-xl" data-testid="acl-editor-modal">
        <Modal.Header>
          <Modal.Title>Access Control</Modal.Title>
          <Modal.Description>
            Manage who can read or modify this resource. An empty list grants
            access to all authenticated users.
          </Modal.Description>
        </Modal.Header>

        {/* ── Current entries ──────────────────────────────────────────── */}
        <div className="mt-4">
          {entries.length === 0 ? (
            <div className="flex items-center gap-2 py-4 text-sm text-gray-500 dark:text-gray-400">
              <Shield className="w-4 h-4 shrink-0" />
              <span>No entries — all authenticated users have access.</span>
            </div>
          ) : (
            <div className="max-h-56 overflow-y-auto rounded-(--radius-component) border border-gray-200 dark:border-gray-700">
              <Table.Root>
                <Table.Header>
                  <Table.Row>
                    <Table.Head>Principal</Table.Head>
                    <Table.Head>Access</Table.Head>
                    <Table.Head className="w-10" />
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {entries.map((entry) => (
                    <Table.Row
                      key={entry._id}
                      data-testid={`acl-entry-${entry.principal}`}
                    >
                      <Table.Cell className="font-mono text-xs">
                        {entry.principal}
                      </Table.Cell>
                      <Table.Cell>
                        <PermissionBadge permissions={entry.permissions} />
                      </Table.Cell>
                      <Table.Cell className="text-right">
                        <Button
                          variant="ghost"
                          size="icon"
                          type="button"
                          onClick={() => handleRemove(entry._id)}
                          data-testid={`remove-acl-${entry.principal}`}
                          className="text-red-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                        >
                          <X className="w-3.5 h-3.5" />
                        </Button>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </Table.Body>
              </Table.Root>
            </div>
          )}
        </div>

        {/* ── Add entry ─────────────────────────────────────────────────── */}
        <div className="mt-5 space-y-3">
          <p className="text-xs font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500">
            Add Entry
          </p>

          {/* Permission selector */}
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-500 dark:text-gray-400 shrink-0 w-14">
              Access:
            </span>
            <PermissionToggle
              value={addPermissions}
              onChange={setAddPermissions}
            />
          </div>

          {/* Kind toggle + ResourcePicker */}
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-500 dark:text-gray-400 shrink-0 w-14">
              Search:
            </span>
            <KindToggle value={addKind} onChange={setAddKind} />
            <ResourcePicker
              kind={addKind}
              placeholder={
                addKind === "users" ? "Search users…" : "Search groups…"
              }
              onSelect={handleSelect}
              className="flex-1"
              data-testid="acl-principal-picker"
            />
          </div>

          <Paragraph variant="subtle" className="text-xs">
            Select a user or group — they'll be added immediately with{" "}
            <strong>
              {PERMISSION_PRESETS.find((p) => p.value === addPermissions)
                ?.label ?? addPermissions}
            </strong>{" "}
            access.
          </Paragraph>
        </div>

        <Modal.Footer>
          <Button
            variant="secondary"
            type="button"
            onClick={() => setOpen(false)}
            data-testid="acl-cancel"
          >
            Cancel
          </Button>
          <Button
            variant="primary"
            type="button"
            onClick={handleSave}
            data-testid="acl-save"
          >
            Save Changes
          </Button>
        </Modal.Footer>
      </Modal.Content>
    </Modal.Root>
  );
}
