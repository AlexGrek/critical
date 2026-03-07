import type { Route } from "./+types/projects";
import { useLoaderData, useRevalidator } from "react-router";
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  H1,
  Paragraph,
  PermissionBadge,
  AclEditor,
  Tabs,
  YamlEditor,
} from "~/components";
import type { AccessControlStore } from "~/components";
import { AlertCircle, Lock } from "lucide-react";
import { formatDate } from "~/lib/utils";
import { useState, useMemo, useCallback } from "react";

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

interface Project {
  id: string;
  name: string;
  labels?: Record<string, string>;
  annotations?: Record<string, string>;
  acl?: {
    list?: Array<{
      permissions: number;
      principals: string[];
      scope?: "ROOT" | "WRITE" | "READ";
    }>;
    last_mod_date?: string;
  };
  state?: ResourceState;
  deletion?: DeletionInfo | null;
  hash_code?: string;
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Project - Critical" },
    { name: "description", content: "Project home page" },
  ];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

export async function loader({ request, params }: Route.LoaderArgs) {
  const { project_id } = params;

  if (!project_id) {
    throw new Response("Project ID is required", { status: 400 });
  }

  const response = await fetch(
    `http://localhost:3742/api/v1/global/projects/${project_id}`,
    {
      headers: {
        Cookie: request.headers.get("Cookie") || "",
      },
    }
  );

  if (!response.ok) {
    if (response.status === 404) {
      throw new Response("Project not found", { status: 404 });
    }
    throw new Response("Failed to load project", { status: response.status });
  }

  const project: Project = await response.json();
  return { project };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function ProjectPage() {
  const { project } = useLoaderData<typeof loader>();
  const [currentAcl, setCurrentAcl] = useState<AccessControlStore>(
    (project.acl as AccessControlStore) || { list: [], last_mod_date: new Date().toISOString() }
  );
  const [isSavingAcl, setIsSavingAcl] = useState(false);
  const revalidator = useRevalidator();

  const handleAclSave = async (newAcl: AccessControlStore) => {
    setIsSavingAcl(true);
    try {
      const response = await fetch(
        `http://localhost:3742/api/v1/global/projects/${project.id}`,
        {
          method: "PUT",
          credentials: "include",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ ...project, acl: newAcl }),
        }
      );

      if (response.ok) {
        setCurrentAcl(newAcl);
      }
    } finally {
      setIsSavingAcl(false);
    }
  };

  /** Stable object for the YAML tab. */
  const yamlValue = useMemo<Record<string, unknown>>(
    () => project as unknown as Record<string, unknown>,
    [project]
  );

  const handleYamlSave = useCallback(async (parsed: Record<string, unknown>) => {
    const res = await fetch(`/api/v1/global/projects/${project.id}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ ...project, ...parsed }),
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error((body as { message?: string; error?: string }).message || (body as { message?: string; error?: string }).error || `HTTP ${res.status}`);
    }
    revalidator.revalidate();
  }, [project, revalidator]);

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      {/* Header */}
      <div className="border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-800/50">
        <div className="max-w-6xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between gap-4">
            <div className="flex-1 min-w-0">
              <h1
                className="text-xl font-bold text-gray-900 dark:text-gray-100 truncate"
                data-testid="project-name"
              >
                {project.name}
              </h1>
              <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                Project ID: <code className="font-mono">{project.id}</code>
              </p>
            </div>

            {/* Status badge */}
            <div className="flex items-center gap-2 shrink-0">
              {project.deletion ? (
                <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-(--radius-component) bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 text-xs font-medium">
                  <AlertCircle className="w-3.5 h-3.5" />
                  Deleted
                </span>
              ) : (
                <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-(--radius-component) bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 text-xs font-medium">
                  <span className="w-1.5 h-1.5 rounded-full bg-green-500" />
                  Active
                </span>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Tabbed content */}
      <div className="max-w-6xl mx-auto px-4 py-6">
        <Tabs.Root defaultValue="overview">
          <Tabs.List>
            <Tabs.Trigger value="overview" data-testid="project-tab-overview">Overview</Tabs.Trigger>
            <Tabs.Trigger value="access" data-testid="project-tab-access">Access</Tabs.Trigger>
            <Tabs.Trigger value="yaml" data-testid="project-tab-yaml">YAML</Tabs.Trigger>
          </Tabs.List>

          {/* ── Overview tab ── */}
          <Tabs.Content value="overview" className="pt-6">
            {/* Quick info cards */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
              <Card data-testid="created-card">
                <CardHeader>
                  <CardTitle className="text-sm font-semibold">Created</CardTitle>
                </CardHeader>
                <CardContent>
                  <Paragraph className="text-sm">
                    {project.state?.created_at
                      ? formatDate(project.state.created_at)
                      : "Unknown"}
                  </Paragraph>
                  {project.state?.created_by && (
                    <Paragraph className="text-xs text-gray-600 dark:text-gray-400 mt-2">
                      by <code className="font-mono">{project.state.created_by}</code>
                    </Paragraph>
                  )}
                </CardContent>
              </Card>

              <Card data-testid="updated-card">
                <CardHeader>
                  <CardTitle className="text-sm font-semibold">Last Updated</CardTitle>
                </CardHeader>
                <CardContent>
                  <Paragraph className="text-sm">
                    {project.state?.updated_at
                      ? formatDate(project.state.updated_at)
                      : "Unknown"}
                  </Paragraph>
                  {project.state?.updated_by && (
                    <Paragraph className="text-xs text-gray-600 dark:text-gray-400 mt-2">
                      by <code className="font-mono">{project.state.updated_by}</code>
                    </Paragraph>
                  )}
                </CardContent>
              </Card>

              <Card data-testid="hash-card">
                <CardHeader>
                  <CardTitle className="text-sm font-semibold">Hash</CardTitle>
                </CardHeader>
                <CardContent>
                  <code className="text-xs font-mono text-gray-500 dark:text-gray-400 break-all">
                    {project.hash_code || "—"}
                  </code>
                </CardContent>
              </Card>
            </div>

            {/* Labels & Annotations */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              {project.labels && Object.keys(project.labels).length > 0 && (
                <Card data-testid="labels-card">
                  <CardHeader>
                    <CardTitle>Labels</CardTitle>
                    <CardDescription>User-managed metadata</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-2">
                      {Object.entries(project.labels).map(([key, value]) => (
                        <div
                          key={key}
                          className="flex items-center justify-between p-2 bg-gray-100 dark:bg-gray-800 rounded-(--radius-component)"
                        >
                          <code className="text-xs font-mono text-gray-600 dark:text-gray-400">
                            {key}
                          </code>
                          <code className="text-xs font-mono text-gray-900 dark:text-gray-100">
                            {value}
                          </code>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}

              {project.annotations && Object.keys(project.annotations).length > 0 && (
                <Card data-testid="annotations-card">
                  <CardHeader>
                    <CardTitle>Annotations</CardTitle>
                    <CardDescription>Free-form metadata</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-2">
                      {Object.entries(project.annotations).map(([key, value]) => (
                        <div
                          key={key}
                          className="flex items-center justify-between p-2 bg-gray-100 dark:bg-gray-800 rounded-(--radius-component)"
                        >
                          <code className="text-xs font-mono text-gray-600 dark:text-gray-400">
                            {key}
                          </code>
                          <code className="text-xs font-mono text-gray-900 dark:text-gray-100 line-clamp-1">
                            {value}
                          </code>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>

            {(!project.labels || Object.keys(project.labels).length === 0) &&
              (!project.annotations || Object.keys(project.annotations).length === 0) && (
                <Card className="w-full py-12" data-testid="empty-state">
                  <div className="flex flex-col items-center gap-2">
                    <Paragraph className="text-gray-500 dark:text-gray-400">
                      No labels or annotations
                    </Paragraph>
                  </div>
                </Card>
              )}
          </Tabs.Content>

          {/* ── Access tab ── */}
          <Tabs.Content value="access" className="pt-6">
            <Card className="w-full">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="flex items-center gap-2">
                      <Lock className="w-4 h-4" />
                      Access Control List
                    </CardTitle>
                    <CardDescription className="mt-1">
                      Manage who can read or modify this project
                    </CardDescription>
                  </div>
                  <AclEditor
                    acl={currentAcl}
                    onSave={handleAclSave}
                    trigger={
                      <Button
                        variant="outline"
                        size="sm"
                        data-testid="edit-acl-button"
                      >
                        Edit ACL
                      </Button>
                    }
                  />
                </div>
              </CardHeader>
              <CardContent>
                {currentAcl.list.length === 0 ? (
                  <Paragraph variant="muted" className="text-sm">
                    No ACL entries — all authenticated users have access.
                  </Paragraph>
                ) : (
                  <div className="space-y-2">
                    {currentAcl.list.map((entry, idx) => (
                      <div
                        key={idx}
                        className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-(--radius-component) border border-gray-100 dark:border-gray-800"
                        data-testid={`acl-display-entry-${idx}`}
                      >
                        <div className="flex flex-wrap gap-1.5">
                          {entry.principals.map((p) => (
                            <code
                              key={p}
                              className="text-xs font-mono px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 rounded-(--radius-component) text-gray-700 dark:text-gray-300"
                            >
                              {p}
                            </code>
                          ))}
                        </div>
                        <PermissionBadge permissions={entry.permissions} />
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </Tabs.Content>

          {/* ── YAML tab ── */}
          <Tabs.Content value="yaml" className="pt-6 flex flex-col min-h-100">
            <YamlEditor
              value={yamlValue}
              onSave={handleYamlSave}
              readOnlyFields={["state", "hash_code", "deletion"]}
              data-testid="project-yaml-editor"
            />
          </Tabs.Content>
        </Tabs.Root>
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
            <CardTitle>Error loading project</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <Paragraph className="text-gray-600 dark:text-gray-400">
            {error instanceof Error
              ? error.message
              : "Something went wrong while loading the project."}
          </Paragraph>
        </CardContent>
      </Card>
    </div>
  );
}
