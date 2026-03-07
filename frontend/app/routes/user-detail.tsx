import type { Route } from "./+types/user-detail";
import { useLoaderData } from "react-router";
import { Link } from "react-router";
import { H1, Paragraph, Card, CardContent, Tabs, YamlEditor } from "~/components";
import { AlertCircle, ArrowLeft, Briefcase, User2, Users2 } from "lucide-react";
import React, { useMemo } from "react";
import { cn, formatDate } from "~/lib/utils";

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
  created_by?: string;
  updated_at: string;
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
// Avatar helpers (same as users.tsx)
// ---------------------------------------------------------------------------

const AVATAR_COLORS = [
  "bg-red-500",
  "bg-orange-500",
  "bg-amber-500",
  "bg-green-500",
  "bg-emerald-500",
  "bg-teal-500",
  "bg-cyan-500",
  "bg-sky-500",
  "bg-blue-500",
  "bg-indigo-500",
  "bg-violet-500",
  "bg-purple-500",
  "bg-fuchsia-500",
  "bg-pink-500",
  "bg-rose-500",
];

function avatarColor(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) & 0xffff;
  }
  return AVATAR_COLORS[hash % AVATAR_COLORS.length];
}

function initials(name: string): string | null {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (!parts.length) return null;
  if (parts.length >= 2) {
    return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  }
  return parts[0].slice(0, 2).toUpperCase();
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

export function meta({ data }: Route.MetaArgs) {
  const name = (data as { user: UserFull } | undefined)?.user?.personal?.name;
  return [
    { title: name ? `${name} - Critical` : "User - Critical" },
    { name: "description", content: "User profile" },
  ];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

export async function loader({ request, params }: Route.LoaderArgs) {
  const cookie = request.headers.get("Cookie") || "";
  const userId = params.user_id;

  const res = await fetch(
    `http://localhost:3742/api/v1/global/users/${userId}`,
    { headers: { Cookie: cookie } }
  );

  if (!res.ok) {
    if (res.status === 401 || res.status === 403) {
      throw new Response("Unauthorized", { status: 401 });
    }
    if (res.status === 404) {
      throw new Response("User not found", { status: 404 });
    }
    throw new Response("Failed to load user", { status: res.status });
  }

  const user: UserFull = await res.json();
  return { user };
}

// ---------------------------------------------------------------------------
// Page component
// ---------------------------------------------------------------------------

export default function UserDetailPage() {
  const { user } = useLoaderData<typeof loader>();
  const name = user.personal?.name?.trim() || "";
  const letters = initials(name);
  const color = avatarColor(user.id);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-6xl mx-auto">
        {/* Back link */}
        <Link
          to="/users"
          className="inline-flex items-center gap-1.5 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors mb-6"
          data-testid="user-detail-back"
        >
          <ArrowLeft className="w-4 h-4" />
          All users
        </Link>

        {/* Two-column profile layout */}
        <div className="flex flex-col lg:flex-row gap-6" data-testid="user-detail-card">
          {/* Left column — avatar + identity */}
          <Card className="lg:w-72 shrink-0 flex flex-col items-center gap-4 p-8">
            {user.avatar_ulid ? (
              <img
                src={`/api/v1/static/user_avatars/${user.avatar_ulid}_hd.webp`}
                alt={name || user.id}
                className="w-32 h-32 rounded-(--radius-component-lg) object-cover"
                data-testid="user-detail-avatar"
              />
            ) : letters ? (
              <div
                className={cn(
                  "w-32 h-32 rounded-(--radius-component-lg) flex items-center justify-center",
                  "text-white text-5xl font-semibold select-none",
                  color
                )}
                aria-hidden="true"
                data-testid="user-detail-avatar"
              >
                {letters}
              </div>
            ) : (
              <div
                className="w-32 h-32 rounded-(--radius-component-lg) flex items-center justify-center bg-gray-200 dark:bg-gray-700"
                aria-hidden="true"
                data-testid="user-detail-avatar"
              >
                <User2 className="w-14 h-14 text-gray-400 dark:text-gray-500" />
              </div>
            )}

            <div className="text-center w-full">
              <H1 className="text-xl wrap-break-word" data-testid="user-detail-name">
                {name || user.id}
              </H1>
              {user.personal?.job_title && (
                <Paragraph
                  variant="muted"
                  className="mt-1 flex items-center justify-center gap-1.5 text-sm"
                  data-testid="user-detail-job-title"
                >
                  <Briefcase className="w-3.5 h-3.5 shrink-0" />
                  {user.personal.job_title}
                </Paragraph>
              )}
              <code className="mt-3 block text-xs font-mono text-gray-400 dark:text-gray-500 break-all">
                {user.id}
              </code>
            </div>
          </Card>

          {/* Right column — details with tabs */}
          <Card className="flex-1 min-w-0">
            <Tabs.Root defaultValue="profile" className="flex flex-col">
              <Tabs.List className="px-4 pt-2">
                <Tabs.Trigger value="profile" data-testid="user-tab-profile">Profile</Tabs.Trigger>
                <Tabs.Trigger value="yaml" data-testid="user-tab-yaml">YAML</Tabs.Trigger>
              </Tabs.List>

              {/* ── Profile tab ── */}
              <Tabs.Content value="profile">
                <CardContent className="py-6 space-y-5">
                  <SectionTitle>Identity</SectionTitle>

                  <DetailRow
                    icon={<User2 className="w-4 h-4" />}
                    label="User ID"
                    value={<code className="font-mono text-sm">{user.id}</code>}
                    testId="user-detail-id"
                  />

                  {user.personal?.gender && (
                    <DetailRow
                      icon={<User2 className="w-4 h-4" />}
                      label="Gender"
                      value={user.personal.gender}
                      testId="user-detail-gender"
                    />
                  )}

                  {user.personal?.manager && (
                    <DetailRow
                      icon={<Users2 className="w-4 h-4" />}
                      label="Manager"
                      value={
                        <Link
                          to={`/u/${user.personal.manager}`}
                          className="font-mono text-sm text-primary-600 dark:text-primary-400 hover:underline"
                          data-testid="user-detail-manager-link"
                        >
                          {user.personal.manager}
                        </Link>
                      }
                      testId="user-detail-manager"
                    />
                  )}

                  {/* Labels */}
                  {user.labels && Object.keys(user.labels).length > 0 && (
                    <div className="pt-2">
                      <SectionTitle>Labels</SectionTitle>
                      <div className="flex flex-wrap gap-1.5 mt-2">
                        {Object.entries(user.labels).map(([k, v]) => (
                          <span
                            key={k}
                            className="inline-flex items-center px-2.5 py-0.5 rounded-(--radius-component) bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium"
                          >
                            <span className="font-semibold mr-1">{k}:</span>
                            {v}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* Timestamps */}
                  {user.state && (
                    <div className="pt-4 border-t border-gray-100 dark:border-gray-800">
                      <div className="grid grid-cols-2 gap-4 text-xs text-gray-500 dark:text-gray-400">
                        <div>
                          <span className="block font-medium mb-0.5">Created</span>
                          <span data-testid="user-detail-created-at">
                            {formatDate(user.state.created_at)}
                          </span>
                        </div>
                        <div>
                          <span className="block font-medium mb-0.5">Updated</span>
                          <span data-testid="user-detail-updated-at">
                            {formatDate(user.state.updated_at)}
                          </span>
                        </div>
                      </div>
                    </div>
                  )}
                </CardContent>
              </Tabs.Content>

              {/* ── YAML tab ── */}
              <Tabs.Content value="yaml" className="p-4 flex flex-col min-h-80">
                <UserYamlTab user={user} />
              </Tabs.Content>
            </Tabs.Root>
          </Card>
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// UserYamlTab — read-only YAML view of user resource
// ---------------------------------------------------------------------------

function UserYamlTab({ user }: { user: UserFull }) {
  const yamlValue = useMemo<Record<string, unknown>>(() => ({
    id: user.id,
    personal: user.personal,
    ...(user.labels && Object.keys(user.labels).length > 0
      ? { labels: user.labels }
      : {}),
    ...(user.annotations && Object.keys(user.annotations).length > 0
      ? { annotations: user.annotations }
      : {}),
  }), [user]);

  return (
    <YamlEditor
      value={yamlValue}
      onChange={() => {}}
      disabled
      data-testid="user-yaml-editor"
    />
  );
}

// ---------------------------------------------------------------------------
// SectionTitle helper
// ---------------------------------------------------------------------------

function SectionTitle({ children }: { children: React.ReactNode }) {
  return (
    <p className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
      {children}
    </p>
  );
}

// ---------------------------------------------------------------------------
// DetailRow helper
// ---------------------------------------------------------------------------

function DetailRow({
  icon,
  label,
  value,
  testId,
}: {
  icon: React.ReactNode;
  label: string;
  value: React.ReactNode;
  testId?: string;
}) {
  return (
    <div className="flex items-start gap-3" data-testid={testId}>
      <span className="mt-0.5 text-gray-400 dark:text-gray-500 shrink-0">
        {icon}
      </span>
      <div className="flex-1 min-w-0">
        <span className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-0.5">
          {label}
        </span>
        <span className="text-sm text-gray-900 dark:text-gray-100 break-all">
          {value}
        </span>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Error Boundary
// ---------------------------------------------------------------------------

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  const is404 =
    error instanceof Response && error.status === 404;

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900 flex items-center justify-center px-4">
      <Card className="max-w-md w-full p-6">
        <div className="flex items-center gap-3 mb-3">
          <AlertCircle className="w-5 h-5 text-red-500" />
          <span className="font-medium text-gray-900 dark:text-gray-100">
            {is404 ? "User not found" : "Error loading user"}
          </span>
        </div>
        <Paragraph variant="muted">
          {is404
            ? "This user doesn't exist or you don't have access."
            : error instanceof Error
            ? error.message
            : "Something went wrong while loading this user."}
        </Paragraph>
        <Link
          to="/users"
          className="mt-4 inline-flex items-center gap-1.5 text-sm text-primary-600 dark:text-primary-400 hover:underline"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to users
        </Link>
      </Card>
    </div>
  );
}
