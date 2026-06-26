## ADDED Requirements

### Requirement: Auto-scaling for swarm grid workers

The GridController SHALL automatically scale the worker pool up or down based on queue depth, respecting configured min and max worker limits.

#### Scenario: Scale up on queue depth increase

- **WHEN** the job queue depth exceeds `scaleUpThreshold` (configurable, default 10)
- **THEN** the controller SHALL spawn additional worker processes up to `maxWorkers`
- **AND** the scale-up SHALL happen within 5 seconds of the threshold being crossed

#### Scenario: Scale down on idle

- **WHEN** the job queue has been empty for `scaleDownIdleDuration` (configurable, default 60s)
- **AND** current worker count is above `minWorkers`
- **THEN** the controller SHALL gracefully terminate idle workers (one at a time, with 30s cooldown)
- **AND** in-progress jobs on a terminating worker SHALL be allowed to complete

#### Scenario: Scale configuration from swarm config

- **WHEN** the swarm grid starts
- **THEN** it SHALL read `minWorkers`, `maxWorkers`, `scaleUpThreshold`, and `scaleDownIdleDuration` from the swarm configuration

### Requirement: Worker health monitoring and dead worker reassignment

The HealthMonitor SHALL periodically check worker liveness and reassign shards from dead workers to healthy ones.

#### Scenario: Health monitor detects dead worker

- **WHEN** a worker fails to respond to heartbeats for `healthCheckInterval * maxMissedHeartbeats` (configurable)
- **THEN** the HealthMonitor SHALL mark the worker as dead
- **AND** all shards assigned to the dead worker SHALL be re-assigned to live workers
- **AND** queued jobs on the dead worker SHALL be re-queued with retry count incremented
- **AND** a health event SHALL be emitted to the metrics system

#### Scenario: Heartbeat protocol

- **WHEN** the controller sends a heartbeat request to a worker
- **THEN** the worker SHALL respond within `heartbeatTimeout` (default 5s) with its current status (busy, idle, draining)
- **AND** the controller SHALL update the worker's lastHeartbeat timestamp on receipt

### Requirement: Docker-based multi-tenant isolation

The Tenant manager SHALL support creating isolated Docker containers for each tenant, with configurable resource limits and cleanup.

#### Scenario: Docker tenant container created

- **WHEN** a new tenant is registered
- **THEN** a Docker container SHALL be created using the configured base image
- **AND** the container SHALL have CPU/memory limits applied from tenant config
- **AND** the controller SHALL route test jobs to the appropriate tenant container

#### Scenario: Tenant container cleanup

- **WHEN** a tenant is deactivated or removed
- **THEN** the tenant's Docker container SHALL be stopped and removed
- **AND** any associated resources (volumes, networks) SHALL be cleaned up
