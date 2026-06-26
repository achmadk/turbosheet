## Why

The current landing page (`landing-page/index.html`) makes several false claims (three browser engines, fabricated benchmarks, `page.act()` API that doesn't exist, "zero runtime dependencies") while completely omitting the project's genuinely strong, already-built features like the test runner, component testing, trace viewer, visual regression, and network interception. This misrepresentation undermines credibility and fails to attract the right users — engineering teams evaluating E2E testing tools.

## What Changes

- Rewrite the landing page to reflect **only what's actually built** in the codebase
- Remove all fabricated metrics ("67% faster", "50% less memory", "100ms check", "40MB footprint")
- Remove false API claims (`page.act()`, "three engine" claim, "zero dependencies")
- Add sections for the real features: test runner, component testing, trace viewer, visual regression, video recording, network interception, locator API, CLI
- Update the code example to show real API usage (`launch()`, test runner, component mounting)
- Fix package name references and version metadata
- Keep the existing visual design aesthetic (dark futuristic theme)

## Capabilities

### New Capabilities

- `landing-page-content`: Landing page copy, features section, benchmark claims, and code examples aligned with actual codebase capabilities

### Modified Capabilities

_(None — no existing specs are changing)_

## Impact

- Single file: `landing-page/index.html` (full rewrite)
- No API, dependency, or system changes
- No breaking changes to shipped code
