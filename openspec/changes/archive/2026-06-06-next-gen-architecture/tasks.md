## 1. Advanced Actionability Engine

- [ ] 1.1 Implement Shadow DOM piercing in `js/src/injected-actions.ts`
- [ ] 1.2 Implement auto-scrolling into view before `elementFromPoint` hit-tests
- [ ] 1.3 Add checks for `pointer-events: none` and DOM attachment stability

## 2. Web-First Assertions

- [ ] 2.1 Enhance `AssertionEngine` in `src/assertions/matchers.rs` to support polling
- [ ] 2.2 Implement soft assertions functionality
- [ ] 2.3 Add advanced matchers (e.g., `toHaveCount`, `toHaveClass`)

## 3. Network Mocking & HAR Playback

- [ ] 3.1 Implement declarative route mocking API (`page.route("**/*", mock)`) in `src/page.rs` and `src/network/proxy.rs`
- [ ] 3.2 Add HAR file recording functionality in `src/network/proxy.rs`
- [ ] 3.3 Add HAR playback for request matching and stubbing

## 4. Zero-Cost Device Emulation

- [ ] 4.1 Define mobile and tablet device descriptors (UA, viewport, scale factor)
- [ ] 4.2 Implement strict Browser Context API to enforce isolation of cookies/storage

## 5. Trace Viewer UI

- [ ] 5.1 Initialize Vite+ React/Vue project in `packages/trace-viewer`
- [ ] 5.2 Build scrubbable timeline for network and console events
- [ ] 5.3 Implement iframe-based DOM rehydration using snapshot HTML
