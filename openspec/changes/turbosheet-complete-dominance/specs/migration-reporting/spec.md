## ADDED Requirements

### Requirement: Migration Reporting Accuracy

The system SHALL produce accurate migration reports that correctly count converted files, skipped files, errors, and warnings.

#### Scenario: Migration report correctly counts converted files

- **GIVEN** the migration adapter processes a directory of test files
- **WHEN** 1 or more files are successfully converted
- **THEN** the report SHALL show the actual count of converted files (not always 0)
- **AND** SHALL list the converted file paths
- **AND** the count SHALL match the number of files that were written to the output directory

#### Scenario: Migration report correctly counts skipped files

- **GIVEN** the migration adapter encounters files that should be skipped (already converted, unsupported syntax, excluded patterns)
- **WHEN** those files are skipped
- **THEN** the report SHALL show the actual count of skipped files (not always 0)
- **AND** SHALL list each skipped file with the reason for skipping
- **AND** SHALL distinguish between: already-converted, unsupported-construct, excluded-by-pattern, and parse-error

#### Scenario: Migration report includes error details

- **GIVEN** the migration adapter encounters errors during conversion
- **WHEN** any file fails to convert
- **THEN** the report SHALL include:
  - The file path that failed
  - The error message and type
  - The line and column where the error occurred (when available)
  - A snippet of the problematic source code (when available)
- **AND** SHALL continue processing remaining files instead of aborting

#### Scenario: Migration report format consistency

- **GIVEN** a migration report is generated
- **WHEN** the report is output
- **THEN** the report SHALL include these sections:
  - Summary: total files, converted, skipped, errors, warnings, duration
  - Converted files: list of paths
  - Skipped files: list of paths with reasons
  - Errors: list of errors with file paths and details
  - Warnings: list of non-fatal issues
- **AND** SHALL be available in both human-readable (console) and machine-readable (JSON) formats
