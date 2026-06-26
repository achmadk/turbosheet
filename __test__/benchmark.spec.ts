import { describe, it, expect, beforeAll, afterAll } from "vite-plus/test";

// ── Benchmark utilities ──────────────────────────────────────────────

type Measurement = { label: string; durationMs: number };

function report(measurements: Measurement[]) {
  if (measurements.length === 0) return;
  const sorted = [...measurements].sort((a, b) => a.durationMs - b.durationMs);
  const count = sorted.length;
  const p50 = sorted[Math.floor(count * 0.5)].durationMs;
  const p90 = sorted[Math.floor(count * 0.9)].durationMs;
  const p95 = sorted[Math.floor(count * 0.95)].durationMs;
  const p99 = sorted[Math.floor(count * 0.99)].durationMs;
  const min = sorted[0].durationMs;
  const max = sorted[count - 1].durationMs;
  const avg = sorted.reduce((s, m) => s + m.durationMs, 0) / count;
  console.log(`\n  📊 ${measurements[0].label}`);
  console.log(`     Samples : ${count}`);
  console.log(`     Min     : ${min.toFixed(3)}ms`);
  console.log(`     Avg     : ${avg.toFixed(3)}ms`);
  console.log(`     P50     : ${p50.toFixed(3)}ms`);
  console.log(`     P90     : ${p90.toFixed(3)}ms`);
  console.log(`     P95     : ${p95.toFixed(3)}ms`);
  console.log(`     P99     : ${p99.toFixed(3)}ms`);
  console.log(`     Max     : ${max.toFixed(3)}ms`);
}

// ── Benchmark suite ───────────────────────────────────────────────────

