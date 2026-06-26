## ADDED Requirements

### Requirement: Automatic Browser Installation

The system SHALL automatically download and install Chrome for Testing (CfT) when not found on the system.

#### Scenario: Auto-download on launch

- **WHEN** `browser.launch()` is called and Chromium is not installed
- **THEN** the system SHALL query `https://googlechromelabs.github.io/chrome-for-testing/known-good-versions.json` for the latest stable version
- **AND** SHALL download the appropriate binary for the current platform (Linux, macOS, Windows)
- **AND** SHALL extract and cache the browser in `~/.cache/turbosheet/browsers/`
- **AND** SHALL launch the browser after download completes

#### Scenario: Manual install via CLI

- **WHEN** `npx tsheet install` is run
- **THEN** the system SHALL download Chrome for Testing
- **AND** SHALL output progress percentage during download
- **AND** SHALL verify the binary after download (check file hash, test launch)

#### Scenario: Cache management

- **WHEN** a browser version is already cached
- **THEN** the system SHALL skip download and use the cached version
- **WHEN** `npx tsheet install --force` is run
- **THEN** the system SHALL re-download even if cached

#### Scenario: Multi-platform support

- **WHEN** downloading on Linux
- **THEN** SHALL download the Linux binary
- **WHEN** downloading on macOS
- **THEN** SHALL download the macOS binary (both Intel and Apple Silicon as needed)
- **WHEN** downloading on Windows
- **THEN** SHALL download the Windows binary
