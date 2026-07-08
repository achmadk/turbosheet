## ADDED Requirements

### Requirement: PlanBuilder constructs TestPlan from ExtractResponse

The Rust side SHALL have a `PlanBuilder` that receives `ExtractResponse` from each file and constructs a unified `TestPlan` with flattened per-test execution entries, respecting modifier cascade and suite structure.

#### Scenario: PlanBuilder builds plan for simple file

- **WHEN** an `ExtractResponse` contains 3 tests with `Normal` modifier under no describe
- **THEN** `PlanBuilder` produces a `TestPlan` with 3 `ExecutionPlan` entries, each with empty suite_path and no inherited hooks

#### Scenario: PlanBuilder applies only modifier cascade

- **WHEN** an `ExtractResponse` has 2 tests with `Normal` modifier and 1 with `Only`
- **THEN** `PlanBuilder` SHALL include only the `Only` test in the active plan
- **AND** non-only tests SHALL NOT appear in the execution queue

#### Scenario: PlanBuilder applies describe.skip to all children

- **WHEN** a suite has `Skip` type
- **THEN** all tests within that suite SHALL be excluded from the active plan
- **AND** all nested child suites SHALL also be excluded

#### Scenario: PlanBuilder applies describe.only cascade

- **WHEN** `describe.only("Focused")` contains tests with `Normal` modifier
- **AND** there are other tests outside this describe or in other describes
- **THEN** only the tests inside the focused describe SHALL be included in the active plan

#### Scenario: PlanBuilder resolves hook inheritance

- **WHEN** a parent suite has `beforeEach` and a child suite also has `beforeEach`
- **THEN** `ExecutionPlan` for tests in the child suite SHALL include both hooks, parent first then child
- **WHEN** a parent suite has `beforeAll` and a child suite has tests
- **THEN** tests in the child suite SHALL have `run_before_all: true` for the parent's beforeAll

### Requirement: Executor consumes TestPlan for execution

The `TestExecutor` SHALL consume the `TestPlan` instead of raw file paths. The existing worker pool, mpsc channel, and result collection pattern SHALL remain unchanged.

#### Scenario: Executor iterates TestPlan entries

- **WHEN** `TestPlan` contains 10 `ExecutionPlan` entries across 3 files
- **THEN** the executor SHALL dispatch 10 `runPlan` calls to workers (respecting worker count), NOT 3 file-level calls

#### Scenario: Executor sends extraction requests in parallel

- **WHEN** discovering 10 test files
- **THEN** the executor SHALL send `extractTests` for all 10 files in parallel across the worker pool
- **AND** SHALL wait for all extractions to complete before building the plan

### Requirement: Suite-level serial/parallel scheduling

The orchestrator SHALL respect `describe.serial` and `describe.parallel` modifiers when scheduling test execution across workers.

#### Scenario: Serial suite tests run sequentially

- **WHEN** tests belong to a `describe.serial` block
- **THEN** those tests SHALL execute on the same worker, one at a time, in declaration order
- **AND** no other suite's tests SHALL interleave with them

#### Scenario: Parallel suite tests run concurrently

- **WHEN** tests belong to a `describe.parallel` block (or default)
- **THEN** those tests MAY execute on any available worker, in any order
