import { test, expect } from '@playwright/test';
import { generateUsername, registerAndLogin, loginAs, loginAsRoot, TEST_PASSWORD } from './helpers/auth';
import { deleteGroup } from './helpers/groups';

test(
  'Given a freshly registered user with no ACL grants, ' +
  'when they open Settings → Access, ' +
  'then they see their own principal, their auto-granted super-permission, and no group/project grants',
  async ({ page }) => {
    const username = await registerAndLogin(page);
    const userId = `u_${username}`;

    await page.goto('/settings');
    await page.getByTestId('tab-access').click();

    const accessTab = page.getByTestId('access-tab');
    await expect(accessTab).toBeVisible();

    // No godmode — banner absent, and only the default registration permission is listed.
    await expect(page.getByTestId('access-godmode-banner')).not.toBeVisible();
    await expect(page.getByTestId('access-super-permission-usr_create_groups')).toBeVisible();

    // Own ID shows up among the effective principals.
    await expect(page.getByTestId('access-all-principals')).toContainText(userId);

    // No direct memberships yet.
    await expect(page.getByTestId('access-direct-memberships')).not.toBeVisible();

    // No group or project grants yet — empty states shown instead of tables.
    await expect(page.getByTestId('access-groups')).toContainText(
      "No group grants you direct access via its ACL."
    );
    await expect(page.getByTestId('access-projects')).toContainText(
      "No project grants you direct access via its ACL."
    );
  }
);

test(
  'Given a user granted READ access to a group via its ACL, ' +
  'when they open Settings → Access, ' +
  'then the group is listed with a Read badge attributed to their own principal',
  async ({ page }) => {
    const readerUsername = generateUsername();
    await page.request.post('/api/v1/register', {
      data: { user: readerUsername, password: TEST_PASSWORD },
    });
    const readerId = `u_${readerUsername}`;

    await loginAsRoot(page);
    const suffix = Math.random().toString(36).slice(2, 8);
    const bareId = `e2e_access_${suffix}`;
    const fullId = `g_${bareId}`;
    const READ_BITS = 1 | 2 | 4; // FETCH | LIST | NOTIFY

    const createRes = await page.request.post('/api/v1/global/groups', {
      data: {
        id: bareId,
        name: `Access Tab Test ${suffix}`,
        acl: {
          list: [{ permissions: READ_BITS, principals: [readerId] }],
          last_mod_date: new Date().toISOString(),
        },
      },
    });
    expect(createRes.ok()).toBeTruthy();

    await loginAs(page, readerUsername);
    await page.goto('/settings');
    await page.getByTestId('tab-access').click();

    const row = page.getByTestId(`access-groups-row-${fullId}`);
    await expect(row).toBeVisible();
    await expect(row).toContainText('Read');
    await expect(row).toContainText(readerId);

    await loginAsRoot(page);
    await deleteGroup(page, fullId);
  }
);

test(
  'Given the root (godmode) user, ' +
  'when they open Settings → Access, ' +
  'then the godmode banner is shown and adm_godmode is listed among their super-permissions',
  async ({ page }) => {
    await loginAsRoot(page);
    await page.goto('/settings');
    await page.getByTestId('tab-access').click();

    await expect(page.getByTestId('access-godmode-banner')).toBeVisible();
    await expect(page.getByTestId('access-super-permission-adm_godmode')).toBeVisible();
  }
);
