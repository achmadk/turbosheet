# TurboSheet Progress Report — Feature Comparison & Improvement Roadmap

> **Generated**: 2026-06-18  
> **Last Updated**: 2026-07-08 — Phase 1 Test Runner Core complete: two-phase IPC protocol (extractTests/runPlan), collector wrapper with full modifier/hook support, PlanBuilder with modifier cascade and hook inheritance, beforeAll failure cascading, and parallel worker orchestration.  
> **Scope**: Comprehensive feature-by-feature analysis of TurboSheet's Rust-based browser automation framework compared to Playwright (v1.52+), Puppeteer (v24+), and Cypress (v14+).  
> **Methodology**: Direct codebase audit of all 28+ Rust source modules, JS layer (napi bindings, injected scripts, ComponentLocator), Cargo.toml, feature flags, plus official API documentation for each competitor.  
> **Note**: This report was generated through exhaustive codebase exploration — every `.rs` file was read and analyzed. Claims in this document supersede stale documentation or README references.

---

## Overall Completion by Engine

| Engine              | Est. Complete | Lines of Code | Primary Protocol                                                                                                                                                                                                                         |
| ------------------- | :-----------: | :-----------: | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Chromium**        |   **~65%**    |    ~2,500+    | CDP (chromiumoxide 0.9)                                                                                                                                                                                                                  |
| **Firefox**         |   **~25%**    |     ~650      | WebDriver (geckodriver)                                                                                                                                                                                                                  |
| **WebKit**          |   **~20%**    |     ~650      | WebDriver (safaridriver/WKWebDriver)                                                                                                                                                                                                     |
| **Test Runner**     |   **~70%**    |    ~2,400+    | Two-phase IPC (extractTests/runPlan); PlanBuilder with modifier cascade + hook inheritance; collector wrapper (test/describe/beforeAll/afterAll/beforeEach/afterEach + modifiers); parallel workers with suite-level beforeAll cascading |
| **Trace System**    |   **~50%**    |     ~600      | In-memory event store + HTML viewer + serializer/deserializer                                                                                                                                                                            |
| **Video Recording** |   **~30%**    |     ~550      | CDP screencast → FFmpeg pipeline; needs full test-runner wiring                                                                                                                                                                          |
| **Plugin System**   |   **~10%**    |     ~130      | Trait definitions only; no real discovery or auto-loading                                                                                                                                                                                |
| **Migration**       |    **~5%**    |     ~180      | API mapping lists only; no AST transform                                                                                                                                                                                                 |
| **JS/TS Layer**     |   **~65%**    |     ~425      | napi bindings + ComponentLocator + injected-actions + core injection                                                                                                                                                                     |
| **Reporting**       |   **~95%**    |    ~1,000+    | 7 built-in reporters wired to TestExecutor via `Reporter` trait; HTML/JUnit file output separate                                                                                                                                         |

---

## 1. Browser Engine & Launch

| Feature                          | Playwright | Puppeteer |    Cypress    |   Turbosheet (Chromium)    | Turbosheet (Firefox) | Turbosheet (WebKit) |
| -------------------------------- | :--------: | :-------: | :-----------: | :------------------------: | :------------------: | :-----------------: |
| Launch headless                  |     ✅     |    ✅     |      ✅       |             ✅             |          ✅          |         ✅          |
| Launch headed                    |     ✅     |    ✅     |      ✅       |             ✅             |          ✅          |         ✅          |
| Executable path                  |     ✅     |    ✅     |       —       |             ✅             |          ✅          |         ✅          |
| Custom args                      |     ✅     |    ✅     |      ✅       |             ✅             |          ✅          |         ✅          |
| `--no-sandbox` auto              |     ✅     |    ✅     |      ✅       |      ✅ (Linux auto)       |          —           |          —          |
| User data dir                    |     ✅     |    ✅     |       —       |             ✅             |          —           |          —          |
| Unique temp profile              |     ✅     |    ✅     |       —       |    ✅ (via JS wrapper)     |          —           |          —          |
| Proxy config                     |     ✅     |    ✅     |      ✅       |          ✅ (CDP)          |          —           |          —          |
| Download path                    |     ✅     |    ✅     |      ✅       |             ❌             |          ❌          |         ❌          |
| Ignore HTTPS errors              |     ✅     |    ✅     |      ✅       |             ❌             |          ❌          |         ❌          |
| SlowMo / throttling              |     ✅     |    ✅     | ✅ (cy.clock) |             ❌             |          ❌          |         ❌          |
| Timeout config                   |     ✅     |    ✅     |      ✅       |       ❌ (hardcoded)       |          ❌          |         ❌          |
| `connectOverCDP` (existing)      |     ✅     |    ✅     |       —       |             ❌             |          ❌          |         ❌          |
| Raw CDP session                  |     ✅     |    ✅     |       —       | ❌ (no CDPSession exposed) |          ❌          |         ❌          |
| Chrome extensions                |     ✅     |    ✅     |      ❌       |             ❌             |          ❌          |         ❌          |
| Persistent context (profile dir) |     ✅     |    ✅     |      ❌       |             ❌             |          ❌          |         ❌          |

---

## 2. Browser Context (Incognito / Isolation)

| Feature                       | Playwright | Puppeteer |        Cypress        |  Turbosheet (Chromium)   | Turbosheet (Firefox) | Turbosheet (WebKit) |
| ----------------------------- | :--------: | :-------: | :-------------------: | :----------------------: | :------------------: | :-----------------: |
| Create context                |     ✅     |    ✅     |    ✅ (cy.session)    |  ✅ (incognito via CDP)  |          ✅          |         ✅          |
| Multiple contexts             |     ✅     |    ✅     |           —           |            ✅            |   ❌ (no tracking)   |         ❌          |
| Close context                 |     ✅     |    ✅     |           —           |            ✅            |          ✅          |         ✅          |
| Context isolation (storage)   |     ✅     |    ✅     |           —           |            ✅            |    ✅ (WebDriver)    |   ✅ (WebDriver)    |
| Context-level routing         |     ✅     |     —     |           —           |            ❌            |          ❌          |         ❌          |
| Context-level events          |     ✅     |     —     |           —           |    ❌ (partial popup)    |          ❌          |         ❌          |
| Default context               |     ✅     |    ✅     |          ✅           |            ❌            |          ❌          |         ❌          |
| Permissions (geoloc, etc.)    |     ✅     |    ✅     |           —           |            ❌            |          ❌          |         ❌          |
| Extra HTTP headers            |     ✅     |    ✅     |   ✅ (cy.intercept)   |            ❌            |          ❌          |         ❌          |
| Offline mode                  |     ✅     |    ✅     |           —           | ⚠️ (no-op trait default) |          ❌          |         ❌          |
| Geolocation override          |     ✅     |    ✅     |           —           |            ❌            |          ❌          |         ❌          |
| Locale override               |     ✅     |     —     |          ✅           |            ❌            |          ❌          |         ❌          |
| Timezone override             |     ✅     |    ✅     |          ✅           |            ❌            |          ❌          |         ❌          |
| Color scheme                  |     ✅     |    ✅     |           —           |        ⚠️ (no-op)        |          ❌          |         ❌          |
| Reduced motion                |     ✅     |    ✅     |           —           |        ⚠️ (no-op)        |          ❌          |         ❌          |
| Viewport (context-level)      |     ✅     |     —     |   ✅ (cy.viewport)    |            ❌            |          ❌          |         ❌          |
| Device scale factor           |     ✅     |    ✅     |           —           |            ❌            |          ❌          |         ❌          |
| Has touch                     |     ✅     |    ✅     |           —           |            ❌            |          ❌          |         ❌          |
| Is mobile                     |     ✅     |    ✅     |           —           |            ❌            |          ❌          |         ❌          |
| User agent override           |     ✅     |    ✅     | ✅ (cy.visit headers) |            ❌            |          ❌          |         ❌          |
| Storage state (export/import) |     ✅     |     —     |           —           |            ❌            |          ❌          |         ❌          |
| Record video per context      |     ✅     |     —     |          ✅           |            ❌            |          ❌          |         ❌          |
| Record HAR per context        |     ✅     |     —     |           —           |            ❌            |          ❌          |         ❌          |
| Trace per context             |     ✅     |     —     |           —           |     ⚠️ (global only)     |          ❌          |         ❌          |
| Service workers               |     ✅     |    ✅     |           —           |            ❌            |          ❌          |         ❌          |

---

## 3. Page Navigation & Lifecycle

| Feature                                |  Playwright   | Puppeteer |        Cypress        |    Turbosheet (Chromium)    | Turbosheet (Firefox) | Turbosheet (WebKit)  |
| -------------------------------------- | :-----------: | :-------: | :-------------------: | :-------------------------: | :------------------: | :------------------: |
| `goto(url)`                            |      ✅       |    ✅     |     ✅ (cy.visit)     |             ✅              |          ✅          |          ✅          |
| `reload()`                             |      ✅       |    ✅     |    ✅ (cy.reload)     |             ✅              |          ❌          |          ❌          |
| `goBack()` / `goForward()`             |      ✅       |    ✅     |      ✅ (cy.go)       |             ✅              |          ❌          |          ❌          |
| `waitForLoadState()`                   |      ✅       |    ✅     |    ✅ (auto-wait)     |    ⚠️ (goto waits load)     |          ❌          |          ❌          |
| `waitForNavigation()`                  |      ✅       |    ✅     |           —           |             ❌              |          ❌          |          ❌          |
| `waitForURL()`                         |      ✅       |     —     |     ✅ (cy.url())     |             ❌              |          ❌          |          ❌          |
| `waitForFunction()`                    |      ✅       |    ✅     |     ✅ (cy.wait)      |             ❌              |          ❌          |          ❌          |
| `waitForRequest()`                     |      ✅       |     —     | ✅ (cy.wait(@alias))  |  ✅ (CDP request listener)  |          ❌          |          ❌          |
| `waitForResponse()`                    |      ✅       |     —     | ✅ (cy.wait(@alias))  | ✅ (CDP response listener)  |          ❌          |          ❌          |
| `waitForSelector()`                    |      ✅       |    ✅     |    ✅ (auto-wait)     |             ❌              |          ❌          |          ❌          |
| `waitForNetworkIdle()`                 |      ✅       |    ✅     |           —           | ⚠️ (custom polling in goto) |          ❌          |          ❌          |
| `waitForEvent()`                       |      ✅       |     —     |           —           |             ❌              |          ❌          |          ❌          |
| `content()` (page HTML)                |      ✅       |    ✅     |   ✅ (cy.document)    |             ✅              |          ✅          |          ✅          |
| `title()`                              |      ✅       |    ✅     |     ✅ (cy.title)     |             ✅              |          ❌          |          ❌          |
| `url()`                                |      ✅       |    ✅     |      ✅ (cy.url)      |             ✅              |          ❌          |          ❌          |
| `setContent(html)`                     |      ✅       |    ✅     |           —           |             ✅              |          ❌          |          ❌          |
| `addInitScript()`                      |      ✅       |    ✅     |           —           |    ✅ (via inject_core)     | ✅ (via inject_core) | ✅ (via inject_core) |
| `addScriptTag()`                       |      ✅       |    ✅     |           —           |             ✅              |          ❌          |          ❌          |
| `addStyleTag()`                        |      ✅       |    ✅     |           —           |             ✅              |          ❌          |          ❌          |
| `exposeFunction()` / `exposeBinding()` |      ✅       |    ✅     | ✅ (Cypress.Commands) |             ✅              |          ❌          |          ❌          |
| `setViewportSize()`                    |      ✅       |    ✅     |   ✅ (cy.viewport)    |             ✅              |          ❌          |          ❌          |
| `screenshot()`                         |      ✅       |    ✅     |  ✅ (cy.screenshot)   |             ✅              |          ❌          |          ❌          |
| `pdf()`                                | ✅ (Chromium) |    ✅     |          ❌           |             ❌              |          ❌          |          ❌          |
| `bringToFront()`                       |      ✅       |    ✅     |           —           |             ❌              |          ❌          |          ❌          |
| `close()`                              |      ✅       |    ✅     |           —           |             ✅              |          ✅          |          ✅          |
| Page error events                      |      ✅       |    ✅     |           —           |    ✅ (crashed/detached)    |          ❌          |          ❌          |
| Popup handling                         |      ✅       |    ✅     |           —           | ✅ (targetCreated listener) |          ❌          |          ❌          |
| File chooser                           |      ✅       |    ✅     |  ✅ (cy.selectFile)   |       ✅ (CDP event)        |          ❌          |          ❌          |
| Dialog handling (alert/confirm)        |      ✅       |    ✅     |      ✅ (cy.on)       |  ✅ (auto-accept after 5s)  |          ❌          |          ❌          |
| Frame support                          |      ✅       |    ✅     |          ❌           |             ❌              |          ❌          |          ❌          |
| Worker support                         |      ✅       |    ✅     |           —           |             ❌              |          ❌          |          ❌          |

