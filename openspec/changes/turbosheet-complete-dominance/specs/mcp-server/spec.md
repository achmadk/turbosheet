## ADDED Requirements

### Requirement: Rust-Native MCP Server

The system SHALL expose TurboSheet as a Model Context Protocol (MCP) server, enabling AI agents to control browsers through TurboSheet's high-performance Rust core.

#### Scenario: MCP server startup

- **WHEN** `npx tsheet mcp` is run
- **THEN** the system SHALL start an MCP server on the configured port (default: 3001)
- **AND** SHALL expose available tools for AI agent discovery

#### Scenario: Browser navigation via MCP

- **WHEN** an AI agent calls the `navigate` MCP tool with a URL
- **THEN** the system SHALL launch a browser (if not already running), navigate to the URL
- **AND** SHALL return the page title and current URL

#### Scenario: Element interaction via MCP

- **WHEN** an AI agent calls the `click` MCP tool with a selector
- **THEN** the system SHALL find the element and click it
- **AND** SHALL return success or the error message
- **WHEN** an AI agent calls the `extract` MCP tool with a selector
- **THEN** the system SHALL return the element's text content

#### Scenario: AI state snapshot via MCP

- **WHEN** an AI agent calls the `getSnapshot` MCP tool
- **THEN** the system SHALL call `page.getAgentSnapshot()` internally
- **AND** SHALL return the compact, LLM-optimized state representation

#### Scenario: Screenshot via MCP

- **WHEN** an AI agent calls the `screenshot` MCP tool
- **THEN** the system SHALL capture a page screenshot
- **AND** SHALL return the screenshot as a base64-encoded image or file path
