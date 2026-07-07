## ADDED Requirements

### Requirement: Completion progress bars

The landing page SHALL display a set of visual progress bars showing the current completion percentage per engine/component, sourced from the PROGRESS.md "Overall Completion by Engine" table.

The following components SHALL be shown:

- Chromium (65%)
- JS/TS Layer (65%)
- Reporting (60%)
- Trace System (50%)
- Test Runner (35%)
- Video Recording (30%)
- Firefox (25%)
- WebKit (20%)
- Plugin System (10%)
- Migration (5%)

An overall aggregate percentage (~35%) SHALL also be displayed.

Each bar SHALL display the component name, a filled/unfilled bar visualization, and the percentage label. Line-of-code counts from PROGRESS.md MAY be included as secondary detail.

#### Scenario: All bars visible

- **WHEN** the page loads
- **THEN** each of the 10 components SHALL display a progress bar with the correct percentage width and label

### Requirement: Hard blockers callout

The landing page SHALL display a callout section listing the three critical gaps that block v1.0, as identified in PROGRESS.md:

1. `test()`/`describe()`/hooks are empty stubs — cannot define tests
2. 7 built-in reporters exist but are not wired to TestExecutor — no test output
3. No CLI binary or config file parser — cannot integrate in CI

#### Scenario: Hard blockers visible

- **WHEN** the page loads
- **THEN** the three hard blockers SHALL be displayed in a visually distinct callout block

### Requirement: Phase-by-phase roadmap

The landing page SHALL display all 8 implementation phases from the PROGRESS.md "Implementation Priority Roadmap", each showing:

- Phase number and name
- Estimated duration
- Goal summary
- Key deliverables (2-4 bullet points per phase)

The 8 phases in order SHALL be:

1. Phase 0 — Quick Wins (1-2 weeks)
2. Phase 1 — Test Runner Core (3-4 weeks)
3. Phase 2 — Auto-Waiting & Actionability (3-4 weeks)
4. Phase 3 — Locator Engine Completion (2-3 weeks)
5. Phase 4 — Error Hierarchy & Events (1-2 weeks)
6. Phase 5 — Firefox/WebKit Completeness (4-6 weeks)
7. Phase 6 — CLI, CI/CD & Config (2-3 weeks)
8. Phase 7 — Advanced Features (6-8 weeks)
9. Phase 8 — Enterprise Readiness (4-6 weeks)

Phase 0 SHALL be visually marked as "current" / "now".

#### Scenario: All roadmap phases displayed

- **WHEN** the page loads
- **THEN** all 8 phases SHALL be displayed in order with name, duration, and key deliverables

### Requirement: Effort estimates

The landing page SHALL display total effort estimates:

- Estimated time to v1.0 (Phases 0-2, 6): 10-14 weeks for a small team
- Estimated time to Playwright parity (all phases): 6-9 months for a small team

#### Scenario: Estimates displayed

- **WHEN** the page loads
- **THEN** the effort estimates SHALL be shown at the bottom of the roadmap section

### Requirement: Data freshness attribution

The roadmap section SHALL include a note attributing the data to PROGRESS.md with its last generation date (2026-06-18).

#### Scenario: Attribution visible

- **WHEN** the page loads
- **THEN** a muted text attribution SHALL be displayed noting the data source and date

### Requirement: Visual design consistency

The roadmap section SHALL match the existing page's design language:

- Background: var(--bg-deep) (#06060f)
- Accent color: var(--accent) (oklch(0.75 0.18 200))
- Cards with var(--bg-surface) and var(--border)
- Monospace font for headers and labels (var(--font-display))
- Responsive: stacks vertically on mobile (max-width: 640px)
