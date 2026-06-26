describe("My Cypress Test", () => {
  it("visits example", () => {
    cy.visit("https://example.com");
    cy.get(".btn").click();
  });
});