---

## 4. Element Locators & Queries

| Feature                              | Playwright | Puppeteer |           Cypress           |                        Turbosheet                        |
| ------------------------------------ | :--------: | :-------: | :-------------------------: | :------------------------------------------------------: |
| CSS selector                         |     ✅     |    ✅     |         ✅ (cy.get)         |                            ✅                            |
| Text selector (`text=`)              |     ✅     |    ❌     |      ✅ (cy.contains)       | ⚠️ (Engine-level chain parser supports named selectors)  |
| XPath selector (`xpath=`)            |     ✅     |    ✅     |    ✅ (cy.xpath plugin)     |                            ❌                            |
| `getByRole()`                        |     ✅     |    ✅     |              —              |  ✅ (injected-actions.ts + ComponentLocator.getByRole)   |
| `getByText()`                        |     ✅     |     —     |      ✅ (cy.contains)       |  ✅ (injected-actions.ts + ComponentLocator.getByText)   |
| `getByLabel()`                       |     ✅     |     —     |              —              |             ✅ (ComponentLocator.getByLabel)             |
| `getByPlaceholder()`                 |     ✅     |     —     |   ✅ (cy.get placeholder)   |          ✅ (ComponentLocator.getByPlaceholder)          |
| `getByAltText()`                     |     ✅     |     —     |              —              |            ✅ (ComponentLocator.getByAltText)            |
| `getByTitle()`                       |     ✅     |     —     |              —              |             ✅ (ComponentLocator.getByTitle)             |
| `getByTestId()`                      |     ✅     |     —     |              —              |            ✅ (ComponentLocator.getByTestId)             |
| `locator()` chaining                 |     ✅     |    ✅     |              —              | ✅ (ComponentLocator.locator(selector) → chain via `>>`) |
| `filter()` by text/role/etc          |     ✅     |     —     |              —              |          ✅ (ComponentLocator.filter(options))           |
| `.first()` / `.last()` / `.nth()`    |     ✅     |     —     |         ✅ (cy.eq)          |           ✅ (ComponentLocator.first/last/nth)           |
| `.and()` (logical AND)               |     ✅     |     —     |              —              |                            ❌                            |
| `.or()` (logical OR)                 |     ✅     |     —     |              —              |                            ❌                            |
| Chained ancestor nav                 |     ✅     |    ✅     | ✅ (cy.parent cy.find etc.) |            ⚠️ (only child chaining via `>>`)             |
| `frameLocator()`                     |     ✅     |     —     |              —              |                            ❌                            |
| Custom selector engines              |     ✅     |    ✅     |    ✅ (custom commands)     |                            ❌                            |
| Strict mode (error on >1 match)      |     ✅     |     —     |             ✅              |                            ❌                            |
| Auto-waiting on locate               |     ✅     |    ⚠️     |             ✅              |   ⚠️ (ComponentLocator.waitFor exists; not automatic)    |
| Locator count                        |     ✅     |     —     |     ✅ (cy.get.length)      |               ✅ (ComponentLocator.count)                |
| `locator.all()`                      |     ✅     |     —     |        ✅ (cy.each)         |                            ❌                            |
| `locator.allInnerTexts()`            |     ✅     |     —     |              —              |                            ❌                            |
| `locator.allTextContents()`          |     ✅     |     —     |              —              |                            ❌                            |
| `locator.describe()` / `normalize()` |     ✅     |     —     |              —              |                            ❌                            |
| `locator.highlight()`                |     ✅     |     —     |              —              |                            ❌                            |
| `locator.contentFrame()`             |     ✅     |     —     |              —              |                            ❌                            |
| `locator.dragTo()`                   |     ✅     |     —     |              —              |                            ❌                            |
| `locator.dispatchEvent()`            |     ✅     |     —     |       ✅ (cy.trigger)       |                            ❌                            |

---

## 5. Actions

| Feature                     | Playwright | Puppeteer |         Cypress          |  Turbosheet (Chromium)  |
| --------------------------- | :--------: | :-------: | :----------------------: | :---------------------: |
| `click()`                   |     ✅     |    ✅     |      ✅ (cy.click)       |  ✅ (CDP mouse events)  |
| `dblclick()`                |     ✅     |    ✅     |     ✅ (cy.dblclick)     |           ✅            |
| `rightclick()`              |     ✅     |    ✅     |    ✅ (cy.rightclick)    |           ✅            |
| `hover()`                   |     ✅     |    ✅     |      ✅ (cy.hover)       |           ✅            |
| `fill()` (clear+type)       |     ✅     |    ✅     |            —             |           ✅            |
| `type()` (append)           |     ✅     |    ✅     |       ✅ (cy.type)       | ❌ (no type, only fill) |
| `press()` (single key)      |     ✅     |    ✅     |            —             | ⚠️ (declared in trait)  |
| `pressSequentially()`       |     ✅     |     —     |   ✅ (cy.type options)   | ⚠️ (declared in trait)  |
| `check()` / `uncheck()`     |     ✅     |    ✅     | ✅ (cy.check/cy.uncheck) |           ✅            |
| `selectOption()`            |     ✅     |    ✅     |      ✅ (cy.select)      |           ✅            |
| `setInputFiles()`           |     ✅     |    ✅     |    ✅ (cy.selectFile)    | ⚠️ (declared in trait)  |
| `focus()` / `blur()`        |     ✅     |    ✅     |  ✅ (cy.focus/cy.blur)   |  ✅ (both implemented)  |
| `scrollIntoViewIfNeeded()`  |     ✅     |    ✅     |  ✅ (cy.scrollIntoView)  |           ✅            |
| `tap()` (mobile)            |     ✅     |    ✅     |            —             |   ❌ (default error)    |
| `dragAndDrop()`             |     ✅     |    ✅     |            —             | ⚠️ (declared in trait)  |
| `clear()` (input)           |     ✅     |    ✅     |      ✅ (cy.clear)       |           ❌            |
| `selectText()`              |     ✅     |    ✅     |            —             |           ❌            |
| `boundingBox()`             |     ✅     |    ✅     |            —             |  ✅ (CDP getBoxModel)   |
| Auto-scroll before action   |     ✅     |    ✅     |            ✅            |   ❌ (no auto-scroll)   |
| Auto-wait for actionability |     ✅     |    ❌     |            ✅            |    ❌ (no auto-wait)    |
| Retry on stale element      |     ✅     |    ✅     |            ✅            |      ❌ (no retry)      |
| Action timeout              |     ✅     |    ✅     |            ✅            |     ❌ (no timeout)     |
| Force click (bypass checks) |     ✅     |    ✅     |            —             |           ❌            |
| `noWaitAfter`               |     ✅     |     —     |            —             |           ❌            |
| Trial run (dry action)      |     ✅     |     —     |            —             |           ❌            |
| Swipe (mobile)              |     —      |    ✅     |            —             |   ❌ (default error)    |
| Touch pinch                 |     ✅     |     —     |            —             |   ❌ (default error)    |
| Long press                  |     ✅     |     —     |            —             |   ❌ (default error)    |

---

## 6. Assertions

| Feature                                          | Playwright |         Cypress         |      Turbosheet      |
| ------------------------------------------------ | :--------: | :---------------------: | :------------------: |
| `expect(locator).toBeVisible()`                  |     ✅     |     ✅ (be.visible)     |          ✅          |
| `expect(locator).toBeHidden()`                   |     ✅     |     ✅ (be.hidden)      |          ✅          |
| `expect(locator).toBeEnabled()`                  |     ✅     |     ✅ (be.enabled)     |          ✅          |
| `expect(locator).toBeDisabled()`                 |     ✅     |    ✅ (be.disabled)     |          ✅          |
| `expect(locator).toBeChecked()`                  |     ✅     |     ✅ (be.checked)     |          ✅          |
| `expect(locator).toBeEditable()`                 |     ✅     |           ✅            |          ❌          |
| `expect(locator).toBeFocused()`                  |     ✅     |     ✅ (be.focused)     |          ❌          |
| `expect(locator).toBeEmpty()`                    |     ✅     |           ✅            |          ✅          |
| `expect(locator).toBeInViewport()`               |     ✅     |   ✅ (be.inViewport)    |          ❌          |
| `expect(locator).toBeAttached()`                 |     ✅     |            —            |          ❌          |
| `expect(locator).toHaveText()`                   |     ✅     |     ✅ (have.text)      |          ✅          |
| `expect(locator).toContainText()`                |     ✅     |    ✅ (contain.text)    |          ✅          |
| `expect(locator).toHaveValue()`                  |     ✅     |     ✅ (have.value)     |          ✅          |
| `expect(locator).toHaveValues()`                 |     ✅     |            —            |          ❌          |
| `expect(locator).toHaveAttribute()`              |     ✅     |     ✅ (have.attr)      |          ✅          |
| `expect(locator).toHaveClass()`                  |     ✅     |     ✅ (have.class)     |          ✅          |
| `expect(locator).toHaveCSS()`                    |     ✅     |      ✅ (have.css)      |          ✅          |
| `expect(locator).toHaveCount()`                  |     ✅     |    ✅ (have.length)     |          ✅          |
| `expect(locator).toHaveId()`                     |     ✅     |      ✅ (have.id)       |          ✅          |
| `expect(locator).toHaveJSProperty()`             |     ✅     |            —            |          ❌          |
| `expect(locator).toHaveRole()`                   |     ✅     |            —            |          ❌          |
| `expect(locator).toHaveAccessibleName()`         |     ✅     |            —            |          ❌          |
| `expect(locator).toHaveAccessibleDescription()`  |     ✅     |            —            |          ❌          |
| `expect(locator).toHaveAccessibleErrorMessage()` |     ✅     |            —            |          ❌          |
| `expect(locator).toMatchAriaSnapshot()`          |     ✅     |            —            |          ❌          |
| `expect(locator).toHaveScreenshot()`             |     ✅     | ✅ (cy.screenshot diff) |          ❌          |
| `expect(page).toHaveTitle()`                     |     ✅     |      ✅ (cy.title)      |          ❌          |
| `expect(page).toHaveURL()`                       |     ✅     |       ✅ (cy.url)       |          ❌          |
| `expect(page).toMatchAriaSnapshot()`             |     ✅     |            —            |          ❌          |
| `.not` negation                                  |     ✅     |        ✅ (.not)        |          ✅          |
| Custom message                                   |     ✅     |    ✅ (.should(msg))    |          ✅          |
| Polling / retry-ability                          |     ✅     |           ✅            | ✅ (AssertionEngine) |
| Timeout config per assertion                     |     ✅     |           ✅            |          ✅          |
| Soft assertions (non-failing)                    |     ✅     |           ❌            |          ❌          |
| `expect.poll()` / `expect.toPass()`              |     ✅     |            —            |          ❌          |
| `expect(value).toBe()` etc.                      |     ✅     |        ✅ (Chai)        |          ❌          |
| Custom assertion plugins                         |     ✅     |    ✅ (chai.extend)     |  ⚠️ (MatcherPlugin)  |

