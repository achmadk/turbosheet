## ADDED Requirements

### Requirement: AI-Powered Flake Diagnosis

The system SHALL automatically classify test failures and suggest fixes with confidence scores.

#### Scenario: Failure classification

- **WHEN** a test fails
- **THEN** the system SHALL classify the failure into one of: Timeout, Assertion Mismatch, Network Error, Browser Crash, Element Not Found, Navigation Failure, or Unknown
- **AND** SHALL compute a confidence score (0.0-1.0) for the classification

#### Scenario: Fix suggestion generation

- **WHEN** a failure is classified as Element Not Found
- **THEN** the system SHALL suggest: "Increase timeout or verify selector exists in current page state"
- **WHEN** a failure is classified as Network Error
- **THEN** the system SHALL suggest: "Check if the server is running or use route mocking"
- **WHEN** a failure is classified as Browser Crash
- **THEN** the system SHALL suggest: "Increase available memory or reduce parallel context count"

#### Scenario: Flake score computation

- **WHEN** the same test has been run multiple times across CI
- **THEN** the system SHALL compute a flake score: (failure count / total runs) × 100
- **AND** SHALL mark tests with score > 10 as "flaky"

#### Scenario: Report annotation

- **WHEN** a test report is generated
- **THEN** each failure SHALL include the classification, confidence score, and suggested fix
- **AND** SHALL include the flake score for tests that have been run multiple times
