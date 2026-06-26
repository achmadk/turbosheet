## ADDED Requirements

### Requirement: CLI `--headed` flag runs browser with visible UI

The `tsheet test` command SHALL accept `--headed` to launch the browser in non-headless mode. When `--headed` is passed, the browser window SHALL be visible during test execution.

#### Scenario: --headed overrides default headless mode

- **WHEN** user runs `tsheet test --headed`
- **THEN** the browser SHALL launch with a visible window
- **THEN** the `headless` field in the config passed to `run_tests()` SHALL be `false`

#### Scenario: default is headless

- **WHEN** user runs `tsheet test` without `--headed`
- **THEN** the browser SHALL launch in headless mode (no visible window)

### Requirement: CLI `--browser` flag selects browser engine

The `tsheet test` command SHALL accept `--browser <type>` to select the browser engine. Valid values SHALL be `chromium`, `firefox`, and `webkit`.

#### Scenario: --browser chromium selects Chromium

- **WHEN** user runs `tsheet test --browser chromium`
- **THEN** the `browser` field in the config SHALL be `"chromium"`

#### Scenario: --browser with invalid value warns user

- **WHEN** user runs `tsheet test --browser safari`
- **THEN** the CLI SHALL print a warning that `"safar"` is not a known browser type and default to `chromium`

#### Scenario: default browser is chromium

- **WHEN** user runs `tsheet test` without `--browser`
- **THEN** the `browser` field in the config SHALL be `None` (native layer defaults to chromium)

### Requirement: CLI `--output` flag sets output directory root

The `tsheet test` command SHALL accept `--output <dir>` to set the root directory for all test output (screenshots, videos, reports).

#### Scenario: --output sets screenshot and video directories

- **WHEN** user runs `tsheet test --output ./test-results`
- **THEN** the `screenshot_dir` in config SHALL be `"./test-results/screenshots"`
- **THEN** the `video_dir` in config SHALL be `"./test-results/videos"`

#### Scenario: --output sets reporter output directory

- **WHEN** user runs `tsheet test --reporter html,junit --output ./reports`
- **THEN** the HTML reporter SHALL write to `./reports/report.html`
- **THEN** the JUnit reporter SHALL write to `./reports/results.xml`

### Requirement: CLI `--update-snapshots` flag enables snapshot updates

The `tsheet test` command SHALL accept `--update-snapshots` to update visual and text snapshots instead of comparing against them.

#### Scenario: --update-snapshots sets config flag

- **WHEN** user runs `tsheet test --update-snapshots`
- **THEN** the `update_snapshots` field in config SHALL be `true`

#### Scenario: snapshots not updated by default

- **WHEN** user runs `tsheet test` without `--update-snapshots`
- **THEN** the `update_snapshots` field in config SHALL be `false` (or `None`)

### Requirement: CLI `--project` flag selects project config

The `tsheet test` command SHALL accept `--project <name>` to select which project configuration to use from a multi-project config file.

#### Scenario: --project sets active project

- **WHEN** user runs `tsheet test --project e2e`
- **THEN** only the project named `"e2e"` SHALL be executed

#### Scenario: no --project runs all projects

- **WHEN** user runs `tsheet test` without `--project`
- **THEN** all configured projects SHALL be executed

### Requirement: CLI help output lists all test flags

The `tsheet help` command SHALL display all available test flags in the "Test Options" section.

#### Scenario: help includes new flags

- **WHEN** user runs `tsheet help`
- **THEN** the output SHALL include `--headed`, `--browser`, `--output`, `--update-snapshots`, and `--project` in the Test Options section
