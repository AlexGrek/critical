# Critical E2E Tests (Cypress)

End-to-end tests for the Critical project frontend using Cypress.

## Setup

### Install Dependencies

```bash
cd cypress
npm install
```

### Environment Setup

Ensure the frontend development server is running:

```bash
cd frontend
npm run dev
```

The tests will run against `http://localhost:5173` by default.

## Running Tests

### Open Cypress Test Runner (Interactive)

```bash
npm run cy:open
```

This opens the Cypress UI where you can see tests run in real-time and debug failures.

### Run Tests Headless

```bash
npm run cy:run
```

Runs all tests in headless mode and generates reports.

### Run Tests in Headed Mode

```bash
npm run cy:run:headed
```

Runs tests with the browser window visible.

### Run Tests in Specific Browser

```bash
npm run cy:run:chrome
npm run cy:run:firefox
```

## Project Structure

```
cypress/
├── e2e/                    # End-to-end test files
│   ├── auth.cy.ts         # Authentication flow tests
│   └── home.cy.ts         # Home page tests
├── fixtures/              # Test data and fixtures
├── support/
│   ├── commands.ts        # Custom Cypress commands
│   └── e2e.ts            # E2E support configuration
├── cypress.config.ts      # Cypress configuration
├── tsconfig.json          # TypeScript configuration
└── package.json           # NPM dependencies
```

## Writing Tests

### Basic Test Example

```typescript
describe('Feature Name', () => {
  beforeEach(() => {
    cy.visit('/page-url');
  });

  it('should do something', () => {
    cy.get('button').click();
    cy.get('.result').should('contain.text', 'Success');
  });
});
```

### Using Custom Commands

```typescript
// Login and verify redirect
cy.login('username', 'password');
cy.url().should('not.include', '/sign-in');
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

Use in tests:

```typescript
cy.fixture('user').then((user) => {
  cy.login(user.username, user.password);
});
```

## Configuration

See `cypress.config.ts` for configuration options:

- `baseUrl` — Base URL for tests (default: `http://localhost:5173`)
- `viewportWidth` / `viewportHeight` — Viewport dimensions
- `defaultCommandTimeout` — Timeout for commands
- `video` — Record video of test runs
- `screenshotOnRunFailure` — Capture screenshots on failure

## CI/CD Integration

To integrate with CI/CD pipelines, add to your workflow:

```yaml
- name: Run Cypress E2E Tests
  run: |
    cd cypress
    npm install
    npm run cy:run
```

## Troubleshooting

### Tests fail with "Cannot find element"

- Ensure the frontend dev server is running on `http://localhost:5173`
- Check that selectors match your app's HTML (use DevTools to inspect)
- Wait for elements: use `cy.get(selector, { timeout: 20000 })`

### Port already in use

Change `baseUrl` in `cypress.config.ts` to match your frontend URL.

### Video/Screenshot files too large

Adjust `videoCompression` in `cypress.config.ts` (higher = more compression).

## Resources

- [Cypress Documentation](https://docs.cypress.io/)
- [Best Practices](https://docs.cypress.io/guides/references/best-practices)
- [API Reference](https://docs.cypress.io/api/table-of-contents)
