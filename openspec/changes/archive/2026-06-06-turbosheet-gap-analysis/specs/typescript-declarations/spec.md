## ADDED Requirements

### Requirement: Complete TypeScript declaration file

The root `index.d.ts` SHALL provide complete, accurate, and documented type declarations for every public API exported by the TurboSheet npm package. The N-API boundary exports functions from `src/lib.rs`, `src/page.rs`, `src/locator.rs`, `src/browser.rs`, `src/context.rs`, `src/assertions/mod.rs`, `src/migrate/mod.rs`, `src/test_runner/mod.rs`.

#### Scenario: All N-API exported functions have TypeScript declarations

- **WHEN** a TypeScript project imports from 'turbo-sheet'
- **THEN** every exported function, class, interface, and type SHALL have a corresponding declaration in index.d.ts with accurate parameter types and return types

#### Scenario: JsLocator has chainable method declarations

- **WHEN** a user types `page.locator('...').` in VS Code
- **THEN** autocomplete SHALL show all locator methods: `click`, `dblclick`, `fill`, `hover`, `focus`, `blur`, `press`, `pressSequentially`, `check`, `uncheck`, `select`, `scrollIntoView`, `boundingBox`, `screenshot`, `dragAndDrop`, `setInputFiles`, `textContent`, `innerText`, `innerHtml`, `getAttribute`, `isVisible`, `isEnabled`, `isDisabled`, `first`, `last`, `nth`, `filter`, `and`, `or`
- **AND** each method SHALL have correct parameter types and return types

#### Scenario: JsPage has full page API declarations

- **WHEN** a user types `page.` in VS Code
- **THEN** autocomplete SHALL show all page methods: `goto`, `screenshot`, `evaluate`, `content`, `title`, `reload`, `goBack`, `goForward`, `setViewportSize`, `viewportSize`, `waitForRequest`, `waitForResponse`, `waitForSelector`, `addScriptTag`, `addStyleTag`, `exposeFunction`, `route`, `unrouteAll`, `routeWebSocket`, `unrouteWebSocket`, `unrouteAllWebSockets`, `throttle`, `setOffline`, `emulateMedia`, `on`, `off`, `locator`, `getByRole`, `getByText`, `getByTestId`, `getByPlaceholder`, `getByLabel`, `getByAltText`, `close`

#### Scenario: Assertion types are declared

- **WHEN** a user types `expect(page).` or `expect(locator).`
- **THEN** autocomplete SHALL show all assertion methods: `toBeVisible`, `toBeHidden`, `toBeEnabled`, `toBeDisabled`, `toHaveText`, `toHaveAttribute`, `toHaveValue`, `toHaveURL`, `toHaveTitle`, `toHaveScreenshot`, `toHaveNoAccessibilityViolations`
- **AND** each assertion SHALL accept optional options parameter with `timeout`

### Requirement: JSDoc documentation on all declarations

Every type declaration SHALL include JSDoc comments describing the method, its parameters, return values, and usage examples. This ensures rich IntelliSense documentation in editors.

#### Scenario: Methods display documentation in editor

- **WHEN** a user hovers over a method name in VS Code
- **THEN** they SHALL see a description, parameter documentation, and optionally a usage example

### Requirement: Type-safe options interfaces

All optional parameters SHALL have named interfaces (e.g., `ScreenshotOptions`, `LocatorOptions`, `AssertionOptions`, `RouteOptions`, `LaunchOptions`, `ContextOptions`) with all fields typed and documented.

#### Scenario: LaunchOptions interface

- **WHEN** `launch()` is called with options
- **THEN** TypeScript SHALL validate: `browser` ('chromium' | 'firefox' | 'webkit'), `headless` (boolean), `executablePath` (string), `args` (string[]), `viewport` ({width, height}), `deviceScaleFactor` (number)
