## ADDED Requirements

### Requirement: Mobile Device Cloud Integration

The system SHALL support running tests on real mobile devices via BrowserStack, SauceLabs, and LambdaTest.

#### Scenario: Cloud provider configuration

- **WHEN** a cloud provider is configured in `turbosheet.config.ts`
- **THEN** the system SHALL connect to the provider's API using the configured credentials
- **AND** SHALL negotiate a device session with the specified capabilities

#### Scenario: Running on real devices

- **WHEN** `browser.newContext({ cloud: { provider: 'browserstack', device: 'iPhone 15', osVersion: '17' } })` is called
- **THEN** the system SHALL request a device session from the provider
- **AND** SHALL route all browser commands through the provider's tunnel

#### Scenario: Local testing tunnel

- **WHEN** running tests against a local development server with a cloud provider
- **THEN** the system SHALL start a local binary tunnel (BrowserStack Local, Sauce Connect)
- **AND** SHALL make the local server accessible to the cloud device

#### Scenario: Device configuration options

- **WHEN** configuring a cloud device
- **THEN** the system SHALL support: device name, OS version, orientation (portrait/landscape), real device vs simulator, geolocation, and custom user agent

#### Scenario: Multi-device parallel testing

- **WHEN** `testConfig.workers > 1` with cloud devices
- **THEN** the system SHALL run tests in parallel across multiple device sessions
- **AND** SHALL respect the provider's concurrent session limit
