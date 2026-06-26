## ADDED Requirements

### Requirement: Tree-Based Browser→Context→Page Ownership

The system SHALL replace global `DashMap` registries with a hierarchical ownership model where Browser owns Context owns Page, with Weak backlinks for upward traversal.

#### Scenario: Browser creates context

- **WHEN** `browser.newContext()` is called
- **THEN** the new Context SHALL be owned by the Browser
- **AND** `browser.contexts()` SHALL return all contexts created by this browser
- **AND** closing the Browser SHALL cascade-close all its contexts and pages

#### Scenario: Context creates page

- **WHEN** `context.newPage()` is called
- **THEN** the new Page SHALL be owned by the Context
- **AND** `context.pages()` SHALL return all pages in this context (not empty `vec![]`)
- **AND** closing the Context SHALL cascade-close all its pages
- **AND** the Page SHALL have a Weak reference back to its parent Context

#### Scenario: Page lifecycle tracking

- **WHEN** a Page is navigated or refreshed
- **THEN** its engine reference SHALL be reused (not replaced)
- **AND** the Context's page list SHALL remain valid after navigation
- **WHEN** a Page is closed
- **THEN** it SHALL be removed from its parent Context's page list
- **AND** the Page's Drop implementation SHALL clean up all engine resources

#### Scenario: Memory leak prevention

- **WHEN** a Browser is dropped without explicit close
- **THEN** its Drop impl SHALL close all contexts and pages
- **AND** no orphaned pages SHALL remain in global state
- **AND** Weak references from pages to contexts SHALL gracefully return None if context is gone
