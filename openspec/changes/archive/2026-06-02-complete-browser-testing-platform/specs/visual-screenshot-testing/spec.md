# visual-screenshot-testing

## ADDED Requirements

### Requirement: toHaveScreenshot matcher

The system SHALL provide a `toHaveScreenshot()` assertion for visual regression testing.

#### Scenario: Screenshot matches baseline

- **WHEN** user calls `await expect(page).toHaveScreenshot('homepage.png')`
- **THEN** if current screenshot matches baseline within threshold, assertion passes

#### Scenario: Screenshot differs from baseline

- **WHEN** user calls `await expect(page).toHaveScreenshot('homepage.png')`
- **AND** current screenshot differs from baseline beyond threshold
- **THEN** assertion fails and diff image is saved

### Requirement: toHaveScreenshot with locator

The system SHALL support `toHaveScreenshot()` on locator assertions.

#### Scenario: Element screenshot matches

- **WHEN** user calls `await expect(page.locator('#header')).toHaveScreenshot('header.png')`
- **THEN** if element screenshot matches baseline, assertion passes

### Requirement: Screenshot baseline management

The system SHALL automatically create baseline screenshots when they don't exist.

#### Scenario: Create baseline on first run

- **WHEN** user calls `expect(page).toHaveScreenshot('new-page.png')`
- **AND** baseline does not exist
- **THEN** system saves current screenshot as baseline and passes

#### Scenario: Update baseline via CLI flag

- **WHEN** user runs `tsheet test --update-screenshots`
- **THEN** all `toHaveScreenshot` assertions update baselines instead of failing

### Requirement: Screenshot comparison options

The system SHALL support configurable comparison options.

#### Scenario: Custom threshold

- **WHEN** user calls `expect(page).toHaveScreenshot('page.png', { threshold: 0.5 })`
- **THEN** assertion allows up to 50% pixel difference

#### Scenario: Ignore specific regions

- **WHEN** user calls `expect(page).toHaveScreenshot('page.png', { ignoreRegions: [{ x: 0, y: 0, width: 100, height: 50 }] })`
- **THEN** those regions are excluded from comparison

### Requirement: Screenshot directories

The system SHALL use configured directories for baseline and diff storage.

#### Scenario: Default screenshot directories

- **WHEN** user runs tests without configuration
- **THEN** system uses `./screenshots/baseline/` and `./screenshots/diff/`

#### Scenario: Custom screenshot directory

- **WHEN** user sets `screenshotsDir: './visual-results'` in config
- **THEN** system uses `./visual-results/baseline/` and `./visual-results/diff/`

### Requirement: CI screenshot storage

The system SHALL support storing baselines in cloud storage for CI environments.

#### Scenario: Upload baseline on CI first run

- **WHEN** `TURBOSHEET_SCREENSHOTS_BUCKET` is set and baseline missing locally
- **THEN** system downloads baseline from cloud storage

#### Scenario: Upload diff on CI failure

- **WHEN** screenshot comparison fails on CI
- **THEN** system uploads diff image to cloud storage
