## ADDED Requirements

### Requirement: Component mounting

The system SHALL mount framework components in lightweight headless context.

#### Scenario: Mount React component

- **WHEN** `component.mount(<App />)` is called with React
- **THEN** system renders component in jsdom-like environment
- **AND** provides page-like API for interaction

#### Scenario: Mount Vue component

- **WHEN** `component.mount(VueComponent)` is called with Vue
- **THEN** system renders component using Vue test utils
- **AND** provides page-like API

#### Scenario: Mount Svelte component

- **WHEN** `component.mount(SvelteComponent)` is called with Svelte
- **THEN** system renders component using svelte register
- **AND** provides page-like API

### Requirement: Component props and state

The system SHALL allow setting props and accessing component state.

#### Scenario: Set props

- **WHEN** `component.mount(<App title="Hello" />)` is called
- **THEN** system passes props to component

#### Scenario: Access component state

- **WHEN** `component.locator('state=count')` is called
- **THEN** system accesses component internal state
- **AND** returns current value

#### Scenario: Trigger state update

- **WHEN** `component.locator('button').click()` triggers state change
- **THEN** component state updates
- **AND** can be verified with `expect(component.locator('state=count')).toHaveText('1')`

### Requirement: Component cleanup

The system SHALL properly cleanup mounted components.

#### Scenario: Cleanup after test

- **WHEN** component is mounted in test
- **THEN** system unmounts component after test completes
- **AND** releases all event listeners

#### Scenario: Cleanup on timeout

- **WHEN** component mount hangs
- **THEN** system times out
- **AND** forcefully unmounts component

### Requirement: Framework detection

The system SHALL auto-detect which framework is being tested.

#### Scenario: Detect React from imports

- **WHEN** test file imports from 'react'
- **THEN** system uses React mounting strategy

#### Scenario: Detect Vue from imports

- **WHEN** test file imports from 'vue'
- **THEN** system uses Vue mounting strategy

#### Scenario: Detect Svelte from imports

- **WHEN** test file imports from 'svelte'
- **THEN** system uses Svelte mounting strategy

### Requirement: Component locator API

The system SHALL provide locator methods for component internals.

#### Scenario: Find element by component selector

- **WHEN** `component.locator('button.submit')` is called
- **THEN** system finds button element within component

#### Scenario: Access computed style

- **WHEN** `component.locator('.item').evaluate(el => getComputedStyle(el).color)` is called
- **THEN** system evaluates function in component context

#### Scenario: Wait for component render

- **WHEN** `component.locator('.loading').waitFor({ state: 'hidden' })` is called
- **THEN** system waits for loading to complete

### Requirement: Component testing configuration

The system SHALL support component-specific configuration.

#### Scenario: Configure jsdom

- **WHEN** `component.mount(App, { jsdom: { url: 'http://localhost' } })` is called
- **THEN** system uses specified jsdom configuration

#### Scenario: Stub external dependencies

- **WHEN** `component.mount(App, { stubs: { Fetch: mockFetch } })` is called
- **THEN** system stubs specified modules
