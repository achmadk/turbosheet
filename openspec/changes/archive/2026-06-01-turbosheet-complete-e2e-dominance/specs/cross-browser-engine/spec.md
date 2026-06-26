## ADDED Requirements

### Requirement: Firefox via WebDriver BiDi

TurboSheet SHALL support Firefox browser automation via the WebDriver BiDi protocol using the `geckodriver` process.

#### Scenario: Launch Firefox with default options

- **WHEN** user calls `tsheet.launch({ browser: 'firefox' })`
- **THEN** TurboSheet SHALL spawn a `geckodriver` process and connect via BiDi WebSocket

#### Scenario: Execute actions in Firefox

- **WHEN** user navigates, clicks, and types in Firefox
- **THEN** all standard TurboSheet actions SHALL work identically to Chromium mode

#### Scenario: Firefox BiDi stream pre-processing

- **WHEN** Firefox emits high-frequency BiDi events (mouse move, DOM mutations)
- **THEN** TurboSheet SHALL filter and batch events in Rust before forwarding to Node.js

### Requirement: WebKit via WebDriver BiDi

TurboSheet SHALL support WebKit browser automation via WebDriver BiDi protocol using `webkit2gtk` (Linux) and Safari remote WebDriver (macOS).

#### Scenario: Launch WebKit on Linux

- **WHEN** user calls `tsheet.launch({ browser: 'webkit' })` on Linux
- **THEN** TurboSheet SHALL connect to `webkit2gtk` via BiDi

#### Scenario: Launch WebKit on macOS

- **WHEN** user calls `tsheet.launch({ browser: 'webkit' })` on macOS
- **THEN** TurboSheet SHALL enable Safari's remote WebDriver and connect via BiDi

### Requirement: Cross-browser selector compatibility

TurboSheet SHALL ensure CSS and text selectors work identically across Chromium, Firefox, and WebKit engines.

#### Scenario: CSS selector parity

- **WHEN** user queries `page.locator('.submit-button')` in any browser
- **THEN** the same selector SHALL return equivalent elements across all three engines

#### Scenario: Text selector parity

- **WHEN** user queries `page.getByText('Submit')` in any browser
- **THEN** the text matching logic SHALL produce identical results across all three engines

### Requirement: Browser binary management

TurboSheet SHALL manage browser binary downloads and installation for all three engines via CLI.

#### Scenario: Install all browsers

- **WHEN** user runs `tsheet install` with no arguments
- **THEN** TurboSheet SHALL download and install Chromium, Firefox, and WebKit binaries

#### Scenario: Install specific browser

- **WHEN** user runs `tsheet install firefox`
- **THEN** TurboSheet SHALL download and install only Firefox
