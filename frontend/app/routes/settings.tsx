import type { Route } from "./+types/settings";
import { useLoaderData, useRevalidator } from "react-router";
import { redirect } from "react-router";
import { Fragment, useState, useRef, useCallback, useEffect, useMemo } from "react";
import type { ReactNode } from "react";
import {
  Button,
  Input,
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  H1,
  H2,
  Paragraph,
  Tabs,
  Table,
  PermissionBadge,
  PrincipalChip,
} from "~/components";
import { usePrincipals } from "~/lib/usePrincipals";
import {
  User2,
  ImagePlus,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Upload,
  X,
  ShieldCheck,
  Crown,
  Users,
  FolderKanban,
  Info,
} from "lucide-react";
import { cn } from "~/lib/utils";
import type { PrincipalMap } from "~/lib/principals";

// ---------------------------------------------------------------------------
// API types
// ---------------------------------------------------------------------------

interface PersonalInfo {
  name: string;
  gender: string;
  job_title: string;
  manager?: string | null;
}

interface ResourceState {
  created_at: string;
  updated_at: string;
  created_by?: string;
  updated_by?: string;
}

interface UserFull {
  id: string;
  personal: PersonalInfo;
  labels?: Record<string, string>;
  annotations?: Record<string, string>;
  state: ResourceState;
  avatar_ulid?: string | null;
  wallpaper_ulid?: string | null;
  hash_code?: string;
}

// ---------------------------------------------------------------------------
// Access-check API types (GET /v1/accesscheck/me/acls)
// ---------------------------------------------------------------------------

interface PermissionFlags {
  bits: number;
  can_fetch: boolean;
  can_list: boolean;
  can_notify: boolean;
  can_create: boolean;
  can_modify: boolean;
}

interface ScopedGrant {
  scope: string;
  permission: PermissionFlags;
  via_principals: string[];
}

interface AclGrant {
  id: string;
  name: string | null;
  permission: PermissionFlags;
  via_principals: string[];
  scoped_grants: ScopedGrant[];
}

interface MyAclsReport {
  user_id: string;
  is_godmode: boolean;
  super_permissions: string[];
  principals: string[];
  direct_memberships: string[];
  groups: AclGrant[];
  projects: AclGrant[];
}

// ---------------------------------------------------------------------------
// Identity helper — the auth cookie is HttpOnly, so the frontend cannot decode
// its own user ID from document.cookie. /accesscheck/me/permissions doubles as
// a lightweight "whoami" and echoes back `user_id` for authenticated callers.
// ---------------------------------------------------------------------------

async function whoami(): Promise<string | null> {
  const res = await fetch(`/api/v1/accesscheck/me/permissions`, {
    credentials: "include",
  });
  if (!res.ok) return null;
  const body = (await res.json()) as { user_id: string };
  return body.user_id;
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Settings - Critical" },
    { name: "description", content: "Account settings" },
  ];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

export async function clientLoader() {
  const [userId, aclsRes] = await Promise.all([
    whoami(),
    fetch(`/api/v1/accesscheck/me/acls`, { credentials: "include" }),
  ]);
  if (!userId) throw redirect("/sign-in");

  const res = await fetch(`/api/v1/global/users/${userId}`, { credentials: "include" });
  if (!res.ok) {
    if (res.status === 401 || res.status === 403) throw redirect("/sign-in");
    throw new Response("Failed to load user", { status: res.status });
  }

  const user: UserFull = await res.json();
  const acls: MyAclsReport | null = aclsRes.ok ? await aclsRes.json() : null;
  return { user, acls };
}

// ---------------------------------------------------------------------------
// Action — profile update
// ---------------------------------------------------------------------------

type ActionResult =
  | { success: true; intent: string }
  | { error: string; intent: string };

