## ADDED Requirements

### Requirement: Component Mount Reliability

The system SHALL mount UI components with deterministic timeout behavior, clear failure modes, and proper DOM state verification.

#### Scenario: Mount operation times out instead of hanging indefinitely

- **GIVEN** `src/component/mount.rs` executes a component mount operation
- **WHEN** the mount target page fails to render the component (e.g., network error, JavaScript error, or infinite loading state)
- **THEN** the mount SHALL fail after a configurable timeout (default: 30 seconds)
- **AND** SHALL NOT hang indefinitely (current behavior: `child_body_wait` can wait forever as there is no timeout)
- **AND** SHALL return a `TurbosheetError::TimeoutError` with details about what was being waited on
- **AND** SHALL include the current page state (URL, readyState, last 10 console messages) in the error for debugging

#### Scenario: Placeholder navigation provides correct component context

- **GIVEN** the mount system uses placeholder navigation as a workaround to establish a page context
- **WHEN** the placeholder page loads
- **THEN** the system SHALL verify the placeholder loaded successfully before proceeding
- **AND** SHALL replace the placeholder URL with the component's actual URL
- **AND** SHALL clean up any leftover placeholder state (history entries, cached resources)
- **AND** SHALL log the navigation as a debug event (not visible to user by default)

#### Scenario: Mount component state verification

- **GIVEN** a component mount operation completes
- **WHEN** the mount result is returned to the caller
- **THEN** the system SHALL verify the mounted component is actually interactive:
  - The root element is present in the DOM
  - No uncaught JavaScript errors occurred during mount
  - The component's detected framework (React, Vue, Angular, or vanilla) matches expectations
- **AND** SHALL return a clear error if any verification fails, including the DOM snapshot at failure point
