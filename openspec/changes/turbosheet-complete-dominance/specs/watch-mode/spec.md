## ADDED Requirements

### Requirement: Watch Mode

The system SHALL automatically re-run affected tests when source files change.

#### Scenario: Starting watch mode

- **WHEN** `npx tsheet test --watch` is executed
- **THEN** the system SHALL start a file watcher on the project directory
- **AND** SHALL run the full test suite on startup
- **AND** SHALL wait for file changes after the suite completes

#### Scenario: File change detection

- **WHEN** a `.ts`, `.js`, `.tsx`, `.jsx`, or `.vue` file changes in the project
- **THEN** the system SHALL detect the change via file system events
- **AND** SHALL debounce for 300ms before triggering a re-run

#### Scenario: Scoped re-run

- **WHEN** a specific test file changes
- **THEN** the system SHALL only re-run that test file (not the entire suite)
- **WHEN** a source file under `src/` changes
- **THEN** the system SHALL re-run all test files that import from that source (if source→test mapping exists)
- **AND** SHALL fall back to full suite run if mapping is unavailable

#### Scenario: Watch mode output

- **WHEN** a re-run is triggered
- **THEN** the system SHALL display which file change triggered the re-run
- **AND** SHALL display the re-run scope (single file, multiple files, or full suite)
- **AND** SHALL display test results inline

#### Scenario: Manual re-run

- **WHEN** the user presses `Ctrl+R` in watch mode
- **THEN** the system SHALL re-run the last test suite
- **AND** SHALL display a single-file re-run (last changed file) if available
