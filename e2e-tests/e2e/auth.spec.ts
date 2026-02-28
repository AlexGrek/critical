import { test, expect } from '@playwright/test';
import { generateUsername, TEST_PASSWORD } from './helpers/auth';

test.describe('Authentication', () => {
  test(
    'Given a new visitor fills the sign-up form with valid credentials, ' +
    'when they submit, ' +
    'then they are auto-logged-in and land on the home page',
    async ({ page }) => {
      const username = generateUsername();

      await page.goto('/sign-up');
      await page.getByTestId('sign-up-username').fill(username);
      await page.getByTestId('sign-up-password').fill(TEST_PASSWORD);
      await page.getByTestId('sign-up-confirm-password').fill(TEST_PASSWORD);
      await page.getByTestId('sign-up-submit').click();

      await expect(page).toHaveURL('/');
      await expect(page.getByRole('heading', { name: 'Critical' })).toBeVisible();
    }
  );

  test(
    'Given a user enters wrong credentials on the sign-in form, ' +
    'when they submit, ' +
    'then an error message is displayed and they remain on the sign-in page',
    async ({ page }) => {
      await page.goto('/sign-in');
      await page.getByTestId('sign-in-username').fill('no_such_user_xyz');
      await page.getByTestId('sign-in-password').fill('WrongPassword!');
      await page.getByTestId('sign-in-submit').click();

      await expect(page).toHaveURL(/\/sign-in/);
      await expect(page.getByTestId('sign-in-error')).toBeVisible();
    }
  );
});
