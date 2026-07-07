## Why

The landing page (`landing-page/index.html`) presents TurboSheet's features as polished and complete, with no indication of actual development progress. `PROGRESS.md` contains a comprehensive feature audit with completion estimates per engine, critical gaps, and a detailed 8-phase implementation roadmap — but none of this data is surfaced on the marketing page. Adding a roadmap section builds trust through transparency, sets realistic expectations for evaluators, and shows momentum to potential contributors.

## What Changes

- Add a "Progress & Roadmap" section to `landing-page/index.html`, placed before the footer
- Create progress bars showing current completion % per engine/component (from the Overall Completion table in PROGRESS.md)
- Add a "Hard Blockers" callout highlighting the 3 critical gaps blocking v1.0
- Add a phase-by-phase roadmap (Phases 0-8) with durations, goals, and key deliverables
- Include overall effort estimates (time to v1.0, time to Playwright parity)
- All data sourced from `PROGRESS.md` — with a note that it was last updated 2026-06-18

No changes to any code, API, or build system. Single-file change to the landing page HTML.

## Capabilities

### New Capabilities

- `roadmap-section`: Landing page section showing project completion status by engine, critical gaps, and implementation roadmap phases with estimated durations

### Modified Capabilities

_(None — no existing specs are changing)_

## Impact

- Single file: `landing-page/index.html` (~150-250 lines added)
- No API, dependency, or system changes
- Data sourced from existing `PROGRESS.md` (no new audit required)
- Section added between the Code Example section and the Footer
