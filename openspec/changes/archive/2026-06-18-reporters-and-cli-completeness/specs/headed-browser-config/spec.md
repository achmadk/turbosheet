## ADDED Requirements

### Requirement: TestConfig headless field controls browser visibility

The `TestConfig` napi struct SHALL include an optional `headless` boolean field. When `headless` is `Some(false)`, the browser SHALL launch with a visible window. When `headless` is `Some(true)` or `None`, the browser SHALL launch in headless mode.

#### Scenario: headless set to false

- **WHEN** `TestConfig` has `{ headless: false }`
- **THEN** the native browser launch SHALL pass `headless: false` to the launch options

#### Scenario: headless set to true

- **WHEN** `TestConfig` has `{ headless: true }`
- **THEN** the native browser launch SHALL pass `headless: true` (headless mode)

#### Scenario: headless is None (default)

- **WHEN** `TestConfig` has no `headless` field
- **THEN** the browser SHALL launch in headless mode (current default behavior preserved)

### Requirement: TestConfig browser field selects engine

The `TestConfig` napi struct SHALL include an optional `browser` string field. When present, it SHALL be passed to the browser launch to select the engine. Valid values SHALL include `"chromium"`, `"firefox"`, and `"webkit"`.

#### Scenario: browser set to chromium

- **WHEN** `TestConfig` has `{ browser: "chromium" }`
- **THEN** the Chromium engine SHALL be used for test execution

#### Scenario: browser set to firefox

- **WHEN** `TestConfig` has `{ browser: "firefox" }`
- **THEN** the Firefox engine SHALL be used for test execution

#### Scenario: browser is None (default)

- **WHEN** `TestConfig` has no `browser` field
- **THEN** the browser SHALL default to Chromium

### Requirement: Config fields are serializable

The `headless` and `browser` fields SHALL be serialized/deserialized via Serde, compatible with config file loading in `config_loader.rs`.

#### Scenario: config file sets headed and browser

- **WHEN** a config file contains `{ "headless": false, "browser": "firefox" }`
- **THEN** `ConfigLoader::load()` SHALL produce a `TestConfig` with matching field values
