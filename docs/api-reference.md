# API Reference

Complete API documentation for TurboSheet.

## Table of Contents

- [Configuration](#configuration)
- [Test Functions](#test-functions)
- [Page Object](#page-object)
- [Locator API](#locator-api)
- [Assertions](#assertions)
- [Fixtures](#fixtures)
- [Browser Context](#browser-context)
- [Network Routing](#network-routing)
- [Mobile Emulation](#mobile-emulation)

---

## Configuration

### `defineConfig(config)`

Defines the test configuration.

```typescript
import { defineConfig } from "turbosheet";

export default defineConfig({
  testDir: "./tests",
  workers: 4,
  timeout: 30000,
  retries: 1,
  reporter: "html",
});
```

### Config Options

| Option                | Type                 | Default                                     | Description                   |
| --------------------- | -------------------- | ------------------------------------------- | ----------------------------- |
| `testDir`             | `string`             | `"."`                                       | Directory to discover tests   |
| `testMatch`           | `string[]`           | `["**/*.tsheet.ts", "**/*.tsheet.spec.ts"]` | Glob patterns for test files  |
| `workers`             | `number`             | CPU count                                   | Number of parallel workers    |
| `timeout`             | `number`             | `30000`                                     | Default test timeout in ms    |
| `retries`             | `number`             | `0`                                         | Retry failed tests N times    |
| `reporter`            | `string`             | `"list"`                                    | Reporter(s) to use            |
| `projects`            | `ProjectReference[]` | `[]`                                        | Monorepo project references   |
| `globalSetup`         | `string`             | `null`                                      | Path to global setup file     |
| `globalTeardown`      | `string`             | `null`                                      | Path to global teardown file  |
| `grep`                | `string`             | `null`                                      | Filter tests by pattern       |
| `shard`               | `ShardConfig`        | `null`                                      | Shard configuration           |
| `screenshotOnFailure` | `boolean`            | `true`                                      | Capture screenshot on failure |
| `screenshotDir`       | `string`             | `"test-results/screenshots"`                | Screenshot output directory   |

---

## Test Functions

### `test(name, fn)`

Defines a test case.

```typescript
import { test, expect } from "turbosheet";

test("login works", async ({ page }) => {
  await page.goto("/login");
  await page.fill('[name="username"]', "user");
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL("/dashboard");
});
```

### `test.only(name, fn)`

Runs only this test.

```typescript
test.only("debug this", async ({ page }) => {
  // Only this test runs
});
```

### `test.skip(name, fn)`

Skips this test.

```typescript
test.skip("not ready", async ({ page }) => {
  // Will be skipped
});
```

### `test.fail(name, fn)`

Marks test as expected to fail.

```typescript
test.fail("known issue", async ({ page }) => {
  // If this passes, test fails
  // If this fails, test passes
});
```

### `test.slow(name, fn)`

Triples the timeout for this test.

### `test.describe(name, fn)`

Groups related tests.

```typescript
test.describe("login", () => {
  test("with valid credentials", async ({ page }) => {
    // ...
  });

  test("with invalid credentials", async ({ page }) => {
    // ...
  });
});
```

### `test.describe.serial(name, fn)`

Runs tests in this describe block serially.

### `test.beforeAll(fn)`

Runs once before all tests in the describe block.

```typescript
test.describe("dashboard", () => {
  test.beforeAll(async ({ page }) => {
    await page.goto("/login");
    await page.fill('[name="username"]', "admin");
    await page.click("button");
  });

  test("shows user info", async ({ page }) => {
    await expect(page.locator(".user-name")).toHaveText("admin");
  });
});
```

### `test.afterAll(fn)`

Runs once after all tests in the describe block.

### `test.beforeEach(fn)`

Runs before each test in the describe block.

### `test.afterEach(fn)`

Runs after each test in the describe block.

### `test.extend(fixtures)`

Creates extended test with custom fixtures.

```typescript
const myTest = test.extend({
  db: async ({ page }, use) => {
    const database = await createDatabase();
    await use(database);
    await database.close();
  },
});

myTest("uses db fixture", async ({ db }) => {
  const users = await db.query("SELECT * FROM users");
  expect(users).toHaveLength(5);
});
```

---

## Page Object

### `page.goto(url, options?)`

Navigates to a URL.

```typescript
await page.goto("https://example.com");
await page.goto("/login", { waitUntil: "networkidle" });
```

Options:

- `waitUntil`: `"load"` | `"domcontentloaded"` | `"networkidle"`

### `page.click(selector, options?)`

Clicks an element.

```typescript
await page.click("button#submit");
await page.click("text=Continue", { modifiers: ["Shift"] });
```

### `page.fill(selector, value)`

Fills an input field.

```typescript
await page.fill('[name="email"]', "user@example.com");
```

### `page.locator(selector)`

Creates a locator for an element.

```typescript
const button = page.locator("button.submit");
await button.click();
```

### `page.waitForSelector(selector, options?)`

Waits for element to appear.

```typescript
await page.waitForSelector(".loaded", { state: "visible" });
await page.waitForSelector(".modal", { state: "hidden" });
```

### `page.screenshot(options?)`

Takes a screenshot.

```typescript
await page.screenshot();
await page.screenshot({ fullPage: true });
await page.screenshot({ path: "screenshot.png" });
```

### `page.evaluate(fn)`

Executes JavaScript in the page context.

```typescript
const title = await page.evaluate(() => document.title);
const result = await page.evaluate(({ x, y }) => x + y, { x: 1, y: 2 });
```

### `page.route(pattern, handler)`

Intercepts network requests.

```typescript
await page.route("**/api/**", async (route) => {
  if (route.request().url().includes("user")) {
    await route.fulfill({
      status: 200,
      body: JSON.stringify({ id: 1, name: "Test" }),
    });
  } else {
    await route.continue();
  }
});
```

### `page.unrouteAll()`

Removes all route handlers.

```typescript
page.unrouteAll();
```

### `page.emulate(options)`

Emulates device or media.

```typescript
await page.emulate({
  viewport: { width: 375, height: 812 },
  userAgent: "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0)",
  deviceScaleFactor: 3,
  hasTouch: true,
  isMobile: true,
});
```

### `page.emulateMedia(options)`

Emulates media features.

```typescript
await page.emulateMedia({ colorScheme: "dark" });
await page.emulateMedia({ reducedMotion: "reduce" });
```

### `page.setGeolocation(geolocation)`

Sets geolocation.

```typescript
await page.setGeolocation({ latitude: 37.7749, longitude: -122.4194 });
```

### `page.setOffline(offline)`

Sets offline mode.

```typescript
await page.setOffline(true);
```

### `page.throttle(configuration)`

Throttles network.

```typescript
await page.throttle({ download: 1000, upload: 500, latency: 100 });
```

---

## Locator API

### `.locator(selector)`

Creates a child locator.

```typescript
const form = page.locator("form");
await form.locator("input").fill("value");
```

### `.getByText(text, options?)`

Finds element by text.

```typescript
await page.getByText("Submit").click();
await page.getByText("Welcome,", { exact: false }).click();
```

### `.getByRole(role, options?)`

Finds element by ARIA role.

```typescript
await page.getByRole("button", { name: "Submit" }).click();
await page.getByRole("textbox", { name: "Email" }).fill("test@example.com");
```

### `.getByTestId(testId)`

Finds element by test ID.

```typescript
await page.getByTestId("submit-button").click();
```

### `.first()`

Returns first matching element.

```typescript
await page.locator("li").first().click();
```

### `.last()`

Returns last matching element.

### `.nth(index)`

Returns element at index.

### `.all()`

Returns all matching elements.

```typescript
const items = await page.locator("li").all();
for (const item of items) {
  console.log(await item.textContent());
}
```

### `.count()`

Returns number of matching elements.

```typescript
const count = await page.locator("li").count();
```

### `.wait()`

Waits for locator to meet condition.

```typescript
await page.locator(".loaded").wait({ state: "visible" });
```

---

## Assertions

All assertions auto-retry until timeout.

### `expect(value)`

Creates an assertion.

```typescript
await expect(page).toHaveURL(/\/dashboard/);
await expect(page.locator(".user-name")).toHaveText("admin");
```

### Page Assertions

| Assertion                          | Description             |
| ---------------------------------- | ----------------------- |
| `toHaveURL(pattern)`               | URL matches pattern     |
| `toHaveTitle(title)`               | Page title matches      |
| `toHaveScreenshot(name, options?)` | Page matches screenshot |

### Locator Assertions

| Assertion                       | Description           |
| ------------------------------- | --------------------- |
| `toBeVisible()`                 | Element is visible    |
| `toBeHidden()`                  | Element is hidden     |
| `toBeEnabled()`                 | Element is enabled    |
| `toBeDisabled()`                | Element is disabled   |
| `toHaveText(text)`              | Element has text      |
| `toHaveAttribute(name, value?)` | Element has attribute |
| `toHaveValue(value)`            | Input has value       |
| `toHaveCount(count)`            | Element count matches |

### Options

```typescript
await expect(locator).toBeVisible({ timeout: 5000 });
await expect(locator).toHaveText("Hello", { ignoreCase: true });
```

---

## Fixtures

### Built-in Fixtures

| Fixture   | Type                | Description      |
| --------- | ------------------- | ---------------- |
| `page`    | `Page`              | Browser page     |
| `context` | `BrowserContext`    | Browser context  |
| `browser` | `Browser`           | Browser instance |
| `request` | `APIRequestContext` | API requests     |

### Custom Fixtures

```typescript
const test = test.extend({
  apiClient: async ({}, use) => {
    const client = new APIClient("https://api.example.com");
    await use(client);
    await client.cleanup();
  },
});
```

### Fixture Scopes

- **test-level**: Created per test, shared within single test execution
- **worker-level**: Created per worker, shared across tests in same worker
- **global**: Created once, shared across all tests

```typescript
// Worker-level (default)
test.extend({
  db: async ({ workerIndex }, use) => {
    const db = await createDatabase(workerIndex);
    await use(db);
  },
});

// Global
test.extend(
  {
    config: async ({}, use) => {
      const config = await loadGlobalConfig();
      await use(config);
    },
  },
  { scope: "global" },
);
```

---

## Browser Context

### `context.newPage()`

Creates a new page in context.

```typescript
const page = await context.newPage();
```

### `context.addInitScript(script)`

Adds script to run before each page.

```typescript
await context.addInitScript(() => {
  window.mockUser = { id: 1, name: "Test User" };
});
```

### `context.addCookies(cookies)`

Adds cookies.

```typescript
await context.addCookies([
  {
    name: "session",
    value: "abc123",
    domain: ".example.com",
    path: "/",
  },
]);
```

### `context.clearCookies()`

Clears all cookies.

### `context.grantPermissions(permissions)`

Grants browser permissions.

```typescript
await context.grantPermissions(["geolocation"]);
```

---

## Network Routing

### `page.route(url, handler)`

Intercepts matching requests.

```typescript
await page.route("**/*.{png,jpg,jpeg}", (route) => {
  route.abort();
});
```

### `route.abort(errorCode?)`

Aborts the request.

```typescript
route.abort("failed");
route.abort("timedout");
```

### `route.continue(requestOverrides?)`

Continues the request with modifications.

```typescript
route.continue({
  method: 'POST',
  headers: { ... },
  postData: JSON.stringify({ data: 123 }),
});
```

### `route.fulfill(response)`

Fulfills with mock response.

```typescript
route.fulfill({
  status: 200,
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ success: true }),
});
```

---

## Mobile Emulation

### Built-in Devices

```typescript
import { devices } from "turbosheet";

test("works on iPhone", async ({ page }) => {
  await page.emulate(devices["iPhone 13"]);
});
```

### Available Devices

- `iPhone 6`, `iPhone 6 Plus`
- `iPhone 7`, `iPhone 7 Plus`
- `iPhone 8`, `iPhone 8 Plus`
- `iPhone X`, `iPhone XR`, `iPhone XS Max`
- `iPhone 11`, `iPhone 11 Pro`, `iPhone 11 Pro Max`
- `iPhone 12`, `iPhone 12 mini`, `iPhone 12 Pro`, `iPhone 12 Pro Max`
- `iPhone 13`, `iPhone 13 mini`, `iPhone 13 Pro`, `iPhone 13 Pro Max`
- `iPhone 14`, `iPhone 14 Plus`, `iPhone 14 Pro`, `iPhone 14 Pro Max`
- `iPad`, `iPad Mini`, `iPad Pro`, `iPad Air`
- `Pixel 3`, `Pixel 3 XL`, `Pixel 4`, `Pixel 4 XL`, `Pixel 5`
- `Samsung Galaxy S5`, `Samsung Galaxy S8`, `Samsung Galaxy S10+`
- `Microsoft Lumia 550`, `Microsoft Lumia 950`
- `Surface Duo`, `Surface Pro 7`

### Custom Emulation

```typescript
await page.emulate({
  viewport: { width: 390, height: 844 },
  userAgent: "Custom User Agent",
  deviceScaleFactor: 2,
  hasTouch: true,
  isMobile: true,
});
```
