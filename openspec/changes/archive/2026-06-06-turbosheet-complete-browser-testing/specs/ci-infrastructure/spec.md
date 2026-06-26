## ADDED Requirements

### Requirement: Docker image

The system SHALL provide official Docker images for CI/CD integration.

#### Scenario: Pull official image

- **WHEN** `docker pull turbosheet/turbosheet:latest` is run
- **THEN** image downloads successfully
- **AND** contains all browser binaries

#### Scenario: Run tests in container

- **WHEN** container runs with test directory mounted
- **THEN** `tsheet test` executes successfully
- **AND** browsers run inside container

#### Scenario: Multi-arch image

- **WHEN** image is pulled on ARM64 machine
- **THEN** system runs ARM64-compatible image
- **AND** browsers work correctly

### Requirement: Browser binary caching

The system SHALL cache browser binaries for faster CI runs.

#### Scenario: Cache browser downloads

- **WHEN** `tsheet install --with-cache` is called
- **THEN** browsers download to cache directory
- **AND** subsequent runs use cached binaries

#### Scenario: GitHub Actions cache integration

- **WHEN** `actions/cache` step is configured
- **THEN** system uses cached browsers
- **AND** CI runs complete faster

### Requirement: GitHub Actions template

The system SHALL provide official GitHub Actions workflow template.

#### Scenario: Basic test workflow

- **WHEN** workflow runs `tsheet test`
- **THEN** tests execute in parallel
- **AND** results are reported to PR

#### Scenario: With caching

- **WHEN** browser cache is configured
- **THEN** workflow uses cached binaries
- **AND** saves 2-5 minutes per run

#### Scenario: With trace upload

- **WHEN** test fails
- **THEN** system uploads trace to artifact storage
- **AND** trace is available for debugging

### Requirement: GitLab CI template

The system SHALL provide official GitLab CI template.

#### Scenario: Include template

- **WHEN** `.gitlab-ci.yml` includes turbosheet template
- **THEN** pipeline runs tests on merge
- **AND** results show in MR discussion

### Requirement: Jenkins shared library

The system SHALL provide Jenkins shared library for enterprise use.

#### Scenario: Use shared library

- **WHEN** Jenkinsfile calls `turbosheet.test()`
- **THEN** tests execute with proper setup
- **AND** results are published

### Requirement: Environment configuration

The system SHALL support environment-specific configuration in CI.

#### Scenario: Headless mode

- **WHEN** `tsheet test --headed=false` is called in CI
- **THEN** browsers run in headless mode

#### Scenario: Retry on failure

- **WHEN** `tsheet test --retries=2` is called
- **THEN** failed tests retry up to 2 times
- **AND** only final result is reported

#### Scenario: Parallel workers in CI

- **WHEN** `tsheet test --workers=4` is called
- **THEN** tests run with 4 parallel workers
- **AND** respects CI resource limits
