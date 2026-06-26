## ADDED Requirements

### Requirement: Zero-Copy Network Interception

The system SHALL use Rust's zero-copy semantics for network request/response bodies, enabling GB-level payload handling without memory pressure.

#### Scenario: Zero-copy request body access

- **GIVEN** a page sends a POST/PUT request with a large payload (>10MB)
- **WHEN** the request is intercepted via CDP `Fetch.requestPaused`
- **THEN** the Rust engine SHALL access the request body via a streaming interface (not buffering into memory)
- **AND** the body data SHALL be readable as a `Stream<Item=Bytes>` (Tokio bytes)
- **AND** the body SHALL be forwardable to the destination without intermediate copies
- **AND** memory usage SHALL NOT exceed the size of the in-flight buffer (configurable, default: 64KB)

#### Scenario: Zero-copy response body passthrough

- **GIVEN** a proxied response has a large body (>100MB, e.g., video stream, WASM binary)
- **WHEN** the response is forwarded through the interception layer
- **THEN** the engine SHALL stream the response body directly from source to destination:
  - Read from CDP `Fetch.fulfillRequest` or `Fetch.continueResponse` stream
  - Transfer via splice/sendfile where available (Linux `splice(2)`, macOS `sendfile`)
  - Fall back to Tokio `copy_buf` with direct buffer reuse
- **AND** the body SHALL NOT be fully buffered in Rust or Node.js memory
- **AND** the peak memory SHALL be bounded by the stream buffer size (default: 64KB, configurable)

#### Scenario: CDP Fetch domain streaming

- **GIVEN** CDP `Fetch.requestPaused` intercepts a large request
- **WHEN** the engine calls `Fetch.continueRequest` or `Fetch.fulfillRequest`
- **THEN** the engine SHALL use CDP streaming APIs where available:
  - `Fetch.continueRequest` with `postData` streaming (Chrome 120+)
  - `Fetch.fulfillRequest` with `body` streaming via `IO` stream handle
- **AND** fall back to buffered mode if the CDP version does not support streaming
- **AND** log a warning when falling back to buffered mode for payloads >10MB

#### Scenario: Node.js FFI bridge zero-copy

- **WHEN** a response body is passed from Rust to Node.js via NAPI-rs
- **THEN** the engine SHALL use NAPI-rs `Buffer` zero-copy (transfer ownership of the Rust `Vec<u8>` to Node.js without copying)
- **AND** for large bodies (>1MB), the engine SHALL use a streaming FFI protocol:
  - Rust writes chunks to a shared memory ring buffer
  - Node.js reads chunks via async callbacks
  - No single allocation exceeds `maxChunkSize` (default: 64KB)
- **AND** the Node.js API SHALL expose this as a `ReadableStream` interface

#### Scenario: GB-level payload handling benchmark

- **GIVEN** a 1GB file is transferred through the interception layer
- **WHEN** using zero-copy mode
- **THEN** peak memory SHALL be <1MB (stream buffer only)
- **AND** throughput SHALL be >500MB/s (limited by network, not CPU/memory)
- **AND** the test SHALL complete without garbage collection pauses (>100ms pauses = failure)

#### Scenario: Compatibility with Playwright's buffer-based API

- **GIVEN** the test code calls `response.body()` or `request.postData()`
- **WHEN** the body is accessed via the synchronous/async API
- **THEN** the engine SHALL transparently buffer the streamed body into a `Vec<u8>` for the API call
- **AND** warn in debug mode if a body >10MB is fully buffered (indicating the user should use streaming API)
- **AND** the streaming API SHALL be available as `response.stream()` and `request.stream()`
