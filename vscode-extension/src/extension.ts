import * as vscode from "vscode";
import { TestExplorerProvider } from "./testExplorer";
import { TurboSheetRunner } from "./runner";
import { TraceViewerPanel } from "./traceViewer";
import { disposeAll } from "./utils";

let testExplorer: TestExplorerProvider | undefined;
let runner: TurboSheetRunner | undefined;

export async function activate(context: vscode.ExtensionContext) {
  console.log("TurboSheet extension activating...");

  runner = new TurboSheetRunner();
  testExplorer = new TestExplorerProvider(runner);

  vscode.window.registerTreeDataProvider("tsheet.testExplorer", testExplorer);

  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.discoverTests", async () => {
      await testExplorer?.discoverTests();
      vscode.window.showInformationMessage("TurboSheet: Tests discovered");
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.runTest", async (resource: vscode.Uri) => {
      const testId = resource?.fsPath || vscode.window.activeTextEditor?.document.fileName;
      if (testId) {
        await runner?.runTest(testId);
      }
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.runAllTests", async () => {
      await runner?.runAllTests();
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.debugTest", async (resource: vscode.Uri) => {
      const testId = resource?.fsPath || vscode.window.activeTextEditor?.document.fileName;
      if (testId) {
        await runner?.debugTest(testId);
      }
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.showTrace", async (tracePath: string) => {
      if (!tracePath) {
        const selected = await vscode.window.showOpenDialog({
          filters: { "Trace Files": ["tsheet-trace", "trace"] },
        });
        if (selected && selected.length > 0) {
          tracePath = selected[0].fsPath;
        }
      }
      if (tracePath) {
        TraceViewerPanel.createOrShow(context.extensionUri, tracePath);
      }
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.viewSettings", () => {
      vscode.commands.executeCommand("workbench.action.openSettings", "tsheet");
    }),
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("tsheet")) {
        runner?.updateConfig();
      }
    }),
  );

  await testExplorer.discoverTests();

  console.log("TurboSheet extension activated");
}

export function deactivate() {
  disposeAll();
  runner?.dispose();
}
