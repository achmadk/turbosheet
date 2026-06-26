## ADDED Requirements

### Requirement: Secure Selector Escaping

The system SHALL use `serde_json` serialization for all selector values passed into JavaScript `evaluate()` calls, replacing the current ad-hoc `selector.replace("'", "\\'")` pattern.

#### Scenario: Standard selectors work

- **WHEN** a selector like `button.submit` is used
- **THEN** the system SHALL correctly escape and pass it to `document.querySelector`
- **AND** the element SHALL be found correctly

#### Scenario: Selectors with quotes

- **WHEN** a selector contains single quotes, e.g., `button[data-name="it's fine"]`
- **THEN** the system SHALL correctly escape the quotes
- **AND** SHALL not throw a JavaScript syntax error

#### Scenario: Selectors with backslashes

- **WHEN** a selector contains backslashes, e.g., `button[data-path="C:\users"]`
- **THEN** the system SHALL correctly escape the backslashes
- **AND** SHALL not break the JavaScript string

#### Scenario: Selectors with newlines and special characters

- **WHEN** a selector contains newlines, template literal backticks, or Unicode characters
- **THEN** the system SHALL correctly escape all characters using JSON serialization
- **AND** SHALL not create JavaScript injection vectors

#### Scenario: Implementation pattern

- **GIVEN** the system receives a selector string
- **WHEN** constructing an `evaluate()` JavaScript snippet
- **THEN** the system SHALL use `serde_json::to_string(&selector)` or `JSON.stringify()` equivalent
- **AND** SHALL NOT use string replacement (`replace("'", "\\'")`) or string concatenation
