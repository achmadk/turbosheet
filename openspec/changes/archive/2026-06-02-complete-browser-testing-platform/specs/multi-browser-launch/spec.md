# multi-browser-launch

## ADDED Requirements

### Requirement: Browser type selection

The system SHALL support launching Chromium, Firefox, and WebKit browsers via a unified `launch()` API with a `browser` option.

#### Scenario: Launch Chromium

- **WHEN** user calls `launch({ browser: 'chromium' })`
- **THEN** system launches Chromium via ChromiumEngine

#### Scenario: Launch Firefox

- **WHEN** user calls `launch({ browser: 'firefox' })`
- **THEN** system launches Firefox via FirefoxEngine using geckodriver

#### Scenario: Launch WebKit

- **WHEN** user calls `launch({ browser: 'webkit' })`
- **THEN** system launches WebKit via WebKitEngine using safaridriver

#### Scenario: Default browser is Chromium

- **WHEN** user calls `launch()` without browser option
- **THEN** system launches Chromium by default

### Requirement: Browser-specific launch options

The system SHALL accept browser-specific launch options while maintaining a unified API surface.

#### Scenario: Firefox-specific option ignored on Chromium

- **WHEN** user calls `launch({ browser: 'chromium', firefoxUserPrefs: { key: 'value' } })`
- **THEN** system launches Chromium and ignores the firefox-specific option

#### Scenario: Chromium-specific option ignored on Firefox

- **WHEN** user calls `launch({ browser: 'firefox', chromiumArgs: ['--disable-extensions'] })`
- **THEN** system launches Firefox and ignores the chromium-specific option

### Requirement: Browser binary path configuration

The system SHALL allow users to specify custom browser binary paths for all supported browsers.

#### Scenario: Custom Chromium path

- **WHEN** user calls `launch({ browser: 'chromium', executablePath: '/custom/chromium' })`
- **THEN** system launches the specified Chromium binary

#### Scenario: Custom Firefox path

- **WHEN** user calls `launch({ browser: 'firefox', executablePath: '/custom/firefox' })`
- **THEN** system launches the specified Firefox binary

### Requirement: Headless mode per browser

The system SHALL support headless mode for all browsers via a unified `headless` option.

#### Scenario: Chromium headless

- **WHEN** user calls `launch({ browser: 'chromium', headless: true })`
- **THEN** Chromium launches in headless mode

#### Scenario: Firefox headless

- **WHEN** user calls `launch({ browser: 'firefox', headless: true })`
- **THEN** Firefox launches in headless mode

### Requirement: Browser availability detection

The system SHALL detect whether required browser binaries are installed and provide clear error messages.

#### Scenario: Missing Chromium

- **WHEN** user calls `launch({ browser: 'chromium' })` but Chromium is not installed
- **THEN** system returns error "Chromium not found. Install with: tsheet install chromium"

#### Scenario: Missing Firefox

- **WHEN** user calls `launch({ browser: 'firefox' })` but Firefox is not installed
- **THEN** system returns error "Firefox not found. Install with: tsheet install firefox"
