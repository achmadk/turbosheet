## ADDED Requirements

### Requirement: AI-Optimized State Extraction

The system SHALL provide a `page.getAgentSnapshot()` method that extracts a compact, LLM-optimized representation of the page state instead of raw HTML.

#### Scenario: Extracting the accessibility tree

- **WHEN** `page.getAgentSnapshot()` is called
- **THEN** the system SHALL return a structured JSON object containing:
  - All visible elements with their ARIA roles, accessible names, and states
  - Bounding boxes for each element in viewport coordinates
  - Text content with approximate position (no full HTML dump)
  - Element relationships (parent, children, siblings)
  - Form element values and validation states
- **AND** the output SHALL be at least 60% smaller than equivalent raw HTML

#### Scenario: Depth filtering

- **WHEN** `page.getAgentSnapshot({ depth: 2 })` is called
- **THEN** the system SHALL only traverse DOM to the specified depth
- **AND** elements below that depth SHALL be summarized as count + type

#### Scenario: Include/exclude filters

- **WHEN** `page.getAgentSnapshot({ include: ['button', 'input', 'a'] })` is called
- **THEN** the system SHALL only include nodes matching the specified selectors
- **WHEN** `page.getAgentSnapshot({ exclude: ['script', 'style', 'noscript'] })` is called
- **THEN** the system SHALL exclude nodes matching the specified selectors

#### Scenario: Usage with AI agents

- **WHEN** an AI agent calls `page.getAgentSnapshot()` before making a decision
- **THEN** the system SHALL return the snapshot in under 100ms
- **AND** the snapshot SHALL be directly parseable by LLMs without additional processing
