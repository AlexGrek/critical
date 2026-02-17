import type { Route } from "./+types/groups";
import { useLoaderData, Link, useFetcher, useRevalidator } from "react-router";
import { Button, MorphModal } from "~/components";
import { useState, useEffect } from "react";

// TypeScript types matching the Rust models
interface AccessControlList {
  permissions: number;
  principals: string[];
}

interface AccessControlStore {
  list: AccessControlList[];
  last_mod_date: string;
}

interface Group {
  id: string;
  name: string;
  acl: AccessControlStore;
}

interface GroupsResponse {
  items: Group[];
}

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Groups - Critical" },
    { name: "description", content: "View all available groups" },
  ];
}

export async function loader({ request }: Route.LoaderArgs) {
  // Fetch groups from the API
  const response = await fetch("http://localhost:3742/api/v1/global/groups", {
    headers: {
      // Cookie will be automatically included by the browser
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

export default function Groups() {
  const { groups } = useLoaderData<typeof loader>();
  const fetcher = useFetcher();
  const revalidator = useRevalidator();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [formData, setFormData] = useState({ name: "", id: "" });
  const [error, setError] = useState("");

  const validateGroupId = (id: string): string | null => {
    if (id.length < 2) {
      return "Group ID must be at least 2 characters";
    }

    if (id.length > 63) {
      return "Group ID must be at most 63 characters";
    }

    if (!/^[a-z0-9_]+$/.test(id)) {
      return "Group ID can only contain lowercase letters, numbers, and underscores";
    }

    if (/^[0-9]/.test(id)) {
      return "Group ID cannot start with a digit";
    }

    return null;
  };

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

  // Handle successful creation
  useEffect(() => {
    if (fetcher.data?.success && isModalOpen) {
      setIsModalOpen(false);
      setFormData({ name: "", id: "" });
      setError("");
      revalidator.revalidate();
    }
  }, [fetcher.data, isModalOpen, revalidator]);

  // Handle errors from action
  useEffect(() => {
    if (fetcher.data?.error) {
      setError(fetcher.data.error);
    }
  }, [fetcher.data]);

  return (
    <div className="min-h-screen bg-gray-950 px-4 py-8">
      <div className="max-w-6xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">Groups</h1>
            <p className="text-gray-400">
              View and manage all groups in the system
            </p>
          </div>
          <div className="flex gap-3">
            <MorphModal
              trigger={
                <Button variant="primary" data-testid="create-group-button">
                  Create Group
                </Button>
              }
              modalWidth={500}
              modalHeight={400}
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
                  <h2 className="text-2xl font-bold text-white mb-4">
                    Create New Group
                  </h2>
                  <form onSubmit={handleSubmit} className="flex-1 flex flex-col">
                    <div className="flex-1 space-y-4">
                      {error && (
                        <div
                          className="bg-red-500/10 border border-red-500/50 rounded-lg p-3 text-red-400 text-sm"
                          data-testid="create-group-error"
                        >
                          {error}
                        </div>
                      )}

                      <div>
                        <label
                          htmlFor="group-name"
                          className="block text-sm font-medium text-gray-300 mb-2"
                        >
                          Group Name
                        </label>
                        <input
                          id="group-name"
                          type="text"
                          data-testid="group-name-input"
                          value={formData.name}
                          onChange={(e) =>
                            setFormData({ ...formData, name: e.target.value })
                          }
                          className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500"
                          placeholder="Enter group name"
                          required
                        />
                      </div>

                      <div>
                        <label
                          htmlFor="group-id"
                          className="block text-sm font-medium text-gray-300 mb-2"
                        >
                          Group ID
                        </label>
                        <input
                          id="group-id"
                          type="text"
                          data-testid="group-id-input"
                          value={formData.id}
                          onChange={(e) => {
                            const value = e.target.value.toLowerCase();
                            setFormData({ ...formData, id: value });
                          }}
                          className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 font-mono"
                          placeholder="my_group"
                          required
                        />
                        <p className="mt-1 text-xs text-gray-500">
                          2-63 characters, lowercase letters, numbers, and underscores
                          only (g_ prefix will be added automatically)
                        </p>
                      </div>
                    </div>

                    <div className="flex gap-3 pt-4 border-t border-gray-800">
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
            <Link to="/">
              <Button variant="secondary">Back to Home</Button>
            </Link>
          </div>
        </div>

        {groups.length === 0 ? (
          <div className="bg-gray-900 border border-gray-800 rounded-lg p-12 text-center">
            <p className="text-gray-400 text-lg">No groups found</p>
            <p className="text-gray-500 text-sm mt-2">
              Groups will appear here once they are created
            </p>
          </div>
        ) : (
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {groups.map((group) => (
              <div
                key={group.id}
                data-testid={`group-card-${group.id}`}
                className="bg-gray-900 border border-gray-800 rounded-lg p-6 hover:border-gray-700 transition-colors"
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="flex-1">
                    <h2 className="text-xl font-semibold text-white mb-1">
                      {group.name}
                    </h2>
                    <p className="text-sm text-gray-500 font-mono">{group.id}</p>
                  </div>
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-400">Access Rules:</span>
                    <span className="text-gray-300 font-medium">
                      {group.acl.list.length}
                    </span>
                  </div>
                  {group.acl.list.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-gray-800">
                      <p className="text-xs text-gray-500 mb-2">Principals:</p>
                      <div className="flex flex-wrap gap-1">
                        {group.acl.list.flatMap((acl) =>
                          acl.principals.map((principal, idx) => (
                            <span
                              key={`${principal}-${idx}`}
                              className="inline-flex items-center px-2 py-1 rounded text-xs font-mono bg-gray-800 text-gray-300"
                            >
                              {principal}
                            </span>
                          ))
                        )}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
