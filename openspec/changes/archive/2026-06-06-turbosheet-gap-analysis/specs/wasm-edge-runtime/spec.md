## ADDED Requirements

### Requirement: WASM compilation target

The project SHALL support building a wasm32-wasi compilation target for the test orchestrator subset, excluding browser engine code.

#### Scenario: WASM binary builds successfully

- **WHEN** `cargo build --target wasm32-wasi --features wasm` is executed
- **THEN** it SHALL produce a .wasm binary under 3MB in size
- **AND** the binary SHALL contain: test scheduler, retry logic, result aggregation, binary protocol encoding/decoding
- **AND** the binary SHALL NOT contain: chromiumoxide, WebDriver client, headless browser launching code

### Requirement: WASM-compatible test orchestrator

The WASM binary SHALL implement a test orchestrator that schedules tests, manages retries, and aggregates results — identical in behavior to the Rust test executor but running in edge environments (workerd, deno).

#### Scenario: Test scheduling in WASM

- **WHEN** tests are submitted to the WASM orchestrator
- **THEN** the orchestrator SHALL schedule them for execution on connected grid nodes
- **AND** it SHALL respect retry, timeout, and shard configuration

#### Scenario: Result aggregation from grid nodes

- **WHEN** grid nodes complete test execution and send results
- **THEN** the WASM orchestrator SHALL aggregate results, produce summary statistics, and return a unified report

### Requirement: Edge-to-grid binary protocol

The WASM orchestrator SHALL communicate with grid nodes using a compact binary protocol over WebSocket, using MessagePack encoding.

#### Scenario: Binary protocol message exchange

- **WHEN** the WASM orchestrator sends a test job to a grid node
- **THEN** the message SHALL be encoded as MessagePack: `{ type: "run_test", id: uuid, payload: { file, config } }`
- **AND** the grid node SHALL respond with MessagePack: `{ type: "test_result", id: uuid, payload: { passed, duration, error?, screenshot? } }`

### Requirement: Runtime detection

The WASM binary SHALL detect its runtime environment (workerd, deno, or native) and adapt its I/O strategy accordingly.

#### Scenario: Runtime detection and adaptation

- **WHEN** the WASM module initializes in workerd
- **THEN** it SHALL use WebSocket API for grid communication
- **WHEN** the WASM module initializes in deno
- **THEN** it SHALL use Deno.connect for TCP grid communication
