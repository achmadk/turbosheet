# TurboSheet Migration Guide

Migrating to TurboSheet is fast and automated! TurboSheet provides an AST-based migration tool that will automatically rewrite your existing Playwright and Cypress tests to leverage TurboSheet's high-performance native engine.

## Using the CLI Tool

To automatically migrate your tests, run the built-in migration CLI:

```bash
# Migrate Playwright tests in the current directory
tsheet migrate playwright

# Migrate Cypress tests in a specific directory
tsheet migrate cypress --dir cypress/e2e

# Perform a dry run to see what will change without writing files
tsheet migrate playwright --dry-run
```

## What Gets Migrated

### Playwright

| Playwright                                | TurboSheet                           | Notes                                                                |
| ----------------------------------------- | ------------------------------------ | -------------------------------------------------------------------- |
| `import { test } from '@playwright/test'` | `import { test } from 'tsheet'`      | Fully automated                                                      |
| `page.$eval('selector', cb)`              | `page.evaluate('selector', cb)`      | Fully automated                                                      |
| `page.waitForSelector('selector')`        | `page.locator('selector').waitFor()` | Automated, but note that TurboSheet auto-waits by default on actions |

### Cypress

| Cypress                      | TurboSheet                               | Notes                                                                                       |
| ---------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------- |
| `cy.visit('/url')`           | `await page.goto('/url')`                | Automated. TurboSheet is fully asynchronous.                                                |
| `cy.get('selector').click()` | `await page.locator('selector').click()` | Automated.                                                                                  |
| `describe(...)` / `it(...)`  | `test.describe(...)` / `test(...)`       | Automated. The CLI automatically injects `async ({ page }) => {}` into your test functions. |

## Manual Steps

While the CLI tool handles 90% of the migration, you may need to manually review:

1. **Custom Plugins/Extensions**: If you used Playwright/Cypress specific plugins, you will need to migrate them to TurboSheet plugins.
2. **Network Interception**: Complex `cy.intercept` or `page.route` scenarios might require manual adjustment to match TurboSheet's native N-API network hooking.
3. **Configuration**: You will need to manually port your `playwright.config.ts` or `cypress.config.js` to `tsheet.config.ts`.

## Need Help?

Run `tsheet help migrate` for more options.
