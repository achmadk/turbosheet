## 1. TestConfig — add headless, browser, and output_dir fields

- [x] 1.1 Add `headless: Option<bool>` field to `TestConfig` napi struct in `src/test_runner/config.rs`
- [x] 1.2 Add `browser: Option<String>` field to `TestConfig` napi struct in `src/test_runner/config.rs`
- [x] 1.3 Add `output_dir: Option<String>` field to `TestConfig` napi struct in `src/test_runner/config.rs`
- [x] 1.4 Verify `cargo check` compiles with new fields (napi-rs handles Option as nullable JS prop)

## 2. JUnit reporter — wire dispatch in run_tests()

- [x] 2.1 Add `ReporterType::Junit` match arm in `run_tests()` in `src/test_runner/mod.rs` (after Html arm, around line 92)
- [x] 2.2 In the Junit arm, call `JunitReporter::write(&aggregated)`, write to `test-results/results.xml` by default, and write to stdout when no output_dir is specified
- [x] 2.3 Verify JUnit reporter dispatch works — run `cargo check` to confirm compilation

## 3. Reporter output paths — extend file-based reporters

- [x] 3.1 Add optional `output_path` to `ReporterConfig` in `src/reporters/mod.rs`
- [x] 3.2 Update HTML reporter dispatch to accept `output_dir` from config instead of hardcoded `"test-results/report.html"`
- [x] 3.3 Update JUnit reporter dispatch to write to `{output_dir}/results.xml` when `output_dir` is set
- [x] 3.4 Ensure `run_tests()` passes `output_dir` to file-based reporters

## 4. CLI flags — add to tsheet.js

- [x] 4.1 Add `--headed` parsing in `parseTestArgs()` — maps to `headless: false`
- [x] 4.2 Add `--browser <type>` parsing in `parseTestArgs()`
- [x] 4.3 Add `--output <dir>` parsing in `parseTestArgs()` — sets outputDir, screenshotDir, videoDir
- [x] 4.4 Add `--update-snapshots` parsing in `parseTestArgs()` — maps to `updateSnapshots: true`
- [x] 4.5 Add `--project <name>` parsing in `parseTestArgs()`
- [x] 4.6 Update `runTest()` to pass all new config fields through to `run_tests()` call (around lines 108-117)
- [x] 4.7 Update `showHelp()` to document all 5 new flags in the Test Options section (around lines 65-71)

## 5. Validation

- [x] 5.1 Run `cargo check` — ensure all Rust changes compile cleanly
- [x] 5.2 Verify CLI help output shows all new flags via `node cli/tsheet.js help`
- [x] 5.3 Verify `--reporter junit` is parseable and doesn't error (even if no tests run)
- [x] 5.4 Smoke test: `node cli/tsheet.js test --headed --browser chromium --reporter list` works without errors
