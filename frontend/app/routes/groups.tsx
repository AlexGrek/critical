import type { Route } from "./+types/groups";
import { useLoaderData, Link, useFetcher, useRevalidator } from "react-router";
import {
  Button,
  Input,
  MorphModal,
  Card,
  CardTitle,
  H1,
  H2,
  Paragraph,
} from "~/components";
import { useState, useEffect } from "react";

// ---------------------------------------------------------------------------
// API types — match the actual shapes returned by the backend
// ---------------------------------------------------------------------------

/** An ACL entry granting a set of permissions to a list of principals. */
interface AccessControlList {
  /** Bitfield of Permissions flags (FETCH=1, LIST=2, NOTIFY=4, CREATE=8, MODIFY=16). */
  permissions: number;
  principals: string[];
  /** Optional scope, e.g. "tasks" or "*". Absent means wildcard. */
  scope?: string;
}

interface AccessControlStore {
  list: AccessControlList[];
  last_mod_date: string;
}

/** Server-managed audit timestamps injected by the #[crit_resource] macro. */
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

/**
 * Shape returned by the list endpoint (GET /v1/global/groups).
 * Only brief fields: id, name, labels — per Group::brief_field_names() in the Rust macro.
 */
interface GroupBrief {
  id: string;
  name: string;
  labels: Record<string, string>;
}

/**
 * Full shape returned by the single GET endpoint (GET /v1/global/groups/{id}).
 * Includes all fields injected by the #[crit_resource] macro.
 */
export interface GroupFull extends GroupBrief {
  description?: string;
  annotations: Record<string, string>;
  acl: AccessControlStore;
  state: ResourceState;
  deletion?: DeletionInfo;
  hash_code: string;
}

interface GroupsResponse {
  items: GroupBrief[];
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
  const response = await fetch("http://localhost:3742/api/v1/global/groups", {
    headers: {
      Cookie: request.headers.get("Cookie") || "",
    },
  });

  if (!response.ok) {
    if (response.status === 401 || response.status === 403) {
      throw new Response("Unauthorized", { status: 401 });
    }
    throw new Response("Failed to load groups", { status: response.status });
  }

  const data: GroupsResponse = await response.json();
  return { groups: data.items };
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

export async function action({ request }: Route.ActionArgs) {
  const formData = await request.formData();
  const intent = formData.get("intent");

  if (intent === "create") {
    const name = formData.get("name");
    const id = formData.get("id");

    if (!name || !id) {
      return { error: "Name and ID are required" };
    }

    const response = await fetch("http://localhost:3742/api/v1/global/groups", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Cookie: request.headers.get("Cookie") || "",
      },
      body: JSON.stringify({ name, id }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      return { error: `Failed to create group: ${errorText}` };
    }

    return { success: true };
  }

  return { error: "Unknown action" };
}

// ---------------------------------------------------------------------------
// Client-side validation (mirrors backend validate_group_id)
// ---------------------------------------------------------------------------

function validateGroupId(id: string): string | null {
  // Strip g_ prefix if the user typed it — backend strips it too
  const bare = id.startsWith("g_") ? id.slice(2) : id;
  if (bare.length < 2) return "Group ID must be at least 2 characters (excluding g_ prefix)";
  if (bare.length > 63) return "Group ID must be at most 63 characters (excluding g_ prefix)";
  if (!/^[a-z0-9_]+$/.test(bare)) return "Group ID can only contain lowercase letters, numbers, and underscores";
  if (/^[0-9]/.test(bare)) return "Group ID cannot start with a digit";
  return null;
}

// ---------------------------------------------------------------------------
// Page component
// ---------------------------------------------------------------------------

export default function Groups() {
  const { groups } = useLoaderData<typeof loader>();
  const fetcher = useFetcher();
  const revalidator = useRevalidator();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [formData, setFormData] = useState({ name: "", id: "" });
  const [error, setError] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!formData.name.trim()) {
      setError("Group name is required");
      return;
    }

    if (!formData.id.trim()) {
      setError("Group ID is required");
      return;
    }

    const idError = validateGroupId(formData.id);
    if (idError) {
      setError(idError);
      return;
    }