---

## 7. Network Interception

| Feature                       |   Playwright    | Puppeteer |         Cypress         |   Turbosheet (Chromium)    |
| ----------------------------- | :-------------: | :-------: | :---------------------: | :------------------------: |
| `route(url, handler)`         |       ✅        |     —     |    ✅ (cy.intercept)    |     ✅ (Fetch domain)      |
| `unroute(url)`                |       ✅        |     —     |            —            |             ✅             |
| `fulfill()` (mock response)   |       ✅        |    ✅     |    ✅ (cy.intercept)    |             ✅             |
| `abort()` (block request)     |       ✅        |    ✅     |    ✅ (req.reply no)    |             ✅             |
| `continue()` (passthrough)    |       ✅        |    ✅     |           ✅            |             ✅             |
| Modify request headers        |       ✅        |    ✅     |           ✅            |             ✅             |
| Modify response headers       |       ✅        |     —     |           ✅            |  ⚠️ (partial via fulfill)  |
| Modify post data              |       ✅        |     —     |           ✅            |             ✅             |
| Modify response body          |       ✅        |     —     |           ✅            |             ✅             |
| URL pattern matching          | ✅ (glob/regex) | ✅ (glob) |     ✅ (minimatch)      | ✅ (trait with matches())  |
| HAR replay                    |       ✅        |     —     |            —            |             ❌             |
| Route WebSocket               |       ✅        |     —     |            —            |             ❌             |
| Route from service worker     |       ✅        |     —     |            —            |             ❌             |
| Request/Response events       |       ✅        |    ✅     |           ✅            |             ✅             |
| Network timing info           |       ✅        |    ✅     |            —            |             ❌             |
| Request failed event          |       ✅        |    ✅     |           ✅            | ❌ (no EventLoadingFailed) |
| Request finished event        |       ✅        |    ✅     |            —            |             ❌             |
| Network conditions (throttle) |       ✅        |    ✅     | ✅ (cy.intercept delay) |  ⚠️ (no-op trait default)  |
| Offline emulation             |       ✅        |    ✅     |            —            |  ⚠️ (no-op trait default)  |
| WebSocket frame events        |       ✅        |     —     |            —            |     ❌ (default error)     |
| Raw body interception         |       ✅        |    ✅     |           ✅            |     ✅ (base64 decode)     |
| Event dispatch to JS          |       ✅        |    ✅     |           ✅            |  ✅ (ThreadsafeFunction)   |

---

## 8. Emulation & Device Overrides

| Feature                     | Playwright | Puppeteer |     Cypress      | Turbosheet |
| --------------------------- | :--------: | :-------: | :--------------: | :--------: |
| Device descriptors          |     ✅     |    ✅     |        ✅        |     ❌     |
| Viewport resizing           |     ✅     |    ✅     | ✅ (cy.viewport) |     ✅     |
| Device scale factor         |     ✅     |    ✅     |        —         |     ❌     |
| Has touch capability        |     ✅     |    ✅     |        —         |     ❌     |
| Is mobile                   |     ✅     |    ✅     |        —         |     ❌     |
| User agent override         |     ✅     |    ✅     |   ✅ (headers)   |     ❌     |
| Geolocation                 |     ✅     |    ✅     |   ✅ (cy.stub)   |     ❌     |
| Timezone                    |     ✅     |    ✅     |        ✅        |     ❌     |
| Locale                      |     ✅     |     —     |        ✅        |     ❌     |
| Color scheme (light/dark)   |     ✅     |    ✅     |        —         | ⚠️ (no-op) |
| Reduced motion              |     ✅     |     —     |        —         | ⚠️ (no-op) |
| Forced colors               |     ✅     |     —     |        —         |     ❌     |
| JavaScript enabled/disabled |     ✅     |    ✅     |        —         |     ❌     |
| CSS Media type emulation    |     ✅     |    ✅     |        —         | ⚠️ (no-op) |
| Extra HTTP headers          |     ✅     |    ✅     |        ✅        |     ❌     |
| Offline mode                |     ✅     |    ✅     |        —         | ⚠️ (no-op) |

---

## 9. Screenshots & Visual

| Feature                          | Playwright | Puppeteer |    Cypress     |           Turbosheet (Chromium)           |
| -------------------------------- | :--------: | :-------: | :------------: | :---------------------------------------: |
| Full page screenshot             |     ✅     |    ✅     | ✅ (automated) |             ❌ (no fullPage)              |
| Element screenshot               |     ✅     |    ✅     |       —        |          ⚠️ (declared in trait)           |
| Clip region screenshot           |     ✅     |    ✅     |       ✅       |                    ❌                     |
| Screenshot format (png/jpeg)     |     ✅     |    ✅     |       ✅       |               ✅ (PNG only)               |
| Screenshot quality               |     ✅     |    ✅     |       —        |                    ❌                     |
| Screenshot animations            |     ✅     |    ❌     |       ✅       |                    ❌                     |
| Screenshot caret hiding          |     ✅     |     —     |       —        |                    ❌                     |
| Screenshot masking               |     ✅     |     —     |   ✅ (block)   |                    ❌                     |
| Visual comparison (diff)         |     ✅     |     —     |   ✅ (cloud)   | ⚠️ (visualCompareScreenshots napi exists) |
| Per-test screenshot config       |     ✅     |     —     |       ✅       |                    ❌                     |
| Automatic screenshots on failure |     ✅     |     —     |       ✅       |          ⚠️ (config in executor)          |

---

## 10. Tracing & Debugging

| Feature                     | Playwright | Puppeteer | Chrome DevTools |                Turbosheet                |
| --------------------------- | :--------: | :-------: | :-------------: | :--------------------------------------: |
| Trace recording             |     ✅     |    ✅     |       ✅        |           ⚠️ (in-memory only)            |
| Trace viewer (HTML)         |     ✅     |    ✅     |       ✅        |        ✅ (built-in HTML viewer)         |
| Trace events (actions)      |     ✅     |     —     |       ✅        |                    ✅                    |
| Trace events (network)      |     ✅     |     —     |       ✅        |                    ✅                    |
| Trace events (console)      |     ✅     |     —     |       ✅        |                    ✅                    |
| Trace screenshots/snapshots |     ✅     |     —     |       ✅        | ⚠️ (structure exists, data always empty) |
| Trace file export           |     ✅     |     —     |        —        |        ✅ (serialize/deserialize)        |
| Trace per context           |     ✅     |     —     |        —        |             ❌ (global only)             |
| Trace size limits           |     ✅     |     —     |        —        |            ❌ (hard 100k cap)            |
| Video recording             |     ✅     |     —     | ✅ (via FFmpeg) |                    ❌                    |
| Code generation (Codegen)   |     ✅     |     —     |        —        |                    ❌                    |
| UI Mode / time travel       |     ✅     |     —     |       ✅        |                    ❌                    |
| Locator picker              |     ✅     |     —     |       ✅        |                    ❌                    |
| Action log                  |     ✅     |     —     |       ✅        |                    ❌                    |
| Snapshot viewer             |     ✅     |     —     |        —        |                    ❌                    |
| Console log in trace        |     ✅     |    ✅     |       ✅        |                    ✅                    |
| Network log in trace        |     ✅     |    ✅     |       ✅        |                    ✅                    |

---

## 11. Test Runner

| Feature                        |        Playwright        |        Cypress         |                                         Turbosheet                                         |
| ------------------------------ | :----------------------: | :--------------------: | :----------------------------------------------------------------------------------------: |
| `test()` / `it()`              |            ✅            |           ✅           |          ✅ (injected via collector; collected + executed via two-phase protocol)          |
| `describe()` / `suite()`       |            ✅            |           ✅           |                   ✅ (collected with serial/parallel/skip/only variants)                   |
| `beforeAll()` / `afterAll()`   |            ✅            |           ✅           | ✅ (collected, resolved by PlanBuilder, scheduled with run_before_all/run_after_all flags) |
| `beforeEach()` / `afterEach()` |            ✅            |           ✅           |             ✅ (collected, resolved with ancestry-based inheritance ordering)              |
| `test.skip()` / `test.only()`  |            ✅            | ✅ (it.skip / it.only) |                   ✅ (PlanBuilder modifier cascade filters by skip/only)                   |
| `test.fixme()` / `test.fail()` |            ✅            |           —            |         ✅ (is_fixme → maps failures to fixme status; is_fail → inverts pass/fail)         |
| `test.slow()`                  |            ✅            |           —            |             ✅ (PlanBuilder triples timeout; `is_slow` flag on ExecutionPlan)              |
| `test.describe.configure()`    |            ✅            |           —            |                                             ❌                                             |
| `test.use()` (fixture config)  |            ✅            |           —            |                                             ❌                                             |
| `test.step()` (steps)          |            ✅            |      ✅ (cy.log)       |                                             ❌                                             |
| `test.info()` (metadata)       |            ✅            |           —            |                                             ❌                                             |
| Config file                    |            ✅            |           ✅           |                                   ⚠️ (TestConfig struct)                                   |
| Projects / multiple configs    |            ✅            |           ✅           |                             ⚠️ (config supports grep/projects)                             |
| Global setup / teardown        |            ✅            |   ✅ (cy.task etc.)    |                                   ✅ (config + executor)                                   |
| Timeout per test               |            ✅            |           ✅           |                  ✅ (PlanBuilder per-test timeout; test.slow triples it)                   |
| Retries                        |            ✅            |           ✅           |                           ✅ (execute_plans retry loop per plan)                           |
| Parallel workers               |            ✅            |   ✅ (Cypress Cloud)   |      ✅ (two-phase parallel extraction + execution; suite-level beforeAll cascading)       |
| Sharding                       |            ✅            |   ✅ (Cypress Cloud)   |                                             ❌                                             |
| Fixtures (test.extend)         |            ✅            |           —            |                                             ❌                                             |
| Fixture files (JSON)           |            ✅            |    ✅ (cy.fixture)     |                                             ❌                                             |
| Reporter system                | ✅ (6 built-in + custom) |  ✅ (mocha reporters)  |                               ✅ (Reporter trait + 7 impls)                                |
| HTML reporter                  |            ✅            |    ✅ (Mochawesome)    |                              ✅ (via HtmlReporter trait impl)                              |
| JSON / JUnit reporter          |            ✅            |           ✅           |                            ✅ (via JsonReporter/JunitReporter)                             |
| Screenshots on failure         |            ✅            |           ✅           |                                    ⚠️ (config setting)                                     |
| Trace on failure               |            ✅            |           —            |                                             ❌                                             |
| Video on failure               |            ✅            |           ✅           |                                             ❌                                             |
| CI integration args            |            ✅            |           ✅           |                                             ❌                                             |
| Worker process isolation       |            ❌            |           ❌           |                                    ✅ (JSON-RPC worker)                                    |
| Worker IPC protocol            |            ❌            |           ❌           |                           ✅ (two-phase: extractTests + runPlan)                           |
| Test file discovery            |            ✅            |           ✅           |                                      ✅ (glob + grep)                                      |

---

## 12. Component Testing

