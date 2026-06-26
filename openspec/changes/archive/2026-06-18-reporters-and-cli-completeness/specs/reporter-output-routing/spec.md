## ADDED Requirements

### Requirement: File-based reporters accept output path override

Reporters that write files (HTML, JUnit) SHALL accept an optional output path parameter. When provided, the reporter SHALL write its output to that path instead of the default location. When not provided, the reporter SHALL use its current default path.

#### Scenario: HTML reporter writes to custom path

- **WHEN** HTML reporter is invoked with output path `"./custom/report.html"`
- **THEN** the HTML file SHALL be written to `"./custom/report.html"`

#### Scenario: HTML reporter uses default path

- **WHEN** HTML reporter is invoked without an output path
- **THEN** the HTML file SHALL be written to `"test-results/report.html"` (current default)

### Requirement: JUnit reporter writes XML output

The JUnit reporter SHALL generate an XML file conforming to the JUnit XML schema, suitable for ingestion by CI systems (Jenkins, GitLab CI, GitHub Actions).

#### Scenario: JUnit reporter invoked via --reporter junit

- **WHEN** user runs tests with `--reporter junit` or `reporter: "junit"` in config
- **THEN** the JUnit reporter SHALL produce XML output
- **THEN** the XML SHALL be written to `"test-results/results.xml"` by default

#### Scenario: JUnit reporter with custom output path

- **WHEN** user runs tests with `--reporter junit` and `--output ./ci-reports`
- **THEN** the JUnit XML SHALL be written to `"./ci-reports/results.xml"`

#### Scenario: JUnit XML contains test suite and case elements

- **WHEN** tests complete with mixed results (pass, fail, skip)
- **THEN** the XML SHALL contain a `<testsuite>` element with `tests`, `failures`, `skipped` attributes
- **THEN** each test SHALL have a `<testcase>` element with `name`, `classname`, `time` attributes
- **THEN** failed tests SHALL contain a `<failure>` child element with message and details

### Requirement: Reporter dispatch includes JUnit in run_tests()

The `run_tests()` function in `src/test_runner/mod.rs` SHALL include a match arm for `ReporterType::Junit` that invokes the JUnit reporter.

#### Scenario: junit reporter is dispatched

- **WHEN** user configures `reporter: "junit"`
- **THEN** `ReporterType::Junit` SHALL be matched in the dispatch block
- **THEN** the JUnit reporter SHALL produce output

#### Scenario: multiple reporters including junit

- **WHEN** user configures `reporter: "html,junit"`
- **THEN** both HTML and JUnit reporters SHALL produce output
