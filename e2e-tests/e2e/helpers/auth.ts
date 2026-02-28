import type { Page } from '@playwright/test';

export const TEST_PASSWORD = 'TestPassword123!';

/** Generates a unique username safe for use in tests. */
export function generateUsername(): string {
  return `testuser_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`;
}

/**
 * Registers a new user and logs in via the API.
 * The resulting auth cookie is stored in the page's browser context (page.request
 * shares cookie storage with the browser), so subsequent page.goto() calls will
 * be authenticated.
 *
 * Returns the username used (useful when auto-generated).
 */
export async function registerAndLogin(
  page: Page,
  username: string = generateUsername(),
  password: string = TEST_PASSWORD
): Promise<string> {
  const registerRes = await page.request.post('/api/v1/register', {
    data: { user: username, password },
  });
  if (!registerRes.ok()) {
    throw new Error(`Registration failed for "${username}": ${await registerRes.text()}`);
  }

  const loginRes = await page.request.post('/api/v1/login', {
    data: { user: username, password },
  });
  if (!loginRes.ok()) {
    throw new Error(`Login failed for "${username}": ${await loginRes.text()}`);
  }

  return username;
}

/**
 * Logs in as an already-registered user. Auth cookie is stored in the page's
 * browser context, so subsequent page.goto() calls will be authenticated.
 */
export async function loginAs(
  page: Page,
  username: string,
  password: string = TEST_PASSWORD
): Promise<void> {
  const res = await page.request.post('/api/v1/login', {
    data: { user: username, password },
  });
  if (!res.ok()) {
    throw new Error(`Login failed for "${username}": ${await res.text()}`);
  }
}

/**
 * Logs in as the built-in root user (credentials: root / changeme).
 * Root has all super-permissions (ADM_USER_MANAGER, etc.) and is useful for
 * tests that require elevated privileges, e.g. creating users or groups on
 * behalf of others.
 */
export async function loginAsRoot(page: Page): Promise<void> {
  await loginAs(page, 'root', 'changeme');
}
