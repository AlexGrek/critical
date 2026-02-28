import { test, expect } from '@playwright/test';
import { registerAndLogin } from './helpers/auth';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Creates a group via the API. page must already be authenticated. */
async function createGroup(page: Parameters<typeof registerAndLogin>[0], bareId: string, name: string) {
  const res = await page.request.post('/api/v1/global/groups', {
    data: { id: bareId, name },
  });
  if (!res.ok()) throw new Error(`createGroup failed: ${await res.text()}`);
}

/** Deletes a group via the API. page must already be authenticated. */
async function deleteGroup(page: Parameters<typeof registerAndLogin>[0], fullId: string) {
  await page.request.delete(`/api/v1/global/groups/${fullId}`);
}

// ---------------------------------------------------------------------------
// Unauthenticated access
// ---------------------------------------------------------------------------

test.describe('Groups Page — unauthenticated', () => {
  test('should not show groups content without auth', async ({ page }) => {
    await page.goto('/groups');
    await page.waitForLoadState('domcontentloaded');
    // Loader throws 401 → React Router renders the error boundary, not the page
    await expect(page.getByTestId('groups-page-heading')).not.toBeVisible();
  });

  test('should handle unauthorised access gracefully (no crash)', async ({ page }) => {
    await page.goto('/groups');
    await expect(page.locator('body')).toBeVisible();
    await expect(page.locator('body')).not.toContainText('TypeError');
  });
});

// ---------------------------------------------------------------------------
// Authenticated access — one unique user per test (safe for parallel runs)
// ---------------------------------------------------------------------------

