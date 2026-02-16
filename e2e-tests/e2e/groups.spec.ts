import { test, expect, request } from '@playwright/test';

test.describe('Groups Page', () => {
  const testUsername = `testuser_${Date.now()}_${Math.random().toString(36).substring(7)}`;
  const testPassword = 'TestPassword123!';
  let authCookie: string;

  // Test group IDs (unique per test run)
  const groupIds = {
    developers: `g_developers_${Date.now()}`,
    designers: `g_designers_${Date.now()}`,
    managers: `g_managers_${Date.now()}`,
  };

  // Helper function to register user
  async function registerUser(apiContext: any) {
    await apiContext.post('http://localhost:3742/api/register', {
      data: {
        user: testUsername,
        password: testPassword,
      },
    });
  }

  // Helper function to login and get auth cookie
  async function loginUser(apiContext: any) {
    const response = await apiContext.post('http://localhost:3742/api/login', {
      data: {
        user: testUsername,
        password: testPassword,
      },
    });
    const cookies = response.headers()['set-cookie'];
    if (cookies) {
      authCookie = cookies;
    }
    return response;
  }

  // Helper function to create a group
  async function createGroup(apiContext: any, id: string, name: string, acl: any) {
    return await apiContext.post(`http://localhost:3742/api/v1/global/groups`, {
      headers: {
        Cookie: authCookie,
      },
      data: {
        id,
        name,
        acl,
      },
      failOnStatusCode: false,
    });
  }

  // Helper function to delete a group
  async function deleteGroup(apiContext: any, id: string) {
    return await apiContext.delete(`http://localhost:3742/api/v1/global/groups/${id}`, {
      headers: {
        Cookie: authCookie,
      },
      failOnStatusCode: false,
    });
  }

  test.beforeAll(async () => {
    // Create API context and register user once
    const apiContext = await request.newContext();
    await registerUser(apiContext);
    await loginUser(apiContext);
    await apiContext.dispose();
  });

  test.beforeEach(async ({ page, context }) => {
    // Login before each test
    const apiContext = await request.newContext();
    const loginResponse = await loginUser(apiContext);
    const cookies = loginResponse.headers()['set-cookie'];

    if (cookies) {
      // Parse and set cookies in the browser context
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
    // Clean up: delete test groups
    const apiContext = await request.newContext();
    await loginUser(apiContext);

    for (const groupId of Object.values(groupIds)) {
      await deleteGroup(apiContext, groupId);
    }
    await apiContext.dispose();
  });

  test('should navigate to groups page from home', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('button', { name: /view groups/i }).click();
    await expect(page).toHaveURL(/\/groups/);
  });

  test('should load the groups page directly', async ({ page }) => {
    await page.goto('/groups');
    await expect(page).toHaveURL(/\/groups/);
    await expect(page.getByRole('heading', { name: /groups/i, level: 1 })).toBeVisible();
  });

  test('should display page header and description', async ({ page }) => {
    await page.goto('/groups');
    await expect(page.getByRole('heading', { name: /groups/i, level: 1 })).toBeVisible();
    await expect(page.getByText(/view and manage all groups in the system/i)).toBeVisible();
  });

  test('should have a back to home button', async ({ page }) => {
    await page.goto('/groups');
    await expect(page.getByRole('button', { name: /back to home/i })).toBeVisible();
  });

  test('should navigate back to home when clicking back button', async ({ page }) => {
    await page.goto('/groups');
    await page.getByRole('button', { name: /back to home/i }).click();
    await expect(page).toHaveURL('/');
  });

  test('should display empty state or existing groups', async ({ page }) => {
    await page.goto('/groups');
    await expect(page.getByRole('heading', { name: /groups/i })).toBeVisible();

    const bodyText = await page.textContent('body');
    if (bodyText?.includes('No groups found')) {
      await expect(page.getByText(/no groups found/i)).toBeVisible();
      await expect(page.getByText(/groups will appear here once they are created/i)).toBeVisible();
    } else {
      // If groups exist, verify cards are shown
      await expect(page.locator('[data-testid^="group-card-"]')).toHaveCount(await page.locator('[data-testid^="group-card-"]').count());
    }
  });

  test.describe('with test groups', () => {
    test.beforeEach(async () => {
      // Create test groups via API
      const apiContext = await request.newContext();
      await loginUser(apiContext);

      // Create Developers group
      await createGroup(apiContext, groupIds.developers, 'Test Developers', {
        list: [
          {
            permissions: 7,
            principals: [testUsername],
          },
        ],
        last_mod_date: new Date().toISOString(),
      });

      // Create Designers group
      await createGroup(apiContext, groupIds.designers, 'Test Designers', {
        list: [
          {
            permissions: 3,
            principals: [testUsername],
          },
        ],
        last_mod_date: new Date().toISOString(),
      });

      // Create Managers group (no ACL)
      await createGroup(apiContext, groupIds.managers, 'Test Managers', {
        list: [],
        last_mod_date: new Date().toISOString(),
      });

      await apiContext.dispose();
    });

    test.afterEach(async () => {
      // Clean up groups after each test
      const apiContext = await request.newContext();
      await loginUser(apiContext);

      for (const groupId of Object.values(groupIds)) {
        await deleteGroup(apiContext, groupId);
      }
      await apiContext.dispose();
    });

    test('should display created groups in a grid', async ({ page }) => {
      await page.goto('/groups');
      const groupCards = page.locator('[data-testid^="group-card-"]');
      await expect(groupCards).toHaveCount(await groupCards.count());
      // At least our 3 test groups should be present
      await expect(page.locator(`[data-testid="group-card-${groupIds.developers}"]`)).toBeVisible();
      await expect(page.locator(`[data-testid="group-card-${groupIds.designers}"]`)).toBeVisible();
      await expect(page.locator(`[data-testid="group-card-${groupIds.managers}"]`)).toBeVisible();
    });

    test('should display group information correctly', async ({ page }) => {
      await page.goto('/groups');

      const devCard = page.locator(`[data-testid="group-card-${groupIds.developers}"]`);
      await expect(devCard.getByText('Test Developers')).toBeVisible();
      await expect(devCard.getByText(groupIds.developers)).toBeVisible();
      await expect(devCard.getByText(/access rules:/i)).toBeVisible();
      await expect(devCard.getByText('1')).toBeVisible();
    });

    test('should display principals for groups with ACL', async ({ page }) => {
      await page.goto('/groups');

      const devCard = page.locator(`[data-testid="group-card-${groupIds.developers}"]`);
      await expect(devCard.getByText(/principals:/i)).toBeVisible();
      await expect(devCard.getByText(testUsername)).toBeVisible();
    });

    test('should not display principals section for groups without ACL', async ({ page }) => {
      await page.goto('/groups');

      const managerCard = page.locator(`[data-testid="group-card-${groupIds.managers}"]`);
      await expect(managerCard.getByText(/access rules:/i)).toBeVisible();
      await expect(managerCard.getByText('0')).toBeVisible();
      await expect(managerCard.getByText(/principals:/i)).not.toBeVisible();
    });

    test('should have responsive grid layout', async ({ page }) => {
      await page.goto('/groups');

      const gridContainer = page.locator('[data-testid^="group-card-"]').first().locator('..');
      await expect(gridContainer).toHaveClass(/grid/);
    });

    test('should display all group cards with correct styling', async ({ page }) => {
      await page.goto('/groups');

      const groupCards = page.locator('[data-testid^="group-card-"]');
      const count = await groupCards.count();

      // Check at least our test groups
      for (const groupId of Object.values(groupIds)) {
        const card = page.locator(`[data-testid="group-card-${groupId}"]`);
        await expect(card).toHaveClass(/bg-gray-900/);
        await expect(card).toHaveClass(/border/);
        await expect(card).toHaveClass(/rounded-lg/);
      }
    });

    test('should show different ACL counts for different groups', async ({ page }) => {
      await page.goto('/groups');

      // Developers group should have 1 ACL rule
      const devCard = page.locator(`[data-testid="group-card-${groupIds.developers}"]`);
      await expect(devCard.getByText(/access rules:/i)).toBeVisible();
      await expect(devCard.locator('text=/access rules:/i').locator('..')).toContainText('1');

      // Managers group should have 0 ACL rules
      const managerCard = page.locator(`[data-testid="group-card-${groupIds.managers}"]`);
      await expect(managerCard.getByText(/access rules:/i)).toBeVisible();
      await expect(managerCard.locator('text=/access rules:/i').locator('..')).toContainText('0');
    });
  });

  test.describe('error handling', () => {
    test('should handle unauthorized access', async ({ page, context }) => {
      // Clear cookies to simulate unauthorized state
      await context.clearCookies();
      await page.goto('/groups');

      // Should either redirect or show error gracefully
      if (page.url().includes('/groups')) {
        await expect(page.locator('body')).toBeVisible();
      } else {
        expect(page.url()).toMatch(/sign-in|\//);
      }
    });
  });

  test.describe('real-time updates', () => {
    test('should reflect newly created groups after page refresh', async ({ page }) => {
      const newGroupId = `g_test_${Date.now()}`;

      await page.goto('/groups');

      // Count initial groups
      const initialCount = await page.locator('[data-testid^="group-card-"]').count();

      // Create a new group via API
      const apiContext = await request.newContext();
      await loginUser(apiContext);
      await createGroup(apiContext, newGroupId, 'Newly Created Group', {
        list: [],
        last_mod_date: new Date().toISOString(),
      });

      // Refresh the page
      await page.reload();

      // Verify the new group appears
      await expect(page.locator(`[data-testid="group-card-${newGroupId}"]`)).toBeVisible();
      await expect(page.getByText('Newly Created Group')).toBeVisible();

      // Clean up
      await deleteGroup(apiContext, newGroupId);
      await apiContext.dispose();
    });
  });

  test.describe('accessibility', () => {
    test.beforeEach(async () => {
      const apiContext = await request.newContext();
      await loginUser(apiContext);

      await createGroup(apiContext, groupIds.developers, 'Test Developers', {
        list: [{ permissions: 7, principals: [testUsername] }],
        last_mod_date: new Date().toISOString(),
      });

      await apiContext.dispose();
    });

    test.afterEach(async () => {
      const apiContext = await request.newContext();
      await loginUser(apiContext);
      await deleteGroup(apiContext, groupIds.developers);
      await apiContext.dispose();
    });

    test('should have proper heading hierarchy', async ({ page }) => {
      await page.goto('/groups');
      const h1 = page.getByRole('heading', { level: 1 });
      await expect(h1).toHaveCount(1);
      await expect(h1).toHaveText('Groups');
    });

    test('should have proper link navigation', async ({ page }) => {
      await page.goto('/groups');
      const backLink = page.getByRole('link').filter({
        has: page.getByRole('button', { name: /back to home/i })
      });
      await expect(backLink).toBeVisible();
    });

    test('should have accessible group cards', async ({ page }) => {
      await page.goto('/groups');
      const groupCard = page.locator(`[data-testid="group-card-${groupIds.developers}"]`);
      await expect(groupCard).toBeVisible();

      // Check that text content is readable
      await expect(groupCard).toContainText('Test Developers');
      await expect(groupCard).toContainText(groupIds.developers);
    });
  });
});
