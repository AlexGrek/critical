describe('Authentication Flow', () => {
  beforeEach(() => {
    cy.visit('/');
  });

  it('should navigate to sign-in page', () => {
    cy.visit('/sign-in');
    cy.url().should('include', '/sign-in');
    cy.get('h1, h2').should('contain.text', 'Sign In');
  });

  it('should navigate to sign-up page', () => {
    cy.visit('/sign-up');
    cy.url().should('include', '/sign-up');
    cy.get('h1, h2').should('contain.text', 'Sign Up');
  });

  it('should display sign-in form elements', () => {
    cy.visit('/sign-in');
    cy.get('input[type="text"]').should('exist');
    cy.get('input[type="password"]').should('exist');
    cy.get('button[type="submit"]').should('exist');
  });

  it('should display sign-up form elements', () => {
    cy.visit('/sign-up');
    cy.get('input[type="text"]').should('exist');
    cy.get('input[type="password"]').should('have.length.at.least', 2);
    cy.get('button[type="submit"]').should('exist');
  });

  it('should have navigation links', () => {
    cy.visit('/sign-in');
    // Check for link to sign-up page
    cy.get('a').should('contain.text', 'Sign Up');
  });
});
