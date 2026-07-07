## Context

The landing page at `landing-page/index.html` is a standalone HTML+CSS+JS file with no build step. It was rewritten during the `landing-page-realignment` change to accurately reflect what's built in the codebase. `PROGRESS.md` contains a comprehensive feature audit (1097 lines) including an Overall Completion by Engine table, Critical Gaps, and an 8-phase Implementation Priority Roadmap — but none of this data appears on the landing page.

The target audience is engineering teams evaluating E2E testing tools. This audience values transparency and accurate status information. The roadmap section will serve as a trust signal and expectation-setter.

## Goals / Non-Goals

**Goals:**

- Add a "Progress & Roadmap" section to `landing-page/index.html` placed before the footer
- Render completion % per engine/component as visual progress bars
- Show the 3 hard blockers that must be fixed before v1.0
- Show all 8 roadmap phases with names, durations, goals, and key deliverables
- Include overall effort estimates (time to v1.0, time to Playwright parity)
- Match the existing dark futuristic design aesthetic (var(--accent), monospace, cards, borders)
- Mark data as sourced from PROGRESS.md with generation date for transparency

**Non-Goals:**

- No build tooling or framework migration (remains standalone HTML+CSS+JS)
- No new CSS design system or visual overhaul
- No feature-level drilldown (no individual comparison rows from the 26 comparison tables)
- No quantitative benchmark numbers
- No interactive filtering or sorting

## Decisions

### Decision: Static HTML section with inline CSS (no JS rendering)

The existing page has no JS framework and opens via `file://`. The progress bars and roadmap timeline will be rendered with static HTML + CSS (inline styles, pseudo-elements, flexbox). No JavaScript needed for the visual rendering — JS is only used for the dropdown-style expand/collapse if we choose to add it.

**Alternative considered:** Render bars via canvas or JS — rejected because it adds complexity and breaks the `file://` simplicity.

### Decision: Progress bars use CSS width percentages with inline style

Each bar will be a `<div>` with `style="width: 65%"` set inline. The data comes from PROGRESS.md's Overall Completion table.

### Decision: Three subsections within the roadmap block (no tabs)

The section will have three visual sub-blocks:

1. **Engine Completion** — progress bars per component
2. **Hard Blockers** — 3 critical gaps callout
3. **Roadmap** — phase-by-phase list with durations and deliverables

These are stacked vertically, not tabbed. Tabs would require JS state management and hide information.

### Decision: Data sourced from PROGRESS.md with freshness note

The section will include a small muted note: "Data sourced from PROGRESS.md — last generated 2026-06-18". This sets expectations that the data may be stale if the page isn't updated alongside the progress report.

## Risks / Trade-offs

- **[Risk] Stale data** → The PROGRESS.md data will drift as development progresses. Mitigation: Include the "last generated" date prominently in the section and add a comment referencing PROGRESS.md as the source.
- **[Risk] Page length increase** → The section is estimated at 150-250 lines, which increases total page size by ~20-30%. Mitigation: This is a single-load marketing page, not a web app — bundle size is not a concern.
- **[Trade-off] Honest but not polished** → Showing 5-25% completion on Firefox/WebKit may deter some users. Value: The target audience (engineering teams) respects transparency over hype.
