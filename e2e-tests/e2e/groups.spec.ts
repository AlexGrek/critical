import { test, expect, request } from '@playwright/test';

test.describe('Groups Page', () => {
  const testUsername = `testuser_${Date.now()}_${Math.random().toString(36).substring(7)}`;
  const testPassword = 'TestPassword123!';
  let authCookie: string;

  // Test group IDs (unique per test run, already include g_ prefix)
  const groupIds = {
    developers: `g_developers_${Date.now()}`,
    designers: `g_designers_${Date.now()}`,
    managers: `g_managers_${Date.now()}`,
  };

  async function registerUser(apiContext: any) {
    await apiContext.post('http://localhost:3742/api/register', {
      data: { user: testUsername, password: testPassword },
    });
  }

  async function loginUser(apiContext: any) {
    const response = await apiContext.post('http://localhost:3742/api/login', {
      data: { user: testUsername, password: testPassword },
    });
    const cookies = response.headers()['set-cookie'];
    if (cookies) authCookie = cookies;
    return response;
  }

  async function createGroup(apiContext: any, id: string, name: string) {
    return await apiContext.post('http://localhost:3742/api/v1/global/groups', {
      headers: { Cookie: authCookie },
      data: { id, name },
      failOnStatusCode: false,
    });
  }

  async function deleteGroup(apiContext: any, id: string) {
    return await apiContext.delete(
      `http://localhost:3742/api/v1/global/groups/${id}`,
      {
        headers: { Cookie: authCookie },
        failOnStatusCode: false,
      }
    );
  }

  test.beforeAll(async () => {
    const apiContext = await request.newContext();
    await registerUser(apiContext);
    await loginUser(apiContext);
    await apiContext.dispose();
  });

  test.beforeEach(async ({ page, context }) => {
    const apiContext = await request.newContext();
    const loginResponse = await loginUser(apiContext);
    const cookies = loginResponse.headers()['set-cookie'];

    if (cookies) {
      const cookieParts = cookies.split(';')[0].split('=');
      await context.addCookies([
        {
          name: cookieParts[0],
          value: cookieParts[1],
          domain: 'localhost',
          path: '/',
        },
      ]);
    }
    await apiContext.dispose();
  });

  test.afterAll(async () => {
    const apiContext = await request.newContext();
    await loginUser(apiContext);
    for (const groupId of Object.values(groupIds)) {
      await deleteGroup(apiContext, groupId);
    }
    await apiContext.dispose();
  });

  // ---------------------------------------------------------------------------
  // Navigation
  // ---------------------------------------------------------------------------

  test('should navigate to groups page from home', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('button', { name: /view groups/i }).click();
    await expect(page).toHaveURL(/\/groups/);
  });

  test('should load the groups page directly', async ({ page }) => {
    await page.goto('/groups');
    await expect(page).toHaveURL(/\/groups/);
    await expect(page.getByTestId('groups-page-heading')).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Page structure
  // ---------------------------------------------------------------------------

  test('should display page header and description', async ({ page }) => {
    await page.goto('/groups');
    await expect(page.getByTestId('groups-page-heading')).toContainText('Groups');
    await expect(page.getByTestId('groups-description')).toContainText(
      /view and manage all groups in the system/i
    );
  });

  test('should have a Create Group button', async ({ page }) => {
    await page.goto('/groups');
    await expect(page.getByTestId('create-group-button')).toBeVisible();
  });

  test('should have a Back to Home link', async ({ page }) => {
    await page.goto('/groups');
    await expect(page.getByTestId('back-to-home-link')).toBeVisible();
  });

  test('should navigate back to home when clicking Back to Home', async ({ page }) => {
    await page.goto('/groups');
    await page.getByTestId('back-to-home-link').click();
    await expect(page).toHaveURL('/');
  });

  test('should display empty state or existing groups', async ({ page }) => {
    await page.goto('/groups');
    await expect(page.getByTestId('groups-page-heading')).toBeVisible();

    const emptyState = page.getByTestId('groups-empty-state');
    const grid = page.getByTestId('groups-grid');

    const isEmpty = await emptyState.isVisible();
    if (isEmpty) {
      await expect(emptyState).toContainText(/no groups found/i);
    } else {
      await expect(grid).toBeVisible();
    }
  });

  // ---------------------------------------------------------------------------
  // Create group modal
  // ---------------------------------------------------------------------------

  test.describe('create group modal', () => {
    test('should open modal when clicking Create Group button', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      await expect(page.getByTestId('create-group-modal-title')).toBeVisible();
      await expect(page.getByTestId('group-name-input')).toBeVisible();
      await expect(page.getByTestId('group-id-input')).toBeVisible();
    });

    test('should close modal when clicking Cancel', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      await expect(page.getByTestId('create-group-modal-title')).toBeVisible();
      await page.getByTestId('cancel-create-group').click();
      await expect(page.getByTestId('create-group-modal-title')).not.toBeVisible();
    });

    test('should show error when submitting empty name', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      // Leave fields empty and submit
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toBeVisible();
      await expect(page.getByTestId('create-group-error')).toContainText(
        /name.*required|required/i
      );
    });

    test('should show error when group ID starts with a digit', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('group-name-input').fill('My Group');
      await page.getByTestId('group-id-input').fill('1invalid');
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toBeVisible();
      await expect(page.getByTestId('create-group-error')).toContainText(
        /cannot start with a digit/i
      );
    });

    test('should show error when group ID contains invalid characters', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('group-name-input').fill('My Group');
      await page.getByTestId('group-id-input').fill('invalid-id!');
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toBeVisible();
      await expect(page.getByTestId('create-group-error')).toContainText(
        /lowercase letters, numbers, and underscores/i
      );
    });

    test('should clear error and reset form when modal is closed and reopened', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('submit-create-group').click();
      await expect(page.getByTestId('create-group-error')).toBeVisible();
      await page.getByTestId('cancel-create-group').click();
      // Reopen — error should be gone
      await page.getByTestId('create-group-button').click();
      await expect(page.getByTestId('create-group-error')).not.toBeVisible();
      await expect(page.getByTestId('group-name-input')).toHaveValue('');
      await expect(page.getByTestId('group-id-input')).toHaveValue('');
    });

    test('should force group ID to lowercase', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      await page.getByTestId('group-id-input').fill('MyGroup');
      await expect(page.getByTestId('group-id-input')).toHaveValue('mygroup');
    });

    test('should show the group-id hint text', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      await expect(page.getByTestId('group-id-hint')).toBeVisible();
    });
  });

  // ---------------------------------------------------------------------------
  // Group cards (with test groups created via API)
  // ---------------------------------------------------------------------------

  test.describe('with test groups', () => {
    test.beforeEach(async () => {
      const apiContext = await request.newContext();
      await loginUser(apiContext);
      await createGroup(apiContext, groupIds.developers, 'Test Developers');
      await createGroup(apiContext, groupIds.designers, 'Test Designers');
      await createGroup(apiContext, groupIds.managers, 'Test Managers');
      await apiContext.dispose();
    });

    test.afterEach(async () => {
      const apiContext = await request.newContext();
      await loginUser(apiContext);
      for (const groupId of Object.values(groupIds)) {
        await deleteGroup(apiContext, groupId);
      }
      await apiContext.dispose();
    });

    test('should display created groups in a grid', async ({ page }) => {
      await page.goto('/groups');
      await expect(page.getByTestId('groups-grid')).toBeVisible();
      await expect(
        page.locator(`[data-testid="group-card-${groupIds.developers}"]`)
      ).toBeVisible();
      await expect(
        page.locator(`[data-testid="group-card-${groupIds.designers}"]`)
      ).toBeVisible();
      await expect(
        page.locator(`[data-testid="group-card-${groupIds.managers}"]`)
      ).toBeVisible();
    });

    test('should display group name and ID in each card', async ({ page }) => {
      await page.goto('/groups');

      const devCard = page.locator(`[data-testid="group-card-${groupIds.developers}"]`);
      await expect(devCard.getByText('Test Developers')).toBeVisible();
      await expect(devCard.getByTestId(`group-id-label-${groupIds.developers}`)).toContainText(
        groupIds.developers
      );
    });

    test('should not show labels section when group has no labels', async ({ page }) => {
      await page.goto('/groups');
      const devCard = page.locator(`[data-testid="group-card-${groupIds.developers}"]`);
      await expect(devCard).toBeVisible();
      await expect(
        page.locator(`[data-testid="group-labels-${groupIds.developers}"]`)
      ).not.toBeVisible();
    });

    test('should have responsive grid layout', async ({ page }) => {
      await page.goto('/groups');
      const grid = page.getByTestId('groups-grid');
      await expect(grid).toHaveClass(/grid/);
    });

    test('should display all group cards with border and background', async ({ page }) => {
      await page.goto('/groups');
      for (const groupId of Object.values(groupIds)) {
        const card = page.locator(`[data-testid="group-card-${groupId}"]`);
        await expect(card).toBeVisible();
        // Cards have border class from the Card component
        await expect(card).toHaveClass(/border/);
      }
    });
  });

  // ---------------------------------------------------------------------------
  // Real-time updates
  // ---------------------------------------------------------------------------

  test.describe('real-time updates', () => {
    test('should reflect newly created groups after page refresh', async ({ page }) => {
      const newGroupId = `g_test_${Date.now()}`;

      await page.goto('/groups');
      const initialCount = await page.locator('[data-testid^="group-card-"]').count();

      const apiContext = await request.newContext();
      await loginUser(apiContext);
      await createGroup(apiContext, newGroupId, 'Newly Created Group');

      await page.reload();
      await expect(
        page.locator(`[data-testid="group-card-${newGroupId}"]`)
      ).toBeVisible();
      await expect(page.getByText('Newly Created Group')).toBeVisible();

      await deleteGroup(apiContext, newGroupId);
      await apiContext.dispose();
    });
  });

  // ---------------------------------------------------------------------------
  // Error handling
  // ---------------------------------------------------------------------------

  test.describe('error handling', () => {
    test('should handle unauthorized access gracefully', async ({ page, context }) => {
      await context.clearCookies();
      await page.goto('/groups');
      // Should either redirect or show error gracefully — not crash
      await expect(page.locator('body')).toBeVisible();
      if (page.url().includes('/groups')) {
        // Stayed on page — either shows an error boundary or empty content
        await expect(page.locator('body')).not.toContainText('TypeError');
      } else {
        expect(page.url()).toMatch(/sign-in|\//);
      }
    });
  });

  // ---------------------------------------------------------------------------
  // Accessibility
  // ---------------------------------------------------------------------------

  test.describe('accessibility', () => {
    test.beforeEach(async () => {
      const apiContext = await request.newContext();
      await loginUser(apiContext);
      await createGroup(apiContext, groupIds.developers, 'Test Developers');
      await apiContext.dispose();
    });

    test.afterEach(async () => {
      const apiContext = await request.newContext();
      await loginUser(apiContext);
      await deleteGroup(apiContext, groupIds.developers);
      await apiContext.dispose();
    });

    test('should have a single h1 heading containing "Groups"', async ({ page }) => {
      await page.goto('/groups');
      const h1 = page.getByRole('heading', { level: 1 });
      await expect(h1).toHaveCount(1);
      await expect(h1).toContainText('Groups');
    });

    test('should have a navigable Back to Home link', async ({ page }) => {
      await page.goto('/groups');
      const backLink = page.getByTestId('back-to-home-link');
      await expect(backLink).toBeVisible();
      // Should be an <a> tag (Link component)
      await expect(backLink).toHaveAttribute('href', '/');
    });

    test('should have accessible group cards with readable text', async ({ page }) => {
      await page.goto('/groups');
      const groupCard = page.locator(
        `[data-testid="group-card-${groupIds.developers}"]`
      );
      await expect(groupCard).toBeVisible();
      await expect(groupCard).toContainText('Test Developers');
      await expect(groupCard).toContainText(groupIds.developers);
    });

    test('create group form inputs should have associated labels', async ({ page }) => {
      await page.goto('/groups');
      await page.getByTestId('create-group-button').click();
      // htmlFor association — Playwright can find by label text
      await expect(page.getByLabel(/group name/i)).toBeVisible();
      await expect(page.getByLabel(/group id/i)).toBeVisible();
    });
  });
});
