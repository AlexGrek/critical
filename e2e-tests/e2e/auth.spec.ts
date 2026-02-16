import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should navigate to sign-in page', async ({ page }) => {
    await page.goto('/sign-in');
    await expect(page).toHaveURL(/\/sign-in/);
    await expect(page.locator('h1, h2')).toContainText(/sign in/i);
  });

  test('should navigate to sign-up page', async ({ page }) => {
    await page.goto('/sign-up');
    await expect(page).toHaveURL(/\/sign-up/);
    await expect(page.locator('h1')).toContainText(/create account/i);
  });

  test('should display sign-in form elements', async ({ page }) => {
    await page.goto('/sign-in');
    await expect(page.locator('input[type="text"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('should display sign-up form elements', async ({ page }) => {
    await page.goto('/sign-up');
    await expect(page.locator('input[type="text"]')).toBeVisible();
    const passwordInputs = page.locator('input[type="password"]');
    await expect(passwordInputs).toHaveCount(2);
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('should have navigation links', async ({ page }) => {
    await page.goto('/sign-in');
    await expect(page.getByRole('link', { name: /sign up/i })).toBeVisible();
  });
});
