import type { Route } from "./+types/projects-list";
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
  Paragraph,
  PermissionBadge,
} from "~/components";
import { useState, useEffect, useCallback, useRef } from "react";
import {
  Plus,
  Trash2,
  Pencil,
  Loader2,
  AlertCircle,
  Lock,
  Users,
  ArrowRight,
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

interface ProjectBrief {
  id: string;
  name: string;
  labels?: Record<string, string>;
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

function validateProjectId(id: string): string | null {
  if (!id || id.length < 2 || id.length > 63)
    return "ID must be 2–63 characters";
  if (!/^[a-z0-9_]+$/.test(id))
    return "ID must contain only lowercase letters, numbers, and underscores";
  return null;
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

interface ErrorBannerProps {
  message: string;
  onDismiss?: () => void;
  className?: string;
  "data-testid"?: string;
}

function ErrorBanner({
  message,
  onDismiss,
  className,
  "data-testid": testid,
}: ErrorBannerProps) {
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
    { title: "Projects - Critical" },
    { name: "description", content: "Manage projects" },
  ];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

export async function loader({ request }: Route.LoaderArgs) {
  const cookie = request.headers.get("Cookie") || "";

  const res = await fetch("http://localhost:3742/api/v1/global/projects", {
    headers: { Cookie: cookie },
  });

  if (!res.ok) {
    if (res.status === 401 || res.status === 403) {
      throw new Response("Unauthorized", { status: 401 });
    }
    throw new Response("Failed to load projects", { status: res.status });
  }

  const data: { items: ProjectBrief[] } = await res.json();
  const projects = data.items;

  // Fetch access-check results in parallel for all projects
  const accessEntries = await Promise.all(
    projects.map(async (p) => {
      const r = await fetch(
        `http://localhost:3742/api/v1/accesscheck/global/projects/${p.id}`,
        { headers: { Cookie: cookie } }
      );
      if (!r.ok) {
        return [
          p.id,
          {
            can_fetch: false,
            can_list: false,
            can_notify: false,
            can_create: false,
            can_modify: false,
            permission_bits: 0,
          },
        ] as const;
      }
      const check: AccessCheckResult = await r.json();
      return [p.id, check] as const;
    })
  );

  const accessMap: Record<string, AccessCheckResult> = Object.fromEntries(
    accessEntries
  );

  return { projects, accessMap };
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

export async function action({ request }: Route.ActionArgs): Promise<ActionResult> {
  const formData = await request.formData();
  const intent = (formData.get("intent") as string) ?? "";
  const cookie = request.headers.get("Cookie") || "";

  switch (intent) {
    case "create": {
      const name = formData.get("name") as string;
      const id = formData.get("id") as string;
      if (!name || !id) return { error: "Name and ID are required", intent };

      const res = await fetch("http://localhost:3742/api/v1/global/projects", {
        method: "POST",
        headers: { "Content-Type": "application/json", Cookie: cookie },
        body: JSON.stringify({ name, id }),
      });
      if (!res.ok) return { error: await parseApiError(res), intent };
      return { success: true, intent };
    }

    case "delete": {
      const projectId = formData.get("projectId") as string;
      const res = await fetch(
        `http://localhost:3742/api/v1/global/projects/${projectId}`,
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
// Page component
// ---------------------------------------------------------------------------

export default function ProjectsListPage() {
  const { projects, accessMap } = useLoaderData<typeof loader>();
  const fetcher = useFetcher<ActionResult>();
  const revalidator = useRevalidator();

  // Create modal
  const [createOpen, setCreateOpen] = useState(false);
  const [createForm, setCreateForm] = useState({ name: "", id: "" });
  const [createError, setCreateError] = useState("");

  // Delete confirmation
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);
  const [tableError, setTableError] = useState("");

  const isSubmitting = fetcher.state === "submitting";

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setCreateError("");
    const idErr = validateProjectId(createForm.id);
    if (idErr) {
      setCreateError(idErr);
      return;
    }
    const form = new FormData();
    form.append("intent", "create");
    form.append("name", createForm.name);
    form.append("id", createForm.id);
    fetcher.submit(form, { method: "POST" });
  };

  const submitDelete = (projectId: string) => {
    setTableError("");
    const form = new FormData();
    form.append("intent", "delete");
    form.append("projectId", projectId);
    fetcher.submit(form, { method: "POST" });
  };

  // Handle fetcher completion
  useEffect(() => {
    if (fetcher.state === "idle" && fetcher.data) {
      const result = fetcher.data as ActionResult;
      if (result.intent === "create" && "success" in result && result.success) {
        setCreateOpen(false);
        setCreateForm({ name: "", id: "" });
        setCreateError("");
        revalidator.revalidate();
      }
      if (result.intent === "delete" && "success" in result && result.success) {
        setConfirmDeleteId(null);
        setTableError("");
        revalidator.revalidate();
      }
      if ("error" in result) {
        if (result.intent === "create") {
          setCreateError(result.error);
        } else if (result.intent === "delete") {
          setTableError(result.error);
        }
      }
    }
  }, [fetcher.state, fetcher.data, revalidator]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-7xl mx-auto">
        {/* Page header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <H1 data-testid="projects-page-heading">{"{!} "}Projects</H1>
            <Paragraph variant="muted" className="mt-0.5">
              Manage your projects and collaborate with teams
            </Paragraph>
          </div>

          <MorphModal
            trigger={
              <Button variant="primary" data-testid="create-project-button">
                <Plus className="w-4 h-4 mr-1.5" />
                New Project
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
                <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
                  <h3 className="text-xs font-mono uppercase tracking-wider text-gray-900 dark:text-gray-100" data-testid="create-project-modal-title">
                    Create New Project
                  </h3>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={close}
                    data-testid="cancel-create-project"
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
                        data-testid="create-project-error"
                        message={createError}
                      />
                    )}
                    <div>
                      <label
                        htmlFor="project-name"
                        className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5"
                      >
                        Project Name
                      </label>
                      <Input
                        id="project-name"
                        data-testid="project-name-input"
                        value={createForm.name}
                        onChange={(e) =>
                          setCreateForm({
                            ...createForm,
                            name: e.target.value,
                          })
                        }
                        placeholder="My Awesome Project"
                        required
                      />
                    </div>
                    <div>
                      <label
                        htmlFor="project-id"
                        className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5"
                      >
                        Project ID
                      </label>
                      <Input
                        id="project-id"
                        monospace
                        data-testid="project-id-input"
                        value={createForm.id}
                        onChange={(e) =>
                          setCreateForm({
                            ...createForm,
                            id: e.target.value.toLowerCase(),
                          })
                        }
                        placeholder="my_awesome_project"
                        required
                      />
                      <p className="mt-1 text-xs text-gray-600 dark:text-gray-400" data-testid="project-id-hint">
                        2–63 chars, lowercase + underscores
                      </p>
                    </div>
                  </div>
                  <div className="flex gap-3 p-4 border-t border-gray-200 dark:border-gray-800 shrink-0">
                    <Button
                      type="submit"
                      variant="primary"
                      size="sm"
                      data-testid="submit-create-project"
                      disabled={isSubmitting}
                    >
                      {isSubmitting ? (
                        <>
                          <Loader2 className="w-4 h-4 mr-1.5 animate-spin" />
                          Creating…
                        </>
                      ) : (
                        "Create Project"
                      )}
                    </Button>
                  </div>
                </form>
              </div>
            )}
          </MorphModal>
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

        {/* Projects grid */}
        {projects.length === 0 ? (
          <Card className="w-full py-20" data-testid="projects-empty-state">
            <div className="flex flex-col items-center gap-4 px-8">
              <div className="flex items-center justify-center w-12 h-12 rounded-(--radius-component-lg) bg-gray-100 dark:bg-gray-800">
                <Lock className="w-6 h-6 text-gray-400" />
              </div>
              <H2>No projects yet</H2>
              <Paragraph className="text-gray-600 dark:text-gray-400 text-center max-w-prose">
                Create your first project to get started with collaboration and
                resource management.
              </Paragraph>
              <Button variant="primary" size="lg">
                <Plus className="w-4 h-4 mr-2" />
                Create First Project
              </Button>
            </div>
          </Card>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {projects.map((project) => (
              <Card
                key={project.id}
                className="hover:shadow-md transition-shadow dark:hover:shadow-lg dark:hover:shadow-gray-800 group"
                data-testid={`project-card-${project.id}`}
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <Link
                        to={`/p/${project.id}`}
                        className="block group/link"
                        data-testid={`project-link-${project.id}`}
                      >
                        <CardTitle className="truncate group-hover/link:text-blue-600 dark:group-hover/link:text-blue-400 transition-colors">
                          {project.name}
                        </CardTitle>
                      </Link>
                      <code className="text-xs font-mono text-gray-500 dark:text-gray-400 truncate block">
                        {project.id}
                      </code>
                    </div>
                    <div className="flex gap-1 shrink-0">
                      {confirmDeleteId === project.id ? (
                        <>
                          <Button
                            variant="destructive"
                            size="sm"
                            onClick={() => submitDelete(project.id)}
                            disabled={isSubmitting}
                            data-testid={`confirm-delete-project-${project.id}`}
                          >
                            Yes
                          </Button>
                          <Button
                            variant="secondary"
                            size="sm"
                            onClick={() => setConfirmDeleteId(null)}
                            data-testid={`cancel-delete-project-${project.id}`}
                          >
                            No
                          </Button>
                        </>
                      ) : (
                        <>
                          <Button
                            variant="ghost"
                            size="sm"
                            disabled={!accessMap[project.id]?.can_modify}
                            title={
                              !accessMap[project.id]?.can_modify
                                ? "You don't have write access to this project"
                                : undefined
                            }
                            onClick={() =>
                              setConfirmDeleteId(project.id)
                            }
                            data-testid={`delete-project-${project.id}`}
                            className="text-red-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </Button>
                          <Link
                            to={`/p/${project.id}`}
                            className="inline-flex"
                          >
                            <Button
                              variant="ghost"
                              size="sm"
                              data-testid={`open-project-${project.id}`}
                              className="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
                            >
                              <ArrowRight className="w-3.5 h-3.5" />
                            </Button>
                          </Link>
                        </>
                      )}
                    </div>
                  </div>
                </CardHeader>

                {/* Labels section */}
                {project.labels && Object.keys(project.labels).length > 0 && (
                  <CardContent className="pt-0 pb-3">
                    <div className="flex flex-wrap gap-1">
                      {Object.entries(project.labels)
                        .slice(0, 3)
                        .map(([key, value]) => (
                          <span
                            key={key}
                            className="inline-flex items-center px-2.5 py-0.5 rounded-(--radius-component) bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium"
                          >
                            <span className="font-semibold mr-1">{key}:</span>
                            <span>{value}</span>
                          </span>
                        ))}
                      {Object.keys(project.labels).length > 3 && (
                        <span className="inline-flex items-center px-2.5 py-0.5 text-xs text-gray-500 dark:text-gray-400">
                          +{Object.keys(project.labels).length - 3} more
                        </span>
                      )}
                    </div>
                  </CardContent>
                )}

                {/* Footer with action badge */}
                <CardDescription className="px-4 py-3 border-t border-gray-100 dark:border-gray-800 flex items-center justify-between text-xs">
                  <span className="text-gray-600 dark:text-gray-400">
                    {accessMap[project.id]?.can_modify ? "You can edit" : "Read-only"}
                  </span>
                  {accessMap[project.id]?.permission_bits > 0 && (
                    <PermissionBadge
                      permissions={accessMap[project.id].permission_bits}
                    />
                  )}
                </CardDescription>
              </Card>
            ))}
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
            <CardTitle>Error loading projects</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <Paragraph className="text-gray-600 dark:text-gray-400">
            {error instanceof Error
              ? error.message
              : "Something went wrong while loading projects."}
          </Paragraph>
        </CardContent>
      </Card>
    </div>
  );
}
