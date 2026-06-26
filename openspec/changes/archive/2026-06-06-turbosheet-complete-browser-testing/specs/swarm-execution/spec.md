## ADDED Requirements

### Requirement: Context pooling

The system SHALL pool browser contexts for reuse across tests.

#### Scenario: Reuse context from pool

- **WHEN** test needs a new context
- **THEN** system retrieves from available pool
- **AND** resets context state before use

#### Scenario: Return context to pool

- **WHEN** test completes
- **THEN** system returns context to pool
- **AND** context is not closed

#### Scenario: Pool exhaustion

- **WHEN** all contexts are in use
- **THEN** system waits for available context
- **OR** creates new context up to max pool size

### Requirement: Per-context tokio task

The system SHALL use dedicated tokio task per browser context (not thread).

#### Scenario: Task-per-context model

- **WHEN** context is created
- **THEN** system spawns dedicated tokio task
- **AND** all CDP communication happens within task

#### Scenario: High concurrency

- **WHEN** 1000 contexts are active
- **THEN** system uses 1000 tokio tasks (not threads)
- **AND** memory usage is proportional to active operations

#### Scenario: Context isolation

- **WHEN** task panics
- **THEN** system isolates panic to that context
- **AND** other contexts continue unaffected

### Requirement: Parallel test execution

The system SHALL execute tests in parallel across workers and sharding.

#### Scenario: Shard distribution

- **WHEN** `tsheet test --shard=1/5` is called
- **THEN** system executes 1/5 of total tests
- **AND** each shard processes different tests

#### Scenario: Worker parallelism

- **WHEN** `tsheet test --workers=4` is called
- **THEN** system spawns 4 worker processes
- **AND** each processes tests in parallel

#### Scenario: File-level parallelism

- **WHEN** multiple test files exist
- **THEN** system distributes files across workers
- **AND** each worker runs tests within assigned files

### Requirement: Load balancing

The system SHALL balance test distribution across workers by estimated duration.

#### Scenario: Dynamic work distribution

- **WHEN** workers report progress
- **THEN** system tracks test durations
- **AND** assigns shorter tests to faster workers

#### Scenario: Weighted distribution

- **WHEN** `tsheet test --weight-by-duration` is called
- **THEN** system uses historical test duration data
- **AND** balances workload across workers

### Requirement: Distributed execution

The system SHALL support distributing tests across multiple machines.

#### Scenario: Connect to swarm coordinator

- **WHEN** `tsheet test --swarm= coordinator:port` is called
- **THEN** system connects to swarm coordinator
- **AND** receives test assignments

#### Scenario: Register as worker

- **WHEN** `tsheet worker --coordinator=host:port` is called
- **THEN** system registers as worker
- **AND** receives test tasks
- **AND** reports results

#### Scenario: Scalability target

- **WHEN** 10,000 concurrent browser contexts are needed
- **THEN** system achieves this with swarm architecture
- **AND** single machine can handle 1000+ contexts

### Requirement: Resource limits

The system SHALL enforce per-context and total resource limits.

#### Scenario: Memory limit per context

- **WHEN** context exceeds memory limit
- **THEN** system terminates context
- **AND** reports error

#### Scenario: CPU limit per worker

- **WHEN** worker exceeds CPU threshold
- **THEN** system throttles new test starts
- **AND** logs warning

#### Scenario: Global resource cap

- **WHEN** total open contexts reaches limit
- **THEN** system queues new test requests
- **AND** prevents system overload
