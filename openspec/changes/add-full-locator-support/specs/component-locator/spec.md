## ADDED Requirements

### Requirement: ComponentLocator.locator creates scoped sub-locator

`ComponentLocator` SHALL provide a `.locator(selector: string)` method that returns a new `ComponentLocator` scoped to the child selector, using Rust `JsLocator`'s selector-chaining semantics (`>>` separator).

#### Scenario: Chained locator selects nested element

- **WHEN** `component.locator('form').locator('button')` is called
- **THEN** the resulting locator targets `form >> button` in the Rust locator system

#### Scenario: Locator preserves page context

- **WHEN** `component.locator('div')` returns a new locator
- **THEN** the new locator shares the same `pageId` and page context as the parent

### Requirement: ComponentLocator.frameLocator scopes to iframe

`ComponentLocator` SHALL provide a `.frameLocator(frameSelector: string)` method that returns a `ComponentLocator` scoped to a matching iframe, using `JsFrameLocator` semantics.

#### Scenario: Frame locator targets iframe content

- **WHEN** `component.frameLocator('iframe[name="my-frame"]')` is called
- **THEN** the locator scopes all subsequent queries to the matching iframe

### Requirement: ComponentLocator get-by methods for semantic selection

`ComponentLocator` SHALL provide `.getByRole()`, `.getByText()`, `.getByLabel()`, `.getByPlaceholder()`, `.getByAltText()`, `.getByTitle()`, and `.getByTestId()` methods that delegate to the corresponding Rust `JsLocator` get-by methods.

#### Scenario: getByRole finds element by ARIA role

- **WHEN** `component.getByRole('button', { name: 'Submit' })` is called
- **THEN** the resulting locator selects elements matching role=button with name "Submit"

#### Scenario: getByText finds element by text content

- **WHEN** `component.getByText('Click me')` is called
- **THEN** the resulting locator selects elements containing the text "Click me"

#### Scenario: getByText exact match

- **WHEN** `component.getByText('Submit', { exact: true })` is called
- **THEN** the resulting locator selects elements whose text exactly matches "Submit"

#### Scenario: getByLabel finds element by aria-label

- **WHEN** `component.getByLabel('Username')` is called
- **THEN** the resulting locator selects elements with aria-label or associated label text "Username"

#### Scenario: getByPlaceholder finds input by placeholder

- **WHEN** `component.getByPlaceholder('Enter email')` is called
- **THEN** the resulting locator selects input elements with placeholder "Enter email"

#### Scenario: getByAltText finds image by alt attribute

- **WHEN** `component.getByAltText('Logo')` is called
- **THEN** the resulting locator selects img elements with alt text "Logo"

#### Scenario: getByTitle finds element by title attribute

- **WHEN** `component.getByTitle('Tooltip text')` is called
- **THEN** the resulting locator selects elements with title attribute "Tooltip text"

#### Scenario: getByTestId finds element by data-testid

- **WHEN** `component.getByTestId('submit-btn')` is called
- **THEN** the resulting locator selects elements with data-testid="submit-btn"

### Requirement: ComponentLocator actions route through Rust JsLocator

All `ComponentLocator` action methods — `.click()`, `.fill()`, `.dblclick()`, `.hover()`, `.press()`, `.check()`, `.uncheck()`, `.selectOption()`, `.focus()`, `.blur()`, `.scrollIntoView()`, `.tap()` — SHALL route through the underlying Rust `JsLocator` for proper actionability checks (wait-for-stable, wait-for-visible, wait-for-enabled) before dispatching the action.

#### Scenario: Click waits for actionability

- **WHEN** `component.locator('button').click()` is called
- **THEN** the locator waits for the element to be visible, enabled, and stable before clicking

#### Scenario: Fill clears and types value

- **WHEN** `component.locator('input').fill('hello')` is called
- **THEN** the locator clears the input and types the value character by character

#### Scenario: Press dispatches keyboard event

- **WHEN** `component.locator('input').press('Enter')` is called
- **THEN** a keyboard event with key "Enter" is dispatched on the focused element

### Requirement: ComponentLocator query methods route through Rust JsLocator

Query methods — `.textContent()`, `.innerText()`, `.innerHtml()`, `.getAttribute()`, `.isVisible()`, `.isEnabled()`, `.isDisabled()`, `.boundingBox()` — SHALL delegate to the Rust `JsLocator` query methods.

#### Scenario: isVisible checks computed styles

- **WHEN** `component.locator('.hidden').isVisible()` is called
- **THEN** the result is false for elements with display:none or visibility:hidden

#### Scenario: textContent returns trimmed text

- **WHEN** `component.locator('div').textContent()` is called
- **THEN** the method returns the textContent of the matched element

#### Scenario: getAttribute returns attribute value

- **WHEN** `component.locator('a').getAttribute('href')` is called
- **THEN** the method returns the href attribute value

### Requirement: ComponentLocator waitFor waits with actionability

`ComponentLocator.waitFor()` SHALL use the Rust locator's `wait_for` method with configurable timeout and state options (`attached`, `detached`, `visible`, `hidden`).

#### Scenario: waitFor waits for element state

- **WHEN** `component.locator('.loading').waitFor({ state: 'hidden' })` is called
- **THEN** the method waits until the element is not visible, up to the default timeout

### Requirement: ComponentLocator.filter narrows matching elements

`ComponentLocator` SHALL provide a `.filter(options)` method that narrows the matched elements by text, role, or other attribute filters, delegating to `JsLocator.filter()`.

#### Scenario: Filter by text content

- **WHEN** `component.locator('li').filter({ hasText: 'Active' })` is called
- **THEN** the resulting locator matches only list items containing "Active"

### Requirement: ComponentLocator.first/last/nth for index-based selection

`ComponentLocator` SHALL provide `.first()`, `.last()`, and `.nth(index)` methods that delegate to the corresponding Rust `JsLocator` methods.

#### Scenario: first selects first matching element

- **WHEN** `component.locator('button').first()` is called
- **THEN** the locator targets only the first matching button

#### Scenario: nth selects by index

- **WHEN** `component.locator('button').nth(2)` is called
- **THEN** the locator targets only the third matching button (0-indexed)

### Requirement: JsLocator exported from napi module as Locator class

The Rust `JsLocator` SHALL be exported from the napi native module as a `Locator` class, constructible with `(pageId: string, selector: string)`.

#### Scenario: Locator constructible from JS

- **WHEN** `new nativeBinding.Locator(pageId, 'button')` is called
- **THEN** it returns a JsLocator instance bound to the given page and selector
