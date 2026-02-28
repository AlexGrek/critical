import type { Route } from "./+types/groups";
import { useLoaderData, useFetcher, useRevalidator } from "react-router";
import {
  Button,
  Input,
  MorphModal,
  AclEditor,
  PermissionBadge,
  Card,
  H1,
  H2,
  H3,
  Paragraph,
  Table,
  Tabs,
  ResourcePicker,
} from "~/components";
import type { AccessControlStore } from "~/components";
import { useState, useEffect, useCallback, useRef } from "react";
import {
  Plus,
  X,
  Trash2,
  Pencil,
  Loader2,
  ChevronRight,
  AlertCircle,
  CheckCircle2,
} from "lucide-react";
import { cn, formatDate } from "~/lib/utils";

// ---------------------------------------------------------------------------
// API types
// ---------------------------------------------------------------------------

interface ResourceState {
  created_at: string;
  created_by?: string;
  updated_at: string;
  updated_by?: string;
}

interface DeletionInfo {
  deleted_at: string;
  deleted_by: string;
}

interface GroupBrief {
  id: string;
  name: string;
  labels: Record<string, string>;
}

interface GroupFull extends GroupBrief {
  description?: string;
  annotations: Record<string, string>;
  acl: AccessControlStore;
  state: ResourceState;
  deletion?: DeletionInfo;
  hash_code: string;
}

