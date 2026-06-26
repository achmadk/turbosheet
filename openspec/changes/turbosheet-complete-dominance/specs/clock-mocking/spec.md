## ADDED Requirements

### Requirement: Clock Mocking and Fake Timers

The system SHALL support mocking time-related APIs for deterministic, time-independent testing.

#### Scenario: Fixed time

- **WHEN** `page.clock.setFixedTime(new Date('2026-06-06T12:00:00Z'))` is called
- **THEN** the system SHALL override `Date.now()`, `new Date()`, and `performance.now()` to return the specified time
- **AND** time SHALL NOT advance until explicitly told to

#### Scenario: Time installation with advancement

- **WHEN** `page.clock.install()` is called
- **THEN** the system SHALL install fake timers overriding `setTimeout`, `setInterval`, `setImmediate`, and `requestAnimationFrame`
- **AND** SHALL start time at the current real time
- **WHEN** `page.clock.fastForward(5000)` is called
- **THEN** the system SHALL advance time by 5000ms
- **AND** SHALL execute any pending timers that become due

#### Scenario: Timer control

- **WHEN** `setTimeout(() => callback(), 10000)` is called with fake timers installed
- **THEN** the callback SHALL NOT execute until `fastForward(10000)` is called
- **AND** SHALL execute immediately when time advances past the timer's deadline

#### Scenario: Cleanup

- **WHEN** the page is closed or navigated
- **THEN** the system SHALL restore all original time APIs
- **AND** SHALL clear any pending fake timers
