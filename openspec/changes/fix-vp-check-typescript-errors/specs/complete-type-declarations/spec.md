## ADDED Requirements

### Requirement: `index.d.ts` exposes all public JS API symbols

The TypeScript declaration file SHALL declare every public symbol exported by `index.js`.

#### Scenario: `launch` is declared as exported function

- **WHEN** TypeScript checks `import("../index.js").launch`
- **THEN** it resolves to a callable function type `(options?: LaunchOptions) => Promise<Browser>`

#### Scenario: `version` is declared as exported function

- **WHEN** TypeScript checks `import("../index.js").version`
- **THEN** it resolves to a callable function type `() => string`

#### Scenario: `Browser` is declared as exported interface

- **WHEN** TypeScript checks `import("../index.js").Browser`
- **THEN** it resolves to an interface with `newContext()` and `close()` methods

#### Scenario: `Page` interface includes `goto`, `locator`, `close`, `content`, `title`, `url`, `evaluate`

- **WHEN** TypeScript checks `page.goto()`, `page.locator()`, `page.close()`, `page.content()`, `page.title()`, `page.url()`, `page.evaluate()`
- **THEN** each resolves to a valid method signature

### Requirement: `vp check --fix` exits with zero type errors

The TypeScript checking pipeline SHALL pass cleanly after all declaration and config fixes.

#### Scenario: Clean type check

- **WHEN** running `vp check --fix`
- **THEN** no TypeScript errors are reported and exit code is 0
