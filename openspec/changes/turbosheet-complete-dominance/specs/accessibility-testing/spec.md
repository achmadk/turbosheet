## ADDED Requirements

### Requirement: Accessibility Testing with axe-core

The system SHALL integrate with axe-core to provide automated accessibility violation detection.

#### Scenario: Running accessibility audit

- **WHEN** `expect(page).toPassAxe()` is called
- **THEN** the system SHALL inject the axe-core runner script into the page
- **AND** SHALL execute a full accessibility audit
- **AND** SHALL pass if no violations are found

#### Scenario: Impact level filtering

- **WHEN** `expect(page).toPassAxe({ impact: ['critical', 'serious'] })` is called
- **THEN** the system SHALL only fail on violations with "critical" or "serious" impact
- **AND** SHALL report but not fail on "moderate" or "minor" violations

#### Scenario: Rule-specific audit

- **WHEN** `expect(page).toPassAxe({ rules: ['color-contrast'] })` is called
- **THEN** the system SHALL only run the specified rules
- **AND** SHALL ignore all other axe-core rules

#### Scenario: Violation reporting

- **WHEN** an accessibility violation is found
- **THEN** the test report SHALL include: rule ID, impact level, description, element selector, ARIA reference link, and suggested fix

#### Scenario: Accessibility snapshot regression

- **WHEN** `expect(page).toPassAxe({ snapshot: true })` is called
- **THEN** the system SHALL store the violations list as a snapshot
- **AND** SHALL fail on subsequent runs if new violations are introduced
- **AND** SHALL pass if violations match the stored snapshot
