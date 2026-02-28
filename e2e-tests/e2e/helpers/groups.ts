import type { Page } from '@playwright/test';

/**
 * Creates a group via the API. The page must already be authenticated.
 * Returns the full group ID (e.g. "g_my_group").
 */
export async function createGroup(page: Page, bareId: string, name: string): Promise<string> {
  const res = await page.request.post('/api/v1/global/groups', {
    data: { id: bareId, name },
  });
  if (!res.ok()) throw new Error(`createGroup failed: ${await res.text()}`);
  return `g_${bareId}`;
}

/**
 * Soft-deletes a group via the API. The page must already be authenticated.
 * Silently ignores failures (cleanup best-effort).
 */
export async function deleteGroup(page: Page, fullId: string): Promise<void> {
  await page.request.delete(`/api/v1/global/groups/${fullId}`);
}
