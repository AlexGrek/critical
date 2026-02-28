import { test, expect } from '@playwright/test';
import { registerAndLogin } from './helpers/auth';
import { createGroup, deleteGroup } from './helpers/groups';

test(
  'Given an unauthenticated user, ' +
  'when they navigate to /groups, ' +
  'then the groups page content is not shown',
  async ({ page }) => {
    await page.goto('/groups');
    await page.waitForLoadState('domcontentloaded');
    await expect(page.getByTestId('groups-page-heading')).not.toBeVisible();
  }
);

test(
  'Given an authenticated user with no groups, ' +
  'when they create a group via the UI modal, ' +
  'then the group immediately appears in the grid without a page reload',
  async ({ page }) => {
    await registerAndLogin(page);
    await page.goto('/groups');

    // Empty state shown initially
    await expect(page.getByTestId('groups-empty-state')).toBeVisible();

    // Open modal and fill in the form
    const suffix = Math.random().toString(36).slice(2, 8);
    const bareId = `e2e_${suffix}`;
    const fullId = `g_${bareId}`;
    const name = `E2E Group ${suffix}`;

    await page.getByTestId('create-group-button').click();
    await page.getByTestId('group-name-input').fill(name);
    await page.getByTestId('group-id-input').fill(bareId);
    await page.getByTestId('submit-create-group').click();

    // Modal closes and row appears — no reload needed
    await expect(page.getByTestId('create-group-modal-title')).not.toBeVisible();
    await expect(page.getByTestId(`group-row-${fullId}`)).toBeVisible();
    await expect(page.getByTestId(`group-id-label-${fullId}`)).toContainText(fullId);

    await deleteGroup(page, fullId);
  }
);

test(
  'Given an authenticated user, ' +
  'when groups exist in the system, ' +
  'then each group is listed with its name and full ID',
  async ({ page }) => {
    await registerAndLogin(page);

    const suffix = Math.random().toString(36).slice(2, 8);
    const groups = [
      { bareId: `e2e_alpha_${suffix}`, name: 'Alpha Team' },
      { bareId: `e2e_beta_${suffix}`,  name: 'Beta Team' },
    ];

    const fullIds: string[] = [];
    for (const { bareId, name } of groups) {
      fullIds.push(await createGroup(page, bareId, name));
    }

    await page.goto('/groups');

    await expect(page.getByTestId('groups-table-card')).toBeVisible();
    for (let i = 0; i < groups.length; i++) {
      const row = page.getByTestId(`group-row-${fullIds[i]}`);
      await expect(row).toBeVisible();
      await expect(row.getByText(groups[i].name)).toBeVisible();
      await expect(row.getByTestId(`group-id-label-${fullIds[i]}`)).toContainText(fullIds[i]);
    }

    for (const id of fullIds) await deleteGroup(page, id);
  }
);
