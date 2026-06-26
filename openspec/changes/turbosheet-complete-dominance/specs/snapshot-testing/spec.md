## ADDED Requirements

### Requirement: Snapshot Testing (Textual)

The system SHALL support snapshot testing for verifying output against stored reference values.

#### Scenario: Creating a snapshot

- **WHEN** `expect(page.textContent()).toMatchSnapshot('homepage-title')` is called during the first run
- **THEN** the system SHALL create a snapshot file with the current value
- **AND** SHALL mark the test as passed (generated baseline)

#### Scenario: Matching a snapshot

- **WHEN** `expect(value).toMatchSnapshot('homepage-title')` is called on subsequent runs
- **THEN** the system SHALL compare the current value against the stored snapshot
- **AND** SHALL pass if they match exactly
- **AND** SHALL fail with a diff if they don't match

#### Scenario: Inline snapshots

- **WHEN** `expect(value).toMatchSnapshot()` is called without a name (inline mode)
- **THEN** the system SHALL embed the snapshot value directly in the test source file
- **AND** SHALL update the source file on snapshot update

#### Scenario: Updating snapshots

- **WHEN** `npx tsheet test --update-snapshots` is run
- **THEN** the system SHALL update all snapshots to the current values
- **AND** SHALL mark tests as passed even if values changed

#### Scenario: CI mode

- **WHEN** `CI=true` is set and a snapshot doesn't match
- **THEN** the system SHALL fail the test
- **AND** SHALL output the diff and the `--update-snapshots` command to run
