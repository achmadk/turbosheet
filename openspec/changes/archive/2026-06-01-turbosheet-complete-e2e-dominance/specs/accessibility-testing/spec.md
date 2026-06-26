## ADDED Requirements

### Requirement: aXe accessibility engine integration

TurboSheet SHALL integrate the aXe accessibility engine for WCAG compliance scanning.

#### Scenario: Run accessibility scan

- **WHEN** user calls `await page.checkAccessibility()`
- **THEN** TurboSheet SHALL run the aXe engine against the current page state

#### Scenario: Scan results

- **WHEN** accessibility scan completes
- **THEN** results SHALL include violations, passes, incomplete, and inapplicable checks

### Requirement: Accessibility assertion

TurboSheet SHALL provide an assertion matcher for accessibility violations.

#### Scenario: Expect no violations

- **WHEN** user calls `await expect(page).toHaveNoAccessibilityViolations()`
- **THEN** test SHALL fail if any WCAG violations are found

#### Scenario: Filter violations by severity

- **WHEN** user calls `expect(page).toHaveNoAccessibilityViolations({ severity: ['critical', 'serious'] })`
- **THEN** only critical and serious violations SHALL cause test failure

#### Scenario: Custom rules

- **WHEN** user calls `expect(page).toHaveNoAccessibilityViolations({ rules: { 'color-contrast': { enabled: false } } })`
- **THEN** the specified rules SHALL be skipped

### Requirement: WCAG level configuration

Users SHALL specify WCAG compliance level (A, AA, AAA) for accessibility scans.

#### Scenario: WCAG AA compliance

- **WHEN** user sets `page.checkAccessibility({ standard: 'wcag21aa' })`
- **THEN** only WCAG 2.1 AA-level rules SHALL be checked

### Requirement: Accessibility report output

Accessibility scan results SHALL be exportable as HTML or JSON.

#### Scenario: HTML accessibility report

- **WHEN** user calls `page.checkAccessibility({ reporter: 'html' })`
- **THEN** an HTML report SHALL be generated with categorized violations and element references

#### Scenario: JSON output

- **WHEN** user calls `page.checkAccessibility({ reporter: 'json' })`
- **THEN** results SHALL be returned as a structured JSON object
