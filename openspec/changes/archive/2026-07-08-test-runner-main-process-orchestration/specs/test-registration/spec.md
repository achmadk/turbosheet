## ADDED Requirements

### Requirement: napi test() functions register into TestRegistry

The existing `#[napi]` exported `test()`, `test.only()`, `test.skip()`, `test.fixme()`, `test.fail()`, `test.slow()` functions in `mod.rs` SHALL continue to register tests into the static `TestRegistry` with correct modifiers and suite context. No changes needed — this already works.

#### Scenario: test with modifier registers correctly

- **WHEN** user code calls any `test.*()` variant from the native module
- **THEN** a `RegisteredTest` is created in the `TestRegistry` with the corresponding `TestModifier`
- **AND** the test is associated with the current suite (if inside a describe block)

### Requirement: napi describe() functions manage suite stack

The existing `#[napi]` exported `describe()`, `describe.serial()`, `describe.parallel()`, `describe.skip()`, `describe.only()` functions SHALL continue to manage the suite stack in `TestRegistry`. No changes needed — this already works.

#### Scenario: describe nesting maintains correct suite stack

- **WHEN** nested describe blocks are called
- **THEN** `begin_suite()` pushes a new suite onto the stack
- **AND** `end_suite()` pops the stack
- **AND** tests registered between begin/end are associated with the correct suite

### Requirement: TestRegistry feeds into PlanBuilder

The `TestRegistry` data SHALL be consumable by the `PlanBuilder` to construct the `TestPlan`. This is the NEW integration point — the registry data was previously orphaned.

#### Scenario: PlanBuilder reads from TestRegistry

- **WHEN** `PlanBuilder.build()` is called with a reference to `TestRegistry`
- **THEN** it SHALL read `get_suites()`, `get_tests()`, and `get_hooks()` to construct the plan
- **AND** it SHALL apply `collect_active_tests()` logic for modifier cascade

#### Scenario: PlanBuilder handles per-run registry

- **WHEN** `run_tests()` is called
- **THEN** the registry SHALL be cleared at the start (`reg.clear()`)
- **AND** repopulated via napi calls during test file loading
- **AND** consumed by `PlanBuilder` before the next `clear()`
