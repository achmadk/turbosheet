## ADDED Requirements

### Requirement: BeforeAll Hook Execution

`test.beforeAll()` hooks SHALL run once before any test in the describe block.

#### Scenario: BeforeAll runs before tests

- **WHEN** `test.describe("suite", () => { test.beforeAll(() => { ... }); test("t1", ...); test("t2", ...); })`
- **THEN** the beforeAll callback SHALL execute exactly once
- **AND** SHALL complete before "t1" begins

#### Scenario: BeforeAll failure prevents tests

- **WHEN** `test.beforeAll()` throws an error
- **THEN** all tests in that describe block SHALL be skipped
- **AND** tests in nested describe blocks SHALL also be skipped

### Requirement: AfterAll Hook Execution

`test.afterAll()` hooks SHALL run once after all tests in the describe block complete.

#### Scenario: AfterAll runs after tests

- **WHEN** `test.describe("suite", () => { test.afterAll(() => { ... }); test("t1", ...); test("t2", ...); })`
- **THEN** the afterAll callback SHALL execute exactly once
- **AND** SHALL run after both "t1" and "t2" complete (pass, fail, or skipped)

#### Scenario: AfterAll runs even if beforeAll failed

- **WHEN** `test.beforeAll()` throws and tests are skipped
- **THEN** `test.afterAll()` SHALL still execute

### Requirement: BeforeEach Hook Execution

`test.beforeEach()` hooks SHALL run before each test in the describe block.

#### Scenario: BeforeEach runs before each test

- **WHEN** `test.describe("suite", () => { test.beforeEach(() => { ... }); test("t1", ...); test("t2", ...); })`
- **THEN** beforeEach SHALL execute before "t1"
- **AND** beforeEach SHALL execute before "t2"

#### Scenario: BeforeEach receives test info

- **WHEN** `test.beforeEach(async ({ page }) => { ... })` is defined
- **THEN** the hook SHALL receive the page fixture for the current test

### Requirement: AfterEach Hook Execution

`test.afterEach()` hooks SHALL run after each test in the describe block.

#### Scenario: AfterEach runs after each test

- **WHEN** `test.describe("suite", () => { test.afterEach(() => { ... }); test("t1", ...); test("t2", ...); })`
- **THEN** afterEach SHALL execute after "t1" completes
- **AND** afterEach SHALL execute after "t2" completes

#### Scenario: AfterEach runs on test failure

- **WHEN** test "t1" fails
- **THEN** afterEach SHALL still execute after "t1"

### Requirement: Hook Scoping in Nested Describes

Hooks SHALL only apply to tests in their describe block and descendant blocks.

#### Scenario: Nested beforeEach

- **WHEN** outer describe has `test.beforeEach(() => outer_setup)`
- **AND** inner describe has `test.beforeEach(() => inner_setup)`
- **AND** there is a test in inner describe
- **THEN** both outer_setup AND inner_setup SHALL run before the test (outer first)

#### Scenario: Inner beforeEach doesn't affect outer tests

- **WHEN** inner describe has `test.beforeEach(() => inner_setup)`
- **AND** there is a test in outer describe (not in inner)
- **THEN** inner_setup SHALL NOT run for the outer test
