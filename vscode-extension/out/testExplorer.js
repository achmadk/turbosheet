"use strict";
var __createBinding =
  (this && this.__createBinding) ||
  (Object.create
    ? function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        var desc = Object.getOwnPropertyDescriptor(m, k);
        if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
          desc = {
            enumerable: true,
            get: function () {
              return m[k];
            },
          };
        }
        Object.defineProperty(o, k2, desc);
      }
    : function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        o[k2] = m[k];
      });
var __setModuleDefault =
  (this && this.__setModuleDefault) ||
  (Object.create
    ? function (o, v) {
        Object.defineProperty(o, "default", { enumerable: true, value: v });
      }
    : function (o, v) {
        o["default"] = v;
      });
var __importStar =
  (this && this.__importStar) ||
  (function () {
    var ownKeys = function (o) {
      ownKeys =
        Object.getOwnPropertyNames ||
        function (o) {
          var ar = [];
          for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
          return ar;
        };
      return ownKeys(o);
    };
    return function (mod) {
      if (mod && mod.__esModule) return mod;
      var result = {};
      if (mod != null)
        for (var k = ownKeys(mod), i = 0; i < k.length; i++)
          if (k[i] !== "default") __createBinding(result, mod, k[i]);
      __setModuleDefault(result, mod);
      return result;
    };
  })();
Object.defineProperty(exports, "__esModule", { value: true });
exports.TestExplorerProvider = exports.TestItemImpl = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
class TestItemImpl {
  constructor(id, label, uri) {
    this.id = id;
    this.label = label;
    this.uri = uri;
    this.children = new Map();
    this.status = "idle";
  }
}
exports.TestItemImpl = TestItemImpl;
class TestExplorerProvider {
  constructor(runner) {
    this.runner = runner;
    this._onDidChangeTreeData = new vscode.EventEmitter();
    this.onDidChangeTreeData = this._onDidChangeTreeData.event;
    this.tests = new Map();
    this.testResults = new Map();
    this.runner.onTestResult((result) => {
      this.updateTestResult(result);
    });
  }
  refresh() {
    this._onDidChangeTreeData.fire(undefined);
  }
  async discoverTests() {
    const config = vscode.workspace.getConfiguration("tsheet");
    const testPattern = config.get("testPattern", "**/*.tsheet.ts");
    const files = await vscode.workspace.findFiles(testPattern, "**/node_modules/**");
    this.tests.clear();
    for (const file of files) {
      await this.parseTestFile(file);
    }
    this.refresh();
    vscode.commands.executeCommand("setContext", "tsheet:hasTests", this.tests.size > 0);
  }
  async parseTestFile(uri) {
    const doc = await vscode.workspace.openTextDocument(uri);
    const content = doc.getText();
    const fileName = path.basename(uri.fsPath, ".tsheet.ts");
    const suiteItem = new TestItemImpl(`suite:${uri.fsPath}`, fileName, uri);
    const testRegex = /test\s*\(\s*['"`]([^'"`]+)['"`]/g;
    let match;
    while ((match = testRegex.exec(content)) !== null) {
      const testName = match[1];
      const testItem = new TestItemImpl(`test:${uri.fsPath}:${testName}`, testName, uri);
      testItem.parent = suiteItem;
      suiteItem.children.set(testItem.id, testItem);
      this.tests.set(testItem.id, testItem);
    }
    if (suiteItem.children.size > 0) {
      this.tests.set(suiteItem.id, suiteItem);
    }
  }
  updateTestResult(result) {
    const testItem = this.tests.get(`test:${result.file}:${result.name}`);
    if (testItem) {
      testItem.status = result.status === "passed" ? "passed" : "failed";
      testItem.error = result.error;
      testItem.duration = result.duration;
      this.testResults.set(testItem.id, result);
    }
    this.refresh();
  }
  getTreeItem(element) {
    const treeItem = new vscode.TreeItem(
      element.label,
      element.children.size > 0
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None,
    );
    treeItem.contextValue = element.id.startsWith("suite:") ? "suite" : "test";
    treeItem.resourceUri = element.uri;
    if (element.id.startsWith("suite:")) {
      const passedCount = Array.from(element.children.values()).filter(
        (c) => c.status === "passed",
      ).length;
      const totalCount = element.children.size;
      treeItem.description = `${passedCount}/${totalCount} passed`;
      treeItem.iconPath = new vscode.ThemeIcon("folder");
    } else {
      switch (element.status) {
        case "passed":
          treeItem.iconPath = new vscode.ThemeIcon(
            "check",
            new vscode.ThemeColor("testing.iconPassed"),
          );
          break;
        case "failed":
          treeItem.iconPath = new vscode.ThemeIcon(
            "x",
            new vscode.ThemeColor("testing.iconFailed"),
          );
          break;
        case "running":
          treeItem.iconPath = new vscode.ThemeIcon("loading~spin");
          break;
        default:
          treeItem.iconPath = new vscode.ThemeIcon("circle-outline");
      }
      treeItem.description = element.duration ? `${(element.duration / 1000).toFixed(2)}s` : "";
      treeItem.tooltip = element.error || `${element.status}`;
    }
    return treeItem;
  }
  getChildren(element) {
    if (!element) {
      return Array.from(this.tests.values()).filter((t) => !t.parent);
    }
    return Array.from(element.children.values());
  }
  getParent(element) {
    return element.parent;
  }
}
exports.TestExplorerProvider = TestExplorerProvider;
//# sourceMappingURL=testExplorer.js.map
