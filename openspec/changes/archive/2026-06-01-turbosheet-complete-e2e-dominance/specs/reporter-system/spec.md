## ADDED Requirements

### Requirement: HTML reporter

TurboSheet SHALL generate a self-contained HTML report with test results, screenshots, and embedded trace viewer.

#### Scenario: Generate HTML report

- **WHEN** user runs `tsheet test --reporter html`
- **THEN** an HTML file SHALL be generated at `test-results/report.html`

#### Scenario: Report structure

- **WHEN** user opens the HTML report
- **THEN** it SHALL show: overall pass/fail summary, test list with status, duration per test, failure screenshots, and error messages

#### Scenario: Failed test trace link

- **WHEN** a test fails and traces are enabled
- **THEN** the HTML report SHALL include a "View Trace" link that opens the embedded trace viewer

### Requirement: JSON reporter

TurboSheet SHALL output results as JSON for CI pipeline ingestion.

#### Scenario: JSON output

- **WHEN** user runs `tsheet test --reporter json`
- **THEN** TurboSheet SHALL output a JSON object with all test results to stdout

#### Scenario: JSON structure

- **WHEN** JSON output is captured
- **THEN** it SHALL include: `{ suites, tests, passes, failures, duration, results[] }` with each result containing `{ title, status, duration, error, screenshotPaths }`

### Requirement: JUnit reporter

TurboSheet SHALL output JUnit XML for Jenkins, GitLab, and other CI systems.

#### Scenario: JUnit XML output

- **WHEN** user runs `tsheet test --reporter junit`
- **THEN** TurboSheet SHALL write a `junit.xml` file in standard JUnit format

### Requirement: GitHub Annotations reporter

TurboSheet SHALL output GitHub Workflow Command annotations for inline PR review comments.

#### Scenario: Annotations on failure

- **WHEN** tests run in GitHub Actions with `--reporter github`
- **THEN** failed tests SHALL appear as inline annotations on the PR diff

#### Scenario: Error message in annotation

- **WHEN** a test fails
- **THEN** the annotation SHALL include the test name, file path, line number, and error message

### Requirement: Terminal reporters

TurboSheet SHALL provide `dot` (compact) and `line` (verbose) terminal reporters.

#### Scenario: Dot reporter

- **WHEN** user runs `tsheet test --reporter dot`
- **THEN** each passing test SHALL print a green `.` and each failure a red `F`

#### Scenario: Line reporter

- **WHEN** user runs `tsheet test --reporter line`
- **THEN** each test SHALL print its full name and status on a separate line

### Requirement: Multiple reporters

TurboSheet SHALL support multiple simultaneous reporters.

#### Scenario: CI-friendly multi-reporter

- **WHEN** user runs `tsheet test --reporter html --reporter json --reporter junit`
- **THEN** all three report files SHALL be generated simultaneously
