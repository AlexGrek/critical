describe('Home Page', () => {
  beforeEach(() => {
    cy.visit('/');
  });

  it('should load the home page', () => {
    cy.url().should('include', '/');
  });

  it('should be accessible without authentication', () => {
    cy.visit('/');
    cy.get('body').should('be.visible');
  });

  it('should have navigation elements', () => {
    cy.get('nav, header, [role="navigation"]').should('exist');
  });

  it('should have links to auth pages', () => {
    cy.get('a').then(($links) => {
      const hasSignIn = Array.from($links).some((el) =>
        el.textContent?.toLowerCase().includes('sign in')
      );
      const hasSignUp = Array.from($links).some((el) =>
        el.textContent?.toLowerCase().includes('sign up')
      );
      expect(hasSignIn || hasSignUp).to.be.true;
    });
  });
});
