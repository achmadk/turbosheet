## ADDED Requirements

### Requirement: BiDi script.addPreloadScript

The BiDi client SHALL support `script.addPreloadScript` to inject JavaScript that runs on every page load before any page script executes.

#### Scenario: Add preload script

- **WHEN** `client.add_preload_script("console.log('injected')")` is called
- **THEN** a BiDi `script.addPreloadScript` command SHALL be sent with the JS source
- **AND** the returned script ID SHALL be stored for later deregistration

#### Scenario: Preload script survives navigation

- **WHEN** the page navigates to a new URL
- **THEN** the preload script SHALL execute again on the new page automatically

#### Scenario: Remove preload script

- **WHEN** `client.remove_preload_script(script_id)` is called
- **THEN** a `script.removePreloadScript` command SHALL be sent
- **AND** subsequent navigations SHALL NOT execute the script

### Requirement: BiDi script.callFunctionOn

The BiDi client SHALL support `script.callFunctionOn` to call a JavaScript function in the page context and return the result.

#### Scenario: Call function declaration

- **WHEN** `client.call_function(function_declaration, params)` is called
- **THEN** a BiDi `script.callFunctionOn` command SHALL be sent with the function body as a string
- **AND** the result SHALL be returned as a `script.EvaluateResult`

#### Scenario: Call function with arguments

- **WHEN** a function is called with arguments `[1, "hello", {key: "val"}]`
- **THEN** the arguments SHALL be serialized as BiDi `script.LocalValue` and passed in the command

### Requirement: expose_function via preload script

`FirefoxPageEngine::expose_function` SHALL use `script.addPreloadScript` to inject function stubs that survive navigation.

#### Scenario: Exposed function in page context

- **WHEN** `expose_function("myFunc", "function(x) { return x + 1; }")` is called
- **THEN** a preload script SHALL be added that creates `window.myFunc`
- **AND** `window.myFunc(5)` from page context SHALL execute the function body and return `6`
- **AND** the function SHALL work after page navigation (because it's a preload script)

### Requirement: register_binding via preload script

`FirefoxPageEngine::register_binding` SHALL use `script.addPreloadScript` to register named bindings accessible from page context.

#### Scenario: Binding callable from page

- **WHEN** `register_binding("__ts_test_binding")` is called
- **THEN** a preload script SHALL inject `window.__ts_test_binding` that routes calls via BiDi `script.callFunctionOn`
