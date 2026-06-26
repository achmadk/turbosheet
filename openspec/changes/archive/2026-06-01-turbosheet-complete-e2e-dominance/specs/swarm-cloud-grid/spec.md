## ADDED Requirements

### Requirement: Grid controller

TurboSheet SHALL provide a grid controller that distributes test execution across multiple worker nodes.

#### Scenario: Submit test to grid

- **WHEN** user runs `tsheet test --grid http://grid-controller:3000`
- **THEN** the test suite SHALL be submitted to the grid controller for distribution

#### Scenario: Controller job queue

- **WHEN** multiple test suites are submitted simultaneously
- **THEN** the grid controller SHALL queue jobs and distribute to available workers

#### Scenario: Worker health tracking

- **WHEN** a worker node becomes unreachable
- **THEN** the controller SHALL re-assign its jobs to other workers

### Requirement: Swarm worker node

Each worker node SHALL leverage Tokio threading to maximize context-per-server density.

#### Scenario: High-density context creation

- **WHEN** a worker node receives a batch of 500 test contexts
- **THEN** all 500 contexts SHALL be created as Tokio tasks, not OS processes

#### Scenario: Context isolation

- **WHEN** tests run on the same worker node
- **THEN** each test context SHALL be isolated (separate storage, cookies, cache)

### Requirement: Auto-scaling

The swarm grid SHALL support auto-scaling based on queue depth.

#### Scenario: Scale up

- **WHEN** queue depth exceeds 100 pending jobs
- **THEN** the controller SHALL launch additional worker nodes (configurable)

#### Scenario: Scale down

- **WHEN** queue has been empty for 5 minutes
- **THEN** the controller SHALL terminate idle workers (configurable)

### Requirement: Sharded test distribution

The grid SHALL distribute sharded test chunks across workers.

#### Scenario: Auto-shard

- **WHEN** 1000 tests are submitted to a grid with 10 workers
- **THEN** each worker SHALL receive approximately 100 tests (auto-sharded)

### Requirement: Result aggregation

The grid controller SHALL aggregate results from all workers into a unified report.

#### Scenario: Aggregated report

- **WHEN** all workers complete their shards
- **THEN** the controller SHALL produce a single aggregated HTML report with all results

#### Scenario: Partial failure reporting

- **WHEN** some workers fail or time out
- **THEN** the controller SHALL include partial results and report worker failures
