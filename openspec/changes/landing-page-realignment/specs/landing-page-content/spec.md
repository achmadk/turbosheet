## ADDED Requirements

### Requirement: Landing page accurately represents browser engine support

The landing page SHALL only claim browser engine support that is actually implemented in the codebase.

#### Scenario: Chromium support listed

- **WHEN** a user reads the landing page features
- **THEN** the page MAY list Chromium as a supported engine
- **AND** the page SHALL NOT list Firefox or WebKit as "supported" without noting their feature-flag status

### Requirement: Landing page states only verified performance claims

The landing page SHALL NOT include any quantitative performance metrics (speed comparisons, memory usage, timing benchmarks) that are not backed by reproducible benchmarks in the repository.

#### Scenario: No fabricated benchmark claims

- **WHEN** a user reads the performance section
- **THEN** the page SHALL NOT display concrete numbers like "67% faster" or "50% less memory"

### Requirement: Landing page code examples match actual API

All code examples on the landing page SHALL use correct API signatures that match the shipped `index.js` and `index.d.ts` exports.

#### Scenario: launch() API shown correctly

- **WHEN** a code example shows browser launch
- **THEN** it SHALL use `launch()` (not `chromium.launch()`)

#### Scenario: No fabricated API methods

- **WHEN** the page shows API usage
- **THEN** it SHALL NOT reference methods that do not exist (e.g., `page.act()`)

### Requirement: Landing page features section reflects real capabilities

The features section SHALL include all major capabilities that are actually implemented in the codebase.

#### Scenario: Core features listed

- **WHEN** a user reads the features section
- **THEN** the page SHALL include: test runner (`test`/`expect`/`describe`), component testing (`mount`/`Component`), locator API (`getByRole`/`getByText`/etc.), trace viewer (record/serialize/HTML viewer), visual regression testing (screenshot comparison/diff), video recording, network interception (`page.route()`), and cross-platform prebuilt binaries

#### Scenario: No fictional features

- **WHEN** a user reads the features section
- **THEN** the page SHALL NOT mention "three browser engines" as a shipped capability

### Requirement: Landing page retains existing visual design system

The landing page SHALL keep its existing dark futuristic aesthetic, CSS design system, animations, and responsive layout.

#### Scenario: Visual identity preserved

- **WHEN** the page is rewritten
- **THEN** the color scheme (`#06060f` background, cyan accent `oklch(0.75 0.18 200)`) SHALL be preserved
- **AND** the particle animation, grid background, and glow orbs SHALL remain
- **AND** the typography system (monospace display + sans-serif body) SHALL be preserved

### Requirement: Landing page shows a realistic code example

The code example SHALL demonstrate at least three real API capabilities in a concise snippet.

#### Scenario: Realistic multi-feature code example

- **WHEN** a user views the code example
- **THEN** it SHALL show at least: page navigation + locator interaction, component mounting, and test/expect usage
