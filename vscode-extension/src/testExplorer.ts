import * as vscode from "vscode";
import * as path from "path";
import { TurboSheetRunner, TestResult } from "./runner";

export interface TestItem {
  id: string;
  label: string;
  uri: vscode.Uri;
  parent?: TestItem;
  children: Map<string, TestItem>;
  status: "idle" | "running" | "passed" | "failed";
  error?: string;
  duration?: number;
}

export class TestItemImpl implements TestItem {
  id: string;
  label: string;
  uri: vscode.Uri;
  parent?: TestItem;
  children: Map<string, TestItem>;
  status: "idle" | "running" | "passed" | "failed";
  error?: string;
  duration?: number;

  constructor(id: string, label: string, uri: vscode.Uri) {
    this.id = id;
    this.label = label;
    this.uri = uri;
    this.children = new Map();
    this.status = "idle";
  }
}

export class TestExplorerProvider implements vscode.TreeDataProvider<TestItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<TestItem | undefined | null>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private tests: Map<string, TestItem> = new Map();
  private testResults: Map<string, TestResult> = new Map();

  constructor(private runner: TurboSheetRunner) {
    this.runner.onTestResult((result) => {
      this.updateTestResult(result);
    });
  }

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  async discoverTests(): Promise<void> {
    const config = vscode.workspace.getConfiguration("tsheet");
    const testPattern = config.get<string>("testPattern", "**/*.tsheet.ts");

    const files = await vscode.workspace.findFiles(testPattern, "**/node_modules/**");

    this.tests.clear();

    for (const file of files) {
      await this.parseTestFile(file);
    }

    this.refresh();
    vscode.commands.executeCommand("setContext", "tsheet:hasTests", this.tests.size > 0);
  }

  private async parseTestFile(uri: vscode.Uri): Promise<void> {
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

  private updateTestResult(result: TestResult): void {
    const testItem = this.tests.get(`test:${result.file}:${result.name}`);
    if (testItem) {
      testItem.status = result.status === "passed" ? "passed" : "failed";
      testItem.error = result.error;
      testItem.duration = result.duration;
      this.testResults.set(testItem.id, result);
    }
    this.refresh();
  }

  getTreeItem(element: TestItem): vscode.TreeItem {
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

  getChildren(element?: TestItem): TestItem[] {
    if (!element) {
      return Array.from(this.tests.values()).filter((t) => !t.parent);
    }
    return Array.from(element.children.values());
  }

  getParent(element: TestItem): TestItem | undefined {
    return element.parent;
  }
}