export async function clientAction({ request }: Route.ClientActionArgs): Promise<ActionResult> {
  const userId = await whoami();
  if (!userId) return { error: "Not authenticated", intent: "update_profile" };

  const form = await request.formData();
  const intent = (form.get("intent") as string) ?? "";

  if (intent === "update_profile") {
    const res = await fetch(
      `/api/v1/global/users/${userId}`,
      {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({
          id: userId,
          personal: {
            name: form.get("name") as string,
            gender: form.get("gender") as string,
            job_title: form.get("job_title") as string,
            manager: (form.get("manager") as string) || null,
          },
        }),
      }
    );
    if (!res.ok) {
      let msg = `Error ${res.status}`;
      try {
        const j = await res.json() as Record<string, unknown>;
        if (typeof j.error === "string") msg = j.error;
        else if (typeof j.message === "string") msg = j.message;
      } catch { /* ignore */ }
      return { error: msg, intent };
    }
    return { success: true, intent };
  }

  return { error: "Unknown action", intent };
}

// ---------------------------------------------------------------------------
// Avatar helpers
// ---------------------------------------------------------------------------

const AVATAR_COLORS = [
  "bg-red-500", "bg-orange-500", "bg-amber-500", "bg-green-500",
  "bg-emerald-500", "bg-teal-500", "bg-cyan-500", "bg-sky-500",
  "bg-blue-500", "bg-indigo-500", "bg-violet-500", "bg-purple-500",
  "bg-fuchsia-500", "bg-pink-500", "bg-rose-500",
];

function avatarColor(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) hash = (hash * 31 + id.charCodeAt(i)) & 0xffff;
  return AVATAR_COLORS[hash % AVATAR_COLORS.length];
}

function initials(name: string): string | null {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (!parts.length) return null;
  if (parts.length >= 2) return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  return parts[0].slice(0, 2).toUpperCase();
}

// ---------------------------------------------------------------------------
// Page component
// ---------------------------------------------------------------------------

