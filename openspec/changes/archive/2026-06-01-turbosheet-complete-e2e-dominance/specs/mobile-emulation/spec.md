## ADDED Requirements

### Requirement: Device emulation presets

TurboSheet SHALL provide built-in device presets for common mobile and tablet devices.

#### Scenario: Set iPhone preset

- **WHEN** user configures `test.use({ ...devices['iPhone 15 Pro'] })`
- **THEN** the browser SHALL emulate iPhone 15 Pro viewport, user agent, touch events, and device scale factor

#### Scenario: Set iPad preset

- **WHEN** user configures `test.use({ ...devices['iPad Pro 12.9'] })`
- **THEN** the browser SHALL emulate iPad Pro landscape/portrait dimensions

#### Scenario: Device list

- **WHEN** user runs `tsheet devices`
- **THEN** a list of all 50+ supported device presets SHALL be displayed

### Requirement: Custom device configuration

Users SHALL define custom device emulation settings.

#### Scenario: Custom viewport

- **WHEN** user sets `test.use({ viewport: { width: 360, height: 780 } })`
- **THEN** the browser viewport SHALL be set to the custom dimensions

#### Scenario: Custom user agent

- **WHEN** user sets `test.use({ userAgent: 'Custom/1.0' })`
- **THEN** HTTP requests SHALL use the custom user agent string

### Requirement: Touch event emulation

TurboSheet SHALL emulate touch events for mobile testing.

#### Scenario: Tap instead of click

- **WHEN** emulating a mobile device and user calls `element.tap()`
- **THEN** TurboSheet SHALL generate a touch event sequence (touchstart, touchend) instead of a click

#### Scenario: Swipe gesture

- **WHEN** user calls `page.swipe({ from: { x: 100, y: 200 }, to: { x: 100, y: 400 } })`
- **THEN** TurboSheet SHALL generate a touch move event sequence

### Requirement: Geolocation mocking

TurboSheet SHALL override browser geolocation.

#### Scenario: Set geolocation

- **WHEN** user sets `context.setGeolocation({ latitude: 40.7128, longitude: -74.0060 })`
- **THEN** the browser SHALL report the specified coordinates to `navigator.geolocation`

#### Scenario: Grant geolocation permission

- **WHEN** user configures `context.grantPermissions(['geolocation'])`
- **THEN** the browser SHALL auto-grant geolocation permission

### Requirement: Color scheme and motion emulation

TurboSheet SHALL emulate `prefers-color-scheme` and `prefers-reduced-motion`.

#### Scenario: Dark mode

- **WHEN** user sets `page.emulateMedia({ colorScheme: 'dark' })`
- **THEN** `matchMedia('(prefers-color-scheme: dark)')` SHALL return true

#### Scenario: Reduced motion

- **WHEN** user sets `page.emulateMedia({ reducedMotion: 'reduce' })`
- **THEN** `matchMedia('(prefers-reduced-motion: reduce)')` SHALL return true
