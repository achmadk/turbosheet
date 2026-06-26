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
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const testExplorer_1 = require("./testExplorer");
const runner_1 = require("./runner");
const traceViewer_1 = require("./traceViewer");
const utils_1 = require("./utils");
let testExplorer;
let runner;
async function activate(context) {
  console.log("TurboSheet extension activating...");
  runner = new runner_1.TurboSheetRunner();
  testExplorer = new testExplorer_1.TestExplorerProvider(runner);
  vscode.window.registerTreeDataProvider("tsheet.testExplorer", testExplorer);
  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.discoverTests", async () => {
      await testExplorer?.discoverTests();
      vscode.window.showInformationMessage("TurboSheet: Tests discovered");
    }),
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.runTest", async (resource) => {
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
    vscode.commands.registerCommand("tsheet.debugTest", async (resource) => {
      const testId = resource?.fsPath || vscode.window.activeTextEditor?.document.fileName;
      if (testId) {
        await runner?.debugTest(testId);
      }
    }),
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("tsheet.showTrace", async (tracePath) => {
      if (!tracePath) {
        const selected = await vscode.window.showOpenDialog({
          filters: { "Trace Files": ["tsheet-trace", "trace"] },
        });
        if (selected && selected.length > 0) {
          tracePath = selected[0].fsPath;
        }
      }
      if (tracePath) {
        traceViewer_1.TraceViewerPanel.createOrShow(context.extensionUri, tracePath);
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
function deactivate() {
  (0, utils_1.disposeAll)();
  runner?.dispose();
}
//# sourceMappingURL=extension.js.map
