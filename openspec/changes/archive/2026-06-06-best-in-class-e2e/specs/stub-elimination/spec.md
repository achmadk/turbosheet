## ADDED Requirements

### Requirement: CDP Method Implementation

The system SHALL implement previously stubbed Chromium methods using their respective CDP commands.

#### Scenario: Setting viewport size

- **WHEN** `page.set_viewport_size` is called
- **THEN** the system MUST execute `Emulation.setDeviceMetricsOverride` and await success

#### Scenario: Navigating history

- **WHEN** `page.go_back` or `page.go_forward` is called
- **THEN** the system MUST execute `Page.navigateHistory` with the appropriate delta

### Requirement: Explicit Failure for Unsupported Methods

The system SHALL throw an error for API methods that are explicitly unsupported, rather than returning a silent success.

#### Scenario: Calling unsupported touch methods

- **WHEN** `page.swipe`, `page.tap`, or `page.pinch` is called
- **THEN** the system MUST throw a `NotImplementedError` indicating the feature is not supported
