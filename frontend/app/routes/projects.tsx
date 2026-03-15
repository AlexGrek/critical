import type { Route } from "./+types/projects";
import { useLoaderData, useRevalidator, Link } from "react-router";
import {
  Button,
  Input,
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
  YamlEditorDrawer,
  Modal,
  PrincipalChip,
} from "~/components";
import type { AccessControlStore } from "~/components";
import { AlertCircle, Lock, Settings, Plus, Github, GitBranch, Trash2, ExternalLink } from "lucide-react";
import { formatDate, cn } from "~/lib/utils";
import { useState, useMemo, useCallback, useRef } from "react";
import { resolvePrincipalsClient } from "~/lib/principals";
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

type RepoProvider = "git" | "github" | "gitlab" | "bitbucket" | "svn" | "mercurial" | "custom";

interface RepoLink {
  url: string;
  provider: RepoProvider;
  name?: string;
  default_branch?: string;
}

interface Project {
  id: string;
  name: string;
  description?: string;
  repositories?: RepoLink[];
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

export async function clientLoader({ params }: Route.ClientLoaderArgs) {
  const { project_id } = params;

  if (!project_id) {
    throw new Response("Project ID is required", { status: 400 });
  }

  const response = await fetch(
    `/api/v1/global/projects/${project_id}`,
    { credentials: "include" }
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

  const principals = await resolvePrincipalsClient(principalIds);

  return { project, principals };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function ProjectPage() {
  const { project, principals } = useLoaderData<typeof clientLoader>();
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
  const [isSavingFeatures, setIsSavingFeatures] = useState(false);
  const [featureError, setFeatureError] = useState("");
  const revalidator = useRevalidator();

  const AVAILABLE_FEATURES = [
    { id: "tickets", name: "Tickets", description: "Issue tracking and ticket management" },
    { id: "pipelines", name: "Pipelines", description: "CI/CD pipeline management" },
  ];

  const handleToggleFeature = async (featureId: string) => {
    setFeatureError("");
    setIsSavingFeatures(true);
    try {
      const currentServices = project.enabled_services || [];
      const isEnabled = currentServices.includes(featureId);
      const newServices = isEnabled
        ? currentServices.filter((s) => s !== featureId)
        : [...currentServices, featureId];

      const response = await fetch(
        `/api/v1/global/projects/${project.id}`,
        {
          method: "PUT",
          credentials: "include",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ ...project, enabled_services: newServices }),
        }
      );

      if (!response.ok) {
        const body = await response.json().catch(() => ({})) as Record<string, unknown>;
        throw new Error(String(body.message ?? body.error ?? `HTTP ${response.status}`));
      }
      revalidator.revalidate();
    } catch (err) {
      setFeatureError(err instanceof Error ? err.message : "Failed to save features");
    } finally {
      setIsSavingFeatures(false);
    }
  };

  const handleAclSave = async (newAcl: AccessControlStore) => {
    setIsSavingAcl(true);
    try {
      const response = await fetch(
        `/api/v1/global/projects/${project.id}`,
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
  const repositories = project.repositories || [];

  // ── Repositories ──────────────────────────────────────────────────────────
  const [isAddingRepo, setIsAddingRepo] = useState(false);
  const [repoForm, setRepoForm] = useState<{
    url: string;
    provider: RepoProvider;
    name: string;
    default_branch: string;
  }>({ url: "", provider: "github", name: "", default_branch: "" });
  const [repoSaving, setRepoSaving] = useState(false);
  const [repoError, setRepoError] = useState("");
  const [removingIdx, setRemovingIdx] = useState<number | null>(null);
  const urlInputRef = useRef<HTMLInputElement>(null);

  function detectProvider(url: string): RepoProvider {
    if (url.includes("github.com")) return "github";
    if (url.includes("gitlab.com")) return "gitlab";
    if (url.includes("bitbucket.org")) return "bitbucket";
    return "git";
  }

  const saveRepositories = useCallback(
    async (repos: RepoLink[]) => {
      const res = await fetch(`/api/v1/global/projects/${project.id}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ ...project, repositories: repos }),
      });
      if (!res.ok) {
        const body = await res.json().catch(() => ({})) as Record<string, unknown>;
        throw new Error(String(body.message ?? body.error ?? `HTTP ${res.status}`));
      }
      revalidator.revalidate();
    },
    [project, revalidator]
  );

  const handleAddRepo = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      setRepoError("");
      if (!repoForm.url.trim()) return;
      setRepoSaving(true);
      try {
        const newRepo: RepoLink = {
          url: repoForm.url.trim(),
          provider: repoForm.provider,
          ...(repoForm.name.trim() ? { name: repoForm.name.trim() } : {}),
          ...(repoForm.default_branch.trim() ? { default_branch: repoForm.default_branch.trim() } : {}),
        };
        await saveRepositories([...repositories, newRepo]);
        setIsAddingRepo(false);
        setRepoForm({ url: "", provider: "github", name: "", default_branch: "" });
      } catch (err) {
        setRepoError(err instanceof Error ? err.message : "Failed to add repository");
      } finally {
        setRepoSaving(false);
      }
    },
    [repoForm, repositories, saveRepositories]
  );

  const handleRemoveRepo = useCallback(
    async (idx: number) => {
      setRemovingIdx(idx);
      try {
        await saveRepositories(repositories.filter((_, i) => i !== idx));
      } finally {
        setRemovingIdx(null);
      }
    },
    [repositories, saveRepositories]
  );

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

              <YamlEditorDrawer
                value={yamlValue}
                onSave={handleYamlSave}
                readOnlyFields={["state", "hash_code", "deletion"]}
                allowedTopLevelKeys={["id", "name", "description", "repositories", "enabled_services", "labels", "annotations", "acl", "state", "hash_code", "deletion"]}
                triggerLabel="YAML"
                drawerTitle="Edit Project YAML"
                data-testid="project-yaml-drawer"
              />

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

            {/* Repositories */}
            <Card className="mb-8" data-testid="repositories-card">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle>Repositories</CardTitle>
                    <CardDescription>Source code repositories linked to this project</CardDescription>
                  </div>
                  {!isAddingRepo && (
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => {
                        setIsAddingRepo(true);
                        setTimeout(() => urlInputRef.current?.focus(), 50);
                      }}
                      data-testid="add-repo-button"
                      title="Add repository"
                    >
                      <Plus className="w-4 h-4" />
                    </Button>
                  )}
                </div>
              </CardHeader>
              <CardContent>
                {repositories.length === 0 && !isAddingRepo && (
                  <Paragraph variant="muted" size="sm" data-testid="no-repos-message">
                    No repositories linked yet.
                  </Paragraph>
                )}

                {/* Repo list */}
                {repositories.length > 0 && (
                  <div className="space-y-2 mb-4" data-testid="repo-list">
                    {repositories.map((repo, idx) => (
                      <div
                        key={idx}
                        className="flex items-center gap-3 px-3 py-2 bg-gray-50 dark:bg-gray-800/60 rounded-(--radius-component) border border-gray-100 dark:border-gray-800"
                        data-testid={`repo-entry-${idx}`}
                      >
                        <span className="shrink-0 text-gray-400 dark:text-gray-500">
                          {repo.provider === "github" ? (
                            <Github className="w-4 h-4" />
                          ) : (
                            <GitBranch className="w-4 h-4" />
                          )}
                        </span>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 min-w-0">
                            <a
                              href={repo.url}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-sm font-mono text-primary-600 dark:text-primary-400 hover:underline truncate"
                              data-testid={`repo-url-${idx}`}
                            >
                              {repo.name || repo.url}
                            </a>
                            <ExternalLink className="w-3 h-3 shrink-0 text-gray-400" />
                          </div>
                          <div className="flex items-center gap-2 mt-0.5 flex-wrap">
                            <span className="text-xs text-gray-400 dark:text-gray-500 capitalize">{repo.provider}</span>
                            {repo.name && (
                              <span className="text-xs font-mono text-gray-500 dark:text-gray-400 truncate">{repo.url}</span>
                            )}
                            {repo.default_branch && (
                              <span className="text-xs text-gray-400 dark:text-gray-500">
                                branch: <code className="font-mono">{repo.default_branch}</code>
                              </span>
                            )}
                          </div>
                        </div>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleRemoveRepo(idx)}
                          disabled={removingIdx === idx}
                          data-testid={`remove-repo-${idx}`}
                          title="Remove repository"
                          className="shrink-0 text-gray-400 hover:text-red-500"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </Button>
                      </div>
                    ))}
                  </div>
                )}

                {/* Add repo form */}
                {isAddingRepo && (
                  <form
                    onSubmit={handleAddRepo}
                    className="space-y-3 pt-2 border-t border-gray-100 dark:border-gray-800 mt-2"
                    data-testid="add-repo-form"
                  >
                    {repoError && (
                      <p className="text-xs text-red-600 dark:text-red-400" data-testid="repo-error">{repoError}</p>
                    )}
                    <div>
                      <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Repository URL <span className="text-red-500">*</span>
                      </label>
                      <Input
                        ref={urlInputRef}
                        data-testid="repo-url-input"
                        monospace
                        value={repoForm.url}
                        onChange={(e) => {
                          const url = e.target.value;
                          setRepoForm((f) => ({
                            ...f,
                            url,
                            provider: url ? detectProvider(url) : f.provider,
                          }));
                        }}
                        placeholder="https://github.com/org/repo"
                        required
                      />
                    </div>
                    <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                      <div>
                        <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Provider</label>
                        <select
                          value={repoForm.provider}
                          onChange={(e) => setRepoForm((f) => ({ ...f, provider: e.target.value as RepoProvider }))}
                          data-testid="repo-provider-select"
                          className="w-full px-3 py-2 text-sm bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-(--radius-component) text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                        >
                          <option value="github">GitHub</option>
                          <option value="gitlab">GitLab</option>
                          <option value="bitbucket">Bitbucket</option>
                          <option value="git">Git</option>
                          <option value="svn">SVN</option>
                          <option value="mercurial">Mercurial</option>
                          <option value="custom">Custom</option>
                        </select>
                      </div>
                      <div>
                        <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                          Display name <span className="text-gray-400">(optional)</span>
                        </label>
                        <Input
                          data-testid="repo-name-input"
                          value={repoForm.name}
                          onChange={(e) => setRepoForm((f) => ({ ...f, name: e.target.value }))}
                          placeholder="my-repo"
                        />
                      </div>
                      <div>
                        <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                          Default branch <span className="text-gray-400">(optional)</span>
                        </label>
                        <Input
                          data-testid="repo-branch-input"
                          monospace
                          value={repoForm.default_branch}
                          onChange={(e) => setRepoForm((f) => ({ ...f, default_branch: e.target.value }))}
                          placeholder="main"
                        />
                      </div>
                    </div>
                    <div className="flex gap-2 pt-1">
                      <Button
                        type="submit"
                        variant="primary"
                        size="sm"
                        disabled={repoSaving || !repoForm.url.trim()}
                        data-testid="save-repo-button"
                      >
                        {repoSaving ? "Saving…" : "Add repository"}
                      </Button>
                      <Button
                        type="button"
                        variant="secondary"
                        size="sm"
                        onClick={() => {
                          setIsAddingRepo(false);
                          setRepoError("");
                          setRepoForm({ url: "", provider: "github", name: "", default_branch: "" });
                        }}
                        data-testid="cancel-add-repo"
                      >
                        Cancel
                      </Button>
                    </div>
                  </form>
                )}
              </CardContent>
            </Card>

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
              {service === "tickets" ? (
                <Card className="w-full">
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <div>
                        <CardTitle>Ticket Groups</CardTitle>
                        <CardDescription>Manage issue types and workflows</CardDescription>
                      </div>
                      <Link to={`/p/${project.id}/tickets`}>
                        <Button variant="primary" size="sm" data-testid="manage-tickets-button">
                          Manage Ticket Groups
                          <Plus className="w-3.5 h-3.5 ml-1.5" />
                        </Button>
                      </Link>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <Paragraph variant="muted" size="sm">
                      Define ticket types and their workflows. Click "Manage Ticket Groups" to configure issue types, statuses, and custom fields.
                    </Paragraph>
                  </CardContent>
                </Card>
              ) : (
                <Card className="w-full py-12">
                  <div className="flex flex-col items-center gap-4 px-8">
                    <H1 className="capitalize">{service}</H1>
                    <Paragraph className="text-center max-w-prose text-gray-600 dark:text-gray-400">
                      {service === "pipelines" ? "CI/CD pipeline management coming soon" : "Content coming soon"}
                    </Paragraph>
                  </div>
                </Card>
              )}
            </Tabs.Content>
          ))}

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
              Enable Features
            </Modal.Title>
            <Modal.Description>
              Select which features to enable for this project
            </Modal.Description>
          </Modal.Header>

          <div className="px-6 py-4">
            {featureError && (
              <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-(--radius-component) text-red-700 dark:text-red-400 text-sm">
                {featureError}
              </div>
            )}
            <div className="space-y-3">
              {AVAILABLE_FEATURES.map((feature) => {
                const isEnabled = project.enabled_services?.includes(feature.id) ?? false;
                return (
                  <button
                    key={feature.id}
                    onClick={() => handleToggleFeature(feature.id)}
                    disabled={isSavingFeatures}
                    className={cn(
                      "w-full px-4 py-3 rounded-(--radius-component) border-2 transition-colors text-left",
                      isEnabled
                        ? "border-primary-500 bg-primary-50 dark:bg-primary-900/20"
                        : "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600",
                      isSavingFeatures && "opacity-50 cursor-not-allowed"
                    )}
                    data-testid={`feature-toggle-${feature.id}`}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1">
                        <p className={cn("font-medium text-sm", isEnabled ? "text-primary-700 dark:text-primary-300" : "text-gray-900 dark:text-gray-100")}>
                          {feature.name}
                        </p>
                        <p className={cn("text-xs mt-0.5", isEnabled ? "text-primary-600 dark:text-primary-400" : "text-gray-600 dark:text-gray-400")}>
                          {feature.description}
                        </p>
                      </div>
                      <div className={cn("w-5 h-5 rounded-(--radius-component) border-2 flex items-center justify-center shrink-0 transition-colors", isEnabled ? "bg-primary-500 border-primary-500" : "border-gray-300 dark:border-gray-600")}>
                        {isEnabled && (
                          <svg className="w-3 h-3 text-white" viewBox="0 0 12 10" fill="none">
                            <path d="M1 5L4 8L11 1" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                          </svg>
                        )}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
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
