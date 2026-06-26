## ADDED Requirements

### Requirement: Core Web Vitals Measurement

The system SHALL measure Core Web Vitals (LCP, INP, CLS, FID, TTFB) via the PerformanceObserver API.

#### Scenario: Collecting vitals

- **WHEN** `page.getWebVitals()` is called
- **THEN** the system SHALL inject a script that registers PerformanceObserver for LCP, INP, CLS, FID, and TTFB
- **AND** SHALL return a structured report with metric name, value, unit, and rating (good/needs-improvement/poor)

#### Scenario: LCP measurement

- **WHEN** the Largest Contentful Paint occurs
- **THEN** the system SHALL record the LCP value in milliseconds
- **AND** SHALL rate: <2500ms "good", 2500-4000ms "needs improvement", >4000ms "poor"

#### Scenario: CLS measurement

- **WHEN** the Cumulative Layout Shift score stabilizes
- **THEN** the system SHALL record the CLS score
- **AND** SHALL rate: <0.1 "good", 0.1-0.25 "needs improvement", >0.25 "poor"

#### Scenario: INP measurement

- **WHEN** the Interaction to Next Paint is observed
- **THEN** the system SHALL record the INP value in milliseconds
- **AND** SHALL rate: <200ms "good", 200-500ms "needs improvement", >500ms "poor"

#### Scenario: Vitals assertion

- **WHEN** `expect(page).toPassVitals({ thresholds: { lcp: 2500, cls: 0.1, inp: 200 } })` is called
- **THEN** the system SHALL wait for all vitals to settle (page idle for 5s)
- **AND** SHALL fail the assertion if any vital exceeds its threshold
