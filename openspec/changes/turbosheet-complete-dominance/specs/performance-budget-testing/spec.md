## ADDED Requirements

### Requirement: Performance Budget Testing

The system SHALL provide native Core Web Vitals measurement and performance budget assertions.

#### Scenario: Collect Core Web Vitals

- **GIVEN** a page has loaded
- **WHEN** `page.getWebVitals()` is called
- **THEN** it SHALL return `{ lcp, inp, cls, fid, ttfb }` with values in milliseconds (except CLS which is unitless)
- **AND** LCP SHALL be measured via `PerformanceObserver` for `largest-contentful-paint`
- **AND** INP SHALL be measured via `PerformanceObserver` for `event` with `duration` and `interactionId`
- **AND** CLS SHALL be measured via `PerformanceObserver` for `layout-shift` with `value` summed
- **AND** FID SHALL be measured via `PerformanceObserver` for `first-input`
- **AND** TTFB SHALL be measured via `PerformanceNavigationTiming.responseStart`

#### Scenario: Assert performance budget

- **GIVEN** a page interaction has completed
- **WHEN** `expect(page).toPassPerformanceBudget({ lcp: 2500, cls: 0.1, inp: 200 })` is called
- **THEN** the system SHALL collect vitals and compare each metric against its threshold
- **AND** pass if ALL metrics are within budget
- **AND** fail with a detailed report showing which metrics exceeded budget and by how much
- **AND** the failure message SHALL include: metric name, measured value, threshold, delta

#### Scenario: Navigate-and-measure convenience API

- **GIVEN** a test needs to measure page load performance
- **WHEN** `page.gotoAndMeasure(url, { lcp: 2500, cls: 0.1 })` is called
- **THEN** the system SHALL navigate to the URL
- **AND** wait for the page to be fully loaded
- **AND** collect all Core Web Vitals
- **AND** assert they are within the specified budgets
- **AND** return the collected vitals object on success

#### Scenario: Performance regression threshold

- **GIVEN** a test has a historical baseline
- **WHEN** `expect(page).toPassPerformanceBudget({ regression: 0.1 })` is called with a regression threshold
- **THEN** the system SHALL compare current vitals against the stored baseline
- **AND** fail if any metric regressed by more than the threshold fraction (e.g., 10% worse)
- **AND** update the baseline on success (configurable)

#### Scenario: CI gate integration

- **GIVEN** performance budgets are configured in `tsheet.config.{ts,js}`
- **WHEN** tests run in CI mode
- **THEN** the system SHALL enforce all configured performance budgets
- **AND** exit with non-zero code if any budget is exceeded
- **AND** generate a performance report in the CI artifacts directory
- **AND** the report SHALL include pass/fail per metric, trend arrows, and comparison to previous run