export default function SettingsPage() {
  const { user, acls } = useLoaderData<typeof clientLoader>();
  const revalidator = useRevalidator();

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-6xl mx-auto">
        <div className="mb-8">
          <H1 data-testid="settings-heading">Settings</H1>
          <Paragraph variant="muted" className="mt-0.5">
            Manage your account
          </Paragraph>
        </div>

        <Tabs.Root defaultValue="profile">
          <Tabs.List data-testid="settings-tabs">
            <Tabs.Trigger value="profile" data-testid="tab-profile">Profile</Tabs.Trigger>
            <Tabs.Trigger value="avatar" data-testid="tab-avatar">Avatar</Tabs.Trigger>
            <Tabs.Trigger value="access" data-testid="tab-access">Access</Tabs.Trigger>
          </Tabs.List>

          <div className="mt-6">
            <Tabs.Content value="profile">
              <ProfileTab user={user} onSaved={() => revalidator.revalidate()} />
            </Tabs.Content>

            <Tabs.Content value="avatar">
              <AvatarTab user={user} onUploaded={() => revalidator.revalidate()} />
            </Tabs.Content>

            <Tabs.Content value="access">
              <AccessTab acls={acls} onRefresh={() => revalidator.revalidate()} />
            </Tabs.Content>
          </div>
        </Tabs.Root>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Profile tab
// ---------------------------------------------------------------------------

function ProfileTab({
  user,
  onSaved,
}: {
  user: UserFull;
  onSaved: () => void;
}) {
  const [form, setForm] = useState({
    name: user.personal?.name ?? "",
    gender: user.personal?.gender ?? "",
    job_title: user.personal?.job_title ?? "",
    manager: user.personal?.manager ?? "",
  });
  const [saving, setSaving] = useState(false);
  const [status, setStatus] = useState<"idle" | "ok" | "error">("idle");
  const [errorMsg, setErrorMsg] = useState("");

  // Reset form when user data changes (after revalidate)
  useEffect(() => {
    setForm({
      name: user.personal?.name ?? "",
      gender: user.personal?.gender ?? "",
      job_title: user.personal?.job_title ?? "",
      manager: user.personal?.manager ?? "",
    });
    setStatus("idle");
  }, [user]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setStatus("idle");

    const fd = new FormData();
    fd.append("intent", "update_profile");
    fd.append("name", form.name);
    fd.append("gender", form.gender);
    fd.append("job_title", form.job_title);
    fd.append("manager", form.manager);

    try {
      const res = await fetch(window.location.pathname, {
        method: "POST",
        body: fd,
      });
      const data = await res.json() as ActionResult;
      if ("success" in data) {
        setStatus("ok");
        onSaved();
      } else {
        setStatus("error");
        setErrorMsg(data.error);
      }
    } catch (err) {
      setStatus("error");
      setErrorMsg(String(err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card className="w-full">
      <CardContent className="py-6">
        <H2 className="text-base mb-6">Personal information</H2>

        <form onSubmit={handleSubmit} className="space-y-4">
          <FormField label="Full name" htmlFor="settings-name">
            <Input
              id="settings-name"
              data-testid="settings-name"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              placeholder="Alice Smith"
            />
          </FormField>

          <FormField label="Job title" htmlFor="settings-job-title">
            <Input
              id="settings-job-title"
              data-testid="settings-job-title"
              value={form.job_title}
              onChange={(e) => setForm({ ...form, job_title: e.target.value })}
              placeholder="Senior Engineer"
            />
          </FormField>

          <FormField label="Gender" htmlFor="settings-gender">
            <Input
              id="settings-gender"
              data-testid="settings-gender"
              value={form.gender}
              onChange={(e) => setForm({ ...form, gender: e.target.value })}
              placeholder="e.g. Female, Male, Non-binary"
            />
          </FormField>

          <FormField label="Manager (user ID)" htmlFor="settings-manager">
            <Input
              id="settings-manager"
              data-testid="settings-manager"
              monospace
              value={form.manager}
              onChange={(e) => setForm({ ...form, manager: e.target.value })}
              placeholder="u_alice"
            />
          </FormField>

          {status === "error" && (
            <div className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
              <AlertCircle className="w-4 h-4 shrink-0" />
              {errorMsg}
            </div>
          )}
          {status === "ok" && (
            <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
              <CheckCircle2 className="w-4 h-4 shrink-0" />
              Profile saved.
            </div>
          )}

          <div className="pt-2">
            <Button
              type="submit"
              variant="primary"
              disabled={saving}
              data-testid="settings-save-profile"
            >
              {saving ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Saving…
                </>
              ) : (
                "Save changes"
              )}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Avatar tab
// ---------------------------------------------------------------------------

type UploadState =
  | { phase: "idle" }
  | { phase: "preview"; file: File; previewUrl: string }
  | { phase: "uploading" }
  | { phase: "done" }
  | { phase: "error"; message: string };

function AvatarTab({
  user,
  onUploaded,
}: {
  user: UserFull;
  onUploaded: () => void;
}) {
  const [state, setState] = useState<UploadState>({ phase: "idle" });
  const [dragging, setDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dragCounter = useRef(0);

  const name = user.personal?.name?.trim() || "";
  const letters = initials(name);
  const color = avatarColor(user.id);

  const acceptFile = useCallback((file: File) => {
    if (!file.type.startsWith("image/")) {
      setState({ phase: "error", message: "Please select an image file (JPEG, PNG, or WebP)." });
      return;
    }
    const url = URL.createObjectURL(file);
    setState({ phase: "preview", file, previewUrl: url });
  }, []);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current++;
    setDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current--;
    if (dragCounter.current === 0) setDragging(false);
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      dragCounter.current = 0;
      setDragging(false);
      const file = e.dataTransfer.files[0];
      if (file) acceptFile(file);
    },
    [acceptFile]
  );

  const handleFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) acceptFile(file);
      // Reset so the same file can be re-selected
      e.target.value = "";
    },
    [acceptFile]
  );

  const handleUpload = useCallback(async () => {
    if (state.phase !== "preview") return;
    const { file } = state;

    setState({ phase: "uploading" });

    try {
      const fd = new FormData();
      fd.append("file", file);

      const res = await fetch(
        `/api/v1/global/users/${user.id}/upload/avatar`,
        { method: "POST", body: fd }
      );

      if (!res.ok) {
        let msg = `Upload failed (${res.status})`;
        try {
          const j = await res.json() as Record<string, unknown>;
          if (typeof j.error === "string") msg = j.error;
          else if (typeof j.message === "string") msg = j.message;
        } catch { /* ignore */ }
        setState({ phase: "error", message: msg });
        return;
      }

      setState({ phase: "done" });
      // Give the backend a moment to process the image, then refresh
      setTimeout(onUploaded, 800);
    } catch (err) {
      setState({ phase: "error", message: String(err) });
    }
  }, [state, user.id, onUploaded]);

  const handleDiscard = useCallback(() => {
    if (state.phase === "preview") URL.revokeObjectURL(state.previewUrl);
    setState({ phase: "idle" });
  }, [state]);

  return (
    <div className="flex flex-col lg:flex-row gap-6 max-w-3xl">
      {/* Current avatar */}
      <Card className="lg:w-56 shrink-0 flex flex-col items-center gap-4 p-6">
        <p className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 self-start">
          Current
        </p>
        {user.avatar_ulid ? (
          <img
            src={`/api/v1/static/user_avatars/${user.avatar_ulid}_hd.webp`}
            alt={name || user.id}
            className="w-28 h-28 rounded-(--radius-component-lg) object-cover"
            data-testid="avatar-current-image"
          />
        ) : letters ? (
          <div
            className={cn(
              "w-28 h-28 rounded-(--radius-component-lg) flex items-center justify-center",
              "text-white text-4xl font-semibold select-none",
              color
            )}
            data-testid="avatar-current-initials"
          >
            {letters}
          </div>
        ) : (
          <div
            className="w-28 h-28 rounded-(--radius-component-lg) flex items-center justify-center bg-gray-200 dark:bg-gray-700"
            data-testid="avatar-current-placeholder"
          >
            <User2 className="w-12 h-12 text-gray-400 dark:text-gray-500" />
          </div>
        )}
        <p className="text-xs text-gray-400 dark:text-gray-500 text-center break-all">
          {name || user.id}
        </p>
      </Card>

      {/* Upload area */}
      <Card className="flex-1 min-w-0">
        <CardContent className="py-6 flex flex-col gap-4">
          <H2 className="text-base">Upload new avatar</H2>
          <Paragraph variant="muted" className="text-sm">
            JPEG, PNG, or WebP · max 5 MB · square images work best
          </Paragraph>

          {/* Drop zone */}
          {state.phase !== "preview" && state.phase !== "uploading" && state.phase !== "done" && (
            <div
              onDragEnter={handleDragEnter}
              onDragLeave={handleDragLeave}
              onDragOver={handleDragOver}
              onDrop={handleDrop}
              onClick={() => fileInputRef.current?.click()}
              role="button"
              tabIndex={0}
              onKeyDown={(e) => e.key === "Enter" && fileInputRef.current?.click()}
              aria-label="Drop zone — drag and drop an image here or click to select"
              data-testid="avatar-dropzone"
              className={cn(
                "flex flex-col items-center justify-center gap-3 cursor-pointer",
                "h-44 border-2 border-dashed rounded-(--radius-component-lg)",
                "transition-colors select-none",
                dragging
                  ? "border-primary-500 bg-primary-50 dark:bg-primary-950/30"
                  : "border-gray-300 dark:border-gray-700 hover:border-primary-400 dark:hover:border-primary-600 hover:bg-gray-50 dark:hover:bg-gray-800/40"
              )}
            >
              <div
                className={cn(
                  "w-12 h-12 rounded-(--radius-component-lg) flex items-center justify-center",
                  dragging
                    ? "bg-primary-100 dark:bg-primary-900/40"
                    : "bg-gray-100 dark:bg-gray-800"
                )}
              >
                {dragging ? (
                  <ImagePlus className="w-6 h-6 text-primary-600 dark:text-primary-400" />
                ) : (
                  <Upload className="w-6 h-6 text-gray-400 dark:text-gray-500" />
                )}
              </div>
              <div className="text-center">
                <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  {dragging ? "Drop to select" : "Drag & drop an image here"}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                  or <span className="text-primary-600 dark:text-primary-400">click to browse</span>
                </p>
              </div>
            </div>
          )}

          {/* Hidden file input */}
          <input
            ref={fileInputRef}
            type="file"
            accept="image/jpeg,image/png,image/webp"
            className="sr-only"
            onChange={handleFileChange}
            data-testid="avatar-file-input"
          />

          {/* Preview */}
          {state.phase === "preview" && (
            <div className="flex flex-col gap-4" data-testid="avatar-preview">
              <div className="flex items-center gap-4">
                <img
                  src={state.previewUrl}
                  alt="Preview"
                  className="w-24 h-24 rounded-(--radius-component-lg) object-cover border border-gray-200 dark:border-gray-700"
                />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                    {state.file.name}
                  </p>
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                    {(state.file.size / 1024).toFixed(0)} KB
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleDiscard}
                  data-testid="avatar-discard"
                  className="shrink-0 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>
              <div className="flex gap-2">
                <Button
                  variant="primary"
                  onClick={handleUpload}
                  data-testid="avatar-upload-confirm"
                >
                  <Upload className="w-4 h-4 mr-2" />
                  Upload avatar
                </Button>
                <Button
                  variant="secondary"
                  onClick={handleDiscard}
                  data-testid="avatar-upload-cancel"
                >
                  Cancel
                </Button>
              </div>
            </div>
          )}

          {/* Uploading */}
          {state.phase === "uploading" && (
            <div
              className="flex items-center gap-3 text-sm text-gray-600 dark:text-gray-400"
              data-testid="avatar-uploading"
            >
              <Loader2 className="w-5 h-5 animate-spin text-primary-500" />
              Uploading and processing…
            </div>
          )}

          {/* Done */}
          {state.phase === "done" && (
            <div
              className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400"
              data-testid="avatar-upload-done"
            >
              <CheckCircle2 className="w-5 h-5 shrink-0" />
              Avatar uploaded! Processing in the background…
            </div>
          )}

          {/* Error */}
          {state.phase === "error" && (
            <div
              className="flex items-start gap-2 text-sm text-red-600 dark:text-red-400"
              data-testid="avatar-upload-error"
            >
              <AlertCircle className="w-5 h-5 shrink-0 mt-0.5" />
              <div>
                <p className="font-medium">Upload failed</p>
                <p className="text-xs mt-0.5">{state.message}</p>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setState({ phase: "idle" })}
                  className="mt-2 text-red-600 dark:text-red-400 hover:text-red-800"
                >
                  Try again
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Access tab
// ---------------------------------------------------------------------------

const SUPER_PERMISSION_INFO: Record<string, { label: string; description: string }> = {
  adm_godmode: {
    label: "Godmode",
    description: "Full bypass of every ACL check — unrestricted access to everything.",
  },
  adm_user_manager: {
    label: "User manager",
    description: "Full control over all users, groups, and memberships.",
  },
  adm_config_editor: {
    label: "Config editor",
    description: "Edit global configuration and any project.",
  },
  usr_create_groups: {
    label: "Create groups",
    description: "Allowed to create new groups.",
  },
  usr_create_projects: {
    label: "Create projects",
    description: "Allowed to create new projects.",
  },
};

function AccessTab({
  acls,
  onRefresh,
}: {
  acls: MyAclsReport | null;
  onRefresh: () => void;
}) {
  const principalIds = useMemo(() => {
    if (!acls) return [];
    const ids = new Set<string>();
    acls.principals.forEach((p) => ids.add(p));
    acls.direct_memberships.forEach((p) => ids.add(p));
    acls.groups.forEach((g) => {
      ids.add(g.id);
      g.via_principals.forEach((p) => ids.add(p));
    });
    acls.projects.forEach((p) => {
      p.via_principals.forEach((v) => ids.add(v));
      p.scoped_grants.forEach((sg) => sg.via_principals.forEach((v) => ids.add(v)));
    });
    return Array.from(ids);
  }, [acls]);

  const principals = usePrincipals(principalIds);

  if (!acls) {
    return (
      <Card className="w-full" data-testid="access-tab">
        <CardContent className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
          <AlertCircle className="w-4 h-4 shrink-0" />
          Couldn't load your access report.
          <Button variant="ghost" size="sm" onClick={onRefresh} data-testid="access-retry">
            Retry
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="flex flex-col gap-6" data-testid="access-tab">
      {/* Super-permissions */}
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ShieldCheck className="w-4 h-4 text-primary-500" />
            Super-permissions
          </CardTitle>
          <CardDescription>
            Global, coarse-grained capabilities that apply everywhere — independent of
            any single resource's ACL.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          {acls.is_godmode && (
            <div
              className="flex items-center gap-2 px-3 py-2 rounded-(--radius-component) bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-900 text-amber-800 dark:text-amber-300 text-sm"
              data-testid="access-godmode-banner"
            >
              <Crown className="w-4 h-4 shrink-0" />
              You have godmode — full access to every resource, bypassing all ACLs below.
            </div>
          )}
          {acls.super_permissions.length === 0 ? (
            <Paragraph variant="muted" className="text-sm">
              No super-permissions granted.
            </Paragraph>
          ) : (
            <ul className="flex flex-col gap-3" data-testid="access-super-permissions">
              {acls.super_permissions.map((perm) => {
                const info = SUPER_PERMISSION_INFO[perm];
                return (
                  <li
                    key={perm}
                    className="flex items-start gap-2 text-sm"
                    data-testid={`access-super-permission-${perm}`}
                  >
                    <ShieldCheck className="w-3.5 h-3.5 mt-0.5 shrink-0 text-primary-500" />
                    <div>
                      <span className="font-medium text-gray-900 dark:text-gray-100">
                        {info?.label ?? perm}
                      </span>
                      <span className="ml-1.5 font-mono text-xs text-gray-400 dark:text-gray-500">
                        {perm}
                      </span>
                      {info && (
                        <Paragraph variant="muted" className="text-xs mt-0.5">
                          {info.description}
                        </Paragraph>
                      )}
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </CardContent>
      </Card>

      {/* Principals */}
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="w-4 h-4 text-primary-500" />
            Your principals
          </CardTitle>
          <CardDescription>
            Any ACL entry naming one of these grants you access — your own ID, plus every
            group you belong to directly or transitively (up to 10 levels of nesting).
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <div>
            <p className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-2">
              Direct memberships
            </p>
            {acls.direct_memberships.length === 0 ? (
              <Paragraph variant="muted" className="text-sm">
                Not a direct member of any group.
              </Paragraph>
            ) : (
              <div className="flex flex-wrap gap-2" data-testid="access-direct-memberships">
                {acls.direct_memberships.map((id) => (
                  <PrincipalChip key={id} id={id} info={principals[id]} />
                ))}
              </div>
            )}
          </div>
          <div>
            <p className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-2">
              All effective principals
            </p>
            <div className="flex flex-wrap gap-2" data-testid="access-all-principals">
              {acls.principals.map((id) => (
                <PrincipalChip key={id} id={id} info={principals[id]} />
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      <AclGrantsCard
        title="Groups"
        description="Groups whose ACL grants you access, and via which principal."
        icon={<Users className="w-4 h-4 text-primary-500" />}
        grants={acls.groups}
        principals={principals}
        emptyLabel="No group grants you direct access via its ACL."
        testId="access-groups"
      />

      <AclGrantsCard
        title="Projects"
        description="Projects whose ACL grants you access, and via which principal. Scope-restricted entries apply only to a specific resource kind within the project."
        icon={<FolderKanban className="w-4 h-4 text-primary-500" />}
        grants={acls.projects}
        principals={principals}
        emptyLabel="No project grants you direct access via its ACL."
        testId="access-projects"
      />

      <div className="flex items-start gap-2 text-xs text-gray-400 dark:text-gray-500">
        <Info className="w-3.5 h-3.5 shrink-0 mt-0.5" />
        This report reflects document-level ACL entries only. Super-permissions above
        (e.g. godmode) can grant additional access that isn't listed here.
      </div>
    </div>
  );
}

function AclGrantsCard({
  title,
  description,
  icon,
  grants,
  principals,
  emptyLabel,
  testId,
}: {
  title: string;
  description: string;
  icon: ReactNode;
  grants: AclGrant[];
  principals: PrincipalMap;
  emptyLabel: string;
  testId: string;
}) {
  return (
    <Card className="w-full" data-testid={testId}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {icon}
          {title}
        </CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        {grants.length === 0 ? (
          <Paragraph variant="muted" className="text-sm">
            {emptyLabel}
          </Paragraph>
        ) : (
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.Head>Name</Table.Head>
                <Table.Head>Permission</Table.Head>
                <Table.Head>Via</Table.Head>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {grants.map((grant) => (
                <Fragment key={grant.id}>
                  <Table.Row data-testid={`${testId}-row-${grant.id}`}>
                    <Table.Cell>
                      <div className="flex flex-col">
                        <span className="font-medium">{grant.name ?? grant.id}</span>
                        <span className="font-mono text-xs text-gray-400 dark:text-gray-500">
                          {grant.id}
                        </span>
                      </div>
                    </Table.Cell>
                    <Table.Cell>
                      <PermissionBadge permissions={grant.permission.bits} />
                    </Table.Cell>
                    <Table.Cell>
                      <div className="flex flex-wrap gap-1.5">
                        {grant.via_principals.map((id) => (
                          <PrincipalChip key={id} id={id} info={principals[id]} size="xs" />
                        ))}
                      </div>
                    </Table.Cell>
                  </Table.Row>
                  {grant.scoped_grants.map((sg) => (
                    <Table.Row
                      key={`${grant.id}::${sg.scope}`}
                      data-testid={`${testId}-row-${grant.id}-scope-${sg.scope}`}
                    >
                      <Table.Cell className="pl-8 text-xs text-gray-500 dark:text-gray-400">
                        scope: <span className="font-mono">{sg.scope}</span>
                      </Table.Cell>
                      <Table.Cell>
                        <PermissionBadge permissions={sg.permission.bits} />
                      </Table.Cell>
                      <Table.Cell>
                        <div className="flex flex-wrap gap-1.5">
                          {sg.via_principals.map((id) => (
                            <PrincipalChip key={id} id={id} info={principals[id]} size="xs" />
                          ))}
                        </div>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </Fragment>
              ))}
            </Table.Body>
          </Table.Root>
        )}
      </CardContent>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// FormField helper
// ---------------------------------------------------------------------------

function FormField({
  label,
  htmlFor,
  children,
}: {
  label: string;
  htmlFor: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <label
        htmlFor={htmlFor}
        className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5"
      >
        {label}
      </label>
      {children}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Error Boundary
// ---------------------------------------------------------------------------

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  return (
    <div className="min-h-screen bg-white dark:bg-gray-900 flex items-center justify-center px-4">
      <Card className="max-w-md w-full p-6">
        <div className="flex items-center gap-3 mb-3">
          <AlertCircle className="w-5 h-5 text-red-500" />
          <span className="font-medium text-gray-900 dark:text-gray-100">
            Error loading settings
          </span>
        </div>
        <Paragraph variant="muted">
          {error instanceof Error ? error.message : "Something went wrong."}
        </Paragraph>
      </Card>
    </div>
  );
}
