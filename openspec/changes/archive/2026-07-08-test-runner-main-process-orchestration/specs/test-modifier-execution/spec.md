## ADDED Requirements

### Requirement: Worker collector registers modifiers correctly

The worker collector wrapper SHALL register modifiers for `test.only`, `test.fixme`, `test.fail`, `test.slow`, and `test.skip` instead of treating them as no-ops.

#### Scenario: test.only registers with only modifier

- **WHEN** user code calls `test.only("critical", async ({ page }) => { ... })`
- **THEN** the collector SHALL push `{ name: "critical", fn: <function>, modifier: "only" }` to collected tests

#### Scenario: test.fixme registers with fixme modifier

- **WHEN** user code calls `test.fixme("flaky", async ({ page }) => { ... })`
- **THEN** the collector SHALL push `{ name: "flaky", fn: <function>, modifier: "fixme" }` to collected tests

#### Scenario: test.fail registers with fail modifier

- **WHEN** user code calls `test.fail("expected-failure", async ({ page }) => { ... })`
- **THEN** the collector SHALL push `{ name: "expected-failure", fn: <function>, modifier: "fail" }` to collected tests

#### Scenario: test.slow registers with slow modifier

- **WHEN** user code calls `test.slow("slow-one", async ({ page }) => { ... })`
- **THEN** the collector SHALL push `{ name: "slow-one", fn: <function>, modifier: "slow" }` to collected tests

### Requirement: Rust plan builder processes modifiers for execution

The Rust `PlanBuilder` SHALL translate modifier flags into execution behavior in each `ExecutionPlan`.

#### Scenario: fail modifier inverts pass/fail

- **WHEN** a test with `fail` modifier passes (no error thrown)
- **THEN** the result SHALL be marked as failed with message "Expected to fail, but passed"
- **WHEN** a test with `fail` modifier throws an error
- **THEN** the result SHALL be marked as passed

#### Scenario: fixme modifier marks as fixme status

- **WHEN** a test with `fixme` modifier fails
- **THEN** the result SHALL be marked as "fixme" (not "failed")
- **AND** fixme results SHALL NOT cause the overall run to be considered failed

#### Scenario: slow modifier triples timeout

- **WHEN** a test with `slow` modifier is executed
- **THEN** the effective timeout SHALL be `config.timeout * 3`

#### Scenario: skip modifier skips execution entirely

- **WHEN** a test has `skip` modifier
- **THEN** the test SHALL be marked as skipped without execution
- **AND** no worker execution plan SHALL be sent for it

### Requirement: only modifier cascade filters active tests

The modifier cascade for `test.only` and `describe.only` SHALL filter the active test set to include only focused items.

#### Scenario: Single test.only excludes others

- **WHEN** a file has 3 tests, one with `test.only`
- **THEN** only the focused test SHALL appear in the active plan
- **AND** the other 2 tests SHALL be excluded entirely (not executed, not reported as skipped)

#### Scenario: describe.only includes all children

- **WHEN** `describe.only("Focus")` contains 2 tests and there are 3 tests outside it
- **THEN** only the 2 tests inside the focused describe SHALL appear in the active plan

#### Scenario: Mixed test.only and describe.only deduplication

- **WHEN** `describe.only("A")` contains `test("a1")` and root has `test.only("b")`
- **THEN** tests from describe A AND `test.only("b")` SHALL all appear in the active plan
