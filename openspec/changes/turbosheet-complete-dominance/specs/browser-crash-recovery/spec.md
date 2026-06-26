## ADDED Requirements

### Requirement: Browser Crash Recovery

The system SHALL detect browser process crashes, recover gracefully, and retry affected tests.

#### Scenario: Detect browser crash via process exit

- **GIVEN** the browser child process is running
- **WHEN** the browser process exits unexpectedly with a non-zero exit code or signal
- **THEN** the system SHALL detect the exit within 500ms
- **AND** capture the exit code and signal (if killed by signal)
- **AND** log the crash event with: browser PID, exit code, signal, uptime, last known URL
- **AND** mark the affected page(s) as crashed

#### Scenario: Detect browser crash via CDP disconnection

- **GIVEN** a CDP connection to the browser is active
- **WHEN** the WebSocket connection drops unexpectedly
- **THEN** the system SHALL detect the disconnection within 1 second (via WebSocket ping/pong timeout)
- **AND** attempt to read the browser's stderr for crash diagnostics
- **AND** mark the affected page as crashed

#### Scenario: Auto-restart with test retry

- **GIVEN** a test is running on a page whose browser has crashed
- **WHEN** the crash is detected
- **THEN** the system SHALL:
  - Launch a new browser instance (same configuration)
  - Create a new context and page
  - Navigate to the last known URL (from url_cache or frameNavigated event)
  - Retry the failed test (up to `maxRetries` times, configurable, default: 2)
- **AND** the retry SHALL NOT count against the test's normal flakiness retries
- **AND** a crash retry SHALL be reported separately in the test output (e.g., `[CRASH RECOVERY]`)

#### Scenario: Signal handling in Rust

- **GIVEN** the browser process receives a fatal signal (SIGSEGV, SIGABRT, SIGBUS)
- **WHEN** the signal is caught
- **THEN** the Rust engine SHALL:
  - Read the browser's stderr pipe for crash diagnostics
  - Extract the crash reason from the Chromium crash log (`chrome_debug.log`)
  - Include the crash reason in the test output
- **AND** this SHALL work on Linux, macOS, and Windows
- **NOTE:** This is a unique advantage over Node.js-based tools which cannot catch native process signals reliably

#### Scenario: Graceful degradation

- **GIVEN** a browser repeatedly crashes (exceeds `maxRetries`)
- **WHEN** retries are exhausted for the same page
- **THEN** the system SHALL:
  - Mark the test as crashed (not failed — distinct status)
  - Continue running remaining tests on other pages/browsers
  - Report the crash in the final summary with count and affected tests
  - Recommend actions: check browser version, check memory usage, consider --headed mode
- **AND** NOT crash the test runner itself
- **AND** NOT lose results from already-completed tests

#### Scenario: Crash diagnostics collection

- **WHEN** a browser crash is detected
- **THEN** the system SHALL collect:
  - Browser process stdout/stderr from launch to crash
  - Chromium crash report (`~/.config/chromium/Crash Reports/` or equivalent)
  - Last 10 CDP commands sent before disconnection
  - Last known page URL and title
  - Memory usage at time of crash (from OS metrics)
- **AND** save diagnostics to `test-results/crash-diagnostics/{timestamp}/`
- **AND** include a summary in the test output
