## Context

TurboSheet is a Rust-based browser automation framework targeting Node.js via NAPI-Rs. It uses `chromiumoxide` for Chromium CDP access and `isolated-vm` for JavaScript test execution isolation. The architecture separates concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                      Node.js / CLI                          │
├─────────────────────────────────────────────────────────────┤
│  Test Runner (TypeScript)  │  JsRuntime (isolated-vm pool) │
├─────────────────────────────────────────────────────────────┤
│                   NAPI-Rs Boundary                         │
├─────────────────────────────────────────────────────────────┤
│  Engine Traits (Rust)  │  Assertions  │  Fixtures/Hooks   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  ChromiumEngine  │  FirefoxEngine  │  WebKitEngine │    │
│  └─────────────────────────────────────────────────────┘    │
│           CDP / WebDriver Protocol                         │
└─────────────────────────────────────────────────────────────┘
```

**Current State:**

- Chromium: Working via `chromiumoxide`
- Firefox/WebKit: Stubbed (return version strings only)
- TestExecutor: Stubbed (returns dummy results)
- Most locator/page methods: Stubbed no-ops

**Stakeholders:** E2E test developers migrating from Playwright/Cypress/Puppeteer

## Goals / Non-Goals

**Goals:**

1. Complete multi-browser support (Firefox, WebKit) with unified API
2. Real test execution pipeline with worker process isolation
3. Full locator and page API parity with Playwright
4. WebSocket interception for network automation
5. Visual testing with `toHaveScreenshot` and baseline management
6. Playwright/Cypress/Puppeteer migration tools
7. Project configuration via `tsheet.config.ts`

**Non-Goals:**

- Supporting IE11 or legacy browsers
- Direct browser injection (Cypress model)
- Built-in CI/CD integration (external tooling)
- Visual Studio Code extension (separate change)

## Decisions

### 1. Browser Engine Abstraction

**Decision:** Unified `BrowserEngine` trait with browser-specific implementations

**Rationale:** TurboSheet already has this pattern with `ChromiumEngine`. Extending to Firefox/WebKit follows the same architecture.

**Alternatives Considered:**

- Single unified CDP-only approach: Rejected because Firefox/WebKit don't speak CDP natively
- Protocol-agnostic abstraction: More complex than needed; browser-specific engines are cleaner

```rust
// src/engine/mod.rs - Already exists, extend to:
pub trait BrowserEngine: Send + Sync {
    async fn launch(&self, options: LaunchOptions) -> Result<BrowserHandle>;
    async fn close(&self, browser: BrowserHandle) -> Result<()>;
}

pub trait ContextEngine {
    async fn new_page(&self) -> Result<PageHandle>;
    async fn close(&self) -> Result<()>;
}

pub trait PageEngine {
    async fn goto(&self, url: &str, state: LoadState) -> Result<()>;
    async fn evaluate(&self, js: &str) -> Result<String>;
    // ... 20+ methods
}
```

**Firefox Implementation:**

- Use `geckodriver` (Rust) via WebDriver protocol (JSON over HTTP)
- `reqwest` for HTTP client (already in dependencies)
- Maintain async/await pattern

**WebKit Implementation:**

- Use `safaridriver` (system-installed on macOS, webkitwd on others)
- Same WebDriver protocol approach as Firefox
- Note: WebKit support requires macOS for full testing

### 2. Test Execution Architecture

**Decision:** Worker process spawning with IPC for result aggregation

**Rationale:** JavaScript test files must run in isolated processes. Rust's `tokio` handles async; actual process isolation requires forking.

**Alternatives Considered:**

- Thread-based isolation: Rejected - JS contexts can't be truly isolated with threads alone
- Web Workers: Rejected - limited API surface, not true isolation
- Child process with IPC: Selected - cleanest isolation, matches Playwright's model

```
┌──────────────────────────────────────────────────────────────┐
│                      Main Process                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  TestExecutor                                         │   │
│  │  - Discovers tests                                    │   │
│  │  - Spawns N worker processes                          │   │
│  │  - Aggregates results via IPC                        │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
           │                              │
           ▼                              ▼
┌─────────────────────┐      ┌─────────────────────┐
│    Worker Process   │      │    Worker Process   │
│  ┌───────────────┐  │      │  ┌───────────────┐  │
│  │ JsRuntimePool │  │      │  │ JsRuntimePool │  │
│  │ (isolated-vm) │  │      │  │ (isolated-vm) │  │
│  └───────────────┘  │      │  └───────────────┘  │
│  ┌───────────────┐  │      │  ┌───────────────┐  │
│  │  CDP Client   │  │      │  │  CDP Client   │  │
│  │  (browser)    │  │      │  │  (browser)    │  │
│  └───────────────┘  │      │  └───────────────┘  │
└─────────────────────┘      └─────────────────────┘
```

**IPC Protocol:**

- Unix domain sockets or Windows named pipes
- JSON-RPC 2.0 messages
- Events: test_start, test_result, hook_start, hook_result, error

### 3. Locator API Completeness

**Decision:** Add missing methods to `JsLocator` and `LocatorBridge`

**Rationale:** Playwright's locator API is the gold standard. TurboSheet has the foundation; missing methods are straightforward additions.

**New Methods:**

```rust
// src/locator.rs additions
pub struct JsLocator {
    // ... existing fields

