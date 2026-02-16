// Cypress E2E support file
// This is processed and loaded automatically before your test files

// Disable uncaught exception handling for tests that intentionally trigger errors
Cypress.on('uncaught:exception', (err, runnable) => {
  // Return false to prevent Cypress from failing the test
  // Adjust as needed for your application's error handling
  return true;
});
