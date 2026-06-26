## ADDED Requirements

### Requirement: Screenshot capture

TurboSheet SHALL capture full-page, element-level, and viewport screenshots.

#### Scenario: Full page screenshot

- **WHEN** user calls `await page.screenshot({ fullPage: true })`
- **THEN** TurboSheet SHALL capture the entire scrollable page as a PNG/WebP image

#### Scenario: Element screenshot

- **WHEN** user calls `await element.screenshot()`
- **THEN** TurboSheet SHALL capture only the element's bounding box

### Requirement: Visual snapshot comparison

TurboSheet SHALL compare screenshots against stored baselines and report pixel differences.

#### Scenario: Baseline comparison

- **WHEN** user calls `await expect(page).toHaveScreenshot('home.png')`
- **THEN** TurboSheet SHALL compare the current screenshot against the stored baseline

#### Scenario: Baseline creation

- **WHEN** no baseline exists for `home.png`
- **THEN** TurboSheet SHALL create the baseline on first run and mark the test as passed

#### Scenario: Pixel threshold

- **WHEN** screenshots differ by less than `threshold: 0.1` (0.1% pixels)
- **THEN** test SHALL pass despite pixel differences

### Requirement: SIMD-accelerated pixel comparison

Pixel comparison SHALL use Rust SIMD instructions (SSE/AVX on x86, NEON on ARM) for 3-8x faster processing.

#### Scenario: Large screenshot comparison

- **WHEN** comparing two 4K screenshots (3840x2160)
- **THEN** comparison SHALL complete in under 500ms

### Requirement: AI-powered semantic diff

TurboSheet SHALL optionally use AI-native DOM data to distinguish "meaningful" visual changes from "noise" (animations, loading states, dynamic content).

#### Scenario: Ignore dynamic regions

- **WHEN** user sets `semanticDiff: true` and the screenshot differs in a region containing an animated element
- **THEN** the AI semantic diff SHALL identify the region as dynamic and exclude it from failure

#### Scenario: Semantic diff image output

- **WHEN** a visual test fails with `semanticDiff: true`
- **THEN** the output SHALL include a diff image with color-coded regions (red=real diff, yellow=ignored dynamic content)

### Requirement: Diff image output

On comparison failure, TurboSheet SHALL produce a diff image with heatmap overlay.

#### Scenario: Diff image generation

- **WHEN** visual comparison fails
- **THEN** TurboSheet SHALL generate a diff image highlighting changed pixels
- **THEN** the diff image SHALL use green for removed, red for added, and gray-scale for unchanged

### Requirement: Screenshot storage management

Baseline screenshots SHALL be stored in a `.tsheet-snap/` directory adjacent to the test file.

#### Scenario: Baseline location

- **WHEN** user runs visual test for `tests/homepage.tsheet.ts`
- **THEN** baselines SHALL be stored at `tests/.tsheet-snap/homepage/`
