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
  H2,
  Paragraph,
  PermissionBadge,
  AclEditor,
  Tabs,
  YamlEditor,
  Modal,
  PrincipalChip,
} from "~/components";
import type { AccessControlStore } from "~/components";
import { AlertCircle, Lock, Settings, Plus } from "lucide-react";
import { formatDate } from "~/lib/utils";
import { useState, useMemo, useCallback } from "react";
import { resolvePrincipals } from "~/lib/principals";
import type { PrincipalMap } from "~/lib/principals";

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
  enabled_services?: string[];
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

  // Collect all principal IDs present in the document
  const principalIds = [
    project.state?.created_by,
    project.state?.updated_by,
    ...(project.acl?.list?.flatMap((e) => e.principals) ?? []),
  ].filter((id): id is string => !!id);

  const principals = await resolvePrincipals(
    principalIds,
    request.headers.get("Cookie") || ""
  );

  return { project, principals };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function ProjectPage() {
  const { project, principals } = useLoaderData<typeof loader>();
  const [currentAcl, setCurrentAcl] = useState<AccessControlStore>(
    (project.acl as AccessControlStore) || { list: [], last_mod_date: new Date().toISOString() }
  );
  const [isSavingAcl, setIsSavingAcl] = useState(false);
  const [isAccessModalOpen, setIsAccessModalOpen] = useState(false);
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);
  const [isAddFeatureModalOpen, setIsAddFeatureModalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<string>(
    (project.enabled_services?.[0]) || "overview"
  );
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

  const enabledServices = project.enabled_services || [];

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

            {/* Status badge + action buttons */}
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

              <Button
                variant="ghost"
                size="icon"
                onClick={() => setIsAccessModalOpen(true)}
                data-testid="project-access-button"
                title="Manage access"
              >
                <Lock className="w-4 h-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setIsSettingsModalOpen(true)}
                data-testid="project-settings-button"
                title="Settings"
              >
                <Settings className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Tabbed content */}
      <div className="max-w-6xl mx-auto px-4 py-6">
        <Tabs.Root value={activeTab} onValueChange={setActiveTab}>
          <div className="flex items-center gap-2">
            <Tabs.List className="flex-1">
              <Tabs.Trigger value="overview" data-testid="project-tab-overview">Overview</Tabs.Trigger>
              {enabledServices.map((service) => (
                <Tabs.Trigger
                  key={service}
                  value={service}
                  data-testid={`project-tab-${service}`}
                >
                  {service}
                </Tabs.Trigger>
              ))}
              <Tabs.Trigger value="yaml" data-testid="project-tab-yaml">YAML</Tabs.Trigger>
            </Tabs.List>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setIsAddFeatureModalOpen(true)}
              data-testid="project-add-feature-button"
              title="Add feature"
            >
              <Plus className="w-4 h-4" />
            </Button>
          </div>

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
                    <div className="mt-2">
                      <PrincipalChip
                        id={project.state.created_by}
                        info={principals[project.state.created_by]}
                        data-testid="project-created-by"
                      />
                    </div>
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
                    <div className="mt-2">
                      <PrincipalChip
                        id={project.state.updated_by}
                        info={principals[project.state.updated_by]}
                        data-testid="project-updated-by"
                      />
                    </div>
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

          {/* ── Enabled Services tabs ── */}
          {enabledServices.map((service) => (
            <Tabs.Content key={service} value={service} className="pt-6">
              <Card className="w-full py-12">
                <div className="flex flex-col items-center gap-4 px-8">
                  <H1>{service}</H1>
                  <Paragraph className="text-center max-w-prose text-gray-600 dark:text-gray-400">
                    Content for {service} feature will appear here
                  </Paragraph>
                </div>
              </Card>
            </Tabs.Content>
          ))}

          {/* ── YAML tab ── */}
          <Tabs.Content value="yaml" className="pt-6 flex flex-col min-h-100">
            <YamlEditor
              value={yamlValue}
              onSave={handleYamlSave}
              readOnlyFields={["state", "hash_code", "deletion"]}
              allowedTopLevelKeys={["id", "name", "description", "repositories", "enabled_services", "labels", "annotations", "acl", "state", "hash_code", "deletion"]}
              data-testid="project-yaml-editor"
            />
          </Tabs.Content>
        </Tabs.Root>
      </div>

      {/* Access Modal */}
      <Modal.Root open={isAccessModalOpen} onOpenChange={setIsAccessModalOpen}>
        <Modal.Content className="max-w-2xl">
          <Modal.Header>
            <Modal.Title className="flex items-center gap-2">
              <Lock className="w-5 h-5" />
              Access Control
            </Modal.Title>
            <Modal.Description>
              Manage who can read or modify this project
            </Modal.Description>
          </Modal.Header>

          <div className="px-6 py-4">
            <AclEditor
              acl={currentAcl}
              onSave={handleAclSave}
              trigger={
                <Button
                  variant="primary"
                  size="sm"
                  data-testid="edit-acl-modal-button"
                >
                  Edit ACL
                </Button>
              }
            />

            <div className="mt-6 space-y-3">
              {currentAcl.list.length === 0 ? (
                <Paragraph variant="muted" className="text-sm">
                  No ACL entries — all authenticated users have access.
                </Paragraph>
              ) : (
                currentAcl.list.map((entry, idx) => (
                  <div
                    key={idx}
                    className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-(--radius-component) border border-gray-100 dark:border-gray-800"
                    data-testid={`acl-display-entry-${idx}`}
                  >
                    <div className="flex flex-wrap gap-2">
                      {entry.principals.map((p) => (
                        <PrincipalChip
                          key={p}
                          id={p}
                          info={principals[p]}
                          data-testid={`acl-principal-${p}`}
                        />
                      ))}
                    </div>
                    <PermissionBadge permissions={entry.permissions} />
                  </div>
                ))
              )}
            </div>
          </div>

          <Modal.Footer>
            <Modal.Close asChild>
              <Button variant="secondary">Close</Button>
            </Modal.Close>
          </Modal.Footer>
        </Modal.Content>
      </Modal.Root>

      {/* Settings Modal */}
      <Modal.Root open={isSettingsModalOpen} onOpenChange={setIsSettingsModalOpen}>
        <Modal.Content className="max-w-2xl">
          <Modal.Header>
            <Modal.Title className="flex items-center gap-2">
              <Settings className="w-5 h-5" />
              Project Settings
            </Modal.Title>
            <Modal.Description>
              Configure project settings and preferences
            </Modal.Description>
          </Modal.Header>

          <div className="px-6 py-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">Coming Soon</CardTitle>
              </CardHeader>
              <CardContent>
                <Paragraph variant="muted" className="text-sm">
                  Project settings will be available soon
                </Paragraph>
              </CardContent>
            </Card>
          </div>

          <Modal.Footer>
            <Modal.Close asChild>
              <Button variant="secondary">Close</Button>
            </Modal.Close>
          </Modal.Footer>
        </Modal.Content>
      </Modal.Root>

      {/* Add Feature Modal */}
      <Modal.Root
        open={isAddFeatureModalOpen}
        onOpenChange={setIsAddFeatureModalOpen}
      >
        <Modal.Content className="max-w-2xl">
          <Modal.Header>
            <Modal.Title className="flex items-center gap-2">
              <Plus className="w-5 h-5" />
              Enable Feature
            </Modal.Title>
            <Modal.Description>
              Add a new feature to this project
            </Modal.Description>
          </Modal.Header>

          <div className="px-6 py-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">Available Features</CardTitle>
              </CardHeader>
              <CardContent>
                <Paragraph variant="muted" className="text-sm">
                  Feature selection will be available soon
                </Paragraph>
              </CardContent>
            </Card>
          </div>

          <Modal.Footer>
            <Modal.Close asChild>
              <Button variant="secondary">Cancel</Button>
            </Modal.Close>
          </Modal.Footer>
        </Modal.Content>
      </Modal.Root>
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
