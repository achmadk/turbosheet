import * as vscode from "vscode";

export interface TestResult {
  name: string;
  file: string;
  status: "passed" | "failed" | "skipped";
  duration: number;
  error?: string;
  screenshots?: string[];
  traceData?: string;
}

type TestResultCallback = (result: TestResult) => void;

export class TurboSheetRunner {
  private disposables: vscode.Disposable[] = [];
  private resultCallbacks: TestResultCallback[] = [];
  private config: vscode.WorkspaceConfiguration;
  private decorationTypes: Map<string, vscode.TextEditorDecorationType> = new Map();

  constructor() {
    this.config = vscode.workspace.getConfiguration("tsheet");
  }

  updateConfig(): void {
    this.config = vscode.workspace.getConfiguration("tsheet");
  }

  onTestResult(callback: TestResultCallback): void {
    this.resultCallbacks.push(callback);
  }

  private emitResult(result: TestResult): void {
    for (const cb of this.resultCallbacks) {
      cb(result);
    }
  }

  async runTest(testPath: string): Promise<void> {
    const reporter = this.config.get<string>("reporter", "list");
    const headless = this.config.get<boolean>("headless", true);

    const args = ["test", testPath, "--reporter", reporter, headless ? "--headless" : ""].filter(
      Boolean,
    );

    await this.runCommand(args);
  }

  async runAllTests(): Promise<void> {
    const reporter = this.config.get<string>("reporter", "list");
    const workers = this.config.get<number>("workers", 1);
    const timeout = this.config.get<number>("timeout", 30000);

    const args = [
      "test",
      "--reporter",
      reporter,
      "--workers",
      workers.toString(),
      "--timeout",
      timeout.toString(),
    ];

    await this.runCommand(args);
  }

  async debugTest(testPath: string): Promise<void> {
    const debugConfig: vscode.DebugConfiguration = {
      type: "node",
      request: "launch",
      name: "TurboSheet Debug",
      runtimeExecutable: "npx",
      runtimeArgs: ["ts-node", testPath],
      cwd: vscode.workspace.rootPath,
      console: "integratedTerminal",
    };

    await vscode.debug.startDebugging(undefined, debugConfig);
  }

  private getCWD() {
    const activeEditor = vscode.window.activeTextEditor;

    if (activeEditor) {
      const workspaceFolder = vscode.workspace.getWorkspaceFolder(activeEditor.document.uri);
      if (workspaceFolder) {
        return workspaceFolder.uri.fsPath;
      }
    }

    // Fallback to the first workspace folder if no file is open
    return vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0
      ? vscode.workspace.workspaceFolders[0].uri.fsPath
      : undefined;
  }

  private async runCommand(args: string[]): Promise<void> {
    const cwd = this.getCWD();
    const terminal = vscode.window.createTerminal({
      name: "TurboSheet",
      cwd,
    });

    terminal.sendText(`npx tsheet ${args.join(" ")}'`);
    terminal.show();
  }

  applyDecorations(editor: vscode.TextEditor): void {
    const doc = editor.document;
    if (!doc.fileName.endsWith(".tsheet.ts")) return;

    const content = doc.getText();
    const decorations: vscode.DecorationOptions[] = [];

    const testRegex = /test\s*\(\s*['"`]([^'"`]+)['"`]/g;
    let match;

    while ((match = testRegex.exec(content)) !== null) {
      const pos = doc.positionAt(match.index);
      const endPos = doc.positionAt(match.index + match[0].length);

      const decoration = {
        range: new vscode.Range(pos, endPos),
        renderOptions: {
          before: {
            contentText: " ",
            backgroundColor: "rgba(102, 126, 234, 0.3)",
          },
        },
      };

      decorations.push(decoration);
    }

    const decorationType = vscode.window.createTextEditorDecorationType({
      before: {
        contentText: "T",
        backgroundColor: "#667eea",
        color: "white",
        fontWeight: "bold",
      },
    });

    editor.setDecorations(decorationType, decorations);
    this.decorationTypes.set(editor.document.fileName, decorationType);
  }

  clearDecorations(filePath: string): void {
    const decorationType = this.decorationTypes.get(filePath);
    if (decorationType) {
      decorationType.dispose();
      this.decorationTypes.delete(filePath);
    }
  }

  dispose(): void {
    for (const d of this.disposables) {
      d.dispose();
    }
    for (const decoration of this.decorationTypes.values()) {
      decoration.dispose();
    }
    this.decorationTypes.clear();
  }
}

export function parseTestResults(output: string): TestResult[] {
  const results: TestResult[] = [];

  const passMatch = output.matchAll(/✓\s+(.+?)\s+\((\d+\.?\d*)s\)/g);
  for (const m of passMatch) {
    results.push({
      name: m[1],
      file: "",
      status: "passed",
      duration: parseFloat(m[2]) * 1000,
    });
  }

  const failMatch = output.matchAll(/✗\s+(.+?)\s+\((\d+\.?\d*)s\)/g);
  for (const m of failMatch) {
    results.push({
      name: m[1],
      file: "",
      status: "failed",
      duration: parseFloat(m[2]) * 1000,
    });
  }

  return results;
}
