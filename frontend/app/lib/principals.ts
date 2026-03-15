export interface PrincipalInfo {
  type: "user" | "group" | "service_account" | "pipeline_account";
  name: string;
  avatar_ulid?: string;
  error?: string;
}

export type PrincipalMap = Record<string, PrincipalInfo>;

/** Server-side: call from a loader, forwarding the request Cookie header. */
export async function resolvePrincipals(
  ids: string[],
  cookieHeader: string = ""
): Promise<PrincipalMap> {
  const unique = [...new Set(ids)].filter(Boolean);
  if (unique.length === 0) return {};
  try {
    const res = await fetch("http://localhost:3742/api/v1/principals/resolve", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Cookie: cookieHeader,
      },
      body: JSON.stringify({ ids: unique }),
    });
    if (!res.ok) return {};
    return res.json();
  } catch {
    return {};
  }
}

/** Client-side: uses credentials cookie automatically. */
export async function resolvePrincipalsClient(
  ids: string[]
): Promise<PrincipalMap> {
  const unique = [...new Set(ids)].filter(Boolean);
  if (unique.length === 0) return {};
  try {
    const res = await fetch("/api/v1/principals/resolve", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ ids: unique }),
    });
    if (!res.ok) return {};
    return res.json();
  } catch {
    return {};
  }
}

/** Returns the avatar thumbnail URL, or null if no avatar available. */
export function principalAvatarUrl(
  type: string,
  avatarUlid: string | undefined
): string | null {
  if (!avatarUlid) return null;
  const dir =
    type === "user"
      ? "user_avatars"
      : type === "group"
      ? "group_avatars"
      : null;
  if (!dir) return null;
  return `/api/v1/static/${dir}/${avatarUlid}_thumb.webp`;
}

/** Returns up to 2-character initials for a display name. */
export function principalInitials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase();
  return name.slice(0, 2).toUpperCase();
}