interface MembershipBrief {
  id: string;
  principal: string;
  group: string;
  deletion?: DeletionInfo;
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

export function meta({}: Route.MetaArgs) {
  return [
    { title: "{!} Groups - Critical" },
    { name: "description", content: "View and manage all groups in the system" },
  ];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

export async function loader({ request }: Route.LoaderArgs) {
  const res = await fetch("http://localhost:3742/api/v1/global/groups", {
    headers: { Cookie: request.headers.get("Cookie") || "" },
  });

  if (!res.ok) {
    if (res.status === 401 || res.status === 403) {
      throw new Response("Unauthorized", { status: 401 });
    }
    throw new Response("Failed to load groups", { status: res.status });
  }

  const data: { items: GroupBrief[] } = await res.json();
  return { groups: data.items };
}

// ---------------------------------------------------------------------------
// Action helpers
// ---------------------------------------------------------------------------

/**
 * Extracts a user-readable message from a failed API response.
 * Tries JSON first ({"error":"..."}, {"message":"..."}), falls back to text.
 */
async function parseApiError(res: Response): Promise<string> {
  let text = "";
  try {
    text = await res.text();
  } catch {
    return `Request failed (HTTP ${res.status})`;
  }
  try {
    const json = JSON.parse(text) as Record<string, unknown>;
    const top = json.error ?? json.message ?? json.detail ?? json.msg;
    if (typeof top === "string" && top) return top;
    // Handle nested: {"error":{"message":"..."}}
    if (top !== null && typeof top === "object") {
      const nested = (top as Record<string, unknown>).message ?? (top as Record<string, unknown>).detail;
      if (typeof nested === "string" && nested) return nested;
    }
  } catch {
    /* not JSON */
  }
  const trimmed = text.trim();
  return trimmed || `Request failed (HTTP ${res.status})`;
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

type ActionResult =
  | { success: true; intent: string }
  | { error: string; intent: string };

export async function action({ request }: Route.ActionArgs): Promise<ActionResult> {
  const formData = await request.formData();
  const intent = (formData.get("intent") as string) ?? "";
  const cookie = request.headers.get("Cookie") || "";

  switch (intent) {
    case "create": {
      const name = formData.get("name") as string;
      const id = formData.get("id") as string;
      if (!name || !id) return { error: "Name and ID are required", intent };

      const res = await fetch("http://localhost:3742/api/v1/global/groups", {
        method: "POST",
        headers: { "Content-Type": "application/json", Cookie: cookie },
        body: JSON.stringify({ name, id }),
      });
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    case "update-group": {
      const groupId = formData.get("groupId") as string;
      const groupJson = formData.get("groupJson") as string;

      const res = await fetch(
        `http://localhost:3742/api/v1/global/groups/${groupId}`,
        {
          method: "PUT",
          headers: { "Content-Type": "application/json", Cookie: cookie },
          body: groupJson,
        }
      );
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    case "delete-group": {
      const groupId = formData.get("groupId") as string;
      const res = await fetch(
        `http://localhost:3742/api/v1/global/groups/${groupId}`,
        { method: "DELETE", headers: { Cookie: cookie } }
      );
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    case "add-member": {
      const principal = formData.get("principal") as string;
      const groupId = formData.get("groupId") as string;
      const key = `${principal}::${groupId}`;
      const res = await fetch(
        `http://localhost:3742/api/v1/global/memberships/${key}`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json", Cookie: cookie },
          body: JSON.stringify({ id: key, principal, group: groupId }),
        }
      );
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    case "remove-member": {
      const key = formData.get("membershipKey") as string;
      const res = await fetch(
        `http://localhost:3742/api/v1/global/memberships/${key}`,
        { method: "DELETE", headers: { Cookie: cookie } }
      );
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    default:
      return { error: "Unknown action", intent };
  }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

function validateGroupId(id: string): string | null {
  const bare = id.startsWith("g_") ? id.slice(2) : id;
  if (bare.length < 2) return "Must be at least 2 characters (excluding g_ prefix)";
  if (bare.length > 63) return "Must be at most 63 characters (excluding g_ prefix)";
  if (!/^[a-z0-9_]+$/.test(bare))
    return "Only lowercase letters, numbers, and underscores";
  if (/^[0-9]/.test(bare)) return "Cannot start with a digit";
  return null;
}

// ---------------------------------------------------------------------------
// Edit-form shape
// ---------------------------------------------------------------------------

interface LabelEntry {
  key: string;
  value: string;
  _id: string;
}

interface EditForm {
  name: string;
  description: string;
  labels: LabelEntry[];
}

function groupToEditForm(group: GroupFull): EditForm {
  return {
    name: group.name,
    description: group.description ?? "",
    labels: Object.entries(group.labels).map(([key, value]) => ({
      key,
      value,
      _id: `${key}-${Math.random()}`,
    })),
  };
}

// ---------------------------------------------------------------------------
// Page component
// ---------------------------------------------------------------------------

export default function Groups() {
  const { groups } = useLoaderData<typeof loader>();
  const fetcher = useFetcher<ActionResult>();
  const revalidator = useRevalidator();
  const prevFetcherState = useRef<string>("idle");

  // ---- Create modal ----
  const [createOpen, setCreateOpen] = useState(false);
  const [createForm, setCreateForm] = useState({ name: "", id: "" });
  const [createError, setCreateError] = useState("");

  // ---- Editor panel ----
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [editingGroup, setEditingGroup] = useState<GroupFull | null>(null);
  const [editingMembers, setEditingMembers] = useState<MembershipBrief[]>([]);
  const [editorLoading, setEditorLoading] = useState(false);

  // Separate error states — each surfaced near the relevant action
  const [loadError, setLoadError] = useState("");     // open editor / reload
  const [saveError, setSaveError] = useState("");     // PUT update-group
  const [memberError, setMemberError] = useState(""); // add / remove member
  const [tableError, setTableError] = useState("");   // delete-group row action

  // Brief "Saved!" indicator — auto-clears after 2.5 s
  const [saveSuccess, setSaveSuccess] = useState(false);
  const saveSuccessTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [editForm, setEditForm] = useState<EditForm>({
    name: "",
    description: "",
    labels: [],
  });
  // Pending ACL changes — null means "use what came from the server"
  const [editAcl, setEditAcl] = useState<AccessControlStore | null>(null);
  const [confirmDeleteGroupId, setConfirmDeleteGroupId] = useState<
    string | null
  >(null);

  // ---- Load group details + memberships ----
  const loadGroup = useCallback(async (groupId: string) => {
    setEditorLoading(true);
    setLoadError("");
    try {
      const [groupRes, membershipsRes] = await Promise.all([
        fetch(`/api/v1/global/groups/${groupId}`),
        fetch(`/api/v1/global/memberships`),
      ]);

      if (groupRes.status === 401 || groupRes.status === 403)
        throw new Error("Not authorized to view this group");
      if (groupRes.status === 404)
        throw new Error("Group not found");
      if (!groupRes.ok)
        throw new Error(`Failed to load group (HTTP ${groupRes.status})`);
      if (!membershipsRes.ok)
        throw new Error(`Failed to load memberships (HTTP ${membershipsRes.status})`);

      const group: GroupFull = await groupRes.json();
      const { items }: { items: MembershipBrief[] } = await membershipsRes.json();

      setEditingGroup(group);
      setEditingMembers(
        items.filter((m) => m.group === groupId && !m.deletion)
      );
      setEditForm(groupToEditForm(group));
      setEditAcl(null);
    } catch (e) {
      setLoadError(e instanceof Error ? e.message : "Failed to load group details");
    } finally {
      setEditorLoading(false);
    }
  }, []);

  const handleEdit = (groupId: string) => {
    setSelectedGroupId(groupId);
    setLoadError("");
    setSaveError("");
    setMemberError("");
    setConfirmDeleteGroupId(null);
    loadGroup(groupId);
  };

  const closeEditor = () => {
    setSelectedGroupId(null);
    setEditingGroup(null);
    setEditingMembers([]);
    setEditAcl(null);
    setLoadError("");
    setSaveError("");
    setMemberError("");
    setConfirmDeleteGroupId(null);
    if (saveSuccessTimer.current) clearTimeout(saveSuccessTimer.current);
    setSaveSuccess(false);
  };

  // ---- Mutations ----
  const submitSave = () => {
    if (!editingGroup) return;
    setSaveError("");
    const labels = Object.fromEntries(
      editForm.labels
        .filter((l) => l.key.trim())
        .map((l) => [l.key.trim(), l.value])
    );
    const updated: GroupFull = {
      ...editingGroup,
      name: editForm.name,
      description: editForm.description || undefined,
      labels,
      acl: editAcl ?? editingGroup.acl,
    };
    const form = new FormData();
    form.append("intent", "update-group");
    form.append("groupId", editingGroup.id);
    form.append("groupJson", JSON.stringify(updated));
    fetcher.submit(form, { method: "POST" });
  };

  const submitAddMember = (principalId: string) => {
    if (!selectedGroupId) return;
    setMemberError("");
    const form = new FormData();
    form.append("intent", "add-member");
    form.append("principal", principalId);
    form.append("groupId", selectedGroupId);
    fetcher.submit(form, { method: "POST" });
  };

  const submitRemoveMember = (membershipKey: string) => {
    setMemberError("");
    const form = new FormData();
    form.append("intent", "remove-member");
    form.append("membershipKey", membershipKey);
    fetcher.submit(form, { method: "POST" });
  };

  const submitDeleteGroup = (groupId: string) => {
    setTableError("");
    const form = new FormData();
    form.append("intent", "delete-group");
    form.append("groupId", groupId);
    fetcher.submit(form, { method: "POST" });
  };

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setCreateError("");
    const idErr = validateGroupId(createForm.id);
    if (idErr) { setCreateError(idErr); return; }
    const form = new FormData();
    form.append("intent", "create");
    form.append("name", createForm.name);
    form.append("id", createForm.id);
    fetcher.submit(form, { method: "POST" });
  };

  // ---- Handle fetcher completion ----
  useEffect(() => {
    // React Router fetcher goes submitting → loading → idle (loaders revalidate
    // after the action). Track any transition INTO idle from a non-idle state.
    const prevState = prevFetcherState.current;
    prevFetcherState.current = fetcher.state;
    if (fetcher.state !== "idle" || prevState === "idle" || !fetcher.data) return;

    const data = fetcher.data;

    if ("error" in data) {
      switch (data.intent) {
        case "create":
          setCreateError(data.error);
          break;
        case "update-group":
          setSaveError(data.error);
          break;
        case "add-member":
        case "remove-member":
          setMemberError(data.error);
          break;
        case "delete-group":
          setTableError(data.error);
          setConfirmDeleteGroupId(null);
          break;
      }
      return;
    }

    // success
    const { intent } = data;

    if (intent === "create") {
      setCreateOpen(false);
      setCreateForm({ name: "", id: "" });
      setCreateError("");
      revalidator.revalidate();
    }

    if (
      (intent === "update-group" ||
        intent === "add-member" ||
        intent === "remove-member") &&
      selectedGroupId
    ) {
      loadGroup(selectedGroupId);
      revalidator.revalidate();
    }

    if (intent === "update-group") {
      setSaveError("");
      setSaveSuccess(true);
      if (saveSuccessTimer.current) clearTimeout(saveSuccessTimer.current);
      saveSuccessTimer.current = setTimeout(() => setSaveSuccess(false), 2500);
    }

    if (intent === "add-member" || intent === "remove-member") {
      setMemberError("");
    }

    if (intent === "delete-group") {
      closeEditor();
      setTableError("");
      revalidator.revalidate();
    }
  }, [fetcher.state, fetcher.data, selectedGroupId, loadGroup, revalidator]);

  // Cleanup saveSuccess timer on unmount
  useEffect(
    () => () => {
      if (saveSuccessTimer.current) clearTimeout(saveSuccessTimer.current);
    },
    []
  );

  const isSubmitting = fetcher.state === "submitting";

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-7xl mx-auto">

        {/* Page header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <H1 data-testid="groups-page-heading">{"{!} "}Groups</H1>
            <Paragraph variant="muted" className="mt-0.5">
              Manage groups and memberships
            </Paragraph>
          </div>

          <MorphModal
            trigger={
              <Button variant="primary" data-testid="create-group-button">
                <Plus className="w-4 h-4 mr-1.5" />
                New Group
              </Button>
            }
            modalWidth={480}
            modalHeight={400}
            isOpen={createOpen}
            onOpenChange={(open) => {
              setCreateOpen(open);
              if (!open) {
                setCreateForm({ name: "", id: "" });
                setCreateError("");
              }
            }}
          >
            {(close) => (
              <div className="flex flex-col h-full">
                <H2 data-testid="create-group-modal-title" className="mb-6">
                  Create New Group
                </H2>
                <form
                  onSubmit={handleCreateSubmit}
                  className="flex-1 flex flex-col"
                >
                  <div className="flex-1 space-y-4">
                    {createError && (
                      <ErrorBanner
                        data-testid="create-group-error"
                        message={createError}
                      />
                    )}
                    <div>
                      <label
                        htmlFor="group-name"
                        className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5"
                      >
                        Group Name
                      </label>
                      <Input
                        id="group-name"
                        data-testid="group-name-input"
                        value={createForm.name}
                        onChange={(e) =>
                          setCreateForm({ ...createForm, name: e.target.value })
                        }
                        placeholder="Engineering Team"
                        required
                      />
                    </div>
                    <div>
                      <label
                        htmlFor="group-id"
                        className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5"
                      >
                        Group ID
                      </label>
                      <Input
                        id="group-id"
                        monospace
                        data-testid="group-id-input"
                        value={createForm.id}
                        onChange={(e) =>
                          setCreateForm({
                            ...createForm,
                            id: e.target.value.toLowerCase(),
                          })
                        }
                        placeholder="engineering_team"
                        required
                      />
                      <Paragraph
                        variant="subtle"
                        className="mt-1 text-xs"
                        data-testid="group-id-hint"
                      >
                        2–63 chars, lowercase + underscores. The{" "}
                        <span className="font-mono">g_</span> prefix is added
                        automatically.
                      </Paragraph>
                    </div>
                  </div>
                  <div className="flex gap-3 pt-4 border-t border-gray-200 dark:border-gray-800">
                    <Button
                      type="submit"
                      variant="primary"
                      data-testid="submit-create-group"
                      disabled={isSubmitting}
                    >
                      {isSubmitting ? (
                        <>
                          <Loader2 className="w-4 h-4 mr-1.5 animate-spin" />
                          Creating…
                        </>
                      ) : (
                        "Create Group"
                      )}
                    </Button>
                    <Button
                      type="button"
                      variant="secondary"
                      onClick={close}
                      data-testid="cancel-create-group"
                    >
                      Cancel
                    </Button>
                  </div>
                </form>
              </div>
            )}
          </MorphModal>
        </div>

        {/* Table-level error (delete-group failures) */}
        {tableError && (
          <ErrorBanner
            message={tableError}
            onDismiss={() => setTableError("")}
            className="mb-4"
            data-testid="table-error"
          />
        )}

        {/* Main grid: table + optional editor */}
        <div
          className={cn(
            "grid gap-6 items-start",
            selectedGroupId
              ? "grid-cols-1 xl:grid-cols-[1fr_420px]"
              : "grid-cols-1"
          )}
        >
          {/* Groups table */}
          <Card className="overflow-hidden" data-testid="groups-table-card">
            <Table.Root>
              <Table.Header>
                <Table.Row>
                  <Table.Head>Name</Table.Head>
                  <Table.Head>ID</Table.Head>
                  <Table.Head>Labels</Table.Head>
                  <Table.Head className="text-right pr-4">Actions</Table.Head>
                </Table.Row>
              </Table.Header>

              {groups.length === 0 ? (
                <Table.Empty colSpan={4} data-testid="groups-empty-state">
                  No groups yet — create one to get started
                </Table.Empty>
              ) : (
                <Table.Body>
                  {groups.map((group) => (
                    <Table.Row
                      key={group.id}
                      selected={group.id === selectedGroupId}
                      data-testid={`group-row-${group.id}`}
                    >
                      <Table.Cell className="font-medium">
                        {group.name}
                      </Table.Cell>

                      <Table.Cell>
                        <span
                          className="font-mono text-xs text-gray-500 dark:text-gray-400"
                          data-testid={`group-id-label-${group.id}`}
                        >
                          {group.id}
                        </span>
                      </Table.Cell>

                      <Table.Cell>
                        <LabelBadges labels={group.labels} max={3} />
                      </Table.Cell>

                      <Table.Cell className="text-right pr-4">
                        <div className="flex items-center justify-end gap-2">
                          {confirmDeleteGroupId === group.id ? (
                            <>
                              <Paragraph
                                variant="danger"
                                className="text-xs mr-1"
                              >
                                Delete?
                              </Paragraph>
                              <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => submitDeleteGroup(group.id)}
                                disabled={isSubmitting}
                                data-testid={`confirm-delete-group-${group.id}`}
                              >
                                Yes
                              </Button>
                              <Button
                                variant="secondary"
                                size="sm"
                                onClick={() => setConfirmDeleteGroupId(null)}
                                data-testid={`cancel-delete-group-${group.id}`}
                              >
                                No
                              </Button>
                            </>
                          ) : (
                            <>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() =>
                                  setConfirmDeleteGroupId(group.id)
                                }
                                data-testid={`delete-group-${group.id}`}
                                className="text-red-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                              >
                                <Trash2 className="w-3.5 h-3.5" />
                              </Button>
                              <Button
                                variant={
                                  group.id === selectedGroupId
                                    ? "primary"
                                    : "outline"
                                }
                                size="sm"
                                onClick={() =>
                                  group.id === selectedGroupId
                                    ? closeEditor()
                                    : handleEdit(group.id)
                                }
                                data-testid={`edit-group-${group.id}`}
                              >
                                <Pencil className="w-3.5 h-3.5 mr-1" />
                                {group.id === selectedGroupId ? "Close" : "Edit"}
                                {group.id !== selectedGroupId && (
                                  <ChevronRight className="w-3.5 h-3.5 ml-0.5" />
                                )}
                              </Button>
                            </>
                          )}
                        </div>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </Table.Body>
              )}
            </Table.Root>
          </Card>

          {/* Editor panel */}
          {selectedGroupId && (
            <GroupEditor
              groupId={selectedGroupId}
              group={editingGroup}
              members={editingMembers}
              loading={editorLoading}
              loadError={loadError}
              saveError={saveError}
              memberError={memberError}
              saveSuccess={saveSuccess}
              editForm={editForm}
              setEditForm={setEditForm}
              editAcl={editAcl}
              onAclChange={setEditAcl}
              isSubmitting={isSubmitting}
              onClose={closeEditor}
              onSave={submitSave}
              onAddMember={submitAddMember}
              onRemoveMember={submitRemoveMember}
              onDismissLoadError={() => setLoadError("")}
              onDismissSaveError={() => setSaveError("")}
              onDismissMemberError={() => setMemberError("")}
            />
          )}
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// GroupEditor — right-hand editor panel
// ---------------------------------------------------------------------------

interface GroupEditorProps {
  groupId: string;
  group: GroupFull | null;
  members: MembershipBrief[];
  loading: boolean;
  loadError: string;
  saveError: string;
  memberError: string;
  saveSuccess: boolean;
  editForm: EditForm;
  setEditForm: React.Dispatch<React.SetStateAction<EditForm>>;
  editAcl: AccessControlStore | null;
  onAclChange: (acl: AccessControlStore) => void;
  isSubmitting: boolean;
  onClose: () => void;
  onSave: () => void;
  onAddMember: (principalId: string) => void;
  onRemoveMember: (key: string) => void;
  onDismissLoadError: () => void;
  onDismissSaveError: () => void;
  onDismissMemberError: () => void;
}

function GroupEditor({
  groupId,
  group,
  members,
  loading,
  loadError,
  saveError,
  memberError,
  saveSuccess,
  editForm,
  setEditForm,
  editAcl,
  onAclChange,
  isSubmitting,
  onClose,
  onSave,
  onAddMember,
  onRemoveMember,
  onDismissLoadError,
  onDismissSaveError,
  onDismissMemberError,
}: GroupEditorProps) {
  const currentAcl = editAcl ?? group?.acl;

  return (
    <Card
      className="overflow-hidden flex flex-col sticky top-4 max-h-[calc(100vh-120px)]"
      data-testid="group-editor-panel"
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-800 shrink-0">
        <div className="min-w-0">
          <H3 className="text-base truncate">{group?.name ?? groupId}</H3>
          <span className="font-mono text-xs text-gray-400 dark:text-gray-500">
            {groupId}
          </span>
        </div>
        <Button
          variant="ghost"
          size="icon"
          onClick={onClose}
          data-testid="close-editor"
          className="shrink-0 ml-2"
        >
          <X className="w-4 h-4" />
        </Button>
      </div>

      {loading ? (
        <div
          className="flex flex-1 items-center justify-center py-16"
          data-testid="editor-loading"
        >
          <Loader2 className="w-5 h-5 animate-spin text-gray-400" />
        </div>
      ) : (
        <Tabs.Root
          defaultValue="details"
          className="flex flex-col flex-1 min-h-0"
        >
          {/* Tab strip */}
          <Tabs.List className="shrink-0 px-2">
            <Tabs.Trigger value="details" data-testid="tab-details">
              Details
            </Tabs.Trigger>
            <Tabs.Trigger value="labels" data-testid="tab-labels">
              Labels
              {editForm.labels.length > 0 && (
                <span className="ml-1.5 inline-flex items-center justify-center w-4 h-4 rounded-full text-[10px] font-mono bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300">
                  {editForm.labels.length}
                </span>
              )}
            </Tabs.Trigger>
            <Tabs.Trigger value="members" data-testid="tab-members">
              Members
              {members.length > 0 && (
                <span className="ml-1.5 inline-flex items-center justify-center w-4 h-4 rounded-full text-[10px] font-mono bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300">
                  {members.length}
                </span>
              )}
            </Tabs.Trigger>
            <Tabs.Trigger value="access" data-testid="tab-access">
              Access
              {editAcl && (
                <span className="ml-1.5 w-1.5 h-1.5 rounded-full bg-amber-400 inline-block" />
              )}
            </Tabs.Trigger>
          </Tabs.List>

          {/* Load error — visible on any tab */}
          {loadError && (
            <div className="shrink-0 px-4 pt-3">
              <ErrorBanner
                message={loadError}
                onDismiss={onDismissLoadError}
                data-testid="editor-load-error"
              />
            </div>
          )}

          {/* Scrollable tab content */}
          <div className="flex-1 overflow-y-auto min-h-0">

            {/* ── Details ── */}
            <Tabs.Content value="details" className="p-4 space-y-3">
              <div>
                <label
                  htmlFor="edit-group-name"
                  className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1"
                >
                  Name
                </label>
                <Input
                  id="edit-group-name"
                  value={editForm.name}
                  onChange={(e) =>
                    setEditForm((f) => ({ ...f, name: e.target.value }))
                  }
                  placeholder="Group name"
                  data-testid="edit-group-name"
                />
              </div>
              <div>
                <label
                  htmlFor="edit-group-description"
                  className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1"
                >
                  Description
                </label>
                <Input
                  id="edit-group-description"
                  value={editForm.description}
                  onChange={(e) =>
                    setEditForm((f) => ({
                      ...f,
                      description: e.target.value,
                    }))
                  }
                  placeholder="Optional description"
                  data-testid="edit-group-description"
                />
              </div>
              {group?.state && (
                <div className="pt-1 space-y-1">
                  <Paragraph variant="subtle" className="text-xs">
                    Created:{" "}
                    <span className="font-mono">
                      {formatDate(group.state.created_at)}
                    </span>
                  </Paragraph>
                  {group.state.updated_at && (
                    <Paragraph variant="subtle" className="text-xs">
                      Updated:{" "}
                      <span className="font-mono">
                        {formatDate(group.state.updated_at)}
                      </span>
                    </Paragraph>
                  )}
                </div>
              )}
            </Tabs.Content>

            {/* ── Labels ── */}
            <Tabs.Content value="labels" className="p-4">
              <div className="flex items-center justify-between mb-3">
                <Paragraph variant="muted" className="text-xs">
                  {editForm.labels.length === 0
                    ? "No labels yet"
                    : `${editForm.labels.length} label${editForm.labels.length !== 1 ? "s" : ""}`}
                </Paragraph>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() =>
                    setEditForm((f) => ({
                      ...f,
                      labels: [
                        ...f.labels,
                        { key: "", value: "", _id: Math.random().toString() },
                      ],
                    }))
                  }
                  data-testid="add-label-button"
                >
                  <Plus className="w-3.5 h-3.5 mr-1" />
                  Add
                </Button>
              </div>
              <div className="space-y-2">
                {editForm.labels.map((label, idx) => (
                  <div key={label._id} className="flex gap-2 items-center">
                    <Input
                      value={label.key}
                      onChange={(e) =>
                        setEditForm((f) => {
                          const labels = [...f.labels];
                          labels[idx] = {
                            ...labels[idx],
                            key: e.target.value,
                          };
                          return { ...f, labels };
                        })
                      }
                      placeholder="key"
                      monospace
                      className="flex-1 min-w-0"
                      data-testid={`label-key-${idx}`}
                    />
                    <span className="text-gray-400 shrink-0">=</span>
                    <Input
                      value={label.value}
                      onChange={(e) =>
                        setEditForm((f) => {
                          const labels = [...f.labels];
                          labels[idx] = {
                            ...labels[idx],
                            value: e.target.value,
                          };
                          return { ...f, labels };
                        })
                      }
                      placeholder="value"
                      monospace
                      className="flex-1 min-w-0"
                      data-testid={`label-value-${idx}`}
                    />
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() =>
                        setEditForm((f) => ({
                          ...f,
                          labels: f.labels.filter((_, i) => i !== idx),
                        }))
                      }
                      data-testid={`remove-label-${idx}`}
                      className="shrink-0"
                    >
                      <X className="w-3.5 h-3.5" />
                    </Button>
                  </div>
                ))}
              </div>
            </Tabs.Content>

            {/* ── Members ── */}
            <Tabs.Content value="members" className="p-4 space-y-3">
              <Table.Root>
                <Table.Header>
                  <Table.Row>
                    <Table.Head>Principal</Table.Head>
                    <Table.Head className="w-12" />
                  </Table.Row>
                </Table.Header>
                {members.length === 0 ? (
                  <Table.Empty colSpan={2}>No members</Table.Empty>
                ) : (
                  <Table.Body>
                    {members.map((member) => (
                      <Table.Row
                        key={member.id}
                        data-testid={`member-row-${member.principal}`}
                      >
                        <Table.Cell className="font-mono text-xs">
                          {member.principal}
                        </Table.Cell>
                        <Table.Cell className="text-right">
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => onRemoveMember(member.id)}
                            disabled={isSubmitting}
                            data-testid={`remove-member-${member.principal}`}
                            className="text-red-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </Button>
                        </Table.Cell>
                      </Table.Row>
                    ))}
                  </Table.Body>
                )}
              </Table.Root>

              <ResourcePicker
                kind="users"
                placeholder="Search users to add…"
                onSelect={(id) => onAddMember(id)}
                disabled={isSubmitting}
                data-testid="add-member-picker"
              />

              {memberError && (
                <ErrorBanner
                  message={memberError}
                  onDismiss={onDismissMemberError}
                  data-testid="member-error"
                />
              )}
            </Tabs.Content>

            {/* ── Access ── */}
            <Tabs.Content value="access" className="p-4 space-y-3">
              {group && currentAcl ? (
                <>
                  <div className="flex items-center justify-between">
                    <Paragraph variant="muted" className="text-xs">
                      {currentAcl.list.length === 0
                        ? "Open to all authenticated users"
                        : `${currentAcl.list.length} ACL entr${currentAcl.list.length !== 1 ? "ies" : "y"}`}
                    </Paragraph>
                    <AclEditor
                      acl={currentAcl}
                      onSave={onAclChange}
                      trigger={
                        <Button
                          variant="outline"
                          size="sm"
                          data-testid="edit-acl-button"
                        >
                          Edit
                        </Button>
                      }
                    />
                  </div>

                  {currentAcl.list.length > 0 && (
                    <div className="space-y-1.5">
                      {currentAcl.list.flatMap((entry) =>
                        entry.principals.map((principal) => (
                          <div
                            key={`${principal}-${entry.permissions}`}
                            className="flex items-center gap-2"
                          >
                            <span className="font-mono text-xs text-gray-600 dark:text-gray-400 truncate flex-1">
                              {principal}
                            </span>
                            <PermissionBadge permissions={entry.permissions} />
                          </div>
                        ))
                      )}
                    </div>
                  )}

                  {editAcl && (
                    <Paragraph variant="warning" className="text-xs">
                      Unsaved ACL changes — click "Save Changes" to apply.
                    </Paragraph>
                  )}
                </>
              ) : (
                <Paragraph variant="subtle" className="text-xs">
                  Load a group to view its access control list.
                </Paragraph>
              )}
            </Tabs.Content>

          </div>
        </Tabs.Root>
      )}

      {/* Footer */}
      {!loading && group && (
        <div className="border-t border-gray-200 dark:border-gray-800 px-4 py-3 shrink-0 space-y-3">
          {saveError && (
            <ErrorBanner
              message={saveError}
              onDismiss={onDismissSaveError}
              data-testid="save-error"
            />
          )}

          <div className="flex items-center gap-2 justify-end">
            {saveSuccess && (
              <span
                className="flex items-center gap-1.5 text-sm text-green-600 dark:text-green-400"
                data-testid="save-success"
              >
                <CheckCircle2 className="w-4 h-4" />
                Saved
              </span>
            )}

            <Button
              variant="secondary"
              onClick={onClose}
              data-testid="cancel-edit"
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={onSave}
              disabled={isSubmitting || !editForm.name.trim()}
              data-testid="save-group-button"
            >
              {isSubmitting ? (
                <>
                  <Loader2 className="w-4 h-4 mr-1.5 animate-spin" />
                  Saving…
                </>
              ) : (
                "Save Changes"
              )}
            </Button>
          </div>
        </div>
      )}
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

function ErrorBanner({
  message,
  onDismiss,
  className,
  ...props
}: {
  message: string;
  onDismiss?: () => void;
  className?: string;
} & Omit<React.HTMLAttributes<HTMLDivElement>, "className">) {
  return (
    <div
      role="alert"
      className={cn(
        "flex items-start gap-2 rounded-(--radius-component) px-3 py-2.5 text-sm",
        "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800",
        "text-red-700 dark:text-red-400",
        className
      )}
      {...props}
    >
      <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
      <span className="flex-1">{message}</span>
      {onDismiss && (
        <button
          type="button"
          onClick={onDismiss}
          aria-label="Dismiss"
          className="shrink-0 transition-colors hover:text-red-600 dark:hover:text-red-200"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      )}
    </div>
  );
}


function LabelBadges({
  labels,
  max = 3,
}: {
  labels: Record<string, string>;
  max?: number;
}) {
  const entries = Object.entries(labels);
  if (entries.length === 0) return null;
  const visible = entries.slice(0, max);
  const overflow = entries.length - max;
  return (
    <div className="flex flex-wrap gap-1">
      {visible.map(([k, v]) => (
        <span
          key={k}
          className="inline-flex items-center px-1.5 py-0.5 rounded-(--radius-component) text-xs font-mono bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400"
        >
          {k}={v}
        </span>
      ))}
      {overflow > 0 && (
        <span className="inline-flex items-center px-1.5 py-0.5 rounded-(--radius-component) text-xs font-mono bg-gray-100 dark:bg-gray-800 text-gray-500 dark:text-gray-500">
          +{overflow}
        </span>
      )}
    </div>
  );
}
