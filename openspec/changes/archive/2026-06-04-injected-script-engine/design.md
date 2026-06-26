## Context

TurboSheet currently relies heavily on ad-hoc `page.evaluate()` strings for interactions in Chromium, Firefox, and WebKit. This causes massive JS logic duplication, extremely high CDP round-trips for simple interactions (like `.click()`), and lacks reactive event handling. Additionally, network interception via the local proxy is incomplete (501 Not Implemented, no HTTPS tunnel, missing POST payloads). The goal is to move to a state-of-the-art injected script architecture, matching or exceeding Playwright's efficiency.

## Goals / Non-Goals

**Goals:**

- Inject a core Javascript payload into all new documents to handle DOM bindings and mutation observation.
- Lazy-load an actionability payload only when interactions are requested.
- Establish a CDP Event Dispatcher to reactively handle network events and binding callbacks.
- Repair or replace the broken Network Proxy to allow intercepting requests properly.

**Non-Goals:**

- Modifying the public API of TurboSheet. Users should experience no breaking changes in how they call `.click()` or other interactions.
- Completely rewriting the engine trait from scratch (we will extend it, not rewrite it).

## Decisions

- **Decision 1: Hybrid Injection Approach over Monolithic Payload**
  _Rationale_: A monolithic script creates excessive CPU overhead on pages with many iframes. The hybrid approach (small core + on-demand actions) keeps memory and execution cost low.
- **Decision 2: CDP Bindings (`Runtime.addBinding`) over Polling**
  _Rationale_: Emitting results back via bindings allows Rust to avoid busy-waiting. UUID request-correlation provides a simple RPC layer.
- **Decision 3: CDP Event Dispatcher instead of inline streaming loops**
  _Rationale_: Centralizing event dispatching enables features like URL tracking on navigation without active polling, and sets the foundation for proper network interception and dialog handling.

## Risks / Trade-offs

- [Risk] Injected script breaks on complex SPAs or detecting bots.
  → Mitigation: Use strict IIFE isolation and randomize binding globals.
- [Risk] Missing CDP event subscriptions could drop messages during rapid navigation.
  → Mitigation: Implement navigation-aware message queues and auto-re-inject on `frameNavigated`.
