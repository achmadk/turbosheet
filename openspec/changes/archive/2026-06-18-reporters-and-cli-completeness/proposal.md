## Why

TurboSheet's reporter system has 7 modules but only 6 are wired into the test runner — JUnit XML dispatch is silently skipped, making CI integration impossible. The JS-based CLI is missing critical flags (`--headed`, `--browser`, `--output`, `--update-snapshots`, `--project`) that every competitor ships, and the native `TestConfig` struct lacks corresponding fields for headed mode and browser selection. Without these, TurboSheet cannot be used in real CI pipelines or multi-browser test suites.

## What Changes

- **JUnit reporter dispatch**: Add the missing `ReporterType::Junit` match arm in `run_tests()` so JUnit XML is written to stdout/file
- **Headed/browser mode**: Add `headless` and `browser` fields to `TestConfig` napi struct, wire through CLI `--headed` and `--browser` flags
- **Output directory flag**: Add `--output` CLI flag mapped to existing `screenshot_dir`/`video_dir` and reporter output paths
- **Update snapshots flag**: Add `--update-snapshots` CLI flag mapped to existing `update_snapshots` config field
- **Project selection flag**: Add `--project` CLI flag for multi-project config support (wires into existing `projects` field on `TestConfig`)
- **Reporter output paths**: Allow reporters that write files (HTML, JUnit, JSON) to accept an `--output-dir` for where reports are written
- **CLI help consistency**: Update `tsheet help` to document all flags

## Capabilities

### New Capabilities

- `cli-flag-completeness`: Parity CLI flags for headed mode, browser selection, output directory, snapshot updates, and project selection — matching Playwright's test CLI surface
- `headed-browser-config`: Native `TestConfig` fields for `headless` (bool) and `browser` (enum: chromium/firefox/webkit) with CLI passthrough
- `reporter-output-routing`: Per-reporter output path support so file-based reporters (HTML, JUnit, JSON) write to a configurable directory instead of hardcoded paths

### Modified Capabilities

- `test-reporter-list` (from phase-1-test-runner-core): Extend reporter dispatch to include JUnit, and add `--output-dir` support for all file-based reporters
- `cli-commands` (implicit — existing in `cli/tsheet.js`): Add `--headed`, `--browser`, `--output`, `--update-snapshots`, `--project` flags to `tsheet test`

## Impact

- `src/test_runner/mod.rs`: Add `ReporterType::Junit` match arm in `run_tests()` (~3 lines)
- `src/test_runner/config.rs`: Add `headless: Option<bool>` and `browser: Option<String>` fields to `TestConfig` struct
- `cli/tsheet.js`: Add 5 new flags to `parseTestArgs()`, update `showHelp()`, pass new config through to `run_tests()`
- `src/reporters/junit.rs`: Verify existing reporter supports configurable output path (add if missing)
- `src/reporters/html.rs`: Accept optional output path override
- `src/reporters/mod.rs`: If needed, extend `ReporterConfig` with `output_path` field
- `index.js`: No changes needed — JS wrapper passes through config as-is
- `index.d.ts`: Add `headless` and `browser` type declarations if `TestConfig` is exposed
