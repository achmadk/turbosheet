const { mount, detectFrameworkFromSource } = require("../index.js");

let passed = 0;
let failed = 0;

function assert(condition, msg) {
  if (condition) {
    console.log("  \u2713", msg);
    passed++;
  } else {
    console.log("  \u2717", msg);
    failed++;
  }
}

async function run() {
  console.log("=== Section 10: Component Testing ===\n");

  // ------- 10.1-10.4: Component Mounting --------------------------
  console.log("--- 10.1-10.4: Component Mounting ---");

  const reactComp = await mount('<button id="btn">Click Me</button>', {
    framework: "react",
    timeout: 3000,
  });
  assert(reactComp.component_id, "React component mounted with ID");
  assert(reactComp.framework === "react", "Framework is react");
  assert(reactComp.page_id, "Page ID present");
  assert(reactComp.html.includes("react"), "HTML includes React scripts");

  const btnText = await reactComp.evaluate(
    'document.getElementById("btn")?.innerText || "NOT_FOUND"',
  );
  assert(btnText.includes("Click Me"), 'Evaluate: button text is "Click Me"');

  try {
    await reactComp.click("#btn");
    assert(true, "Click succeeded");
  } catch (e) {
    assert(false, "Click failed: " + e.message);
  }

  try {
    const screenshot = await reactComp.screenshot();
    assert(screenshot.length > 100, "Screenshot taken, size: " + screenshot.length + " bytes");
  } catch (e) {
    assert(false, "Screenshot failed: " + e.message);
  }

  try {
    const content = await reactComp.content();
    assert(content.includes("Click Me"), "Content includes rendered component");
  } catch (e) {
    assert(false, "Content failed: " + e.message);
  }

  // ------- 10.5: Component Locator API ----------------------------
  console.log("\n--- 10.5: Component Locator API ---");

  const locator = reactComp.locator("#btn");
  assert(!!locator, "locator() returns a locator object");

  try {
    const text = await locator.textContent();
    assert(text.includes("Click Me"), "locator.textContent() returns button text");
  } catch (e) {
    assert(false, "locator.textContent() failed: " + e.message);
  }

  try {
    const visible = await locator.isVisible();
    assert(visible === true, "locator.isVisible() returns true");
  } catch (e) {
    assert(false, "locator.isVisible() failed: " + e.message);
  }

  try {
    const enabled = await locator.isEnabled();
    assert(enabled === true, "locator.isEnabled() returns true");
  } catch (e) {
    assert(false, "locator.isEnabled() failed: " + e.message);
  }

  try {
    const innerHtml = await locator.innerHtml();
    assert(innerHtml.includes("Click Me"), "locator.innerHtml() works");
  } catch (e) {
    assert(false, "locator.innerHtml() failed: " + e.message);
  }

  try {
    await locator.getAttribute("type");
    assert(true, "locator.getAttribute() works");
  } catch (e) {
    assert(false, "locator.getAttribute() failed: " + e.message);
  }

  try {
    const style = await locator.computedStyle("color");
    assert(typeof style === "string", "locator.computedStyle() works");
  } catch (e) {
    assert(false, "locator.computedStyle() failed: " + e.message);
  }

  // ------- 10.6: Props and State Access ---------------------------
  console.log("\n--- 10.6: Props and State Access ---");

  try {
    const props = await reactComp.getProps();
    assert(typeof props === "object", "getProps() returns an object");
  } catch (e) {
    assert(false, "getProps() failed: " + e.message);
  }

  try {
    const state = await reactComp.getState();
    assert(typeof state === "object", "getState() returns an object");
  } catch (e) {
    assert(false, "getState() failed: " + e.message);
  }

  // ------- 10.7: Framework Detection from Source Imports ----------
  console.log("\n--- 10.7: Framework Detection ---");

  assert(
    detectFrameworkFromSource("import React from 'react'") === "react",
    "Detect React from import",
  );
  assert(
    detectFrameworkFromSource('import { ref } from "vue"') === "vue",
    "Detect Vue from import",
  );
  assert(
    detectFrameworkFromSource("import { onMount } from 'svelte'") === "svelte",
    "Detect Svelte from import",
  );
  assert(
    detectFrameworkFromSource('console.log("no framework")') === "unknown",
    "Unknown framework returns unknown",
  );

  // ------- 10.8: Component Cleanup --------------------------------
  console.log("\n--- 10.8: Component Cleanup ---");

  try {
    await reactComp.close();
    assert(reactComp._closed, "Component marked as closed");
    console.log("  \u2713 Component cleanup succeeded");
    passed++;
  } catch (e) {
    assert(false, "Close failed: " + e.message);
  }

  try {
    await reactComp.evaluate("1+1");
    assert(false, "Should reject evaluate on closed component");
  } catch (e) {
    assert(e.message.includes("closed"), "Rejects evaluate on closed component");
    passed++;
  }

  // ------- 10.9: Configuration Options ----------------------------
  console.log("\n--- 10.9: Configuration Options ---");

  try {
    const compWithWrapper = await mount("<span>Wrapped</span>", {
      framework: "react",
      wrapper: "section",
      timeout: 3000,
    });
    assert(compWithWrapper.component_id, "Mount with custom wrapper works");
    await compWithWrapper.close();
    passed++;
  } catch (e) {
    assert(false, "Custom wrapper mount failed: " + e.message);
  }

  console.log("\n--- Vue Component Mount ---");
  try {
    const vueComp = await mount("<div>{{ message }}</div>", { framework: "vue", timeout: 3000 });
    assert(vueComp.component_id, "Vue component mounted");
    await vueComp.close();
    passed++;
  } catch (e) {
    assert(false, "Vue component mount failed: " + e.message);
  }

  // ------- Summary ------------------------------------------------
  console.log("\n=== Component Testing Complete ===");
  console.log("Passed: " + passed + ", Failed: " + failed);
  process.exit(failed > 0 ? 1 : 0);
}

run().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
