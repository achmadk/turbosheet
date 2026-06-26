## ADDED Requirements

### Requirement: Framework-agnostic component mounting

TurboSheet SHALL support mounting frontend framework components (React, Vue, Svelte) in a lightweight headless context.

#### Scenario: Mount React component

- **WHEN** user calls `const component = await mount(<Button label="Click" />)`
- **THEN** TurboSheet SHALL render the component in a minimal headless Chromium context without loading the full application

#### Scenario: Interact with mounted component

- **WHEN** user clicks the mounted component
- **THEN** component SHALL respond to interactions and state changes

#### Scenario: Assert on component output

- **WHEN** user interacts with the component
- **THEN** standard `expect` assertions SHALL work on the component's rendered output

### Requirement: Fast component rendering cycles

Component rendering SHALL use a pre-warmed minimal browser context, avoiding full page load.

#### Scenario: Warm context reuse

- **WHEN** mounting multiple components in the same test file
- **THEN** TurboSheet SHALL reuse a pre-warmed headless context (no new page load per mount)

#### Scenario: First mount speed

- **WHEN** mounting the first component
- **THEN** rendering SHALL complete in under 1 second (excluding framework bundle load time)

### Requirement: Framework auto-detection

TurboSheet SHALL auto-detect the project's frontend framework and apply appropriate rendering configuration.

#### Scenario: React detection

- **WHEN** project has `react` in `package.json`
- **THEN** TurboSheet SHALL use React rendering pipeline automatically

#### Scenario: Vue detection

- **WHEN** project has `vue` in `package.json`
- **THEN** TurboSheet SHALL use Vue rendering pipeline automatically