    const form = new FormData();
    form.append("intent", "create");
    form.append("name", formData.name);
    form.append("id", formData.id);
    fetcher.submit(form, { method: "POST" });
  };

  useEffect(() => {
    if (fetcher.data?.success && isModalOpen) {
      setIsModalOpen(false);
      setFormData({ name: "", id: "" });
      setError("");
      revalidator.revalidate();
    }
  }, [fetcher.data, isModalOpen, revalidator]);

  useEffect(() => {
    if (fetcher.data?.error) {
      setError(fetcher.data.error);
    }
  }, [fetcher.data]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-6xl mx-auto">
        {/* Page header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <H1 data-testid="groups-page-heading">{"{!} "}Groups</H1>
            <Paragraph variant="muted" data-testid="groups-description">
              View and manage all groups in the system
            </Paragraph>
          </div>
          <div className="flex gap-3">
            <MorphModal
              trigger={
                <Button variant="primary" data-testid="create-group-button">
                  Create Group
                </Button>
              }
              modalWidth={500}
              modalHeight={420}
              isOpen={isModalOpen}
              onOpenChange={(open) => {
                setIsModalOpen(open);
                if (!open) {
                  setFormData({ name: "", id: "" });
                  setError("");
                }
              }}
            >
              {(close) => (
                <div className="flex flex-col h-full">
                  <H2 data-testid="create-group-modal-title" className="mb-4">
                    Create New Group
                  </H2>
                  <form onSubmit={handleSubmit} className="flex-1 flex flex-col">
                    <div className="flex-1 space-y-4">
                      {error && (
                        <div
                          className="bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/50 rounded-(--radius-component) p-3 text-red-600 dark:text-red-400 text-sm"
                          data-testid="create-group-error"
                        >
                          {error}
                        </div>
                      )}

                      <div>
                        <label
                          htmlFor="group-name"
                          className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                        >
                          Group Name
                        </label>
                        <Input
                          id="group-name"
                          type="text"
                          data-testid="group-name-input"
                          value={formData.name}
                          onChange={(e) =>
                            setFormData({ ...formData, name: e.target.value })
                          }
                          placeholder="Enter group name"
                          required
                        />
                      </div>

                      <div>
                        <label
                          htmlFor="group-id"
                          className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                        >
                          Group ID
                        </label>
                        <Input
                          id="group-id"
                          type="text"
                          monospace
                          data-testid="group-id-input"
                          value={formData.id}
                          onChange={(e) =>
                            setFormData({
                              ...formData,
                              id: e.target.value.toLowerCase(),
                            })
                          }
                          placeholder="my_group"
                          required
                        />
                        <Paragraph
                          variant="subtle"
                          className="mt-1 text-xs"
                          data-testid="group-id-hint"
                        >
                          2–63 characters, lowercase letters, numbers, and
                          underscores only. The{" "}
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
                        disabled={fetcher.state === "submitting"}
                      >
                        {fetcher.state === "submitting"
                          ? "Creating..."
                          : "Create Group"}
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

            <Link to="/" data-testid="back-to-home-link">
              <Button variant="secondary">Back to Home</Button>
            </Link>
          </div>
        </div>

        {/* Content */}
        {groups.length === 0 ? (
          <Card data-testid="groups-empty-state" className="p-12 text-center">
            <CardTitle className="text-lg">No groups found</CardTitle>
            <Paragraph variant="muted" className="mt-2">
              Groups will appear here once they are created
            </Paragraph>
          </Card>
        ) : (
          <div
            data-testid="groups-grid"
            className="grid gap-4 md:grid-cols-2 lg:grid-cols-3"
          >
            {groups.map((group) => (
              <GroupCard key={group.id} group={group} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Group card
// ---------------------------------------------------------------------------

function GroupCard({ group }: { group: GroupBrief }) {
  const labelEntries = Object.entries(group.labels);

  return (
    <Card
      data-testid={`group-card-${group.id}`}
      className="p-6 hover:border-gray-300 dark:hover:border-gray-700 transition-colors"
    >
      <div className="mb-3">
        <CardTitle className="text-lg mb-1">{group.name}</CardTitle>
        <span
          data-testid={`group-id-label-${group.id}`}
          className="text-sm text-gray-500 dark:text-gray-400 font-mono"
        >
          {group.id}
        </span>
      </div>

      {labelEntries.length > 0 && (
        <div
          className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-800"
          data-testid={`group-labels-${group.id}`}
        >
          <Paragraph variant="subtle" className="text-xs mb-2">
            Labels
          </Paragraph>
          <div className="flex flex-wrap gap-1">
            {labelEntries.map(([key, value]) => (
              <span
                key={key}
                className="inline-flex items-center px-2 py-0.5 rounded-(--radius-component) text-xs font-mono bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300"
              >
                {key}={value}
              </span>
            ))}
          </div>
        </div>
      )}
    </Card>
  );
}
