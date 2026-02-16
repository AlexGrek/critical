/// <reference types="cypress" />

// Custom commands for Critical project tests

declare global {
  namespace Cypress {
    interface Chainable {
      /**
       * Log in as a user with username and password
       * @example cy.login('testuser', 'password123')
       */
      login(username: string, password: string): Chainable<void>;

      /**
       * Register a new user
       * @example cy.register('newuser', 'password123')
       */
      register(username: string, password: string): Chainable<void>;

      /**
       * Navigate to home page
       * @example cy.visitHome()
       */
      visitHome(): Chainable<void>;
    }
  }
}

/**
 * Login command - authenticates user via the sign-in page
 */
Cypress.Commands.add('login', (username: string, password: string) => {
  cy.visit('/sign-in');
  cy.get('input[type="text"]').first().type(username);
  cy.get('input[type="password"]').type(password);
  cy.get('button[type="submit"]').click();
  cy.url().should('not.include', '/sign-in');
});

/**
 * Register command - creates a new user via the sign-up page
 */
Cypress.Commands.add('register', (username: string, password: string) => {
  cy.visit('/sign-up');
  cy.get('input[type="text"]').first().type(username);
  cy.get('input[type="password"]').first().type(password);
  cy.get('input[type="password"]').last().type(password);
  cy.get('button[type="submit"]').click();
});

/**
 * Visit home page
 */
Cypress.Commands.add('visitHome', () => {
  cy.visit('/');
});

export {};
