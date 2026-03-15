import { useState, useEffect, useRef } from "react";
import { resolvePrincipalsClient } from "./principals";
import type { PrincipalMap } from "./principals";

/**
 * Resolves principal IDs to display info (name, avatar) client-side.
 * Accumulates results across incremental loads — already-fetched IDs are not re-fetched.
 */
export function usePrincipals(ids: string[]): PrincipalMap {
  const [resolved, setResolved] = useState<PrincipalMap>({});
  const resolvedRef = useRef<PrincipalMap>({});

  // Stable key: only re-run when the set of IDs actually changes
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const key = ids.join("\0");

  useEffect(() => {
    const toFetch = ids.filter((id) => id && !(id in resolvedRef.current));
    if (toFetch.length === 0) return;
    resolvePrincipalsClient(toFetch).then((result) => {
      resolvedRef.current = { ...resolvedRef.current, ...result };
      setResolved({ ...resolvedRef.current });
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);

  return resolved;
}
