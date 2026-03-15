import type { Route } from "./+types/project-tickets";
import { useLoaderData, useFetcher, useRevalidator, Link } from "react-router";
import {
  Button,
  Input,
  MorphModal,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  H1,
  H2,
  H3,
  Paragraph,
  Tabs,
  YamlEditorDrawer,
  Modal,
} from "~/components";
import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import {
  Plus,
  Trash2,
  Pencil,
  Loader2,
  AlertCircle,
  ChevronRight,
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

interface TicketStatus {
  name: string;
  category: "todo" | "in_progress" | "done";
}

interface TicketFieldDef {
  name: string;
  field_type: string;
  required: boolean;
  description?: string;
  options?: string[];
}

interface TicketTypeDef {
  name: string;
  description?: string;
  statuses: TicketStatus[];
  fields?: TicketFieldDef[];
}

interface TicketGroup {
  id: string;
  name: string;
  description?: string;
  project: string;
  ticket_types?: TicketTypeDef[];
  labels?: Record<string, string>;
  annotations?: Record<string, string>;
  state?: ResourceState;
  deletion?: DeletionInfo | null;
  hash_code?: string;
}

interface TicketGroupBrief {
  id: string;
  name: string;
  description?: string;
  project: string;
  labels?: Record<string, string>;
}

interface ProjectBrief {
  id: string;
  name: string;
}

interface AccessCheckResult {
  can_fetch: boolean;
  can_list: boolean;
  can_notify: boolean;
  can_create: boolean;
  can_modify: boolean;
  permission_bits: number;
}

type ActionResult =
  | { success: true; intent: string }
  | { error: string; intent: string };

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function validateTicketGroupCode(code: string): string | null {
  const raw = code.replace(/^tg_/, "").trim();
  if (!raw || raw.length < 2 || raw.length > 6)
    return "Code must be 2–6 characters";
  if (!/^[A-Z]+$/.test(raw))
    return "Code must contain only capital letters (A–Z)";
  return null;
}

function nameToCodes(name: string): string {
  const letters = name
    .split(/\s+/)
    .map((w) => w[0])
    .filter(Boolean)
    .map((c) => c.toUpperCase())
    .filter((c) => /[A-Z]/.test(c))
    .join("")
    .slice(0, 6);
  if (letters.length >= 2) return letters;
  const fallback = name.replace(/[^A-Za-z]/g, "").toUpperCase().slice(0, 6);
  return fallback.length >= 2 ? fallback : "";
}

async function parseApiError(res: Response): Promise<string> {
  let text = "";
  try {
    text = await res.text();
  } catch {
    return `Request failed (HTTP ${res.status})`;
  }
  try {
    const json = JSON.parse(text) as Record<string, unknown>;
    const top =
      json.error ?? json.message ?? json.detail ?? json.msg;
    if (typeof top === "string" && top) return top;
    if (top !== null && typeof top === "object") {
      const nested =
        (top as Record<string, unknown>).message ??
        (top as Record<string, unknown>).detail;
      if (typeof nested === "string" && nested) return nested;
    }
  } catch {
    /* not JSON */
  }
  const trimmed = text.trim();
  return trimmed || `Request failed (HTTP ${res.status})`;
}

function ErrorBanner({
  message,
  onDismiss,
  className,
  "data-testid": testid,
}: {
  message: string;
  onDismiss?: () => void;
  className?: string;
  "data-testid"?: string;
}) {
  return (
    <div
      className={cn(
        "flex items-center gap-3 px-4 py-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-(--radius-component) text-red-700 dark:text-red-400",
        className
      )}
      data-testid={testid}
    >
      <AlertCircle className="w-4 h-4 shrink-0" />
      <Paragraph className="text-sm flex-1">{message}</Paragraph>
      {onDismiss && (
        <Button
          variant="ghost"
          size="sm"
          onClick={onDismiss}
          className="text-red-700 dark:text-red-400 hover:text-red-900 dark:hover:text-red-300"
        >
          ✕
        </Button>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Ticket Groups - Critical" },
    { name: "description", content: "Manage ticket groups" },
  ];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

export async function clientLoader({ params }: Route.ClientLoaderArgs) {
  const { project_id } = params;
  if (!project_id) {
    throw new Response("Project ID is required", { status: 400 });
  }

  // Fetch project
  const projectRes = await fetch(
    `/api/v1/global/projects/${project_id}`,
    { credentials: "include" }
  );
  if (!projectRes.ok) {
    if (projectRes.status === 401 || projectRes.status === 403) {
      throw new Response("Unauthorized", { status: 401 });
    }
    throw new Response("Failed to load project", { status: projectRes.status });
  }
  const project: ProjectBrief = await projectRes.json();

  // Fetch ticket groups
  const tgRes = await fetch(
    `/api/v1/projects/${project_id}/ticketgroups`,
    { credentials: "include" }
  );
  if (!tgRes.ok) {
    if (tgRes.status === 401 || tgRes.status === 403) {
      throw new Response("Unauthorized", { status: 401 });
    }
    throw new Response("Failed to load ticket groups", { status: tgRes.status });
  }
  const tgData: { items: TicketGroupBrief[] } = await tgRes.json();

  // Fetch access check
  const accessRes = await fetch(
    `/api/v1/accesscheck/global/projects/${project_id}`,
    { credentials: "include" }
  );
  const access: AccessCheckResult = accessRes.ok
    ? await accessRes.json()
    : {
        can_fetch: false,
        can_list: false,
        can_notify: false,
        can_create: false,
        can_modify: false,
        permission_bits: 0,
      };

  return { project, ticketGroups: tgData.items, access };
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

export async function clientAction({ request, params }: Route.ClientActionArgs): Promise<ActionResult> {
  const { project_id } = params;
  if (!project_id) return { error: "Project ID is required", intent: "" };

  const formData = await request.formData();
  const intent = (formData.get("intent") as string) ?? "";

  switch (intent) {
    case "create": {
      const code = (formData.get("code") as string)?.replace(/^tg_/, "").toUpperCase() || "";
      const name = formData.get("name") as string;
      const description = (formData.get("description") as string) || undefined;

      if (!code || !name)
        return { error: "Code and name are required", intent };

      const codeErr = validateTicketGroupCode(code);
      if (codeErr) return { error: codeErr, intent };

      const res = await fetch(
        `/api/v1/projects/${project_id}/ticketgroups`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          credentials: "include",
          body: JSON.stringify({ id: code, name, description, ticket_types: [] }),
        }
      );
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    case "delete": {
      const tgId = formData.get("tgId") as string;
      const res = await fetch(
        `/api/v1/projects/${project_id}/ticketgroups/${tgId}`,
        { method: "DELETE", credentials: "include" }
      );
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    case "update": {
      const tgId = formData.get("tgId") as string;
      const body = formData.get("body") as string;
      if (!tgId || !body) return { error: "TG ID and body required", intent };

      const res = await fetch(
        `/api/v1/projects/${project_id}/ticketgroups/${tgId}`,
        {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          credentials: "include",
          body,
        }
      );
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    default:
      return { error: "Unknown action", intent };
  }
}

// ---------------------------------------------------------------------------
// TicketGroupEditor Component
// ---------------------------------------------------------------------------

interface TicketGroupEditorProps {
  ticketGroup: TicketGroup;
  onLoadGroup: (id: string) => Promise<void>;
  onSave: (intent: string, body: string) => void;
  onClose: () => void;
  isSaving: boolean;
}

function TicketGroupEditor({
  ticketGroup,
  onLoadGroup,
  onSave,
  onClose,
  isSaving,
}: TicketGroupEditorProps) {
  const [activeTab, setActiveTab] = useState("details");
  const [editForm, setEditForm] = useState({
    name: ticketGroup.name,
    description: ticketGroup.description || "",
    ticket_types: ticketGroup.ticket_types || [],
  });
  const [newTypeName, setNewTypeName] = useState("");
  const [newTypeStatusName, setNewTypeStatusName] = useState("");
  const [newTypeStatusCategory, setNewTypeStatusCategory] = useState<"todo" | "in_progress" | "done">("todo");
  const [error, setError] = useState("");
  const [saved, setSaved] = useState(false);
  const savedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleDetailsSubmit = () => {
    setError("");
    if (!editForm.name.trim()) {
      setError("Name is required");
      return;
    }
    const body = JSON.stringify({
      id: ticketGroup.id,
      name: editForm.name,
      description: editForm.description || undefined,
      project: ticketGroup.project,
      hash_code: ticketGroup.hash_code,
      ticket_types: editForm.ticket_types,
    });
    onSave("update", body);
    setSaved(true);
    savedTimerRef.current = setTimeout(() => setSaved(false), 2500);
  };

  const handleAddTicketType = () => {
    setError("");
    if (!newTypeName.trim()) {
      setError("Type name is required");
      return;
    }
    if (!newTypeStatusName.trim()) {
      setError("At least one status is required");
      return;
    }
    const newType: TicketTypeDef = {
      name: newTypeName,
      statuses: [{ name: newTypeStatusName, category: newTypeStatusCategory }],
      fields: [],
    };
    setEditForm({
      ...editForm,
      ticket_types: [...editForm.ticket_types, newType],
    });
    setNewTypeName("");
    setNewTypeStatusName("");
  };

  const handleRemoveTicketType = (index: number) => {
    setEditForm({
      ...editForm,
      ticket_types: editForm.ticket_types.filter((_, i) => i !== index),
    });
  };

  const handleSaveTicketTypes = () => {
    setError("");
    if (editForm.ticket_types.length === 0) {
      setError("At least one ticket type is required");
      return;
    }
    const body = JSON.stringify({
      id: ticketGroup.id,
      name: editForm.name,
      description: editForm.description || undefined,
      project: ticketGroup.project,
      hash_code: ticketGroup.hash_code,
      ticket_types: editForm.ticket_types,
    });
    onSave("update", body);
    setSaved(true);
    savedTimerRef.current = setTimeout(() => setSaved(false), 2500);
  };

  const yamlValue = useMemo<Record<string, unknown>>(
    () => ticketGroup as unknown as Record<string, unknown>,
    [ticketGroup]
  );

  return (
    <Card className="sticky top-4 max-h-[calc(100vh-120px)] overflow-hidden flex flex-col">
      <CardHeader className="shrink-0 border-b border-gray-200 dark:border-gray-800 pb-3">
        <div className="flex items-center justify-between gap-2">
          <div>
            <CardTitle className="font-mono text-sm">{ticketGroup.id}</CardTitle>
            <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
              {ticketGroup.name}
            </p>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={onClose}
            className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
          >
            ✕
          </Button>
        </div>
      </CardHeader>

      <Tabs.Root value={activeTab} onValueChange={setActiveTab} className="flex flex-col flex-1 min-h-0">
        <Tabs.List className="shrink-0 px-2 border-b border-gray-200 dark:border-gray-800">
          <Tabs.Trigger value="details">Details</Tabs.Trigger>
          <Tabs.Trigger value="ticket-types">
            Types {editForm.ticket_types.length > 0 && (
              <span className="ml-1.5 inline-block w-5 h-5 rounded-full bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium flex items-center justify-center">
                {editForm.ticket_types.length}
              </span>
            )}
          </Tabs.Trigger>
        </Tabs.List>

        <div className="flex-1 overflow-y-auto min-h-0">
          {/* Details tab */}
          <Tabs.Content value="details" className="p-4 space-y-4">
            {error && <ErrorBanner message={error} onDismiss={() => setError("")} />}
            <div>
              <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                Name
              </label>
              <Input
                value={editForm.name}
                onChange={(e) => setEditForm({ ...editForm, name: e.target.value })}
                placeholder="e.g. Bug Tracking"
                data-testid="tg-name-input"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                Description
              </label>
              <textarea
                value={editForm.description}
                onChange={(e) => setEditForm({ ...editForm, description: e.target.value })}
                placeholder="What ticket types this group covers"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-(--radius-component) bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 text-sm font-sans focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none"
                rows={3}
                data-testid="tg-description-input"
              />
            </div>
            {ticketGroup.state && (
              <div className="space-y-2 pt-2 border-t border-gray-200 dark:border-gray-800">
                <div>
                  <p className="text-xs text-gray-600 dark:text-gray-400">Created</p>
                  <p className="text-sm font-mono text-gray-900 dark:text-gray-100">
                    {formatDate(ticketGroup.state.created_at)}
                  </p>
                </div>
                {ticketGroup.state.updated_at && (
                  <div>
                    <p className="text-xs text-gray-600 dark:text-gray-400">Updated</p>
                    <p className="text-sm font-mono text-gray-900 dark:text-gray-100">
                      {formatDate(ticketGroup.state.updated_at)}
                    </p>
                  </div>
                )}
              </div>
            )}
          </Tabs.Content>

          {/* Ticket Types tab */}
          <Tabs.Content value="ticket-types" className="p-4 space-y-4">
            {error && <ErrorBanner message={error} onDismiss={() => setError("")} />}

            {/* Existing types */}
            {editForm.ticket_types.length > 0 && (
              <div className="space-y-2">
                <p className="text-xs font-medium text-gray-700 dark:text-gray-300">Existing Types</p>
                {editForm.ticket_types.map((type, idx) => (
                  <div
                    key={idx}
                    className="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-(--radius-component) bg-gray-50 dark:bg-gray-800"
                  >
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                        {type.name}
                      </p>
                      <p className="text-xs text-gray-600 dark:text-gray-400">
                        {type.statuses.length} status{type.statuses.length !== 1 ? "es" : ""}
                      </p>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRemoveTicketType(idx)}
                      className="text-red-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 ml-2 shrink-0"
                      data-testid={`remove-type-${idx}`}
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </Button>
                  </div>
                ))}
              </div>
            )}

            {/* Add new type form */}
            <div className="space-y-3 pt-4 border-t border-gray-200 dark:border-gray-800">
              <p className="text-xs font-medium text-gray-700 dark:text-gray-300">Add New Type</p>
              <div>
                <label className="block text-xs text-gray-600 dark:text-gray-400 mb-1">Type Name</label>
                <Input
                  value={newTypeName}
                  onChange={(e) => setNewTypeName(e.target.value)}
                  placeholder="e.g. Bug"
                  data-testid="new-type-name"
                />
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-xs text-gray-600 dark:text-gray-400 mb-1">First Status</label>
                  <Input
                    value={newTypeStatusName}
                    onChange={(e) => setNewTypeStatusName(e.target.value)}
                    placeholder="e.g. Open"
                    data-testid="new-type-status-name"
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-600 dark:text-gray-400 mb-1">Category</label>
                  <select
                    value={newTypeStatusCategory}
                    onChange={(e) => setNewTypeStatusCategory(e.target.value as typeof newTypeStatusCategory)}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-(--radius-component) bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                    data-testid="new-type-status-category"
                  >
                    <option value="todo">To Do</option>
                    <option value="in_progress">In Progress</option>
                    <option value="done">Done</option>
                  </select>
                </div>
              </div>
              <Button
                variant="secondary"
                size="sm"
                onClick={handleAddTicketType}
                className="w-full"
                data-testid="add-type-button"
              >
                <Plus className="w-3.5 h-3.5 mr-1" />
                Add Type
              </Button>
            </div>
          </Tabs.Content>
        </div>
      </Tabs.Root>

      {/* Footer */}
      <div className="shrink-0 border-t border-gray-200 dark:border-gray-800 p-3 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          {saved && (
            <div className="flex items-center gap-1.5 text-green-600 dark:text-green-400">
              <CheckCircle2 className="w-4 h-4" />
              <span className="text-xs font-medium">Saved!</span>
            </div>
          )}
          <YamlEditorDrawer
            value={yamlValue}
            readOnlyFields={["state", "hash_code", "deletion"]}
            triggerLabel="View YAML"
            drawerTitle={`${ticketGroup.id} - YAML`}
            disabled={false}
            data-testid="tg-yaml-drawer"
          />
        </div>
        <div className="flex gap-2">
          <Button variant="secondary" size="sm" onClick={onClose} data-testid="tg-cancel">
            Cancel
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={activeTab === "ticket-types" ? handleSaveTicketTypes : handleDetailsSubmit}
            disabled={isSaving}
            data-testid="tg-save"
          >
            {isSaving ? (
              <>
                <Loader2 className="w-3.5 h-3.5 mr-1 animate-spin" />
                Saving…
              </>
            ) : (
              "Save Changes"
            )}
          </Button>
        </div>
      </div>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Page Component
// ---------------------------------------------------------------------------

export default function ProjectTicketsPage() {
  const { project, ticketGroups, access } = useLoaderData<typeof clientLoader>();
  const fetcher = useFetcher<ActionResult>();
  const revalidator = useRevalidator();

  const [createOpen, setCreateOpen] = useState(false);
  const [createForm, setCreateForm] = useState({ code: "", name: "", description: "" });
  const [createError, setCreateError] = useState("");
  const [codeLocked, setCodeLocked] = useState(false);

  const [selectedTgId, setSelectedTgId] = useState<string | null>(null);
  const [selectedTg, setSelectedTg] = useState<TicketGroup | null>(null);
  const [loadError, setLoadError] = useState("");
  const [tableError, setTableError] = useState("");
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  const prevFetcherStateRef = useRef<string>("idle");
  const isSubmitting = fetcher.state === "submitting";
  const isLoading = fetcher.state === "loading";

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setCreateError("");
    const codeErr = validateTicketGroupCode(createForm.code);
    if (codeErr) {
      setCreateError(codeErr);
      return;
    }
    if (!createForm.name.trim()) {
      setCreateError("Name is required");
      return;
    }
    const form = new FormData();
    form.append("intent", "create");
    form.append("code", createForm.code);
    form.append("name", createForm.name);
    form.append("description", createForm.description);
    fetcher.submit(form, { method: "POST" });
  };

  const loadTicketGroup = useCallback(
    async (tgId: string) => {
      setLoadError("");
      try {
        const res = await fetch(
          `/api/v1/projects/${project.id}/ticketgroups/${tgId}`,
          { credentials: "include" }
        );
        if (!res.ok) {
          setLoadError("Failed to load ticket group");
          return;
        }
        const tg: TicketGroup = await res.json();
        setSelectedTg(tg);
      } catch {
        setLoadError("Failed to load ticket group");
      }
    },
    [project.id]
  );

  const handleEditClick = (tgId: string) => {
    if (selectedTgId === tgId) {
      setSelectedTgId(null);
      setSelectedTg(null);
    } else {
      setSelectedTgId(tgId);
      loadTicketGroup(tgId);
    }
  };

  const handleSaveTg = (intent: string, body: string) => {
    setTableError("");
    const form = new FormData();
    form.append("intent", intent);
    form.append("tgId", selectedTgId || "");
    form.append("body", body);
    fetcher.submit(form, { method: "POST" });
  };

  const submitDelete = (tgId: string) => {
    setTableError("");
    const form = new FormData();
    form.append("intent", "delete");
    form.append("tgId", tgId);
    fetcher.submit(form, { method: "POST" });
  };

  // Handle fetcher completion
  useEffect(() => {
    const prevState = prevFetcherStateRef.current;
    prevFetcherStateRef.current = fetcher.state;
    if (fetcher.state !== "idle" || prevState === "idle" || !fetcher.data) return;

    const result = fetcher.data as ActionResult;

    if (result.intent === "create" && "success" in result && result.success) {
      setCreateOpen(false);
      setCreateForm({ code: "", name: "", description: "" });
      setCreateError("");
      setCodeLocked(false);
      revalidator.revalidate();
    }
    if (result.intent === "delete" && "success" in result && result.success) {
      setConfirmDeleteId(null);
      setTableError("");
      setSelectedTgId(null);
      setSelectedTg(null);
      revalidator.revalidate();
    }
    if (result.intent === "update" && "success" in result && result.success) {
      if (selectedTgId) {
        loadTicketGroup(selectedTgId);
      }
      revalidator.revalidate();
    }
    if ("error" in result && result.error) {
      if (result.intent === "create") {
        setCreateError(result.error);
      } else {
        setTableError(result.error);
      }
    }
  }, [fetcher.state, fetcher.data, revalidator, selectedTgId, loadTicketGroup]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-7xl mx-auto">
        {/* Breadcrumb & Header */}
        <div className="mb-8">
          <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 mb-3">
            <Link
              to="/projects"
              className="hover:text-gray-900 dark:hover:text-gray-100 transition-colors"
              data-testid="breadcrumb-projects"
            >
              Projects
            </Link>
            <span>/</span>
            <Link
              to={`/p/${project.id}`}
              className="hover:text-gray-900 dark:hover:text-gray-100 transition-colors"
              data-testid="breadcrumb-project"
            >
              {project.name}
            </Link>
            <span>/</span>
            <span className="text-gray-900 dark:text-gray-100">Ticket Groups</span>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <H1 data-testid="page-heading">Ticket Groups</H1>
              <Paragraph variant="muted" className="mt-0.5">
                Manage ticket types and workflows for {project.name}
              </Paragraph>
            </div>

            <MorphModal
              trigger={
                <Button
                  variant="primary"
                  disabled={!access.can_create}
                  data-testid="create-tg-button"
                >
                  <Plus className="w-4 h-4 mr-1.5" />
                  New Ticket Group
                </Button>
              }
              modalWidth={480}
              modalHeight={400}
              isOpen={createOpen}
              onOpenChange={(open) => {
                setCreateOpen(open);
                if (!open) {
                  setCreateForm({ code: "", name: "", description: "" });
                  setCreateError("");
                  setCodeLocked(false);
                }
              }}
            >
              {(close) => (
                <div className="flex flex-col h-full">
                  <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
                    <h3 className="text-xs font-mono uppercase tracking-wider text-gray-900 dark:text-gray-100">
                      Create Ticket Group
                    </h3>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={close}
                      className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
                    >
                      ✕
                    </Button>
                  </div>
                  <form
                    onSubmit={handleCreateSubmit}
                    className="flex-1 flex flex-col"
                  >
                    <div className="flex-1 space-y-4 p-4 overflow-y-auto">
                      {createError && (
                        <ErrorBanner
                          message={createError}
                          data-testid="create-error"
                        />
                      )}
                      <div>
                        <label
                          htmlFor="tg-code"
                          className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5"
                        >
                          Ticket Group Code
                        </label>
                        <Input
                          id="tg-code"
                          monospace
                          value={createForm.code}
                          onChange={(e) => {
                            const code = e.target.value.replace(/^tg_/, "").toUpperCase();
                            setCreateForm((f) => ({
                              ...f,
                              code,
                              name: codeLocked ? f.name : code,
                            }));
                          }}
                          onFocus={() => setCodeLocked(true)}
                          placeholder="BUG"
                          maxLength={6}
                          data-testid="tg-code-input"
                        />
                        <p className="mt-1 text-xs text-gray-600 dark:text-gray-400">
                          2–6 capital letters (A–Z)
                        </p>
                      </div>
                      <div>
                        <label
                          htmlFor="tg-name"
                          className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5"
                        >
                          Name
                        </label>
                        <Input
                          id="tg-name"
                          value={createForm.name}
                          onChange={(e) => {
                            const name = e.target.value;
                            setCreateForm((f) => ({
                              ...f,
                              name,
                              code: codeLocked ? f.code : nameToCodes(name),
                            }));
                          }}
                          placeholder="e.g. Bug Tracking"
                          data-testid="tg-name-input"
                        />
                      </div>
                      <div>
                        <label
                          htmlFor="tg-desc"
                          className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5"
                        >
                          Description (Optional)
                        </label>
                        <Input
                          id="tg-desc"
                          value={createForm.description}
                          onChange={(e) =>
                            setCreateForm({
                              ...createForm,
                              description: e.target.value,
                            })
                          }
                          placeholder="What this group covers"
                          data-testid="tg-description-input"
                        />
                      </div>
                    </div>
                    <div className="flex gap-3 p-4 border-t border-gray-200 dark:border-gray-800 shrink-0">
                      <Button
                        type="submit"
                        variant="primary"
                        size="sm"
                        disabled={isSubmitting}
                        data-testid="create-tg-submit"
                      >
                        {isSubmitting ? (
                          <>
                            <Loader2 className="w-4 h-4 mr-1.5 animate-spin" />
                            Creating…
                          </>
                        ) : (
                          "Create"
                        )}
                      </Button>
                    </div>
                  </form>
                </div>
              )}
            </MorphModal>
          </div>
        </div>

        {/* Table-level error */}
        {tableError && (
          <ErrorBanner
            message={tableError}
            onDismiss={() => setTableError("")}
            className="mb-6"
            data-testid="table-error"
          />
        )}

        {/* Load error */}
        {loadError && (
          <ErrorBanner
            message={loadError}
            onDismiss={() => setLoadError("")}
            className="mb-6"
            data-testid="load-error"
          />
        )}

        {/* Main content */}
        {ticketGroups.length === 0 ? (
          <Card className="w-full py-20" data-testid="empty-state">
            <div className="flex flex-col items-center gap-4 px-8">
              <div className="flex items-center justify-center w-12 h-12 rounded-(--radius-component-lg) bg-gray-100 dark:bg-gray-800">
                <Plus className="w-6 h-6 text-gray-400" />
              </div>
              <H2>No ticket groups yet</H2>
              <Paragraph className="text-center max-w-prose">
                Create your first ticket group to define issue types and workflows.
              </Paragraph>
            </div>
          </Card>
        ) : (
          <div className="grid gap-6 items-start">
            <Card data-testid="tg-list-card">
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-gray-200 dark:border-gray-800">
                      <th className="px-4 py-3 text-left font-medium text-gray-700 dark:text-gray-300 font-semibold">
                        ID
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-700 dark:text-gray-300 font-semibold">
                        Name
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-700 dark:text-gray-300 font-semibold">
                        Description
                      </th>
                      <th className="px-4 py-3 text-right font-medium text-gray-700 dark:text-gray-300 font-semibold">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {ticketGroups.map((tg) => (
                      <tr
                        key={tg.id}
                        className="border-b border-gray-100 dark:border-gray-900 hover:bg-gray-50 dark:hover:bg-gray-900/50 transition-colors"
                        data-testid={`tg-row-${tg.id}`}
                      >
                        <td className="px-4 py-3">
                          <code className="text-xs font-mono text-gray-600 dark:text-gray-400 bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded-(--radius-component)">
                            {tg.id}
                          </code>
                        </td>
                        <td className="px-4 py-3 font-medium text-gray-900 dark:text-gray-100">
                          {tg.name}
                        </td>
                        <td className="px-4 py-3 text-gray-600 dark:text-gray-400 max-w-xs truncate">
                          {tg.description || "—"}
                        </td>
                        <td className="px-4 py-3 text-right">
                          <div className="flex justify-end gap-1">
                            {confirmDeleteId === tg.id ? (
                              <>
                                <Button
                                  variant="destructive"
                                  size="sm"
                                  onClick={() => submitDelete(tg.id)}
                                  disabled={isSubmitting}
                                  data-testid={`confirm-delete-${tg.id}`}
                                >
                                  Yes
                                </Button>
                                <Button
                                  variant="secondary"
                                  size="sm"
                                  onClick={() => setConfirmDeleteId(null)}
                                  data-testid={`cancel-delete-${tg.id}`}
                                >
                                  No
                                </Button>
                              </>
                            ) : (
                              <>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  disabled={!access.can_modify}
                                  title={
                                    !access.can_modify
                                      ? "You don't have write access"
                                      : undefined
                                  }
                                  onClick={() =>
                                    setConfirmDeleteId(tg.id)
                                  }
                                  data-testid={`delete-tg-${tg.id}`}
                                  className="text-red-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                                >
                                  <Trash2 className="w-3.5 h-3.5" />
                                </Button>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  disabled={!access.can_modify && selectedTgId !== tg.id}
                                  onClick={() => handleEditClick(tg.id)}
                                  data-testid={`edit-tg-${tg.id}`}
                                  className="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
                                >
                                  {selectedTgId === tg.id ? (
                                    <>Close</>
                                  ) : (
                                    <>
                                      <Pencil className="w-3.5 h-3.5" />
                                    </>
                                  )}
                                </Button>
                              </>
                            )}
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </Card>

            {/* Editor panel */}
            {selectedTgId && selectedTg && (
              <TicketGroupEditor
                ticketGroup={selectedTg}
                onLoadGroup={loadTicketGroup}
                onSave={handleSaveTg}
                onClose={() => {
                  setSelectedTgId(null);
                  setSelectedTg(null);
                }}
                isSaving={isSubmitting || isLoading}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Error Boundary
// ---------------------------------------------------------------------------

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  return (
    <div className="min-h-screen bg-white dark:bg-gray-900 flex items-center justify-center px-4">
      <Card className="max-w-md w-full">
        <CardHeader>
          <div className="flex items-center gap-3">
            <AlertCircle className="w-5 h-5 text-red-500" />
            <CardTitle>Error loading ticket groups</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <Paragraph className="text-gray-600 dark:text-gray-400">
            {error instanceof Error
              ? error.message
              : "Something went wrong."}
          </Paragraph>
        </CardContent>
      </Card>
    </div>
  );
}
