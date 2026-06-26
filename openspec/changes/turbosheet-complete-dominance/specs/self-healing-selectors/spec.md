## ADDED Requirements

### Requirement: Self-Healing Selector Engine

The system SHALL provide automatic selector healing when the primary selector fails to find an element, using heuristic-based fallback matching.

#### Scenario: Healing on element not found

- **WHEN** a locator action is performed and the primary selector matches 0 elements
- **THEN** the system SHALL automatically attempt to find the element using fallback strategies
- **AND** SHALL log the healing event with original selector, healed selector, and confidence score

#### Scenario: Healing strategies

- **WHEN** the primary selector fails
- **THEN** the system SHALL try the following strategies in order:
  1. Text similarity: find elements with similar text content (Levenshtein distance < 0.3)
  2. ARIA role match: find elements with the same ARIA role in proximity
  3. CSS class overlap: find elements with overlapping CSS class names
  4. DOM position proximity: find elements at similar DOM depth and sibling position
  5. Visual bounding box: if previous successful interaction had a known position, find nearest element

#### Scenario: Confidence threshold

- **WHEN** the confidence score of the best candidate is below 0.7
- **THEN** the system SHALL NOT auto-heal
- **AND** SHALL throw the original "Element not found" error

#### Scenario: Healing rate limit

- **WHEN** a single test exceeds 3 healing events
- **THEN** the system SHALL fail the test with "Exceeded selector healing threshold"
- **AND** SHALL report all healed selectors in the error message
- **AND** the threshold SHALL be configurable in test config

#### Scenario: Healing reporting

- **WHEN** a test completes with one or more healing events
- **THEN** the test report SHALL include a "Healed Selectors" section
- **AND** SHALL show: original selector, healed selector, confidence, and location
