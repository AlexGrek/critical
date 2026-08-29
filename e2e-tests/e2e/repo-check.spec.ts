import { test, expect } from '@playwright/test';
import { loginAsRoot } from './helpers/auth';

test(
  'Given a project, ' +
  'when the user adds a repository with SSH auth backed by a newly-created credential ' +
  'and clicks Check, then a result renders; ' +
  'and when they edit the saved repository, the change persists',
  async ({ page }) => {
    await loginAsRoot(page);

    const suffix = Math.random().toString(36).slice(2, 8);
    const projectId = `e2e_repocheck_${suffix}`;
    const createRes = await page.request.post('/api/v1/global/projects', {
      data: { id: projectId, name: `Repo Check E2E ${suffix}` },
    });
    expect(createRes.ok()).toBeTruthy();

    await page.goto(`/p/${projectId}`);
    await expect(page.getByTestId('repositories-card')).toBeVisible();

    // Open the add-repository form.
    await page.getByTestId('add-repo-button').click();
    await expect(page.getByTestId('add-repo-form')).toBeVisible();

    await page.getByTestId('repo-url-input').fill('git@github.com:octocat/Hello-World.git');
    await page.getByTestId('repo-provider-select').selectOption('git');
    await page.getByTestId('repo-auth-method-select').selectOption('ssh');

    // No credential yet — the picker is shown; open the "New credential" modal instead.
    await expect(page.getByTestId('repo-credential-picker')).toBeVisible();
    await page.getByTestId('new-repo-credential-button').click();

    const credId = `e2e_cred_${suffix}`;
    await expect(page.getByTestId('new-credential-form')).toBeVisible();
    await page.getByTestId('new-credential-name').fill(`E2E credential ${suffix}`);
    await page.getByTestId('new-credential-id').fill(credId);
    await page.getByTestId('new-credential-method-select').selectOption('ssh');
    await page
      .getByTestId('new-credential-secret')
      .fill('-----BEGIN OPENSSH PRIVATE KEY-----\nnotarealkey\n-----END OPENSSH PRIVATE KEY-----\n');
    await page.getByTestId('save-new-credential').click();

    // Modal closes and the credential is now attached (picker replaced by the chip).
    await expect(page.getByTestId('new-credential-form')).not.toBeVisible();
    await expect(page.getByText(`rc_${credId}`)).toBeVisible();

    // Check before saving — exercises the endpoint against the unsaved form values.
    await page.getByTestId('repo-check-form').click();
    await expect(page.getByTestId('repo-check-result-form')).toBeVisible({ timeout: 15000 });

    // Save the repository entry.
    await page.getByTestId('save-repo-button').click();
    await expect(page.getByTestId('add-repo-form')).not.toBeVisible();
    const row = page.getByTestId('repo-entry-0');
    await expect(row).toBeVisible();
    await expect(row).toContainText('SSH');

    // Check the saved row.
    await page.getByTestId('repo-check-0').click();
    await expect(page.getByTestId('repo-check-result-0')).toBeVisible({ timeout: 15000 });

    // Edit the saved row: change the default branch and confirm it persists.
    await page.getByTestId('edit-repo-0').click();
    await expect(page.getByTestId('add-repo-form')).toBeVisible();
    await expect(page.getByTestId('repo-url-input')).toHaveValue('git@github.com:octocat/Hello-World.git');
    await page.getByTestId('repo-branch-input').fill('master');
    await page.getByTestId('save-repo-button').click();
    await expect(page.getByTestId('add-repo-form')).not.toBeVisible();
    await expect(page.getByTestId('repo-entry-0')).toContainText('master');

    await page.request.delete(`/api/v1/global/projects/${projectId}`);
  }
);
