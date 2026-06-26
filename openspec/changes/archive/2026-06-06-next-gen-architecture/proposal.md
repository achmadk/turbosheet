## Why

Turbosheet has a massive structural advantage being built in Rust, but it currently lacks the advanced Developer Experience (DX) and rock-solid reliability features that make Playwright and Cypress the industry standards. To become the best end-to-end testing tool, we need to eliminate test flakiness (via advanced actionability and web-first assertions), provide world-class debugging (via a time-travel trace viewer), and support robust context isolation and network mocking.

## What Changes

- **Advanced Actionability Engine**: Upgrade the injected script engine to support auto-scrolling before actionability checks, Shadow DOM piercing for selectors, and `pointer-events: none` checks.
- **Time-Travel Trace Viewer**: Build a local web UI to load trace JSON files, scrub through a timeline, and rehydrate DOM snapshots correlated with network/console events.
- **Network Matrix**: Introduce HAR file recording/playback and a more ergonomic, high-level route mocking API.
- **Device Emulation**: Add pre-configured device descriptors (viewport, UA, touch) and ensure strict, lightweight Browser Context isolation.
- **Web-First Assertions**: Introduce soft assertions and advanced matchers that automatically retry until a timeout is reached.

## Capabilities

### New Capabilities

- `trace-viewer-ui`: A local web UI for visualizing test traces, DOM snapshots, network, and console logs over a timeline.
- `device-emulation`: Pre-configured device descriptors and strict browser context isolation for mobile/tablet testing.
- `web-first-assertions`: A robust assertion engine supporting soft assertions, auto-retrying, and advanced matchers (e.g., `toHaveCount`).

### Modified Capabilities

- `injected-script-engine`: Modifying requirements to include auto-scrolling, Shadow DOM piercing, and pointer-events checks before actions.
- `network-interception`: Modifying requirements to support HAR file recording/playback and a high-level ergonomic mocking API.

## Impact

- **Rust Backend**: Significant extensions to `src/locator.rs`, `src/network/proxy.rs`, `src/assertions/matchers.rs`, and `src/page.rs`.
- **Injected JS**: Updates to `js/src/injected-actions.ts` to handle Shadow DOM and scrolling.
- **New Frontend Project**: A new Vite+ frontend for the trace viewer UI (likely placed in a `packages/trace-viewer` or similar directory).
