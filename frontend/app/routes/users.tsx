import type { Route } from "./+types/users";
import { useLoaderData } from "react-router";
import { Link } from "react-router";
import { H1, Paragraph, Card } from "~/components";
import { AlertCircle, UserCircle2, User2 } from "lucide-react";
import { cn } from "~/lib/utils";

// ---------------------------------------------------------------------------
// API types
// ---------------------------------------------------------------------------

interface PersonalInfo {
  name: string;
  gender: string;
  job_title: string;
  manager?: string | null;
}

interface UserBrief {
  id: string;
  personal: PersonalInfo;
  labels?: Record<string, string>;
  avatar_ulid?: string | null;
}

// ---------------------------------------------------------------------------
// Avatar helpers
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

/** Returns 1-2 letter initials from a display name, or null if name is empty. */
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

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Users - Critical" },
    { name: "description", content: "Browse all users" },
  ];
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

export async function loader({ request }: Route.LoaderArgs) {
  const cookie = request.headers.get("Cookie") || "";

  const res = await fetch("http://localhost:3742/api/v1/global/users", {
    headers: { Cookie: cookie },
  });

  if (!res.ok) {
    if (res.status === 401 || res.status === 403) {
      throw new Response("Unauthorized", { status: 401 });
    }
    throw new Response("Failed to load users", { status: res.status });
  }

  const data: { items: UserBrief[] } = await res.json();
  return { users: data.items };
}

// ---------------------------------------------------------------------------
// Page component
// ---------------------------------------------------------------------------

export default function UsersPage() {
  const { users } = useLoaderData<typeof loader>();

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 px-4 py-8">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <H1 data-testid="users-page-heading">Users</H1>
          <Paragraph variant="muted" className="mt-0.5">
            {users.length} {users.length === 1 ? "user" : "users"} in the system
          </Paragraph>
        </div>

        {/* Grid */}
        {users.length === 0 ? (
          <Card
            className="flex flex-col items-center justify-center py-16 text-center"
            data-testid="users-empty-state"
          >
            <UserCircle2 className="w-12 h-12 text-gray-300 dark:text-gray-600 mb-4" />
            <Paragraph variant="muted">No users found.</Paragraph>
          </Card>
        ) : (
          <div
            className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4"
            data-testid="users-grid"
          >
            {users.map((user) => (
              <UserCard key={user.id} user={user} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// UserCard
// ---------------------------------------------------------------------------

function UserCard({ user }: { user: UserBrief }) {
  const name = user.personal?.name?.trim() || "";
  const letters = initials(name);
  const jobTitle = user.personal?.job_title;

  return (
    <Link
      to={`/u/${user.id}`}
      className="group block"
      data-testid={`user-card-${user.id}`}
    >
      <div
        className={cn(
          "flex flex-col items-center gap-2 p-4",
          "rounded-(--radius-component-lg)",
          "border border-gray-200 dark:border-gray-800",
          "bg-white dark:bg-gray-900",
          "hover:border-primary-400 dark:hover:border-primary-600",
          "hover:shadow-md dark:hover:shadow-gray-800",
          "transition-all"
        )}
      >
        {/* Avatar */}
        {user.avatar_ulid ? (
          <img
            src={`/api/v1/static/user_avatars/${user.avatar_ulid}_thumb.webp`}
            alt={name || user.id}
            className="w-14 h-14 rounded-(--radius-component) object-cover shrink-0"
          />
        ) : letters ? (
          <div
            className={cn(
              "w-14 h-14 rounded-(--radius-component) flex items-center justify-center shrink-0",
              "text-white text-lg font-semibold select-none",
              avatarColor(user.id)
            )}
            aria-hidden="true"
          >
            {letters}
          </div>
        ) : (
          <div
            className="w-14 h-14 rounded-(--radius-component) flex items-center justify-center shrink-0 bg-gray-200 dark:bg-gray-700"
            aria-hidden="true"
          >
            <User2 className="w-6 h-6 text-gray-400 dark:text-gray-500" />
          </div>
        )}

        {/* Info */}
        <div className="w-full text-center min-w-0">
          <p
            className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate group-hover:text-primary-700 dark:group-hover:text-primary-300 transition-colors"
            data-testid={`user-name-${user.id}`}
          >
            {name || user.id}
          </p>
          {jobTitle && (
            <p className="text-xs text-gray-500 dark:text-gray-400 truncate mt-0.5">
              {jobTitle}
            </p>
          )}
          <code className="text-[10px] font-mono text-gray-400 dark:text-gray-600 truncate block mt-0.5">
            {user.id}
          </code>
        </div>
      </div>
    </Link>
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
            Error loading users
          </span>
        </div>
        <Paragraph variant="muted">
          {error instanceof Error
            ? error.message
            : "Something went wrong while loading users."}
        </Paragraph>
      </Card>
    </div>
  );
}
