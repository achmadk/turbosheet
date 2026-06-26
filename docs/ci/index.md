# CI/CD Integration Guides

Guides for integrating TurboSheet with popular CI/CD platforms.

## Available Guides

- [GitHub Actions](./github-actions.md)
- [GitLab CI](./gitlab-ci.md)
- [Jenkins](./jenkins.md)
- [CircleCI](./circleci.md)

---

## Quick Reference

### Basic CI Command

```bash
npx tsheet test --reporter=list
```

### With JUnit XML Output

```bash
npx tsheet test --reporter=junit --output-file=test-results/junit.xml
```

### With HTML Report

```bash
npx tsheet test --reporter=html
```

### Running Specific Browsers

```bash
npx tsheet test --browser=chromium
npx tsheet test --browser=firefox
npx tsheet test --browser=webkit
```

### Sharding

```bash
npx tsheet test --shard=1/4
```

### Caching Browsers

Cache the following directories:

- Linux: `~/.cache/tsheet`
- macOS: `~/Library/Caches/tsheet`
- Windows: `%LOCALAPPDATA%\tsheet\cache`

### Artifacts

Recommended paths to archive:

- `test-results/screenshots/` - Screenshots on failure
- `test-results/report.html` - HTML test report
- `test-results/*.xml` - JUnit XML reports

---

## Matrix Strategies

### GitHub Actions

```yaml
strategy:
  matrix:
    shard: [1, 2, 3, 4]
```

### GitLab CI

```yaml
test:
  parallel:
    matrix:
      - SHARD: [1, 2, 3, 4]
```

### CircleCI

```yaml
jobs:
  test:
    parameters:
      shard:
        type: integer
```

### Jenkins

```groovy
def tests = [:]
for (int i = 1; i <= 4; i++) {
    tests["Shard ${i}"] = {
        sh "npx tsheet test --shard=${i}/4"
    }
}
parallel tests
```
