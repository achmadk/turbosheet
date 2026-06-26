## ADDED Requirements

### Requirement: Official Docker images

TurboSheet SHALL provide official Docker images with all browsers pre-installed.

#### Scenario: Pull default image

- **WHEN** user runs `docker pull tsheet/tsheet:latest`
- **THEN** the image SHALL contain the TurboSheet binary + Chromium, Firefox, and WebKit

#### Scenario: Browser-specific images

- **WHEN** user runs `docker pull tsheet/tsheet:chromium-only`
- **THEN** the image SHALL contain only Chromium (smaller image for CI)

#### Scenario: Run tests in Docker

- **WHEN** user runs `docker run tsheet/tsheet tsheet test`
- **THEN** tests SHALL execute inside the container with all browsers available

### Requirement: CI provider templates

TurboSheet SHALL provide pre-built CI configuration templates for GitHub Actions and GitLab CI.

#### Scenario: GitHub Actions workflow

- **WHEN** user runs `tsheet init --ci github`
- **THEN** TurboSheet SHALL generate a `.github/workflows/tsheet.yml` file with cached browser binaries

#### Scenario: GitLab CI template

- **WHEN** user runs `tsheet init --ci gitlab`
- **THEN** TurboSheet SHALL generate a `.gitlab-ci.yml` file

#### Scenario: Parallel CI sharding

- **WHEN** the generated CI workflow runs with 4 shards
- **THEN** each shard SHALL run `tsheet test --shard=1/4` through `--shard=4/4`

### Requirement: Multi-architecture builds

Docker images SHALL support `linux/amd64` and `linux/arm64`.

#### Scenario: ARM CI runner

- **WHEN** user pulls the image on an ARM64 GitHub Actions runner
- **THEN** the correct architecture image SHALL be used automatically

### Requirement: Browser cache in CI

CI workflows SHALL cache browser binaries between runs to reduce installation time.

#### Scenario: Cached browser install

- **WHEN** CI runs a second time
- **THEN** browser binaries SHALL be restored from cache instead of re-downloaded