    pub async fn scroll_into_view(&self) -> Result<()>;
    pub async fn focus(&self) -> Result<()>;
    pub async fn blur(&self) -> Result<()>;
    pub async fn bounding_box(&self) -> Result<Option<Rect>>;
    pub async fn screenshot(&self, options: Option<ScreenshotOptions>) -> Result<Buffer>;
    pub async fn drag_and_drop(&self, target: &JsLocator) -> Result<()>;
    pub async fn hover(&self) -> Result<()>;
    pub async fn press(&self, key: &str) -> Result<()>;
    pub async fn press_sequentially(&self, text: &str) -> Result<()>;
    pub async fn set_input_files(&self, files: Vec<String>) -> Result<()>;
}
```

**Implementation:** Each method calls appropriate CDP commands or page.evaluate() JavaScript

### 4. Page API Completeness

**Decision:** Add missing methods to `JsPage`

**New Methods:**

```rust
// src/page.rs additions
pub struct JsPage {
    // ... existing fields

    // Viewport
    pub async fn set_viewport_size(&self, size: ViewportSize) -> Result<()>;
    pub fn viewport_size(&self) -> ViewportSize;

    // Navigation
    pub async fn reload(&self, options: Option<NavigationOptions>) -> Result<()>;
    pub async fn go_back(&self) -> Result<()>;
    pub async fn go_forward(&self) -> Result<()>;

    // Waits
    pub async fn wait_for_request(&self, url: &str, options: Option<TimeoutOptions>) -> Result<Request>;
    pub async fn wait_for_response(&self, url: &str, options: Option<TimeoutOptions>) -> Result<Response>;
    pub async fn wait_for_selector(&self, selector: &str, options: Option<WaitOptions>) -> Result<Locator>;

    // Evaluate handle
    pub async fn evaluate_handle(&self, js: &str) -> Result<JsHandle>;

    // Tags
    pub async fn add_script_tag(&self, options: ScriptTagOptions) -> Result<()>;
    pub async fn add_style_tag(&self, options: StyleTagOptions) -> Result<()>;
    pub async fn expose_function(&self, name: &str, callback: ThreadsafeFunction) -> Result<()>;

    // Events (already partially exists)
    pub fn on(&self, event: &str, handler: Fn) -> Result<()>;
    pub fn off(&self, event: &str) -> Result<()>;
}
```

### 5. Network WebSocket Interception

**Decision:** Extend `NetworkProxy` with WebSocket support

**Rationale:** WebSocket is increasingly used in SPAs; testing requires interception ability.

**Architecture:**

```rust
// src/network/proxy.rs additions
pub struct NetworkProxy {
    pub routes: Arc<DashMap<String, RouteHandler>>,
    pub websocket_routes: Arc<DashMap<String, WebSocketHandler>>,  // NEW
    pub port: u16,
    // ...
}

pub trait WebSocketHandler: Send + Sync {
    fn on_socket_open(&self, socket: WebSocket);
    fn on_socket_message(&self, socket: WebSocket, message: Message);
    fn on_socket_close(&self, socket: WebSocket, code: u16, reason: &str);
}
```

**Implementation:**

- CDP has limited WebSocket support; use DevTools Protocol's `Fetch` domain for request interception
- For WebSocket interception, use browser's built-in DevTools Protocol WebSocket support
- Pattern: `page.routeWebSocket(url_pattern, handler)`

### 6. Visual Testing with Screenshot Comparison

**Decision:** Add `toHaveScreenshot` matcher with automatic baseline management

**Rationale:** Visual regression testing is a key E2E feature; Playwright has this as a strength.

**Architecture:**

```rust
// src/assertions/matchers.rs additions
pub enum Matcher {
    // ... existing
    HaveScreenshot {
        name: String,
        options: VisualComparisonOptions,
    },
}

pub struct VisualComparisonOptions {
    pub threshold: f64,           // 0.0 to 1.0
    pub max_diff_pixels: Option<usize>,
    pub ignore_regions: Vec<Rect>,
    pub animations: AnimationBehavior,
}

