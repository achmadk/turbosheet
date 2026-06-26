## ADDED Requirements

### Requirement: Cross-Engine Consistency Reports

The system SHALL support running the same test across multiple browser engines and reporting behavioral differences.

#### Scenario: Run same test on multiple engines

- **GIVEN** a test file is configured to run on Chromium, Firefox, and WebKit
- **WHEN** the test suite executes
- **THEN** the system SHALL run each test on all three engines
- **AND** collect per-engine results: pass/fail, timing, screenshots, console output, network log
- **AND** produce a consistency report comparing results across engines

#### Scenario: Consistency report format

- **WHEN** a cross-engine test run completes
- **THEN** the system SHALL generate a report containing:
  - Overall consistency score: percentage of tests that produced the same result across all engines
  - Per-test consistency: show which engines passed, which failed, and why
  - Timing comparison: bar chart showing test duration per engine
  - Screenshot comparison: side-by-side or diff overlay for each engine
  - Layout delta report: positions/sizes that differ beyond a configurable threshold (default: 1px)

#### Scenario: Per-engine layout difference detection

- **GIVEN** the same page is rendered on Chromium and Firefox
- **WHEN** consistency checking is enabled
- **THEN** the system SHALL capture full-page screenshots on each engine
- **AND** compute pixel-level diff between engine screenshots
- **AND** report elements whose bounding box differs by more than `maxDelta` pixels
- **AND** the report SHALL include: element selector, property that differs (x/y/width/height), values per engine

#### Scenario: Consistency score

- **WHEN** a cross-engine report is generated
- **THEN** the system SHALL compute a score:
  - `1.0` = identical behavior on all engines (same pass/fail, no layout diff >1px)
  - `0.5-0.99` = behavioral differences detected (different layout, different timing)
  - `0.0-0.49` = functional differences (test passes on one engine, fails on another)
- **AND** the score SHALL be prominently displayed in the test summary output

#### Scenario: CI gate for consistency

- **GIVEN** consistency checks are configured with a minimum score threshold
- **WHEN** the cross-engine score falls below the threshold
- **THEN** the CI run SHALL be marked as unstable/warning (configurable: warn vs fail)
- **AND** the consistency report SHALL be uploaded as a CI artifact
- **AND** developers SHALL be able to suppress per-engine differences with `@tsheet-consistency-ignore` comments in tests
