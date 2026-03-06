import type { Route } from "./+types/projects";
import { useLoaderData } from "react-router";
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
  MorphModal,
} from "~/components";
import { AlertCircle, Lock, History } from "lucide-react";
import { formatDate } from "~/lib/utils";
import { useState } from "react";

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
      principal?: string;
      group?: string;
      permission_bits?: number;
      scope?: "ROOT" | "WRITE" | "READ";
    }>;
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
  const [accessOpen, setAccessOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);

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

            {/* Action buttons - text on desktop, icons on mobile */}
            <div className="flex items-center gap-2 shrink-0">
              {/* History button */}
              <>
                {/* Desktop: text + icon */}
                <MorphModal
                  trigger={
                    <Button
                      variant="outline"
                      size="sm"
                      data-testid="history-button"
                      className="hidden sm:inline-flex gap-2"
                    >
                      <History className="w-4 h-4" />
                      <span>History</span>
                    </Button>
                  }
                  modalWidth={600}
                  modalHeight={500}
                  isOpen={historyOpen}
                  onOpenChange={setHistoryOpen}
                >
                  {(close) => (
                    <div className="flex flex-col h-full">
                      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
                        <h3 className="text-xs font-mono uppercase tracking-wider text-gray-900 dark:text-gray-100">
                          History
                        </h3>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={close}
                          data-testid="close-history"
                          className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
                        >
                          ✕
                        </Button>
                      </div>
                      <div className="flex-1 overflow-y-auto p-4">
                        <p className="text-sm text-gray-600 dark:text-gray-400">
                          History tracking coming soon...
                        </p>
                      </div>
                    </div>
                  )}
                </MorphModal>

                {/* Mobile: icon only */}
                <MorphModal
                  trigger={
                    <Button
                      variant="outline"
                      size="icon"
                      className="sm:hidden"
                      data-testid="history-button-mobile"
                      title="History"
                    >
                      <History className="w-4 h-4" />
                    </Button>
                  }
                  modalWidth={600}
                  modalHeight={500}
                  isOpen={historyOpen}
                  onOpenChange={setHistoryOpen}
                >
                  {(close) => (
                    <div className="flex flex-col h-full">
                      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
                        <h3 className="text-xs font-mono uppercase tracking-wider text-gray-900 dark:text-gray-100">
                          History
                        </h3>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={close}
                          data-testid="close-history"
                          className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
                        >
                          ✕
                        </Button>
                      </div>
                      <div className="flex-1 overflow-y-auto p-4">
                        <p className="text-sm text-gray-600 dark:text-gray-400">
                          History tracking coming soon...
                        </p>
                      </div>
                    </div>
                  )}
                </MorphModal>
              </>

              {/* Access control button */}
              <>
                {/* Desktop: text + icon */}
                <MorphModal
                  trigger={
                    <Button
                      variant="outline"
                      size="sm"
                      data-testid="access-button"
                      className="hidden sm:inline-flex gap-2"
                    >
                      <Lock className="w-4 h-4" />
                      <span>Access</span>
                    </Button>
                  }
                  modalWidth={600}
                  modalHeight={500}
                  isOpen={accessOpen}
                  onOpenChange={setAccessOpen}
                >
                  {(close) => (
                    <div className="flex flex-col h-full">
                      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
                        <h3 className="text-xs font-mono uppercase tracking-wider text-gray-900 dark:text-gray-100">
                          Access Control
                        </h3>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={close}
                          data-testid="close-access"
                          className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
                        >
                          ✕
                        </Button>
                      </div>
                      <div className="flex-1 overflow-y-auto p-4">
                        {project.acl?.list && project.acl.list.length > 0 ? (
                          <div className="space-y-2">
                            {project.acl.list.map((entry, idx) => {
                              const principal =
                                entry.principal || entry.group || "Unknown";
                              const permissionBits = entry.permission_bits || 0;
                              return (
                                <div
                                  key={idx}
                                  className="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800 rounded-(--radius-component) text-sm"
                                >
                                  <code className="font-mono text-gray-900 dark:text-gray-100">
                                    {principal}
                                  </code>
                                  <PermissionBadge permissions={permissionBits} />
                                </div>
                              );
                            })}
                          </div>
                        ) : (
                          <p className="text-sm text-gray-600 dark:text-gray-400">
                            No custom access control entries
                          </p>
                        )}
                      </div>
                    </div>
                  )}
                </MorphModal>

                {/* Mobile: icon only */}
                <MorphModal
                  trigger={
                    <Button
                      variant="outline"
                      size="icon"
                      className="sm:hidden"
                      data-testid="access-button-mobile"
                      title="Access Control"
                    >
                      <Lock className="w-4 h-4" />
                    </Button>
                  }
                  modalWidth={600}
                  modalHeight={500}
                  isOpen={accessOpen}
                  onOpenChange={setAccessOpen}
                >
                  {(close) => (
                    <div className="flex flex-col h-full">
                      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
                        <h3 className="text-xs font-mono uppercase tracking-wider text-gray-900 dark:text-gray-100">
                          Access Control
                        </h3>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={close}
                          data-testid="close-access"
                          className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
                        >
                          ✕
                        </Button>
                      </div>
                      <div className="flex-1 overflow-y-auto p-4">
                        {project.acl?.list && project.acl.list.length > 0 ? (
                          <div className="space-y-2">
                            {project.acl.list.map((entry, idx) => {
                              const principal =
                                entry.principal || entry.group || "Unknown";
                              const permissionBits = entry.permission_bits || 0;
                              return (
                                <div
                                  key={idx}
                                  className="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800 rounded-(--radius-component) text-sm"
                                >
                                  <code className="font-mono text-gray-900 dark:text-gray-100">
                                    {principal}
                                  </code>
                                  <PermissionBadge permissions={permissionBits} />
                                </div>
                              );
                            })}
                          </div>
                        ) : (
                          <p className="text-sm text-gray-600 dark:text-gray-400">
                            No custom access control entries
                          </p>
                        )}
                      </div>
                    </div>
                  )}
                </MorphModal>
              </>
            </div>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="max-w-6xl mx-auto px-4 py-8">
        {/* Quick info cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
          {/* Created */}
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

          {/* Updated */}
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

          {/* Status */}
          <Card data-testid="status-card">
            <CardHeader>
              <CardTitle className="text-sm font-semibold">Status</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center gap-2">
                {project.deletion ? (
                  <>
                    <AlertCircle className="w-4 h-4 text-red-500" />
                    <Paragraph className="text-sm text-red-600 dark:text-red-400">
                      Deleted
                    </Paragraph>
                  </>
                ) : (
                  <>
                    <div className="w-2 h-2 rounded-full bg-green-500" />
                    <Paragraph className="text-sm text-green-600 dark:text-green-400">
                      Active
                    </Paragraph>
                  </>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Labels & Annotations */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Labels */}
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

          {/* Annotations */}
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

        {/* Empty state for no metadata */}
        {(!project.labels || Object.keys(project.labels).length === 0) &&
          (!project.annotations ||
            Object.keys(project.annotations).length === 0) && (
            <Card className="text-center py-12" data-testid="empty-state">
              <Paragraph className="text-gray-500 dark:text-gray-400">
                No labels or annotations
              </Paragraph>
            </Card>
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
