## ADDED Requirements

### Requirement: WASM compilation target

TurboSheet SHALL compile to WebAssembly for execution in edge runtime environments (Cloudflare Workers, Deno).

#### Scenario: WASM build

- **WHEN** user runs `tsheet build --target wasm`
- **THEN** TurboSheet SHALL produce a `.wasm` binary for the core orchestration engine

#### Scenario: Runtime detection

- **WHEN** the WASM binary executes in a `workerd` environment
- **THEN** it SHALL detect the runtime and use available APIs

### Requirement: Edge-compatible test orchestration

The WASM runtime SHALL support test scheduling, retries, and result aggregation at the edge.

#### Scenario: Edge test scheduling

- **WHEN** a test schedule request arrives at a Cloudflare Worker
- **THEN** the WASM runtime SHALL parse the test configuration and dispatch to grid nodes

#### Scenario: Geo-distributed execution

- **WHEN** tests are submitted to multiple edge locations
- **THEN** each edge location SHALL coordinate with the nearest grid node for browser execution

### Requirement: Reduced WASM footprint

The WASM binary SHALL be optimized for size to meet edge platform limits.

#### Scenario: WASM binary size

- **WHEN** compiled for WASM
- **THEN** the binary SHALL be under 5MB (target: 3MB) to fit Cloudflare Workers limits

#### Scenario: Tree-shaking

- **WHEN** compiling to WASM
- **THEN** unused browser engine code SHALL be excluded from the binary

### Requirement: Browser proxy via edge

The WASM runtime SHALL proxy browser connections from the edge to the nearest grid worker node.

#### Scenario: Edge-to-grid bridging

- **WHEN** a test runs at the edge
- **THEN** browser automation commands SHALL be forwarded to a grid node via a lightweight binary protocol

#### Scenario: Latency-optimized routing

- **WHEN** multiple grid nodes are available
- **THEN** the edge runtime SHALL route to the grid node with lowest latency

### Requirement: Edge-native assertion execution

Assertion polling SHALL execute in the WASM runtime at the edge, not on the grid node.

#### Scenario: Edge assertion polling

- **WHEN** `expect(page.locator('.toast')).toBeVisible()` runs at the edge
- **THEN** the polling loop SHALL execute in WASM, with only selector evaluations proxied to the browser
