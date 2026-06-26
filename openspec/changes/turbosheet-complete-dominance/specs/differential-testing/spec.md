## ADDED Requirements

### Requirement: Differential Testing Across Browser Versions

The system SHALL support running test suites against multiple browser versions simultaneously and reporting behavioral differences.

#### Scenario: Run tests on two Chromium versions

- **GIVEN** the test suite is configured with two Chromium versions (e.g., Stable 130 and Beta 131)
- **WHEN** the test suite executes in differential mode
- **THEN** the system SHALL launch two browser instances (one per version)
- **AND** run each test on both versions
- **AND** compare results per test: pass/fail, timing, screenshots, console output
- **AND** report any test that behaves differently between versions

#### Scenario: Differential test report

- **WHEN** differential testing completes
- **THEN** the system SHALL generate a report containing:
  - Version A: name, version string, pass/fail count
  - Version B: name, version string, pass/fail count
  - Differing tests: list of tests with different outcomes between versions
  - New failures: tests that pass on version A but fail on version B
  - New passes: tests that fail on version A but pass on version B
  - Timing regressions: tests that are >20% slower on version B
  - Screenshot diffs: pixel-level comparison for tests that produce different screenshots

#### Scenario: Automated regression flagging

- **GIVEN** a test passes on the current stable version
- **WHEN** the same test fails on a beta/canary version
- **THEN** the system SHALL flag this as a potential regression
- **AND** assign a severity based on:
  - Critical: test was explicitly written for the feature being changed
  - High: test interacts with the feature area indirectly
  - Medium: test is in a different area but the failure pattern matches known changes
  - Low: flaky test with intermittent failure
- **AND** include browser bug tracker links if the regression matches known Chromium issues

#### Scenario: CI gate for browser upgrades

- **GIVEN** a CI pipeline is upgrading Chrome for Testing from v130 to v131
- **WHEN** differential testing is run between the two versions
- **THEN** the system SHALL run the full test suite on both versions
- **AND** block the upgrade if:
  - Any critical-severity regression is detected
  - More than `maxRegressionPct` (configurable, default: 1%) of tests change behavior
  - A Core Web Vitals metric regresses by >10% on the new version
- **AND** allow the upgrade if no regressions are found (green signal)

#### Scenario: Version discovery from Chrome for Testing API

- **GIVEN** an automated browser installation workflow
- **WHEN** differential testing is configured with `versions: ["stable", "beta", "dev", "canary"]`
- **THEN** the system SHALL query `https://googlechromelabs.github.io/chrome-for-testing/known-good-versions.json`
- **AND** resolve "stable", "beta", "dev", "canary" to specific version strings using the latest available in each channel
- **AND** download the resolved versions if not already cached
- **AND** report which specific versions will be compared

#### Scenario: Per-version exclusion rules

- **GIVEN** a test is known to behave differently on beta versions
- **WHEN** differential testing is run
- **THEN** the system SHALL support `@tsheet-version-skip` annotations:
  - `@tsheet-version-skip beta` — skip this test on beta version
  - `@tsheet-version-skip < 131` — skip on versions older than 131
  - `@tsheet-version-skip >= 132` — skip on versions 132 and newer
- **AND** mark skipped tests in the report with the skip reason