pub async fn to_have_screenshot(
    &self,
    locator: &Locator,
    name: &str,
    options: Option<VisualComparisonOptions>,
) -> Result<bool> {
    // 1. Capture current screenshot
    let current = locator.screenshot().await?;

    // 2. Load baseline from disk or generate initial
    let baseline_path = format!("./screenshots/baseline/{}.png", name);
    let baseline = load_or_create_baseline(&baseline_path, &current).await?;

    // 3. Compare with diff generation
    let diff = visual_compare_screenshots(&current, &baseline, options.threshold)?;

    // 4. If diff > threshold, save diff and fail
    if diff.pixel_diff_percent > options.threshold {
        save_diff_image(&diff, &format!("./screenshots/diff/{}.png", name))?;
        return Err(TurbosheetError::VisualMismatch { diff });
    }

    Ok(true)
}
```

**Baseline Storage:**

- Local: `./screenshots/baseline/` directory
- CI: Environment variable `TURBOSHEET_SCREENSHOTS_BUCKET` for cloud storage
- Format: PNG with metadata JSON sidecar

### 7. Project Configuration

**Decision:** Add `tsheet.config.ts` support via cosmiconfig

**Rationale:** Professional E2E tools need project-level configuration; matches Playwright's approach.

**Config Schema:**

```typescript
// tsheet.config.ts
interface TsheetConfig {
  testDir?: string;
  testMatch?: string[];
  timeout?: number;
  retries?: number;
  workers?: number;
  reporter?: ReporterType | ReporterConfig;

  // Browser options
  headless?: boolean;
  browser?: BrowserType | BrowserConfig;

  // Global setup/teardown
  globalSetup?: string;
  globalTeardown?: string;

  // Visual testing
  screenshotsDir?: string;
  baselineDir?: string;

  // Projects (like Playwright)
  projects?: Project[];

  // Webhook for CI
  reportSlowTests?: { threshold: number; callback: string };
}
```

**Loading:**

1. Check `tsheet.config.ts` in project root
2. Check `tsheet.config.js`
3. Check `package.json` `tsheet` field
4. Use defaults if none found

### 8. Migration Tool Architecture

**Decision:** AST-based transformation with mapping tables

**Rationale:** Test migration requires understanding both source and target syntax.

**Architecture:**

```
┌─────────────────────────────────────────────────────────────┐
│  Migration CLI                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │ Playwright  │  │  Cypress    │  │ Puppeteer   │          │
│  │  Parser     │  │  Parser     │  │  Parser     │          │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘          │
│         │                │                │                  │
│         └────────────────┼────────────────┘                  │
│                          ▼                                   │
│              ┌───────────────────────┐                       │
│              │   AST Normalizer      │                       │
│              │  (Common Representation)│                     │
│              └───────────┬───────────┘                       │
│                          ▼                                   │
│              ┌───────────────────────┐                       │
│              │  TurboSheet Generator │                       │
│              └───────────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

**Parser Approach:**

- Playwright: Parse `test()` calls, `expect()` assertions, `page.*` actions
- Cypress: Parse `cy.get()`, `cy.contains()`, `cy.wrap()`, assertions
- Puppeteer: Parse `page.goto()`, `page.click()`, `page.evaluate()`

**Mapping Tables:**

```rust
// src/migrate/mappings.rs
pub struct PlaywrightToTurboMapping;
impl ApiMapping for PlaywrightToTurboMapping {
    fn get_action_map() -> HashMap<&'static str, &'static str> {
        hashmap! {
            "page.click" => "page.locator().click()",
            "page.fill" => "page.locator().fill()",
            "page.type" => "page.locator().pressSequentially()",
            "page.selectOption" => "page.locator().select()",
            // ...
        }
    }

    fn get_assertion_map() -> HashMap<&'static str, &'static str> {
        hashmap! {
            "expect(page).toHaveTitle" => "expect(page).toHaveTitle()",
            "expect(locator).toBeVisible" => "expect(locator).toBeVisible()",
            // ... identical for most Playwright assertions
        }
    }
}
```

## Risks / Trade-offs

**[Risk: Firefox WebDriver compatibility]** → Mitigation: Use geckodriver Rust crate which handles Firefox CDP translation. Test thoroughly on multiple Firefox versions.

**[Risk: WebKit support limited to macOS]** → Mitigation: Document this limitation clearly. WebKit automation on Linux/Windows requires webkitwd server which has limited support.

**[Risk: Process spawning overhead]** → Mitigation: Pool worker processes and reuse for multiple test suites. Limit workers to CPU cores.

**[Risk: Visual screenshot flakiness on CI]** → Mitigation: Provide `CI=true` mode that uses separate baseline comparison strategy. Allow threshold configuration per screenshot.

**[Risk: Migration tool accuracy]** → Mitigation: Start with high-confidence automated migration (60-70% coverage) and generate report of manual review items. Don't aim for 100% automation.

**[Trade-off: Browser-specific vs unified API]** → The unified API means some browser-specific features won't be available. Accept this for simplicity and portability.

## Open Questions

1. **Worker IPC mechanism**: Unix sockets vs named pipes vs stdio JSON? Need to decide based on Windows compatibility requirements.

2. **Screenshot baseline storage**: Local filesystem works for single developer. How should CI handle baseline updates? Push to cloud storage?

3. **Migration priority**: Should we prioritize Playwright migration (most common) or Cypress (often has complex async patterns)?

4. **geckodriver integration**: The `geckodriver` Rust crate exists but may need patches. Should we vendor it or contribute upstream?

5. **Test isolation granularity**: Should each test file run in its own process, or share a process with isolate cleanup? Playwright uses process-per-file.
