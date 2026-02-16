# Critical E2E Tests (Playwright)

End-to-end tests for the Critical project frontend using Playwright.

## Setup

### Install Dependencies

```bash
cd e2e-tests
npm install
```

### Environment Setup

**IMPORTANT**: These tests use **real API calls** to the actual development server and database. You must have both the backend and frontend running:

#### 1. Start the backend with ArangoDB:
```bash
# From project root
make run
```

This starts the backend API on `http://localhost:3742` with a real ArangoDB instance.

#### 2. Start the frontend development server:
```bash
cd frontend
npm run dev
```

This starts the frontend on `http://localhost:5173` which proxies API calls to the backend.

The tests will:
- Create **unique random users** for each test run to avoid conflicts
- Create **test groups** via the actual API
- Clean up created data after tests complete
- Use the real database (not mocked data)

## Running Tests

### Run All Playwright Tests

```bash
npm run test
```

### Run in Headed Mode

```bash
npm run test:headed
```

### Run Specific Browser

```bash
npm run test:chrome
npm run test:firefox
npm run test:webkit
```

### Debug Mode

```bash
npm run test:debug
```

## Project Structure

```
e2e-tests/
├── e2e/                    # End-to-end test files
│   ├── auth.spec.ts       # Authentication flow tests
│   ├── home.spec.ts       # Home page tests
│   └── groups.spec.ts     # Groups page tests
├── fixtures/              # Test data and fixtures
├── playwright.config.ts   # Playwright configuration
├── tsconfig.json          # TypeScript configuration
└── package.json           # NPM dependencies
```

## Test Coverage

### Current Test Suites

#### Authentication Tests (`auth.spec.ts`)
- Navigation to sign-in and sign-up pages
- Form element presence and validation
- Navigation links between auth pages
- Sign-in and sign-up functionality

#### Home Page Tests (`home.spec.ts`)
- Page loading without authentication
- Navigation elements presence
- Links to auth pages

#### Groups Page Tests (`groups.spec.ts`)
- **Navigation**: From home page and direct URL access
- **Page Structure**: Headers, descriptions, and back button
- **Empty State**: Display when no groups exist
- **Groups Display**: Grid layout with **real API data**
  - Creates test groups via API with unique IDs
  - Group name and ID display
  - Access control rules count
  - Principals list for groups with ACL
  - Correct styling and responsive layout
- **Real-time Updates**: Verifies groups appear after creation and page refresh
- **Data Cleanup**: Automatically deletes test groups after tests
- **Error Handling**: Unauthorized access
- **Accessibility**: Proper heading hierarchy and link navigation
- **User Isolation**: Each test run uses unique random usernames to avoid conflicts

## Writing Tests

### Basic Test Example

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/page-url');
  });

  test('should do something', async ({ page }) => {
    await page.getByRole('button').click();
    await expect(page.getByText('Success')).toBeVisible();
  });
});
```

### Working with Fixtures

Create JSON files in the `fixtures/` directory:

```json
{
  "user": {
    "username": "testuser",
    "password": "password123"
  }
}
```

## Configuration

See `playwright.config.ts` for configuration options:

- `baseURL` — Base URL for tests (default: `http://localhost:5173`)
- `viewport` — Viewport dimensions
- `timeout` — Test timeout
- `video` — Record video of test runs
- `screenshot` — Capture screenshots on failure

## CI/CD Integration

To integrate with CI/CD pipelines, add to your workflow:

```yaml
- name: Run Playwright E2E Tests
  run: |
    cd e2e-tests
    npm install
    npx playwright install --with-deps
    npm run test
```

## Troubleshooting

### Tests fail with "Cannot find element"

- Ensure the frontend dev server is running on `http://localhost:5173`
- Check that selectors match your app's HTML (use DevTools to inspect)
- Increase timeout: `await page.getByRole('button', { timeout: 20000 })`

### Port already in use

Change `baseURL` in `playwright.config.ts` to match your frontend URL.

### Video/Screenshot files too large

Adjust video settings in `playwright.config.ts`.

## Resources

- [Playwright Documentation](https://playwright.dev/)
- [Best Practices](https://playwright.dev/docs/best-practices)
- [API Reference](https://playwright.dev/docs/api/class-test)