test.describe('Groups Page — authenticated', () => {
  // Every test in this block gets a fresh unique user + navigates to /groups.
  // page.request shares cookie storage with the browser, so the auth cookie
  // set by registerAndLogin is picked up automatically by page.goto().
  test.beforeEach(async ({ page }) => {
    await registerAndLogin(page);
    await page.goto('/groups');
  });

  // ---------------------------------------------------------------------------
  // Page structure
  // ---------------------------------------------------------------------------

  test('should display page heading and description', async ({ page }) => {
    await expect(page.getByTestId('groups-page-heading')).toContainText('Groups');
    await expect(page.getByTestId('groups-description')).toContainText(
      /view and manage all groups in the system/i
    );
  });

  test('should have a Create Group button and a Back to Home link', async ({ page }) => {
    await expect(page.getByTestId('create-group-button')).toBeVisible();
    await expect(page.getByTestId('back-to-home-link')).toBeVisible();
  });

  test('should navigate back to home', async ({ page }) => {
    await page.getByTestId('back-to-home-link').click();
    await expect(page).toHaveURL('/');
  });

  test('should have a single h1 heading', async ({ page }) => {
    const h1 = page.getByRole('heading', { level: 1 });
    await expect(h1).toHaveCount(1);
    await expect(h1).toContainText('Groups');
  });

  test('Back to Home link should point to /', async ({ page }) => {
    await expect(page.getByTestId('back-to-home-link')).toHaveAttribute('href', '/');
  });

  // ---------------------------------------------------------------------------
  // Empty state (new user has no groups)
  // ---------------------------------------------------------------------------

  test('should show empty state and no grid for a new user', async ({ page }) => {
    await expect(page.getByTestId('groups-empty-state')).toBeVisible();
    await expect(page.getByTestId('groups-empty-state')).toContainText(/no groups found/i);
    await expect(page.getByTestId('groups-grid')).not.toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Create group modal
  // ---------------------------------------------------------------------------

  test.describe('create group modal', () => {
    test('should open modal when clicking Create Group', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await expect(page.getByTestId('create-group-modal-title')).toBeVisible();
      await expect(page.getByTestId('group-name-input')).toBeVisible();
      await expect(page.getByTestId('group-id-input')).toBeVisible();
      await expect(page.getByTestId('submit-create-group')).toBeVisible();
      await expect(page.getByTestId('cancel-create-group')).toBeVisible();
    });

    test('should show the group ID hint text', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await expect(page.getByTestId('group-id-hint')).toBeVisible();
    });

    test('form inputs should have associated labels', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await expect(page.getByLabel(/group name/i)).toBeVisible();
      await expect(page.getByLabel(/group id/i)).toBeVisible();
    });

    test('should close modal on Cancel', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await expect(page.getByTestId('create-group-modal-title')).toBeVisible();
      await page.getByTestId('cancel-create-group').click();
      await expect(page.getByTestId('create-group-modal-title')).not.toBeVisible();
    });

    test('should clear error and reset form when modal is closed and reopened', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toBeVisible();
      await page.getByTestId('cancel-create-group').click();
      await page.getByTestId('create-group-button').click();
      await expect(page.getByTestId('create-group-error')).not.toBeVisible();
      await expect(page.getByTestId('group-name-input')).toHaveValue('');
      await expect(page.getByTestId('group-id-input')).toHaveValue('');
    });

    test('should force group ID input to lowercase', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('group-id-input').fill('MyGroup');
      await expect(page.getByTestId('group-id-input')).toHaveValue('mygroup');
    });

    test('should show error when submitting with empty fields', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toContainText(/required/i);
    });

    test('should reject group ID starting with a digit', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('group-name-input').fill('My Group');
      await page.getByTestId('group-id-input').fill('1invalid');
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toContainText(
        /cannot start with a digit/i
      );
      // Modal stays open
      await expect(page.getByTestId('create-group-modal-title')).toBeVisible();
    });

    test('should reject group ID with invalid characters', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('group-name-input').fill('My Group');
      await page.getByTestId('group-id-input').fill('invalid-id!');
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toContainText(
        /lowercase letters, numbers, and underscores/i
      );
    });

    test('should reject group ID that is too short', async ({ page }) => {
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('group-name-input').fill('My Group');
      await page.getByTestId('group-id-input').fill('x');
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toContainText(/at least 2 characters/i);
    });

    test('should create a group and display it in the grid', async ({ page }) => {
      const suffix = Math.random().toString(36).slice(2, 8);
      const bareId = `testgrp_${suffix}`;
      const fullId = `g_${bareId}`;
      const groupName = `Test Group ${suffix}`;

      await page.getByTestId('create-group-button').click();
      await page.getByTestId('group-name-input').fill(groupName);
      await page.getByTestId('group-id-input').fill(bareId);
      await page.getByTestId('submit-create-group').click();

      // Modal closes after success
      await expect(page.getByTestId('create-group-modal-title')).not.toBeVisible();

      // Group card appears without page reload (revalidator fires)
      await expect(page.getByTestId(`group-card-${fullId}`)).toBeVisible();
      await expect(page.getByTestId(`group-id-label-${fullId}`)).toContainText(fullId);

      // Cleanup
      await deleteGroup(page, fullId);
    });
  });

  // ---------------------------------------------------------------------------
  // Group cards (pre-created via API before each test)
  // ---------------------------------------------------------------------------

  test.describe('with pre-created groups', () => {
    // Group IDs unique per-test to support parallel execution.
    // We declare them at describe scope and populate in beforeEach so each test
    // gets its own set.
    let groupIds: { dev: string; des: string; mgr: string };

    test.beforeEach(async ({ page }) => {
      const suffix = Math.random().toString(36).slice(2, 8);
      groupIds = {
        dev: `g_test_dev_${suffix}`,
        des: `g_test_des_${suffix}`,
        mgr: `g_test_mgr_${suffix}`,
      };
      await createGroup(page, `test_dev_${suffix}`, 'Test Developers');
      await createGroup(page, `test_des_${suffix}`, 'Test Designers');
      await createGroup(page, `test_mgr_${suffix}`, 'Test Managers');
      await page.reload();
    });

    test.afterEach(async ({ page }) => {
      for (const id of Object.values(groupIds)) {
        await deleteGroup(page, id);
      }
    });

    test('should display created groups in a grid', async ({ page }) => {
      await expect(page.getByTestId('groups-grid')).toBeVisible();
      for (const id of Object.values(groupIds)) {
        await expect(page.getByTestId(`group-card-${id}`)).toBeVisible();
      }
    });

    test('should display group name and ID in each card', async ({ page }) => {
      const devCard = page.getByTestId(`group-card-${groupIds.dev}`);
      await expect(devCard.getByText('Test Developers')).toBeVisible();
      await expect(devCard.getByTestId(`group-id-label-${groupIds.dev}`)).toContainText(
        groupIds.dev
      );
    });

    test('should not show labels section when group has no labels', async ({ page }) => {
      await expect(page.getByTestId(`group-card-${groupIds.dev}`)).toBeVisible();
      await expect(
        page.getByTestId(`group-labels-${groupIds.dev}`)
      ).not.toBeVisible();
    });

    test('should display all group cards with a border', async ({ page }) => {
      for (const id of Object.values(groupIds)) {
        await expect(page.getByTestId(`group-card-${id}`)).toHaveClass(/border/);
      }
    });

    test('grid should have responsive CSS classes', async ({ page }) => {
      await expect(page.getByTestId('groups-grid')).toHaveClass(/grid/);
    });

    test('should reflect newly added groups after reload', async ({ page }) => {
      const suffix2 = Math.random().toString(36).slice(2, 8);
      const bareId = `test_new_${suffix2}`;
      const fullId = `g_${bareId}`;

      await createGroup(page, bareId, 'Newly Created Group');
      await page.reload();

      await expect(page.getByTestId(`group-card-${fullId}`)).toBeVisible();
      await expect(page.getByText('Newly Created Group')).toBeVisible();

      // Cleanup the extra group (the three from beforeEach are cleaned by afterEach)
      await deleteGroup(page, fullId);
    });
  });
});
