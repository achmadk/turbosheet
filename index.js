const { existsSync, readFileSync, rmSync, mkdtempSync } = require("fs");
const { join } = require("path");
const os = require("os");

const { platform, arch } = process;

let nativeBinding = null;
let loadError = null;

function loadNative() {
  const triples = [
    // platform-specific triples in priority order
    ["linux", "x64", "linux-x64-gnu"],
    ["linux", "arm64", "linux-arm64-gnu"],
    ["darwin", "x64", "darwin-x64"],
    ["darwin", "arm64", "darwin-arm64"],
    ["win32", "x64", "win32-x64-msvc"],
    ["win32", "arm64", "win32-arm64-msvc"],
  ];

  for (const [plat, arc, triple] of triples) {
    if (plat === platform && arc === arch) {
      const bindingPath = join(__dirname, `turbosheet.${triple}.node`);
      if (existsSync(bindingPath)) {
        try {
          nativeBinding = require(bindingPath);
          return;
        } catch (e) {
          loadError = e;
        }
      }
    }
  }

  // Fallback: try requiring the binary directly
  try {
    nativeBinding = require("./turbosheet.node");
  } catch (e) {
    loadError = loadError || e;
  }
}

loadNative();

if (!nativeBinding) {
  throw new Error(
    `TurboSheet: Failed to load native binding for ${platform}-${arch}. ` +
      `Run 'npm run build' to compile from source, or install a prebuilt binary.` +
      (loadError ? `\nCaused by: ${loadError.message}` : ""),
  );
}

// On Linux, Chrome requires --no-sandbox in container/CI environments
// where kernel sandboxing is unavailable. Inject it by default to avoid
// websocket timeout errors caused by sandbox initialization failure.
const defaultArgs = process.platform === "linux" ? ["--no-sandbox"] : [];

