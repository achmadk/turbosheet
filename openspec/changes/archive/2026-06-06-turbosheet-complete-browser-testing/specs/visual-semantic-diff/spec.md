## ADDED Requirements

### Requirement: Screenshot capture

The system SHALL capture screenshots using real CDP screenshot API.

#### Scenario: Capture viewport screenshot

- **WHEN** `page.screenshot()` is called
- **THEN** system captures current viewport
- **AND** returns PNG bytes

#### Scenario: Capture full page screenshot

- **WHEN** `page.screenshot({ fullPage: true })` is called
- **THEN** system scrolls page and captures entire content
- **AND** returns stitched PNG image

#### Scenario: Capture element screenshot

- **WHEN** `page.locator('.chart').screenshot()` is called
- **THEN** system captures element bounding box
- **AND** returns PNG of element only

### Requirement: Pixel-level comparison

The system SHALL compare screenshots using pixel diff with configurable threshold.

#### Scenario: Compare identical screenshots

- **WHEN** `expect(page).toHaveScreenshot('same.png')` is called for identical pages
- **THEN** comparison passes

#### Scenario: Compare with pixel differences

- **WHEN** comparison finds pixel differences below threshold
- **THEN** comparison passes

#### Scenario: Compare with differences above threshold

- **WHEN** comparison finds pixel differences above threshold
- **THEN** comparison fails
- **AND** error message shows diff percentage

### Requirement: Semantic diff

The system SHALL ignore cosmetic differences using AI-powered semantic comparison.

#### Scenario: Ignore text rendering differences

- **WHEN** font rendering differs but content is same
- **THEN** semantic diff ignores minor pixel variations in text rendering

#### Scenario: Ignore anti-aliasing differences

- **WHEN** subpixel anti-aliasing differs
- **THEN** semantic diff ignores these variations

#### Scenario: Ignore layout-only changes

- **WHEN** content is same but positioned differently due to layout
- **THEN** semantic diff identifies as semantic match

### Requirement: Ignore regions

The system SHALL allow specifying regions to ignore during comparison.

#### Scenario: Ignore dynamic content region

- **WHEN** `page.screenshot({ animations: 'disabled' })` is called
- **THEN** system ignores animated elements

#### Scenario: Custom ignore region

- **WHEN** screenshot comparison includes ignore regions
- **THEN** system excludes those regions from diff
- **AND** compares only non-ignored areas

#### Scenario: Ignore by selector

- **WHEN** `expect(page).toHaveScreenshot('output.png', { ignoreRegions: [{ selector: '.ads' }] })` is called
- **THEN** system ignores elements matching selector

### Requirement: Layout shift detection

The system SHALL detect Cumulative Layout Shift (CLS) violations.

#### Scenario: Detect layout shift

- **WHEN** comparing screenshots with layout shifts
- **THEN** system calculates CLS score
- **AND** reports shift regions

### Requirement: Baseline management

The system SHALL manage screenshot baselines for visual regression testing.

#### Scenario: Update baseline

- **WHEN** `UPDATE_VISUAL_BASELINES=true` env var is set
- **THEN** system writes new screenshots as baselines

#### Scenario: First run creates baseline

- **WHEN** `expect(page).toHaveScreenshot('new.png')` is called
- **AND** no baseline exists
- **THEN** system creates baseline
- **AND** passes

#### Scenario: Missing baseline warning

- **WHEN** comparison is attempted without baseline
- **THEN** system logs warning
- **AND** creates baseline in `__screenshots__/` directory
