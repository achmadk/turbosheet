## ADDED Requirements

### Requirement: Error Handling Foundation

The system SHALL provide comprehensive, consistent error handling across all Rust-to-JS boundaries, including complete `From` trait implementations, panic safety, and graceful failure paths.

#### Scenario: TurbosheetError supports From<serde_json::Error>

- **GIVEN** `TurbosheetError` enum is defined in `src/error.rs`
- **WHEN** any operation uses `serde_json` to parse or serialize data
- **THEN** `From<serde_json::Error>` SHALL be implemented for `TurbosheetError`
- **AND** SHALL map deserialization errors to an appropriate variant (e.g., `InternalError` or a new `DeserializationError`)
- **AND** SHALL include the error message and location context
- **AND** SHALL eliminate the need for `.map_err(|e| TurbosheetError::InternalError(e.to_string()))` patterns

#### Scenario: TurbosheetError supports From<url::ParseError>

- **GIVEN** `TurbosheetError` enum is defined in `src/error.rs`
- **WHEN** a URL string is parsed with the `url` crate
- **THEN** `From<url::ParseError>` SHALL be implemented for `TurbosheetError`
- **AND** SHALL map to an appropriate variant (e.g., `InvalidArgument` or a new `UrlParseError`)
- **AND** SHALL include the malformed URL and parse error details

#### Scenario: TurbosheetError supports From<reqwest::Error>

- **GIVEN** `TurbosheetError` enum is defined in `src/error.rs`
- **WHEN** an HTTP request is made using `reqwest`
- **THEN** `From<reqwest::Error>` SHALL be implemented for `TurbosheetError`
- **AND** SHALL map to an appropriate variant (e.g., `NetworkError`)
- **AND** SHALL preserve the HTTP status code and response body when available
- **AND** SHALL handle both request failures and response errors

#### Scenario: Panic hook installed at library initialization

- **GIVEN** the native Rust library is loaded by Node.js
- **WHEN** the library initializes (in `src/lib.rs` or equivalent entry point)
- **THEN** the system SHALL install a custom panic hook via `std::panic::set_hook()`
- **AND** the hook SHALL capture the panic message and backtrace
- **AND** SHALL format the panic as a proper JavaScript error instead of crashing the Node process
- **AND** SHALL include: panic message, file, line number, and Rust backtrace
- **AND** SHALL throw or reject with a `TurbosheetError::InternalError` containing the panic details

#### Scenario: CDP port bind failure returns error instead of panicking

- **GIVEN** `JsBrowser::new()` attempts to bind to a CDP WebSocket port
- **WHEN** the port is already in use or unavailable
- **THEN** the system SHALL NOT panic/unwrap on the port bind operation
- **AND** SHALL return a `TurbosheetError::BrowserLaunchError` with the port conflict details
- **AND** SHALL suggest trying the next available port or using a configured port range
- **AND** SHALL NOT crash the Node.js process

#### Scenario: Unwrap elimination on all fallible operations

- **GIVEN** the codebase contains `.unwrap()` calls on fallible operations
- **WHEN** any such call can fail at runtime
- **THEN** each `.unwrap()` SHALL be replaced with proper error propagation
- **AND** SHALL use `?` operator with appropriate `From` impls
- **AND** SHALL use `.context()` or `.map_err()` with descriptive error messages where `From` impls are not available
- **AND** SHALL handle the following known unwrap/expect sites:
  - CDP port bind in `JsBrowser::new()`
  - JSON parsing in CDP event handling
  - URL parsing throughout the codebase
  - HashMap conversion from JsObject in migration adapters
  - File I/O in binary manager and screencast paths
