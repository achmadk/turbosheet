# Getting Started with TurboSheet

5-minute quickstart guide to running your first automated browser tests.

## Prerequisites

- Node.js 18+ installed
- npm or pnpm package manager

## Step 1: Install TurboSheet

```bash
npm install -D turbosheet
```

## Step 2: Install Browser

```bash
npx tsheet install
```

This downloads Chromium (the default browser). You can also install Firefox and WebKit:

```bash
npx tsheet install firefox
npx tsheet install webkit
```

## Step 3: Create Your First Test

Create a file `tests/example.spec.ts`:

```typescript
import { test, expect } from "turbosheet";

test("homepage loads correctly", async ({ page }) => {
  await page.goto("https://example.com");

  await expect(page).toHaveTitle(/Example/);
  await expect(page.locator("h1")).toHaveText("Example Domain");
});

test("navigation works", async ({ page }) => {
  await page.goto("https://example.com");
  await page.click('a[href="https://www.iana.org/domains/example"]');

  await expect(page).toHaveURL(/iana\.org/);
});
```

## Step 4: Create Configuration

Create `tsheet.config.ts` in your project root:

```typescript
import { defineConfig } from "turbosheet";

export default defineConfig({
  testDir: "./tests",
  workers: 2,
  timeout: 30000,
  reporter: "list",
});
```

## Step 5: Run Tests

```bash
npx tsheet test
```

You should see output like:

```
✓ homepage loads correctly (1.2s)
✓ navigation works (0.8s)

  2 passed, 0 failed, 0 skipped
```

---

## What Just Happened?

TurboSheet:

1. **Discovered** your test files matching `**/*.tsheet.ts` in the `tests/` directory
2. **Launched** 2 parallel browser workers
3. **Executed** each test in a fresh browser page
4. **Retried** failed assertions automatically until timeout
5. **Reported** results to the terminal

---

## Next Steps

### Add More Tests

Create additional test files following the same pattern:

```typescript
// tests/login.spec.ts
import { test, expect } from "turbosheet";

test.describe("login flow", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/login");
  });

  test("shows validation errors", async ({ page }) => {
    await page.click('button[type="submit"]');
    await expect(page.locator(".error")).toBeVisible();
  });

  test("logs in with valid credentials", async ({ page }) => {
    await page.fill('[name="email"]', "test@example.com");
    await page.fill('[name="password"]', "secret");
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL("/dashboard");
  });
});
```

### Use Selectors

TurboSheet supports multiple selector strategies:

```typescript
// CSS selector
await page.click("button.primary");

// Text content
await page.click("text=Continue");

// ARIA role
await page.getByRole("button", { name: "Submit" }).click();

// Test ID
await page.getByTestId("submit-button").click();

// XPath
await page.locator('xpath=//button[@type="submit"]').click();
```

### Add Assertions

```typescript
await expect(page).toHaveURL("/dashboard");
await expect(page).toHaveTitle("Dashboard");

await expect(page.locator(".welcome")).toHaveText("Welcome, John");
await expect(page.locator(".badge")).toHaveAttribute("data-status", "active");
await expect(page.locator('input[name="email"]')).toHaveValue("john@example.com");
await expect(page.locator(".items li")).toHaveCount(5);
```

### Handle Async Operations

```typescript
// Wait for element to appear
await page.waitForSelector(".loaded");

// Wait for navigation
await page.waitForURL("/dashboard");

// Wait for network idle
await page.goto("/dashboard", { waitUntil: "networkidle" });
```

### Mock Network Requests

```typescript
test("shows mock data", async ({ page }) => {
  await page.route("**/api/user", async (route) => {
    await route.fulfill({
      status: 200,
      body: JSON.stringify({ name: "Mock User", email: "mock@example.com" }),
    });
  });

  await page.goto("/profile");
  await expect(page.locator(".user-name")).toHaveText("Mock User");
});
```

---

## Common Tasks

### Run Specific Tests

```bash
# Run tests matching pattern
npx tsheet test --grep "login"

# Run only this test
npx tsheet test tests/login.spec.ts
```

### Debug a Test

```bash
npx tsheet test --debug
```

### Generate a Test

```bash
npx tsheet codegen --url http://localhost:3000
```

Opens a browser where you can interact and TurboSheet will generate test code.

### View Test Traces

```bash
npx tsheet show-trace test-results/trace.zip
```

### Take Screenshots on Failure

```typescript
// tsheet.config.ts
export default defineConfig({
  screenshotOnFailure: true,
  screenshotDir: "./test-results/screenshots",
});
```

---

## Configuration Reference

```typescript
// tsheet.config.ts
export default defineConfig({
  // Test discovery
  testDir: "./tests",
  testMatch: ["**/*.tsheet.ts", "**/*.tsheet.spec.ts"],

  // Execution
  workers: 4, // Parallel workers
  timeout: 30000, // Test timeout in ms
  retries: 1, // Retry failed tests

  // Reporting
  reporter: "list", // 'list', 'dot', 'line', 'html', 'json'

  // Output
  screenshotOnFailure: true,
  screenshotDir: "./test-results/screenshots",

  // Filtering
  grep: "pattern", // Only run tests matching pattern
  shard: { current: 1, total: 4 }, // Run shard 1 of 4

  // Browser
  browser: "chromium", // 'chromium', 'firefox', 'webkit'
  headed: false, // Show browser window
});
```

---

## Resources

- [API Reference](./api-reference.md) - Complete API documentation
- [Migration Guide](./migration-guide.md) - From Playwright, Cypress, or Puppeteer
- [CI/CD Guides](./ci) - GitHub Actions, GitLab CI, Jenkins, CircleCI

---

## Getting Help

```bash
# Show all available commands
npx tsheet --help

# Show device list
npx tsheet devices

# Show version
npx tsheet --version
```

Report issues at: https://github.com/turbosheet/turbosheet/issues
