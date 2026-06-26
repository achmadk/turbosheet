# GitHub Actions Integration

Complete guide for running TurboSheet tests in GitHub Actions CI.

## Basic Setup

Create `.github/workflows/test.yml`:

```yaml
name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm"

      - name: Install dependencies
        run: npm ci

      - name: Install TurboSheet browsers
        run: npx tsheet install

      - name: Run tests
        run: npx tsheet test

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: test-results/
```

## With Matrix Testing (Multiple Browsers)

```yaml
name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        browser: [chromium, firefox, webkit]
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm"

      - name: Install dependencies
        run: npm ci

      - name: Install ${{ matrix.browser }}
        run: npx tsheet install ${{ matrix.browser }}

      - name: Run tests (${{ matrix.browser }})
        run: npx tsheet test --browser=${{ matrix.browser }}

      - name: Upload screenshots on failure
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: screenshots-${{ matrix.browser }}
          path: test-results/screenshots/

      - name: Upload HTML report
        uses: actions/upload-artifact@v4
        with:
          name: html-report-${{ matrix.browser }}
          path: test-results/report.html
```

## With Sharded Tests

```yaml
name: Sharded Tests

on:
  push:
    branches: [main]

env:
  TURBO_SHEET_VERSION: "0.1.0"

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        shard: [1, 2, 3, 4]
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm"

      - name: Install dependencies
        run: npm ci

      - name: Install TurboSheet browsers
        run: npx tsheet install

      - name: Run tests (shard ${{ matrix.shard }}/4)
        run: npx tsheet test --shard=${{ matrix.shard }}/4

      - name: Upload artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results-shard-${{ matrix.shard }}
          path: test-results/
```

## With Caching

```yaml
name: Tests with Cache

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm"

      - name: Cache TurboSheet browsers
        uses: actions/cache@v4
        with:
          path: ~/.cache/tsheet
          key: browsers-${{ runner.os }}-${{ hashFiles('package.json') }}

      - name: Install dependencies
        run: npm ci

      - name: Install TurboSheet browsers
        run: npx tsheet install

      - name: Run tests
        run: npx tsheet test

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: test-results/
```

## With GitHub Annotations

```yaml
name: Tests with Annotations

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm"

      - name: Install dependencies
        run: npm ci

      - name: Install TurboSheet browsers
        run: npx tsheet install

      - name: Run tests
        run: npx tsheet test --reporter=github

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: test-results/
```

## Complete Example with All Features

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  TURBO_SHEET_VERSION: "0.1.0"
  NODE_VERSION: "20"

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: "npm"
      - run: npm ci
      - run: npm run lint

  test:
    needs: lint
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        shard: [1, 2]
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: "npm"

      - name: Cache TurboSheet browsers
        uses: actions/cache@v4
        with:
          path: ~/.cache/tsheet
          key: browsers-${{ runner.os }}-${{ hashFiles('package.json') }}

      - name: Install dependencies
        run: npm ci

      - name: Install TurboSheet browsers
        run: npx tsheet install chromium

      - name: Run tests (shard ${{ matrix.shard }}/2)
        run: npx tsheet test --shard=${{ matrix.shard }}/2 --reporter=github

      - name: Upload screenshots on failure
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: screenshots-shard-${{ matrix.shard }}
          path: test-results/screenshots/

  merge:
    needs: [lint, test]
    runs-on: ubuntu-latest
    if: always()
    steps:
      - name: Check if all tests passed
        run: |
          if [[ "${{ needs.test.result }}" != "success" ]]; then
            echo "Tests failed"
            exit 1
          fi
          echo "All checks passed!"
```