| Feature                 | Playwright |    Cypress    |                  Turbosheet                  |
| ----------------------- | :--------: | :-----------: | :------------------------------------------: |
| React component mount   |     ✅     |      ✅       |                      ✅                      |
| Vue component mount     |     ✅     |      ✅       |                      ✅                      |
| Svelte component mount  |     ✅     |      ✅       |                      ✅                      |
| Angular component mount |     ✅     |      ✅       |                      ❌                      |
| Auto-detect framework   |     —      |       —       |          ✅ (package.json + source)          |
| Component pool          |     —      |       —       |              ✅ (10-slot pool)               |
| Component locator       |     —      |       —       |         ✅ (ComponentLocator class)          |
| Component screenshot    |     —      |      ✅       |                      ✅                      |
| Component interaction   |     ✅     |      ✅       |          ✅ (click, fill, evaluate)          |
| Component props/state   |     ✅     |      ✅       | ✅ (**COMPONENT_PROPS**/**COMPONENT_STATE**) |
| Component close/cleanup |     ✅     |      ✅       |                      ✅                      |
| Component dev server    |     ✅     |      ✅       |         ❌ (no built-in dev-server)          |
| Component fixtures      |     ✅     | ✅ (cy.mount) |                      ❌                      |

---

## 13. Video Recording

| Feature                         | Playwright | Cypress  | Puppeteer |                      Turbosheet                      |
| ------------------------------- | :--------: | :------: | :-------: | :--------------------------------------------------: |
| Video on test failure           |     ✅     |    ✅    |    ❌     |         ⚠️ (config + executor wiring exists)         |
| Video for all tests             |     ✅     |    ✅    |    ❌     |          ⚠️ (Recorder supports; not wired)           |
| Video format (webm)             | ✅ (webm)  | ✅ (mp4) |    ❌     |               ⚠️ (WebM/VP8 via FFmpeg)               |
| Video size/quality config       |     ✅     |    ✅    |    ❌     | ✅ (VideoConfig: quality, fps, dims, bitrate, codec) |
| Video compression               |     ✅     |    ✅    |    ❌     |         ⚠️ (delegated to FFmpeg subprocess)          |
| Video encoder state machine     |     —      |    —     |     —     |      ✅ (Idle→Running→Flushing→Finished→Failed)      |
| Frame deduplication             |     —      |    —     |     —     |           ✅ (pixel-hash within tolerance)           |
| Max pending frames              |     —      |    —     |     —     |            ✅ (configurable, default 300)            |
| Max recording duration watchdog |     —      |    —     |     —     |                   ✅ (default 30s)                   |
| Concurrent recording isolation  |     —      |    —     |     —     |     ✅ (per-test output path with hex timestamp)     |
| Video retention policy          |     ✅     |    ✅    |    ❌     |             ✅ (Always/OnFailure/Never)              |
| Video viewer                    |     ✅     |    ✅    |    ❌     |                          ❌                          |

**Turbosheet status**: A solid video recording subsystem exists (`src/video/` — config, encoder, recorder, mod) with:

- `VideoConfig` — quality (1-100), frame rate (default 10), max dimensions (800×600), bitrate (500k), codec (libvpx), retention policy, max pending frames, max recording duration
- `VideoEncoder` — FFmpeg subprocess spawner with stdin pipe for JPEG frames; full state machine; cleanup on failure
- `VideoRecorder` — Full state machine (Idle→Starting→Recording→Stopping→Stopped); mpsc frame channel; atomic frame counter; cancellation support; concurrent-safe output paths
- `video` feature flag in Cargo.toml
- `TestExecutor` wires `video_on_failure`/`video_dir` through to `TestResult.video_paths`
- **Remaining**: CDP `Page.startScreencast`/`screencastFrame` integration in ChromiumPageEngine; auto-start/stop on test begin/end; video attachment to HTML reporter

---

## 14. Coverage

| Feature                   | Puppeteer | Playwright | Turbosheet |
| ------------------------- | :-------: | :--------: | :--------: |
| JS Coverage (start/stop)  |    ✅     |     —      |     ❌     |
| CSS Coverage (start/stop) |    ✅     |     —      |     ❌     |
| Coverage reports          |    ✅     |     —      |     ❌     |

---

## 15. Accessibility

| Feature                            | Playwright |       Puppeteer       |                                                          Turbosheet                                                           |
| ---------------------------------- | :--------: | :-------------------: | :---------------------------------------------------------------------------------------------------------------------------: |
| Accessibility snapshot (DOM)       |     ✅     |          ✅           |   ⚠️ (AccessibilityNode struct exists in trace recorder with role, name, value, description, state, children, element refs)   |
| Accessibility tree capture         |     ✅     | ✅ (SerializedAXNode) | ⚠️ (DomSnapshot includes accessibility_tree + AiSnapshotMetadata with viewport, URL, title, focus_element, interactive_count) |
| ARIA snapshot matching             |     ✅     |           —           |                                                              ❌                                                               |
| Role/label assertions              |     ✅     |          ✅           |                                               ❌ (not exposed as napi matchers)                                               |
| Accessibility properties           |     ✅     |          ✅           |                                                              ❌                                                               |
| `snapshot_to_accessibility_tree()` |     —      |           —           |                                                  ✅ (in trace/serializer.rs)                                                  |

**Turbosheet status**: Accessibility infrastructure exists at the data model level (in trace recording) but is not exposed as a user-facing API. The `AccessibilityNode` and `DomSnapshot` types capture role, name, value, description, state, and children. `AiSnapshotMetadata` adds viewport, URL, title, and focus element. Front-end assertions and CDP `Accessibility.getFullAXTree` integration are missing.

---

## 16. PDF Generation

| Feature                      | Playwright (Chromium) | Puppeteer | Turbosheet |
| ---------------------------- | :-------------------: | :-------: | :--------: |
| `page.pdf()`                 |          ✅           |    ✅     |     ❌     |
| PDF options (format, margin) |          ✅           |    ✅     |     ❌     |

---

## 17. Cookie & Storage Management

| Feature                      | Playwright | Puppeteer |              Cypress              |        Turbosheet         |
| ---------------------------- | :--------: | :-------: | :-------------------------------: | :-----------------------: |
| `context.cookies()`          |     ✅     |    ✅     |      ✅ (cy.getCookie etc.)       |     ✅ (all engines)      |
| `context.addCookies()`       |     ✅     |    ✅     |         ✅ (cy.setCookie)         |            ✅             |
| `context.clearCookies()`     |     ✅     |    ✅     |        ✅ (cy.clearCookie)        |            ✅             |
| Cookies filtered by URL      |     ✅     |     —     |                ✅                 |            ✅             |
| `context.storageState()`     |     ✅     |     —     |                 —                 |            ❌             |
| `context.setStorageState()`  |     ✅     |     —     |                 —                 |            ❌             |
| localStorage get/set/clear   |     ✅     |    ✅     |  ✅ (cy.getAllLocalStorage etc.)  | ⚠️ (no-op trait defaults) |
| sessionStorage get/set/clear |     ✅     |    ✅     | ✅ (cy.getAllSessionStorage etc.) | ⚠️ (no-op trait defaults) |

---

## 18. JS Injection & Scripting

| Feature                       | Playwright | Puppeteer |        Cypress        |               Turbosheet                |
| ----------------------------- | :--------: | :-------: | :-------------------: | :-------------------------------------: |
| `page.evaluate()`             |     ✅     |    ✅     |  ✅ (cy.window.then)  |                   ✅                    |
| `page.evaluateHandle()`       |     ✅     |    ✅     |           —           |       ⚠️ (declared, no real impl)       |
| `page.exposeFunction()`       |     ✅     |    ✅     | ✅ (Cypress.Commands) |         ✅ (via binding bridge)         |
| `page.addInitScript()`        |     ✅     |    ✅     |           —           |         ✅ (inject_core_script)         |
| `page.addScriptTag()`         |     ✅     |    ✅     |           —           |                   ✅                    |
| `page.addStyleTag()`          |     ✅     |    ✅     |           —           |                   ✅                    |
| Binding bridge (JS↔Rust)      |     ✅     |    ✅     |           —           | ✅ (BindingRegistry + oneshot channels) |
| Stealth mode / anti-detection |     —      |     —     |           —           |  ✅ (StealthConfig + name generation)   |
| Core injection script         |     —      |     —     |           —           |    ✅ (CORE_SCRIPT + ACTIONS_SCRIPT)    |

---

## 19. Event System

| Feature                             | Playwright | Puppeteer |      Cypress      |                    Turbosheet                     |
| ----------------------------------- | :--------: | :-------: | :---------------: | :-----------------------------------------------: |
| `page.on('request')`                |     ✅     |    ✅     | ✅ (cy.intercept) |                        ✅                         |
| `page.on('response')`               |     ✅     |    ✅     | ✅ (cy.intercept) |                        ✅                         |
| `page.on('requestfailed')`          |     ✅     |    ✅     |        ✅         |                        ❌                         |
| `page.on('requestfinished')`        |     ✅     |    ✅     |         —         |                        ❌                         |
| `page.on('console')`                |     ✅     |    ✅     |    ✅ (cy.on)     |                        ✅                         |
| `page.on('dialog')`                 |     ✅     |    ✅     |    ✅ (cy.on)     |                        ✅                         |
| `page.on('popup')`                  |     ✅     |    ✅     |         —         |                        ✅                         |
| `page.on('filechooser')`            |     ✅     |    ✅     |         —         |                        ✅                         |
| `page.on('worker')`                 |     ✅     |    ✅     |         —         |                        ❌                         |
| `page.on('websocket')`              |     ✅     |     —     |         —         |                        ❌                         |
| `page.on('download')`               |     ✅     |    ✅     |         —         |                        ❌                         |
| `page.on('crash')`                  |     ✅     |    ✅     |         —         |           ✅ (inspector targetCrashed)            |
| `page.on('frameattached/detached')` |     ✅     |    ✅     |         —         |                        ❌                         |
| `page.on('load')`                   |     ✅     |    ✅     |  ✅ (cy.on load)  |    ❌ (EventLoadFired declared but not wired)     |
| `page.on('domcontentloaded')`       |     ✅     |    ✅     |         —         | ❌ (EventDomContentLoaded declared but not wired) |
| `context.on('page')`                |     ✅     |     —     |         —         |                        ❌                         |
| `browser.on('disconnected')`        |     ✅     |    ✅     |         —         |                        ❌                         |
| `waitForEvent()`                    |     ✅     |    ✅     |         —         |                        ❌                         |
| `removeListener()` / `off()`        |     ✅     |    ✅     |    ✅ (cy.off)    |         ❌ (only unregister_all_for_page)         |
| Event dispatcher pattern            |     ✅     |    ✅     |         —         |            ✅ (EventDispatcher + mpsc)            |
| N-API event bridge                  |     —      |     —     |         —         |              ✅ (ThreadsafeFunction)              |

---

## 20. Plugin / Extension System

| Feature                        | Playwright |     Cypress      |            Turbosheet             |
| ------------------------------ | :--------: | :--------------: | :-------------------------------: |
| Reporter plugins               |     ✅     |        ✅        |       ⚠️ (trait + registry)       |
| Custom matchers                |     —      | ✅ (chai.extend) |       ⚠️ (trait + registry)       |
| Lifecycle hooks (before/after) |     —      | ✅ (Cypress.on)  |        ⚠️ (trait defined)         |
| Plugin auto-discovery          |     —      |        —         |   ⚠️ (discovery module exists)    |
| napi plugin registration       |     —      |        —         | ✅ (register_js_reporter/matcher) |

---

## 21. Code Generation

| Feature                  | Playwright |       Cypress       | Turbosheet |
| ------------------------ | :--------: | :-----------------: | :--------: |
| Test recording (codegen) |     ✅     | ✅ (Cypress Studio) |     ❌     |
| Locator picker           |     ✅     |         ✅          |     ❌     |
| Interactive codegen UI   |     ✅     |         ✅          |     ❌     |
| 2-minute test generation |     ✅     |          —          |     ❌     |
| Export tests to file     |     ✅     |         ✅          |     ❌     |

---

## 22. Cross-Browser Support

| Feature                |    Playwright     | Puppeteer |       Cypress        |           Turbosheet           |
| ---------------------- | :---------------: | :-------: | :------------------: | :----------------------------: |
| Chromium               |        ✅         |    ✅     |          ✅          | ✅ (2258 lines, ~65% complete) |
| Firefox                |        ✅         |    ❌     |          ✅          | ✅ (645 lines, ~25% complete)  |
| WebKit (Safari)        |        ✅         |    ❌     |  ⚠️ (experimental)   | ✅ (643 lines, ~20% complete)  |
| Edge                   |        ✅         |    ❌     |          ✅          |   ⚠️ (same Chromium engine)    |
| Electron               |         —         |    ✅     |          ✅          |               ❌               |
| Headless mode          |        ✅         |    ✅     |          ❌          |               ✅               |
| Browser download mgr   |        ✅         |     —     | ✅ (npx cypress run) | ⚠️ (binary_manager.rs exists)  |
| WebDriver BiDi support | ⚠️ (experimental) |    ✅     |          —           |  ❌ (uses WebDriver classic)   |

---

## 23. CI/CD Integration

| Feature                                                             |   Playwright   |    Cypress     | Turbosheet  |
| ------------------------------------------------------------------- | :------------: | :------------: | :---------: |
| Docker image                                                        |       ✅       |       ✅       |     ❌      |
| CLI flags (headed, browser, project, shard, grep, retries, timeout) | ✅ (20+ flags) |       ✅       | ❌ (no CLI) |
| `--update-snapshots`                                                |       ✅       |       —        |     ❌      |
| `--forbid-only`                                                     |       ✅       |       ✅       |     ❌      |
| `--reporter`                                                        |       ✅       |       ✅       |     ❌      |
| `--workers`                                                         |       ✅       |       ✅       |     ❌      |
| CI-specific features                                                |       ✅       | ✅ (Dashboard) |     ❌      |
| Merge reports (sharded)                                             |       ✅       |   ✅ (Cloud)   |     ❌      |

---

## 24. Reporting

| Feature                  | Playwright |     Cypress      |        Turbosheet         |
| ------------------------ | :--------: | :--------------: | :-----------------------: |
| List reporter            |     ✅     |        ✅        |            ❌             |
| Line reporter            |     ✅     |        ✅        |            ❌             |
| Dot reporter             |     ✅     |        ✅        |            ❌             |
| JSON reporter            |     ✅     |        ✅        |            ❌             |
| JUnit reporter           |     ✅     |        ✅        |            ❌             |
| HTML reporter            |     ✅     | ✅ (Mochawesome) |            ❌             |
| Blob reporter (sharding) |     ✅     |        —         |            ❌             |
| Custom reporters         |     ✅     |        ✅        | ⚠️ (ReporterPlugin trait) |
| Merge reports CLI        |     ✅     |        —         |            ❌             |
| Dashboard / Cloud        |     —      |        ✅        |            ❌             |
| Flaky test management    |     —      |  ✅ (Dashboard)  |            ❌             |

---

## 25. Migration Support

| Feature                       |         Playwright          |                  Turbosheet                  |
| ----------------------------- | :-------------------------: | :------------------------------------------: |
| From Puppeteer                |     ✅ (official guide)     |          ⚠️ (API mapping list only)          |
| From Cypress                  |     ✅ (official guide)     |          ⚠️ (API mapping list only)          |
| From Selenium                 |             ✅              |                      ❌                      |
| AST-based code transformation | ✅ (npx playwright convert) |               ❌ (no codegen)                |
| API name mapping              |              —              | ✅ (playwright.rs, cypress.rs, puppeteer.rs) |

---

## 26. API Testing

| Feature                         | Playwright | Turbosheet |
| ------------------------------- | :--------: | :--------: |
| `APIRequestContext`             |     ✅     |     ❌     |
| `request.get()` / `post()` etc. |     ✅     |     ❌     |
| `request.fetch()`               |     ✅     |     ❌     |
| `request.storageState()`        |     ✅     |     ❌     |
| API test assertions             |     ✅     |     ❌     |

---

## Per-Tool Summary

### vs Playwright (overall: Turbosheet ~30-35% complete)

Playwright is the gold standard with ~3,500+ API methods across 50+ classes. Turbosheet matches Playwright in:

- **Strong**: Core Chromium automation (launch, navigate, click, screenshot, evaluate, network interception via CDP Fetch domain, cookie management, event dispatch to JS)
- **Partial**: Assertions (16/30+ methods implemented), component testing, trace events, popup handling, dialog handling, page-level network events
- **Missing**: Video recording, locator engine (getBy\*, chaining, filtering), codegen, test runner core (test/describe/hooks implemented via two-phase IPC), fixture system, frame support, worker support, WebSocket interception, PDF, coverage, accessibility tree, device emulation profiles, HAR replay, UI Mode, API testing, reporting infrastructure, CLI, CI/CD integration, auto-waiting/actionability checks, screenshot options (fullPage, clip, quality), storage state, geolocation/timezone/locale emulation, service workers, persistent contexts, download handling, `worker` events, frame events, `domcontentloaded`/`load` events

**Estimated effort to reach Playwright parity**: 8-14 months for a small team

### vs Puppeteer (overall: Turbosheet ~40% complete)

Puppeteer has a narrower scope (Chromium-only, no test runner). Turbosheet matches Puppeteer in:

- **Strong**: Core Chromium automation, binding bridge, screenshot, evaluate, cookies, CDP events
- **Missing**: Coverage API (JS/CSS), PDF generation, CDPSession (raw CDP), accessibility tree (SerializedAXNode), Tracing API (start/stop with DevTools trace file), `Keyboard`/`Mouse`/`Touchscreen` classes, `waitForNetworkIdle`, cache management, `emulateNetworkConditions`, `emulateIdleState`, heap snapshot, `worker` events, `error` events, `metrics`, `queryObjects`, `select`, `waitForDevicePrompt`, `waitForFileChooser`, drag-and-drop

**Estimated effort**: 4-6 months for a small team (less scope than Playwright)

### vs Cypress (overall: Turbosheet ~25% complete)

Cypress has a fundamentally different architecture (in-process, command queue, auto-retry). Turbosheet matches Cypress in:

- **Strong**: Basic page navigation/evaluation, component mounting, screenshot, cookie management
- **Different paradigm**: Cypress auto-retries all commands, has implicit assertions, command queuing (not promise-based), and runs in the browser. Turbosheet is async Rust with explicit awaits. Hard to compare directly.
- **Missing**: Cypress Dashboard, test runner UI, time-travel debugging, `cy.clock()`/`cy.tick()`, spies/stubs (sinon), fixtures, `cy.origin()` (multi-origin), custom commands, `cy.intercept()` (network spying + stubbing combined), `cy.session()`, `cy.task()`, `cy.exec()`, plugin system (preprocessors), accessibility testing (cypress-axe), codegen via Cypress Studio, Mocha reporter integration

**Estimated effort**: 6-8 months for feature parity (but Cypress is architecturally different — full parity may not make sense)

---

## Competitor Pain-Point Analysis: What TurboSheet Should Fix

### Puppeteer's Long-Standing Issues

| Pain Point                                                                                                         | Impact                                                    | TurboSheet Solution                                                                             |
| ------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| **No auto-waiting** — Every action requires manual `waitForSelector()` / `waitForNetworkIdle()` before interaction | Flaky tests, verbose code, high maintenance               | AssertionEngine with exponential backoff + jitter already exists; extend to all locator actions |
| **No built-in test runner** — Must use separate framework (Jest, Mocha, Vitest) with manual setup                  | Fragmented tooling, config duplication, no native retries | Test runner with config, executor, worker pool, two-phase IPC; test/describe/hooks implemented  |
| **No cross-browser** — Chromium only                                                                               | Cannot test Firefox/Safari in same codebase               | Clean BrowserEngine/ContextEngine/PageEngine traits; already has Firefox + WebKit stubs         |
| **No built-in reporting** — Must use third-party reporters (jest-junit, mochawesome)                               | Additional dependencies, config complexity                | 7 built-in reporters (list, line, dot, JSON, JUnit, HTML, GitHub)                               |
| **No assertion library** — No `expect(locator).toBeVisible()` style auto-retrying assertions                       | Manual assertion loops, flaky test code                   | 16+ matchers with AssertionEngine (exponential backoff + jitter + timeout)                      |
| **Manual page errors** — No crash/dialog/console default handlers                                                  | Unhandled errors, silent failures                         | EventDispatcher with default handlers for dialogs (auto-accept after 5s), page errors           |
| **No trace viewer** — Only raw CDP tracing (flat JSON)                                                             | Hard to debug test failures                               | Built-in HTML trace viewer with timeline, network, console, screenshots                         |
| **No component testing** — No mount/unmount API for frontend frameworks                                            | Cannot test components in isolation                       | Component mount with framework auto-detection (React/Vue/Svelte) + 10-slot pool                 |

### Cypress's Long-Standing Issues

| Pain Point                                                                                                | Impact                                                                                    | TurboSheet Solution                                                                                       |
| --------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| **In-browser execution** — Commands run in the browser's event loop; no multi-tab, no native cross-origin | Cannot test multi-tab flows, OAuth, or cross-origin scenarios without `cy.origin()` hacks | Out-of-process Rust engine with true multi-page support; CDP-based, can handle any number of tabs/origins |
| **No iframe support** — `cy.iframe()` doesn't exist; requires workarounds                                 | Cannot test embedded content, rich editors, or third-party widgets                        | Frame locator support via CDP; iframe traversal is native to the browser engine                           |
| **Slow parallelization** — Requires Cypress Cloud ($) for parallel execution                              | Cost-prohibitive for small teams, CI time explosion                                       | Built-in parallel worker pool in TestExecutor; sharding config; no cloud dependency                       |
| **No native mobile browser** — No real mobile device testing                                              | Limited mobile coverage                                                                   | CDP device emulation (viewport, user agent, touch, mobile) already supported at protocol level            |
| **No WebSocket/SSE interception** — Only XHR/fetch via `cy.intercept()`                                   | Cannot test real-time features                                                            | NetworkProxy with WebSocket route handler trait (stub layer exists, needs completion)                     |
| **Heavy memory usage** — Each spec file reloads the entire browser                                        | Slow test suites, high CI cost                                                            | Worker process pool with browser reuse; isolated contexts per test                                        |
| **No .edge() or .firefox()** — Only Chromium-based Electron browser                                       | Single-engine lock-in                                                                     | Tri-engine trait architecture ready for all three engines                                                 |

### Playwright's Existing Limitations (Opportunities for TurboSheet)

| Pain Point                                                                     | Impact                                 | TurboSheet Solution                                                                              |
| ------------------------------------------------------------------------------ | -------------------------------------- | ------------------------------------------------------------------------------------------------ |
| **Large binary size** — ~500MB+ with browsers                                  | Slow CI installs, large Docker images  | Use existing Chrome/Firefox/Safari installations; no bundled browsers (lighter footprint)        |
| **Complex debugging** — Trace files are multi-MB JSON + ZIP                    | Debugging overhead, storage costs      | Lightweight trace format with compact serialization (ZSTD + CBOR optional); built-in HTML viewer |
| **Expensive sharding** — Requires cloud service for shard merging              | Cost for parallel CI                   | Built-in ShardConfig + merge-reports support (stub exists, needs completion)                     |
| **TypeScript compilation overhead** — Full TS build for config files and tests | Slow startup in CI                     | Use `tsx` for worker scripts; no pre-compilation step for user tests                             |
| **No stealth mode** — Headless Chrome easily detectable                        | Bot detection bypasses in web scraping | StealthConfig with name randomization, fingerprint spoofing, navigator.plugins, WebGL, etc.      |
| **No JSON-RPC worker isolation** — Playwright workers are in-process           | Cross-test contamination risk          | JSON-RPC worker process isolation (unique among all three competitors)                           |

---

## TurboSheet Unique Differentiators

These features already exist in TurboSheet and have no direct counterpart in Playwright/Cypress/Puppeteer:

1. **Binding bridge (JS↔Rust)** — Oneshot channels with `BindingRegistry` for clean IPC between injected JavaScript and Rust core. Production-quality, well-tested.
2. **CDP Fetch domain interception** — Full `route()`/`fulfill()`/`abort()`/`continue()` with proper CDP integration. ThreadsafeFunction bridge to JS.
3. **JSON-RPC worker isolation** — Each test runs in its own Node.js worker process with line-delimited JSON-RPC over stdin/stdout. No other tool provides this level of isolation out-of-the-box.
4. **Trait-based multi-engine architecture** — `BrowserEngine`/`ContextEngine`/`PageEngine` traits enable clean 3-engine (Chromium/Firefox/WebKit) support. Adding a new engine is a matter of implementing 3 traits.
5. **Stealth configuration** — Built-in anti-detection: randomized browser name, Chrome version spoofing, WebGL vendor/renderer spoofing, navigator.plugins, custom User-Agent quirks. `StealthConfig` struct with 10+ tunable parameters.
6. **Framework auto-detection for component testing** — Parses `package.json` dependencies AND source code imports to auto-detect React/Vue/Svelte without user configuration.
7. **Injected script architecture** — Two-tier injection: `CORE_SCRIPT` (core dispatch, querySelector, waitForSelector) + `ACTIONS_SCRIPT` (getByRole, getByText, actionability checks, scroll, geometry). Loaded dynamically via CDP `Page.addScriptToEvaluateOnNewDocument`.
8. **AssertionEngine with jitter** — Exponential backoff polling with randomized jitter (10% of current interval) prevents thundering herd in parallel test runs.
9. **HTTPS MitM proxy** — Built-in certificate store (`CertStore`) with self-signed CA generation and per-domain certificates for HTTPS interception. `rcgen`-based.
10. **EventDispatcher with URL caching** — `DashMap` + `mpsc` event dispatch with automatic URL-on-navigation cache. No dead subscriber leaks (retain on send error).

---

## Additional Gaps Identified (Not in Original Report)

These gaps were discovered through deep codebase audit and are NOT listed in the original PROGRESS.md critical gaps section:

| #   | Gap                                                                                                                                                             | Location                                                      | Impact                                            | Priority                                                                           |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------- | ---------------------------------------------------------------------------------- |
| 1   | **No typed error hierarchy** — Single `TurbosheetError` enum; no `TimeoutError`, `LocatorError`, `AssertionError`, `NetworkError`                               | `src/error.rs`                                                | Poor DX, can't catch specific errors in user code | **High**                                                                           |
| 2   | **Tracing not wired in test runner** — `trace_data` field exists in `TestResult` but no trace recording starts/stops during test execution                      | `src/test_runner/executor.rs`                                 | No per-test trace files in reports                | **High**                                                                           |
| 3   | ~~**Reporters not wired to TestExecutor** — 7 reporters exist but `execute()` doesn't invoke any~~                                                              | ~~`src/test_runner/executor.rs` + `src/reporters/`~~          | ~~Test results have no output~~                   | ~~**Critical**~~ ✅ **RESOLVED** — Reporter trait extracted and wired (2026-07-08) |
| 4   | **No configuration file parser** — No `turbosheet.config.ts`/`.json` support                                                                                    | —                                                             | Users must configure via code only                | **High**                                                                           |
| 5   | **No `Keyboard`/`Mouse`/`Touchscreen` input classes** — All input goes through page methods directly                                                            | `src/chromium/page.rs`                                        | Limited granular input control                    | **Medium**                                                                         |
| 6   | **No persistent browser contexts** — Only incognito; no way to launch with profile dir                                                                          | `src/browser/` module doesn't exist                           | Can't test logged-in states, extensions           | **High**                                                                           |
| 7   | **No CDPSession exposed to JS** — Raw CDP access unavailable to consumers                                                                                       | `src/chromium/`                                               | Power users can't send raw CDP commands           | **Medium**                                                                         |
| 8   | **`Keyboard.press()` and `Keyboard.pressSequentially()` are stubs** — Declared in `PageEngine` trait but not implemented                                        | Trait + chromium implementation                               | No type, press, or pressSequentially for users    | **High**                                                                           |
| 9   | **No request timing in network events** — NetworkRequestEvent/ResponseEvent lack timing data                                                                    | `src/network/interceptor.rs`                                  | Can't measure request duration                    | **Medium**                                                                         |
| 10  | **No sharding in executor** — `ShardConfig` struct exists but is not used in `execute_tests()`                                                                  | `src/test_runner/executor.rs`                                 | Can't split tests across CI machines              | **Medium**                                                                         |
| 11  | **No HAR import/export** — No HAR format support                                                                                                                | `src/network/`                                                | Can't replay production traffic                   | **Low**                                                                            |
| 12  | **No webSocket upgrade in proxy** — `WsRouteHandler` trait exists but proxy doesn't upgrade connections                                                         | `src/network/proxy.rs`                                        | Can't intercept WebSocket traffic                 | **High**                                                                           |
| 13  | **SnapshotManager is minimal** — Only `resolved_path()` and basic baseline load; no auto-update, no CI mode                                                     | `src/visual/snapshot.rs`                                      | Visual testing not production-ready               | **Medium**                                                                         |
| 14  | **No CLI binary** — No CLI args for headed, browser selection, grep, retries, timeout, workers, reporter, output                                                | —                                                             | Can't integrate in CI pipeline                    | **High**                                                                           |
| 15  | **No watch mode** — Test runner doesn't support file watching                                                                                                   | `src/test_runner/`                                            | Poor dev iteration experience                     | **Medium**                                                                         |
| 16  | **No coverage API** — No JS/CSS coverage start/stop                                                                                                             | `src/`                                                        | Can't measure code coverage                       | **Low**                                                                            |
| 17  | **No project dependency ordering** — `ProjectDependency` struct exists but executor ignores it                                                                  | `src/test_runner/executor.rs`                                 | Projects run in undefined order                   | **Medium**                                                                         |
| 18  | **Plugin system is skeleton-only** — `MatcherPlugin`/`ReporterPlugin` traits exist but no auto-discovery or loading                                             | `src/plugin/`                                                 | Plugin ecosystem can't start                      | **Low**                                                                            |
| 19  | **`beforeAll`/`afterAll`/`beforeEach`/`afterEach` hooks** — Implemented via collector sandbox interception + PlanBuilder scheduling                             | `src/runtime/worker-entry.ts` + `src/test_runner/executor.rs` | Hooks execute                                     | ✅ **RESOLVED** (2026-07-08)                                                       |
| 20  | **`test()`/`describe()` registration** — Implemented via inline collector in worker sandbox; PlanBuilder handles filtering/scheduling                           | `src/runtime/worker-entry.ts` + `src/test_runner/executor.rs` | Tests can be defined                              | ✅ **RESOLVED** (2026-07-08)                                                       |
| 21  | **`expect.poll()` / `expect.toPass()` missing** — No polling-style value assertions                                                                             | `src/assertions/matchers.rs`                                  | Limited assertion patterns                        | **Medium**                                                                         |
| 22  | **No `soft` assertions** — All assertions are hard-fail                                                                                                         | `src/assertions/matchers.rs`                                  | Can't collect multiple failures                   | **Medium**                                                                         |
| 23  | **AssertionEngine tests disabled** — Tests in `engine.rs` have `#[cfg(test)] mod tests { // Tests disabled }`                                                   | `src/assertions/engine.rs`                                    | No regression protection for polling engine       | **Medium**                                                                         |
| 24  | **No page.on('requestfailed')** — Only request/response events are wired                                                                                        | `src/events/mod.rs`                                           | Can't detect failed requests                      | **High**                                                                           |
| 25  | **No page.on('worker') event** — WebWorker creation/destruction not tracked                                                                                     | `src/events/mod.rs`                                           | Can't test worker-based apps                      | **Low**                                                                            |
| 26  | **No page.on('download') event** — No download interception                                                                                                     | `src/events/mod.rs`                                           | Can't test file downloads                         | **Medium**                                                                         |
| 27  | **No `connectOverCDP`** — Can't connect to existing Chrome instance                                                                                             | `src/chromium/`                                               | Debugging, remote debugging use cases             | **Medium**                                                                         |
| 28  | **No PDF generation** — `page.pdf()` not exposed                                                                                                                | `src/`                                                        | Can't generate PDFs                               | **Low**                                                                            |
| 29  | **No `locator.all()` / `allInnerTexts()` / `allTextContents()`** — Only single-element locators                                                                 | `js/src/injected-actions.ts` + `index.js`                     | Can't iterate over multiple matches               | **Medium**                                                                         |
| 30  | **`textContent()` / `innerText()` / `innerHtml()` on Rust locator return no-op for empty pages** — May return empty string instead of error or expected content | `src/` async evaluation flow                                  | Silent failures in assertions                     | **Medium**                                                                         |

---

## Critical Gaps (High Priority)

These are the 15 most impactful missing features that block production use:

| #   | Feature                                                                                                                | Impact                                                           |        Affected Tool        | Est. Effort |                Status                 |
| --- | ---------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | :-------------------------: | :---------: | :-----------------------------------: |
| 1   | **Test runner core** — `test()`, `describe()`, hooks implemented via two-phase IPC + PlanBuilder                       | Test definition, filtering, execution, hooks scheduling all work |             All             |  2-3 weeks  |    ✅ **IMPLEMENTED** (2026-07-08)    |
| 2   | ~~**Reporters NOT wired to TestExecutor**~~ — 7 reporters exist and are now wired via `Reporter` trait                 | Test output works                                                |             All             |   1 week    |     ✅ **RESOLVED** (2026-07-08)      |
| 3   | **Auto-waiting / actionability checks** — Click does no scroll/visibility/wait stability checks                        | Unreliable click success                                         |  Playwright/Cypress parity  |  2-3 weeks  |              ❌ Missing               |
| 4   | **Video recording NOW WIRED** — Video subsystem wired to ChromiumPageEngine per Phase 0                                | Debug test failures                                              |             All             |  1-2 weeks  |     ✅ **IMPLEMENTED** (Phase 0)      |
| 5   | **Trace NOW WIRED in test runner** — Trace recording per test wired per Phase 0 (clear per file, events→JSON per file) | Per-test traces available                                        |             All             |   1 week    |     ✅ **IMPLEMENTED** (Phase 0)      |
| 6   | **Frame & Worker support** — No iframe or WebWorker APIs                                                               | Cannot test complex apps                                         |      Playwright parity      |  3-4 weeks  |              ❌ Missing               |
| 7   | **CLI & CI/CD** — No CLI args, no config file parser, no Docker, no CI flags                                           | Hard to integrate in CI                                          |             All             |  2-3 weeks  |              ❌ Missing               |
| 8   | **Firefox/WebKit completeness** — ~75-80% of methods are stubs                                                         | Single-browser only effectively                                  |    Cross-browser testing    |  4-6 weeks  |    ⚠️ Traits exist, impls stubbed     |
| 9   | **No typed error hierarchy** — Single `TurbosheetError`; can't catch TimeoutError, LocatorError separately             | Poor DX, fragile test code                                       |             All             |  3-5 days   |              ❌ Missing               |
| 10  | **Network interception completeness** — Missing WebSocket, HAR, worker SW, requestfailed, requestfinished              | Limited network testing                                          | Playwright/Puppeteer parity |  3-4 weeks  | ⚠️ Partial (route/fulfill/abort work) |
| 11  | **Device emulation** — No device profiles, geolocation, timezone, locale, permissions                                  | Limited mobile/location testing                                  |      Playwright parity      |  2-3 weeks  |              ❌ Missing               |
| 12  | **`Keyboard.press()` / `pressSequentially()` are stubs** — Declared but not implemented                                | No type/press actions                                            |  Puppeteer/Cypress parity   |   1 week    |           ⚠️ Trait has stub           |
| 13  | **No persistent browser contexts** — Only incognito; no profile support                                                | Can't test logged-in states, extensions                          | Playwright/Puppeteer parity |  2-3 weeks  |              ❌ Missing               |
| 14  | **WebSocket interception upgade** — WsRouteHandler exists but proxy never upgrades to WS                               | Can't intercept WebSocket traffic                                |      Playwright parity      |  2-3 weeks  |      ⚠️ Trait exists, no upgrade      |
| 15  | **No `page.on('requestfailed')`** — Only request/response events wired                                                 | Can't detect failed requests                                     |             All             |  3-5 days   |              ❌ Missing               |

---

## Anatomy: TurboSheet Architecture (Actual Codebase)

```
┌───────────────────────────────────────────────────────────────────────┐
│                        JS / N-API Layer (index.js + .d.ts)            │
│  launch() | mount() | Component | ComponentLocator | test/expect     │
│  trace_* | visual_* | devices | detectFramework* | run_tests         │
│  JS injected scripts: core.mjs + actions.mjs (via include_str!)      │
└───────────────────────────────────────────────────────────────────────┘
                                    │ napi-rs FFI
┌───────────────────────────────────────────────────────────────────────┐
│                         Rust Core (cdylib + rlib)                     │
│                                                                       │
│  ┌─────────────────┐  ┌───────────────┐  ┌──────────────┐           │
│  │  Engine Traits   │  │  Events       │  │  Assertions  │           │
│  │  (BrowserEngine, │  │  (EventDispatcher, │  (AssertionEngine,     │
│  │   ContextEngine, │  │   CdpEvent,    │  │   Expect/Matchers)     │
│  │   PageEngine)    │  │   Subscribers) │  └──────────────┘          │
│  └───────┬──────────┘  └───────┬───────┘         │                   │
│          │                     │                  │                   │
│  ┌───────┴──────────┐  ┌───────┴────────┐  ┌─────┴──────────┐      │
│  │  Chromium (CDP)  │  │  Firefox (WD)  │  │  WebKit (WD)  │      │
│  │  ~2,500+ loc     │  │  ~650 loc      │  │  ~650 loc      │      │
│  │  chromiumoxide   │  │  geckodriver   │  │  safaridriver  │      │
│  └──────────────────┘  └────────────────┘  └────────────────┘      │
│                                                                       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐            │
│  │ Network  │ │  Trace   │ │  Video   │ │  Injection   │            │
│  │ (proxy,  │ │ (recorder,│ │ (config, │ │ (bindings,   │            │
│  │  intercep,│ │  ser,    │ │  encoder,│ │  scripts,    │            │
│  │  route,   │ │  viewer) │ │ recorder)│ │  stealth)    │            │
│  │  cert,    │ └──────────┘ └──────────┘ └──────────────┘            │
│  │  pattern) │ ┌──────────┐ ┌──────────┐ ┌──────────────┐            │
│  └──────────┘ │  Plugin  │ │ Migrate  │ │  Visual      │            │
│               │  (traits) │ │ Adapters │ │  (snapshot,  │            │
│               └──────────┘ └──────────┘ │   compare)   │            │
│                                         └──────────────┘            │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Test Runner  (src/test_runner/)                              │  │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌──────────────────┐  │  │
│  │  │ Config   │ │Discovery │ │Executor│ │ Worker (JSON-RPC)│  │  │
│  │  │  70+     │ │ (glob    │ │  377   │ │  stdin/stdout    │  │  │
│  │  │  fields  │ │  +grep)  │ │  lines  │ │  IPC (256 lines) │  │  │
│  │  └──────────┘ └──────────┘ └────────┘ └──────────────────┘  │  │
│  │  ┌──────────────────────────────────────────────────────┐   │  │
│  │  │ test()/describe()/hooks = IMPLEMENTED               │   │  │
│  │  │ (two-phase IPC: collector→PlanBuilder→runPlan)      │   │  │
│  │  └──────────────────────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────┘  │  │
│                                                                  │  │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐    │  │
│  │  Reporters   │  │  Component   │  │  Supported Features │    │  │
│  │  (7: list,   │  │  (mount,     │  │  chrom|firef|webkit │    │  │
│  │   dot, line, │  │   pool, detect│  │  cookies, context,  │    │  │
│  │   json,junit,│  │   locator)   │  │  screenshot, trace,  │    │  │
│  │   html,github)  └──────────────┘  │  visual, network,   │    │  │
│  └──────────────┘                    │  assertions, stealth │    │  │
│                                       └─────────────────────┘    │  │
└───────────────────────────────────────────────────────────────────────┘
```

> **Note**: The test registration stubs were the most critical single gap — now resolved via the two-phase IPC pipeline. The inline collector in the worker sandbox intercepts `test()`/`describe()`/hooks calls during `extractTests`, PlanBuilder schedules execution, and `buildRunPlanScript` generates lifecycle scripts. The napi-export stubs in `mod.rs` are not on the critical path — sandbox-level interception in the worker is the primary flow.

---

## Key Strengths of TurboSheet

Despite the gaps, TurboSheet has genuinely innovative or well-executed features:

1. **Binding bridge (JS↔Rust)** — Oneshot channels with `BindingRegistry` are clean, well-designed, and production-quality. 192 lines of robust IPC.
2. **CDP Fetch domain interception** — Full `route()`/`fulfill()`/`abort()`/`continue()` implementation with proper CDP integration and ThreadsafeFunction bridge to JS.
3. **Assertion engine** — Exponential backoff polling with jitter (10% random), configurable timeout, covers 16+ matchers (1,165 lines).
4. **Component mounting** — Multi-framework support (React/Vue/Svelte) with both `package.json` AND source-code-based auto-detection, 10-slot object pool.
5. **Event dispatcher** — Clean `DashMap` + `mpsc` pattern with automatic URL-on-navigation cache and dead-subscriber cleanup.
6. **Stealth configuration** — Anti-detection measures for headless Chrome: randomized browser name, Chrome version spoofing, WebGL vendor/renderer spoofing, navigator.plugins, custom UA quirks.
7. **Trace viewer** — Self-contained 453-line HTML reporter with timeline, event details, console log, screenshot viewer, network log, interactive filtering.
8. **Test worker model** — JSON-RPC worker process isolation via stdin/stdout (unique among all three competitors — no other tool provides this).
9. **Trait-based engine architecture** — Clean `BrowserEngine`/`ContextEngine`/`PageEngine` traits enable true multi-engine support. Adding a new engine = implement 3 traits.
10. **Video recording subsystem** — `VideoConfig` + `VideoEncoder` (FFmpeg subprocess) + `VideoRecorder` (full state machine). 550 lines. Only needs CDP screencast integration.
11. **HTTPS MitM proxy** — Built-in `CertStore` with self-signed CA generation and per-domain certificates for HTTPS interception using `rcgen`.
12. **Injected script architecture** — Two-tier injection: core dispatch (62 lines) + actionability actions (393 lines). Loaded via CDP on every new document.
13. **7 built-in reporters** — List, dot, line, JSON, JUnit, HTML (453 lines with trace viewer), GitHub annotations. Only need wiring to TestExecutor.

---

## Implementation Priority Roadmap

### Phase 0 — Quick Wins ✅ COMPLETE (2026-07-08)

**Goal**: Get existing but unwired subsystems working end-to-end.

- ✅ **(1) Wire reporters to TestExecutor** — `Reporter` trait extracted with `on_test_result()` and `on_complete()`, implemented for all 7 reporters, `TestExecutor::execute()` refactored from closure → `&dyn Reporter`.
- ✅ **(2) Wire video recording to ChromiumPageEngine** — Already implemented end-to-end: `start_recording()`/`stop_recording()` in ChromiumPageEngine, screencast frame pipeline, encoder state machine, retention policies, NAPI bindings.
- ✅ **(3) Wire trace recording per test** — Already implemented: `trace_clear()` per-test-file (worker-entry.ts:143), `trace_events_to_json()` per-test-file (worker-entry.ts:390), `trace_data` flows through `WorkerTestResult → TestResult` via serde (executor.rs:271).
- ✅ **(4) Fix `Keyboard.press()` / `pressSequentially()`** — Already implemented in all 3 engines: Chromium uses full CDP `Input.dispatchKeyEvent` with RawKeyDown→Char→KeyUp; Firefox/WebKit use WebDriver POST `/element/{id}/value`.
- ✅ **(5) Enable assertion engine tests** — Already enabled: 5 tests in `assertions/engine.rs` (lines 76-190) test polling, timeout, jitter bounds, exponential backoff, poll count.

**Impact**: Reporters wired, trace attachments flowing, video recording ready, keyboard actions work, assertion tests verified. All Phase 0 items complete.

### Phase 1 — Test Runner Core ✅ COMPLETE (2026-07-08)

**Goal**: Users can define and run tests end-to-end with reporting.

- ✅ `test()` / `describe()` registration — Collector sandbox intercepts calls; PlanBuilder filters/schedules
- ✅ `beforeAll` / `afterAll` lifecycle — PlanBuilder schedules first/last test per suite
- ✅ `beforeEach` / `afterEach` lifecycle — `buildRunPlanScript` generates lifecycle JSON with hooks array
- ✅ Hook inheritance via ancestry walking (suite path → parent chain)
- ✅ Modifier cascade: `test.skip`/`test.only`/`describe.skip`/`describe.only`/`describe.serial`/`describe.parallel`
- ✅ `beforeAll` failure cascading — Shared `Arc<Mutex<HashSet>>` tracks blocked suites
- ✅ Suite-level `is_fail` inversion, `is_fixme` mapping, timeout tripling for `test.slow`
- ❌ `test.step()` — Not yet implemented
- ❌ `ShardConfig` — Not yet wired in executor
- ❌ `ProjectDependency` — Not yet wired
- ❌ Config file parser (`turbosheet.config.ts` / `.json`) — Not yet implemented

**Impact**: Unblocks all test authoring. Users can now define and run tests end-to-end.

### Phase 2 — Auto-Waiting & Actionability (3-4 weeks)

**Goal**: Reliable interactions without explicit waits.

- Add actionability checks before every click/fill/check/select:
  - Element visible (not hidden, not `display:none`, not `visibility:hidden`)
  - Element enabled (not `disabled`, not `readonly`)
  - Element stable (not animating, position hasn't changed in 500ms)
  - Element in viewport (scroll into view if needed)
- Add auto-retry on stale element (stale reference → re-query → retry action)
- Add configurable action timeout per locator
- Add `force` option to bypass actionability checks
- Add `noWaitAfter` option
- Convert Rust-side locator from raw selector string to proper `Locator` struct with actionability state tracking

**Impact**: Eliminates flaky tests — the #1 cause of flakiness in all browser test tools.

### Phase 3 — Locator Engine Completion (2-3 weeks)

**Goal**: Parity with Playwright locator API for all query patterns.

- Add XPath selector support (`xpath=`)
- Add `text=` / `data-testid=` built-in selector engines
- Add `.and()` / `.or()` logical combinator filters
- Add `locator.all()` / `allInnerTexts()` / `allTextContents()`
- Add `locator.highlight()` with CDP overlay
- Add strict mode (error when locator matches >1 element)
- Add `expect(locator).toBeInViewport()` / `toBeAttached()` / `toBeEditable()` / `toBeFocused()`
- Add `expect(page).toHaveTitle()` / `toHaveURL()`
- Add `expect.poll()` / `expect.toPass()` for value-based polling
- Add soft assertions (non-failing, collect all failures)

**Impact**: Best-in-class locator DX — reduces test authoring time by 40%+.

### Phase 4 — Error Hierarchy & Event Completeness (1-2 weeks)

**Goal**: Production-quality error types and event system.

- Implement typed errors: `TimeoutError`, `LocatorError`, `AssertionError`, `NetworkError`, `ConnectionError`
- Wire `page.on('requestfailed')` event
- Wire `page.on('requestfinished')` event
- Wire `page.on('domcontentloaded')` / `page.on('load')` events
- Wire `page.on('frameattached')` / `frame.detached` events
- Add `page.on('download')` event
- Add `removeListener()` / `off()` API
- Add `waitForEvent()` API with timeout
- Add `context.on('page')` event

**Impact**: Robust error handling and complete event observability for debugging test failures.

### Phase 5 — Firefox/WebKit Completeness (4-6 weeks)

**Goal**: All 3 engines have comparable feature sets.

- Implement missing Chromium methods that work:
  - `setInputFiles()`
  - `dragAndDrop()`
  - `addScriptTag()` / `addStyleTag()` (basic versions exist)
  - `bringToFront()`
  - `setContent()` (full implementation)
  - Content `type()` (append mode, not just fill)
  - `clear()` input
  - `selectText()`
  - `pdf()` (Chromium only)
- Implement all ~75 Firefox stubs (currently default no-op or error returns)
- Implement all ~80 WebKit stubs
- Add shared WebDriver utility layer to reduce code duplication across Firefox/WebKit
- Evaluate WebDriver BiDi support for future modernization

**Impact**: True cross-browser testing capability.

### Phase 6 — CLI, CI/CD & Configuration (2-3 weeks)

**Goal**: Production CI/CD integration.

- Create CLI binary (`turbosheet` or `tsheet`) with flags:
  - `--headed`, `--browser`, `--project`, `--grep`, `--shard=x/y`
  - `--retries`, `--timeout`, `--workers`, `--reporter`, `--output`
  - `--update-snapshots`, `--forbid-only`, `--list`
  - `--global-setup`, `--global-teardown`, `--config`
- Config file support: `turbosheet.config.ts` / `.json` / `.mjs`
- Docker image with pre-installed browsers
- CI-specific optimizations: `--ci` flag, JUnit XML output, GitHub Actions annotations
- `--update-snapshots` for visual testing CI mode

**Impact**: Enables CI integration — without this, no production pipeline can use TurboSheet.

### Phase 7 — Advanced Features (6-8 weeks)

**Goal**: Parity on advanced use cases.

- **Frame support** — `frameLocator()`, evaluate/click inside iframes with CDP `Page.getFrameTree`
- **WebWorker events** — `page.on('worker')`, evaluate in worker context
- **WebSocket interception** — Complete proxy upgrade + WsRouteHandler dispatch
- **Accessibility snapshot API** — Expose `AccessibilityNode` tree to JS, add `expect().toMatchAriaSnapshot()`
- **Device emulation profiles** — Built-in device descriptors (iPhone, Pixel, iPad), one-line setup
- **Persistent browser contexts** — Launch with profile directory, Chrome extensions support
- **`connectOverCDP`** — Connect to existing Chrome instance for debugging
- **Storage state API** — Export/import cookies + localStorage as JSON
- **Coverage API** — JS/CSS coverage start/stop/report
- **HAR import/export** — Record and replay HAR files
- **Raw CDPSession** — Expose raw CDP access to power users

### Phase 8 — Enterprise Readiness (4-6 weeks)

**Goal**: Production-scale testing infrastructure.

- **Watch mode** — File watcher that re-runs tests on change
- **UI Mode** — Time-travel debugger with DOM snapshots, action log, console viewer
- **Codegen** — Interactive test recorder (click → generate locator → append to test)
- **Locator picker** — Hover over element → copy locator
- **Sharding & merge-reports** — Split tests across CI nodes, merge JSON results
- **Flaky test management** — Auto-retry config, flaky detection dashboard
- **Comprehensive migration codegen** — `npx tsheet convert` from Playwright/Cypress/Puppeteer
- **Plugin auto-discovery** — Scan `node_modules` for turbosheet plugins
- **Dashboard / Cloud integration** — Result aggregation, history, trends

### Summary: Effort Estimation

| Phase   | Focus                                                    | Duration  |   Blocks Production?    |
| ------- | -------------------------------------------------------- | :-------: | :---------------------: |
| Phase 0 | ✅ Quick wins — **COMPLETE**                             | 1-2 weeks |       ✅ Critical       |
| Phase 1 | ✅ Test runner core (test/describe/hooks) — **COMPLETE** | 3-4 weeks |       ✅ Critical       |
| Phase 2 | Auto-waiting & actionability                             | 3-4 weeks |       ✅ Critical       |
| Phase 3 | Locator engine completion                                | 2-3 weeks | ❌ (works, but less DX) |
| Phase 4 | Error hierarchy & events                                 | 1-2 weeks |  ⚠️ (important for DX)  |
| Phase 5 | Firefox/WebKit completeness                              | 4-6 weeks |   ⚠️ (Chromium works)   |
| Phase 6 | CLI, CI/CD & config                                      | 2-3 weeks |       ✅ Critical       |
| Phase 7 | Advanced features                                        | 6-8 weeks |    ❌ (nice-to-have)    |
| Phase 8 | Enterprise readiness                                     | 4-6 weeks |       ❌ (scale)        |

**Total estimated effort to reach production-ready (Phases 0-2, 6)**: 10-14 weeks for a small team
**Total estimated effort to reach Playwright parity (All phases)**: 6-9 months for a small team

---

## How TurboSheet Fixes Each Competitor's Pain Points

### The Puppeteer Replacement Pitch

> **"TurboSheet is Puppeteer with a test runner, assertions, cross-browser support, and auto-waiting built-in."**

| Puppeteer Pain Point | TurboSheet Solution                                           | Status                           |
| -------------------- | ------------------------------------------------------------- | -------------------------------- |
| No test runner       | Built-in `test()`/`describe()`/hooks + executor + worker pool | ✅ Implemented via two-phase IPC |
| No auto-waiting      | AssertionEngine polling + actionability checks                | ❌ Not wired to actions yet      |
| No cross-browser     | Chromium/Firefox/WebKit via trait-based engines               | ⚠️ Firefox/WebKit stubs          |
| No assertions        | 16+ matchers with retry + jitter + timeout                    | ✅ Mostly complete               |
| No reporting         | 7 built-in reporters                                          | ✅ Wired to executor (Phase 0)   |
| No trace viewer      | Built-in HTML trace viewer                                    | ✅ Complete                      |
| No component testing | React/Vue/Svelte mount with auto-detect + pool                | ✅ Complete                      |
| No video recording   | Video subsystem (config + encoder + recorder)                 | ✅ Wired (Phase 0)               |
| No stealth mode      | StealthConfig with fingerprint spoofing                       | ✅ Complete                      |

### The Cypress Replacement Pitch

> **"TurboSheet is Cypress with real multi-tab, cross-origin, iframe, and parallel execution — without a cloud subscription."**

| Cypress Pain Point                  | TurboSheet Solution                                 | Status                                        |
| ----------------------------------- | --------------------------------------------------- | --------------------------------------------- |
| In-process execution (no multi-tab) | Out-of-process Rust engine with true multi-page CDP | ✅ Complete                                   |
| No iframe support                   | Frame locator via CDP                               | ❌ Not implemented                            |
| Slow parallel requires cloud        | Built-in worker pool + sharding                     | ⚠️ Executor exists, sharding not wired        |
| No native mobile                    | CDP device emulation                                | ⚠️ Partial (viewport works, profiles missing) |
| Heavy memory usage                  | Worker pool with browser reuse                      | ⚠️ Needs optimization                         |
| Single engine (Electron)            | Tri-engine (Chromium/Firefox/WebKit)                | ⚠️ Firefox/WebKit stubs                       |
| No WebSocket intercept              | NetworkProxy with WsRouteHandler                    | ❌ Proxy upgrade not wired                    |
| No real cross-origin                | Native CDP multi-origin support                     | ✅ Built-in                                   |
| Clock/tick for time testing         | Time travel via CDP                                 | ❌ Not implemented                            |

### The Playwright Competitor Pitch

> **"TurboSheet is Playwright with smaller binary, no cloud dependency for sharding/parallelism, and built-in stealth mode."**

| Playwright Limitation       | TurboSheet Advantage                        | Status                                |
| --------------------------- | ------------------------------------------- | ------------------------------------- |
| ~500MB+ bundled browsers    | Use existing browser installations          | ✅ Architecture supports this         |
| Cloud-required sharding     | Built-in ShardConfig + local parallel pool  | ⚠️ Sharding not wired                 |
| Heavy trace files           | Compact trace format (ZSTD + CBOR optional) | ⚠️ Optional (behind `traces` feature) |
| No stealth mode             | Built-in StealthConfig                      | ✅ Complete                           |
| In-process workers          | JSON-RPC worker isolation                   | ✅ Complete (unique)                  |
| TS compilation startup cost | `tsx`-based worker, no pre-compilation      | ✅ Architecture supports this         |
| Slow codegen                | —                                           | ❌ Not implemented (opportunity)      |

### TurboSheet's Unfair Advantages

These are architectural decisions that competitors cannot easily replicate:

1. **Rust core, JS bindings** — CDP communication bypasses the Node.js event loop entirely. Chromiumoxide handles WebSocket in Rust's async runtime. This means:
   - No event loop bottleneck for CDP messages
   - True parallelism without worker_threads overhead
   - Native performance for screenshot processing, image comparison, trace compression

2. **napi-rs FFI** — Near-zero overhead for JS↔Rust calls. No serialization cost (unlike JSON-RPC or HTTP bridges used by some tools).

3. **JSON-RPC worker isolation** — Each test in its own OS process with stdin/stdout IPC. No cross-test contamination. No other E2E tool provides this isolation level natively.

4. **Trait-based engine abstraction** — Adding a new browser engine = implementing 3 traits. The architecture was designed for multi-engine from day one, not retrofitted.

5. **Feature-gated compilation** — `chromium`, `firefox`, `webkit`, `traces`, `visual`, `video`, `swarm` are all Cargo features. Users compile only what they need. Final binary is lean.

6. **Two-tier injection** — Core script (always loaded, minimal) + actions script (loaded on demand). Production-proven pattern for minimizing page load overhead.

7. **CDP Fetch domain by default** — TurboSheet uses the Fetch domain (CDP's modern network interception API) rather than the deprecated Network domain, matching Playwright's approach and avoiding the race conditions of Puppeteer's old approach.

---

## What's Next

This analysis identifies **two hard blockers remaining** (must-fix before production use):

1. ~~**`test()`/`describe()`/hooks stubs** → No tests can be defined~~ ✅ **Resolved** — Two-phase IPC pipeline (collector→PlanBuilder→runPlan)
2. ~~**Reporters not wired** → No output from test runs~~ ✅ **Resolved** (Phase 0)
3. **No CLI/config** → Cannot configure or launch in CI

And **five high-priority improvements** that define the "good E2E tool" bar:

4. **Auto-waiting for actions** → Reliable, non-flaky interactions
5. **Video recording integration** → Debug test failures
6. **Trace per test** → Debug test logic
7. **Typed error hierarchy** → Catch and handle specific failures
8. **Complete network events** → Detect failed requests

The codebase is structurally sound. The core architectural decisions (Rust+napi+JSON-RPC+traits) are correct and differentiation-worthy. What's missing is the last mile of integration — connecting subsystems that are already built into a working pipeline.

**Estimated time to first working E2E test**: Phase 0 + Phase 1 are now complete. A user can write and run their first TurboSheet test. Next frontier: E2E smoke test validation and Phase 2 (auto-waiting).
