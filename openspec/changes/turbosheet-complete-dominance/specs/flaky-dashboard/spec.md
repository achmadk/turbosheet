## ADDED Requirements

### Requirement: Flaky Test Dashboard

The system SHALL aggregate flakiness data across CI runs and surface the most flaky tests in a human-readable report.

#### Scenario: Flakiness data collection

- **WHEN** each CI test run completes
- **THEN** the system SHALL output a `tsheet-flakiness.json` report alongside the test results
- **AND** SHALL include per-test: pass count, fail count, total runs, flake score

#### Scenario: Flake score computation

- **WHEN** computing flakiness for a test
- **THEN** the system SHALL compute score = (failure count / total runs) × 100
- **AND** SHALL only consider tests that have run at least 5 times

#### Scenario: Dashboard generation

- **WHEN** `npx tsheet flaky-dashboard` is run against a directory of `tsheet-flakiness.json` files
- **THEN** the system SHALL aggregate data across all runs
- **AND** SHALL generate an HTML report showing:
  - Top-20 flakiest tests ranked by score
  - Flakiness trend per test over time (sparkline or chart)
  - Total flaky tests count (score > 10)
  - Overall suite flakiness percentage

#### Scenario: Flaky trend tracking

- **WHEN** runs are grouped by week
- **THEN** the system SHALL display flakiness trend per-week
- **AND** SHALL highlight tests where flakiness is increasing

#### Scenario: Auto-retry configuration

- **WHEN** a test is marked as flaky (score > 10)
- **THEN** the system SHALL suggest adding `test.retries(2)` to that test
- **AND** SHALL show retry effectiveness: pass rate with/without retries