describe("1.5.5 Engine Performance Benchmarks", () => {
  // oxlint-disable-next-line typescript/no-redundant-type-constituents
  let browser: import("../index.js").Browser | null = null;
  let page: import("../index.js").Page | null = null;

  beforeAll(async () => {
    try {
      const tsheet = await import("../index.js");
      if (tsheet && typeof tsheet.launch === "function") {
        browser = await tsheet.launch({ headless: true });
        const context = await browser.newContext();
        page = await context.newPage();
      }
    } catch (err) {
      console.warn("Browser launch failed, skipping benchmarks:", err);
    }
  }, 60000);

  afterAll(async () => {
    if (page) await page.close();
    if (browser) await browser.close();
  });

  // ── Actionability check (full locator path) ────────────────────

  it("should measure isVisible latency (locator: strict + invoke_action)", async () => {
    if (!page) {
      console.log("  ⚠ Skipped — no browser available");
      expect(page).not.toBeNull();
      return;
    }

    await page.goto(`data:text/html,<div id="a">target</div>`);
    const loc = page.locator("#a");
    const measurements: Measurement[] = [];
    const SAMPLES = 200;

    for (let i = 0; i < SAMPLES; i++) {
      const start = performance.now();
      await loc.isVisible();
      const elapsed = performance.now() - start;
      measurements.push({ label: "isVisible latency (full locator)", durationMs: elapsed });
    }

    report(measurements);
  });

  // ── Raw invoke_action latency (no strict mode) ─────────────────

  it("should measure raw invoke_action round-trips (target: ≤2 per click)", async () => {
    if (!page) {
      console.log("  ⚠ Skipped — no browser available");
      expect(page).not.toBeNull();
      return;
    }

    await page.goto(
      `data:text/html,<button id="btn" style="width:100px;height:40px">click</button>`,
    );

    // Chromium click flow: get_element_center (1 evaluate) + CDP mousePressed + CDP mouseReleased
    // = 3 CDP round-trips (target: ≤2, needs optimization)
    const measurements: Measurement[] = [];
    const SAMPLES = 100;

    for (let i = 0; i < SAMPLES; i++) {
      const start = performance.now();
      await page.locator("#btn").click();
      const elapsed = performance.now() - start;
      measurements.push({ label: "click (full locator)", durationMs: elapsed });
    }

    report(measurements);

    // Theoretical CDP round-trips for Chromium click:
    // 1. get_element_center evaluate → 1 round-trip
    // 2. CDP mousePressed → 1 round-trip
    // 3. CDP mouseReleased → 1 round-trip
    // = 3 total (target: 2 — eliminate get_element_center evaluate by using binding bridge)
    // The binding bridge replaces the evaluate with invoke_action which uses a single CDP evaluate+binding callback
  });

  // ── Click round-trips ──────────────────────────────────────────

  it("should measure click latency (target: ≤2 CDP round-trips)", async () => {
    if (!page) {
      console.log("  ⚠ Skipped — no browser available");
      expect(page).not.toBeNull();
      return;
    }

    await page.goto(
      `data:text/html,<button id="btn" style="width:100px;height:40px">click</button>`,
    );
    const btn = page.locator("#btn");
    const measurements: Measurement[] = [];
    const SAMPLES = 100;

    for (let i = 0; i < SAMPLES; i++) {
      const start = performance.now();
      await btn.click();
      const elapsed = performance.now() - start;
      measurements.push({ label: "click latency", durationMs: elapsed });
    }

    report(measurements);
  });

  // ── invoke_action latency (binding bridge) ─────────────────────

  it("should measure invoke_action latency across all action types", async () => {
    if (!page) {
      console.log("  ⚠ Skipped — no browser available");
      expect(page).not.toBeNull();
      return;
    }

    await page.goto(`data:text/html,
      <div id="el" data-val="x" style="width:50px;height:30px">
        <span>text</span>
        <input id="inp" type="checkbox" />
        <select id="sel"><option value="a">A</option></select>
      </div>
    `);

    type ActionCall = { name: string; fn: () => Promise<any> };
    const actions: ActionCall[] = [
      { name: "content", fn: () => page!.content() },
      { name: "title", fn: () => page!.title() },
      { name: "isVisible", fn: () => page!.locator("#el").isVisible() },
      { name: "isEnabled", fn: () => page!.locator("#el").isEnabled() },
      { name: "isDisabled", fn: () => page!.locator("#el").isDisabled() },
      { name: "focus", fn: () => page!.locator("#inp").focus() },
      { name: "blur", fn: () => page!.locator("#inp").blur() },
      { name: "scrollIntoView", fn: () => page!.locator("#el").scrollIntoView() },
      { name: "check", fn: () => page!.locator("#inp").check() },
      { name: "uncheck", fn: () => page!.locator("#inp").uncheck() },
      { name: "select", fn: () => page!.locator("#sel").select("a") },
      { name: "innerText", fn: () => page!.locator("#el").innerText() },
      { name: "innerHTML", fn: () => page!.locator("#el").innerHTML() },
      { name: "getAttribute", fn: () => page!.locator("#el").getAttribute("data-val") },
      { name: "hover", fn: () => page!.locator("#el").hover() },
      { name: "dblclick", fn: () => page!.locator("#inp").dblclick() },
      { name: "rightClick", fn: () => page!.locator("#inp").rightClick() },
    ];

    const SAMPLES = 30;

    for (const action of actions) {
      const measurements: Measurement[] = [];
      for (let i = 0; i < SAMPLES; i++) {
        const start = performance.now();
        await action.fn();
        const elapsed = performance.now() - start;
        measurements.push({ label: action.name, durationMs: elapsed });
      }

      const sorted = [...measurements].sort((a, b) => a.durationMs - b.durationMs);
      const p95 = sorted[Math.floor(SAMPLES * 0.95)].durationMs;
      console.log(
        `  ${action.name.padEnd(18)} p95: ${p95.toFixed(3)}ms  (avg: ${(sorted.reduce((s, m) => s + m.durationMs, 0) / SAMPLES).toFixed(3)}ms)`,
      );
    }
  });

  // ── Concurrent action throughput ───────────────────────────────

  it("should measure concurrent invoke_action throughput", async () => {
    if (!page) {
      console.log("  ⚠ Skipped — no browser available");
      expect(page).not.toBeNull();
      return;
    }

    await page.goto(`data:text/html,
      <div id="a">A</div>
      <div id="b">B</div>
      <div id="c">C</div>
      <div id="d">D</div>
    `);

    const SAMPLES = 50;
    const start = performance.now();

    for (let i = 0; i < SAMPLES; i++) {
      await Promise.all([
        page!.locator("#a").isVisible(),
        page!.locator("#b").isVisible(),
        page!.locator("#c").isVisible(),
        page!.locator("#d").isVisible(),
      ]);
    }

    const total = performance.now() - start;
    const perBatch = total / SAMPLES;
    const perAction = perBatch / 4;
    console.log(
      `  Concurrent (4×) isVisible ×${SAMPLES}: ${total.toFixed(1)}ms total, ${perBatch.toFixed(3)}ms/batch, ${perAction.toFixed(3)}ms/action`,
    );
  });
});
