import { test, expect } from '@playwright/test';

test(
  'Given any visitor, ' +
  'when they load the home page, ' +
  'then the Critical heading and navigation links to all app sections are visible',
  async ({ page }) => {
    await page.goto('/');

    await expect(page.getByRole('heading', { name: 'Critical' })).toBeVisible();

    for (const route of ['sign-in', 'sign-up', 'groups', 'ui-gallery']) {
      await expect(page.getByTestId(`nav-link-${route}`)).toBeVisible();
    }
  }
);
