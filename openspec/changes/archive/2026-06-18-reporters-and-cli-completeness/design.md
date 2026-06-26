## Context

TurboSheet ships with 7 reporter modules (`dot`, `line`, `list`, `json`, `junit`, `github`, `html`) under `src/reporters/` and a JS-based CLI at `cli/tsheet.js`. The test runner in `src/test_runner/mod.rs` dispatches reporters via a match block in `run_tests()`.

**Known issues found during codebase audit:**

- `ReporterType::Junit` is defined in the enum and parseable, but the match arm is missing → CI integration broken
- `TestConfig` has no `headless` or `browser` field → headed mode impossible to configure
- CLI `--headed`, `--browser`, `--output`, `--update-snapshots`, `--project` flags don't exist
- File-based reporters (HTML, JUnit) write to hardcoded paths like `test-results/report.html`

**Key files:**

- `src/test_runner/mod.rs` — `run_tests()` reporter dispatch (lines 60-104)
- `src/test_runner/config.rs` — `TestConfig` napi struct
- `src/reporters/mod.rs` — `ReporterType` enum, `AggregatedTestResult`
- `src/reporters/junit.rs` — JUnit reporter (exists, never called)
- `cli/tsheet.js` — CLI binary with `parseTestArgs()`, `runTest()`, `showHelp()`

## Goals / Non-Goals

**Goals:**

- Wire JUnit reporter dispatch so CI tools can ingest results
- Add `--headed`, `--browser`, `--output`, `--update-snapshots`, `--project` CLI flags
- Add `headless` and `browser` fields to `TestConfig` napi struct
- Allow file-based reporters to write to a configurable output directory
- Update `tsheet --help` to document all flags

**Non-Goals:**

- No new reporter types (all 7 already exist)
- No Rust CLI binary rewrite (keep JS for now)
- No test registration callback fixes (separate phase)
- No config file auto-generation changes
- No changes to the runner's core execution pipeline

## Decisions

### Decision 1: JUnit dispatch — add match arm, write to file

**Option A (chosen)**: Add `ReporterType::Junit` arm after `ReporterType::Html` in the match block. Use the existing `junit.rs` API — if it outputs to stdout, also write to `{output_dir}/results.xml`.
**Option B**: Refactor JUnit reporter to use a trait method. Unnecessary — existing reporters all use static methods.
**Why A**: Minimal change (3 lines). The JUnit module compiles and works. Only dispatch is missing.

### Decision 2: `headless`/`browser` fields — nullable on TestConfig

**Option A (chosen)**: Add `headless: Option<bool>` (default: Some(true)) and `browser: Option<String>` (default: Some("chromium")) to `TestConfig`. CLI `--headed` sets `headless: false`. CLI `--browser <name>` sets `browser`.
**Option B**: Add a single `launchOptions` field. Over-engineered — the config struct already flattens options.
**Why A**: Minimal surface area. Default preserves current behavior (headless chromium).

### Decision 3: CLI `--output` maps to multiple config fields

**Option A (chosen)**: `--output <dir>` sets `screenshot_dir`, `video_dir`, and a new `output_dir` field on `TestConfig` (for reporter output). Each can still be overridden individually.
**Option B**: `--output` only controls reporter output. Confusing — users expect it to control all output.
**Why A**: Follows Playwright's `--output` convention where one flag controls the output root.

### Decision 4: Reporter output paths — extend `ReporterConfig`, passthrough via `run_tests()`

**Option A (chosen)**: Add optional `output_path` to `ReporterConfig` struct. For JUnit and HTML reporters, use this path (or a sensible default) instead of hardcoded paths.
**Option B**: Thread a global output directory through to each reporter. More invasive — `ReporterConfig` already exists.
**Why A**: Clean, minimal change. Each reporter decides whether to use it.

## Risks / Trade-offs

- **[Risk] CLI is JS, not Rust**: Adding flags to `parseTestArgs()` is straightforward but the config merge logic in `cli/tsheet.js:83-98` could get messy with more fields. → **Mitigation**: Keep `parseTestArgs()` extracting flags into a flat object that gets merged over config file values. Add a simple function to normalize kebab-case CLI args to camelCase config keys.
- **[Risk] `--headed` inverted logic**: Playwright uses `--headed` to show the browser (default is headless). If someone sets `headless: false` in config file and passes `--headed`, the CLI merge should not double-negate. → **Mitigation**: `--headed` maps to `headless: false`. If both `--headed` and config `headless: false` exist, result is `headless: false` (correct). Config `headless: true` + `--headed` → `headless: false` (correct).
- **[Risk] `--browser` value validation**: Only `chromium` works currently (Firefox/WebKit engines are stubs). → **Mitigation**: Accept and pass through the value now. When Firefox/WebKit backends are completed, they'll work without CLI changes.
- **[Risk] JUnit output path**: The current `junit.rs` may write to stdout only. → **Mitigation**: If no output path is specified, write to `test-results/results.xml` by default (directory created if missing).

## Migration Plan

No migration needed — all changes are additive:

1. Add `headless`/`browser` to `TestConfig` (non-breaking — both Optional with defaults)
2. Add CLI flags (new — don't break existing usage)
3. Wire JUnit dispatch (new — was silently skipped before)
4. Reporter output paths (new param, reporters fall back to current behavior if unset)

## Open Questions

- Should `--output` create the directory automatically? **Assume yes** — mkdir -p before writing.
- Should `--project` accept a name or index? **Assume name** — matches `projects[].name` in config.