async function launchWithDefaults(options = {}) {
  // Clean up stale Chrome SingletonLock from previous runs.
  // Chromiumoxide's default profile dir is /tmp/chromiumoxide-runner/.
  if (process.platform === "linux") {
    try {
      rmSync("/tmp/chromiumoxide-runner/SingletonLock", { force: true });
    } catch {}
  }

  // Use a unique temp user-data-dir per launch to avoid profile conflicts.
  const tmpDir = mkdtempSync(join(os.tmpdir(), "tsheet-"));
  const userDataArg = `--user-data-dir=${tmpDir}`;

  const merged = {
    ...options,
    args: [...defaultArgs, userDataArg, ...(options.args || [])],
  };
  try {
    return await nativeBinding.launch(merged);
  } catch (err) {
    // Cleanup temp dir on launch failure
    try {
      rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
    throw err;
  }
}

const {
  launch: _nativeLaunch,
  devices,
  version,
  traceStartRecording,
  traceStopAndSerialize,
  traceLoadAndDeserialize,
  traceGenerateViewerHtml,
  traceWriteViewerToFile,
  traceEventsToJson,
  visualCompareScreenshots,
  visualSaveSnapshot,
  visualGenerateDiffImage,
} = nativeBinding;

const launch = launchWithDefaults;

const COMPONENT_TIMEOUT = 5000;

class ComponentLocator {
  constructor(pageId, selector) {
    this._pageId = pageId;
    this._selector = selector;
    this._inner = nativeBinding.createLocator(pageId, selector);
  }

  // ── Locator chaining ──────────────────────────────────────────

  locator(selector) {
    return new ComponentLocator(this._pageId, `${this._selector} >> ${selector}`);
  }

  // ── Playwright getBy* helpers ─────────────────────────────────
  // Rust get_by_* return JsLocator objects — extract .selector.

  getByRole(role, options) {
    const name = (options && options.name) || undefined;
    return new ComponentLocator(this._pageId, this._inner.getByRole(role, name).selector);
  }

  getByText(text, options) {
    const exact = (options && options.exact) || undefined;
    return new ComponentLocator(this._pageId, this._inner.getByText(text, exact).selector);
  }

  getByLabel(label, options) {
    const exact = (options && options.exact) || undefined;
    return new ComponentLocator(this._pageId, this._inner.getByLabel(label, exact).selector);
  }

  getByPlaceholder(placeholder, options) {
    const exact = (options && options.exact) || undefined;
    return new ComponentLocator(
      this._pageId,
      this._inner.getByPlaceholder(placeholder, exact).selector,
    );
  }

  getByAltText(altText, options) {
    const exact = (options && options.exact) || undefined;
    return new ComponentLocator(this._pageId, this._inner.getByAltText(altText, exact).selector);
  }

  getByTitle(title, options) {
    const exact = (options && options.exact) || undefined;
    return new ComponentLocator(this._pageId, this._inner.getByTitle(title, exact).selector);
  }

  getByTestId(testId) {
    return new ComponentLocator(this._pageId, this._inner.getByTestId(testId).selector);
  }

  // ── Filter / index ────────────────────────────────────────────
  // Rust filter returns a JsLocator — extract .selector.

  filter(options) {
    return new ComponentLocator(this._pageId, this._inner.filter(options || {}).selector);
  }

  first() {
    return new ComponentLocator(this._pageId, this._inner.first().selector);
  }

  last() {
    return new ComponentLocator(this._pageId, this._inner.last().selector);
  }

  nth(index) {
    return new ComponentLocator(this._pageId, this._inner.nth(index).selector);
  }

  // ── Actions (via existing napi component helpers) ─────────────

  async click() {
    return nativeBinding.componentClick(this._pageId, this._selector);
  }

  async fill(value) {
    return nativeBinding.componentFill(this._pageId, this._selector, value);
  }

  // ── Queries via Rust locator ──────────────────────────────────

  async textContent() {
    return this._inner.textContent();
  }

  async innerText() {
    return this._inner.innerText();
  }

  async innerHtml() {
    return this._inner.innerHTML();
  }

  async getAttribute(name) {
    return this._inner.getAttribute(name);
  }

  async isDisabled() {
    return this._inner.isDisabled();
  }

  async isEnabled() {
    return this._inner.isEnabled();
  }

  async isVisible() {
    return this._inner.isVisible();
  }

  async count() {
    return this._inner.count();
  }

  async evaluate(js) {
    return this._inner.evaluate(js);
  }

  // ── Misc (uses component napi helpers directly) ───────────────

  async computedStyle(property) {
    return nativeBinding.componentGetComputedStyle(this._pageId, this._selector, property);
  }

  // ── Wait ──────────────────────────────────────────────────────

  async waitFor(options) {
    return this._inner.waitFor(options || {});
  }

  async wait(options) {
    return this.waitFor(options);
  }
}

class Component {
  constructor(mountedData) {
    this.html = mountedData.html;
    this.framework = mountedData.framework;
    this.component_id = mountedData.componentId || mountedData.component_id;
    this.page_id = mountedData.pageId || mountedData.page_id;
    this.root_html = mountedData.rootHtml || mountedData.root_html;
    this._closed = false;
  }

  locator(selector) {
    if (this._closed) throw new Error("Component is closed");
    return new ComponentLocator(this.page_id, selector);
  }

  async click(selector) {
    if (this._closed) throw new Error("Component is closed");
    return nativeBinding.componentClick(this.page_id, selector);
  }

  async fill(selector, value) {
    if (this._closed) throw new Error("Component is closed");
    return nativeBinding.componentFill(this.page_id, selector, value);
  }

  async evaluate(js) {
    if (this._closed) throw new Error("Component is closed");
    return nativeBinding.componentEvaluate(this.page_id, js);
  }

  async screenshot() {
    if (this._closed) throw new Error("Component is closed");
    return nativeBinding.componentScreenshot(this.page_id);
  }

  async content() {
    if (this._closed) throw new Error("Component is closed");
    return nativeBinding.componentGetContent(this.page_id);
  }

  async close() {
    if (this._closed) return;
    this._closed = true;
    try {
      await nativeBinding.componentClose(this.page_id);
    } catch {
      // Page might already be gone - ignore close errors
    }
    try {
      await nativeBinding.componentMountClose(this.page_id, this.framework);
    } catch {
      // Pool release is best-effort
    }
  }

  async getProps() {
    const raw = await this.evaluate("window.__COMPONENT_PROPS__ || null");
    try {
      return JSON.parse(raw);
    } catch {
      return {};
    }
  }

  async getState() {
    const raw = await this.evaluate("window.__COMPONENT_STATE__ || null");
    try {
      return JSON.parse(raw);
    } catch {
      return {};
    }
  }

  async waitFor(jsCondition, options) {
    const timeout = (options && options.timeout) || COMPONENT_TIMEOUT;
    return nativeBinding.componentWaitFor(this.page_id, jsCondition, timeout);
  }
}

async function mount(component, options = {}) {
  let framework = options.framework;
  if (!framework || framework === "auto") {
    try {
      const pkg = readFileSync(join(process.cwd(), "package.json"), "utf-8");
      framework = nativeBinding.detectFrameworkFromPackageJson(pkg);
      if (framework === "unknown") framework = "auto";
    } catch {
      framework = "auto";
    }
    options.framework = framework;
  }

  const result = await nativeBinding.mountComponent(component, options);
  return new Component(result);
}

module.exports = {
  launch,
  devices,
  version,
  trace_start_recording: traceStartRecording,
  trace_stop_and_serialize: traceStopAndSerialize,
  trace_load_and_deserialize: traceLoadAndDeserialize,
  trace_generate_viewer_html: traceGenerateViewerHtml,
  trace_write_viewer_to_file: traceWriteViewerToFile,
  trace_events_to_json: traceEventsToJson,
  visual_compare_screenshots: visualCompareScreenshots,
  visual_save_snapshot: visualSaveSnapshot,
  visual_generate_diff_image: visualGenerateDiffImage,
  mount,
  Component,
  ComponentLocator,
  detectFrameworkFromPackageJson: nativeBinding.detectFrameworkFromPackageJson,
  detectFrameworkFromSource: nativeBinding.detectFrameworkFromSource,
  run_tests: nativeBinding.runTests,
  test: nativeBinding.test,
  expect: nativeBinding.expect,
  describe: nativeBinding.describe,
  beforeAll: nativeBinding.beforeAll,
  afterAll: nativeBinding.afterAll,
  beforeEach: nativeBinding.beforeEach,
  afterEach: nativeBinding.afterEach,
};
module.exports.default = module.exports;
