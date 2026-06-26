## ADDED Requirements

### Requirement: JS plugin API

TurboSheet SHALL support JavaScript plugins installed via npm for reporters, hooks, and custom matchers.

#### Scenario: Install plugin

- **WHEN** user runs `npm install tsheet-plugin-allure`
- **THEN** TurboSheet SHALL auto-discover the plugin from `node_modules`

#### Scenario: Plugin registration

- **WHEN** plugin exports a `createReporter()` function
- **THEN** user SHALL register it in config: `reporter: ['html', './allure-reporter.js']`

#### Scenario: Custom matcher plugin

- **WHEN** plugin exports a `createMatchers()` function returning custom matchers
- **THEN** user SHALL use `expect(element).toHaveCustomProperty(value)` in tests

### Requirement: Rust plugin API

TurboSheet SHALL support high-performance Rust plugins compiled as shared libraries (.so/.dylib).

#### Scenario: Rust reporter plugin

- **WHEN** user places a `.so` file in `~/.tsheet/plugins/`
- **THEN** TurboSheet SHALL load it and call its exported `report(result)` function

#### Scenario: Plugin performance

- **WHEN** a Rust plugin processes test results
- **THEN** it SHALL execute in the same Tokio runtime as the core (zero IPC overhead)

### Requirement: Lifecycle hooks API

Plugins SHALL hook into the test lifecycle at defined points.

#### Scenario: Pre-test hook

- **WHEN** a plugin implements `onTestStart(test)`
- **THEN** the hook SHALL be called before each test execution

#### Scenario: Post-test hook

- **WHEN** a plugin implements `onTestEnd(test, result)`
- **THEN** the hook SHALL be called after each test with the result

#### Scenario: On-failure hook

- **WHEN** a test fails and plugin implements `onTestFailed(test, error)`
- **THEN** the hook SHALL be called with the error details

### Requirement: Plugin discovery

TurboSheet SHALL auto-discover plugins from `node_modules/tsheet-plugin-*` and `~/.tsheet/plugins/`.

#### Scenario: Automatic discovery

- **WHEN** user installs `tsheet-plugin-slack` via npm
- **THEN** TurboSheet SHALL automatically detect and register the plugin without config changes
