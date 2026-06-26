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
exports.TurboSheetRunner = void 0;
exports.parseTestResults = parseTestResults;
const vscode = __importStar(require("vscode"));
class TurboSheetRunner {
  constructor() {
    this.disposables = [];
    this.resultCallbacks = [];
    this.decorationTypes = new Map();
    this.config = vscode.workspace.getConfiguration("tsheet");
  }
  updateConfig() {
    this.config = vscode.workspace.getConfiguration("tsheet");
  }
  onTestResult(callback) {
    this.resultCallbacks.push(callback);
  }
  emitResult(result) {
    for (const cb of this.resultCallbacks) {
      cb(result);
    }
  }
  async runTest(testPath) {
    const reporter = this.config.get("reporter", "list");
    const headless = this.config.get("headless", true);
    const args = ["test", testPath, "--reporter", reporter, headless ? "--headless" : ""].filter(
      Boolean,
    );
    await this.runCommand(args);
  }
  async runAllTests() {
    const reporter = this.config.get("reporter", "list");
    const workers = this.config.get("workers", 1);
    const timeout = this.config.get("timeout", 30000);
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
  async debugTest(testPath) {
    const debugConfig = {
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
  async runCommand(args) {
    const terminal = vscode.window.createTerminal({
      name: "TurboSheet",
      cwd: vscode.workspace.rootPath,
    });
    terminal.sendText(`npx tsheet ${args.join(" ")}'`);
    terminal.show();
  }
  applyDecorations(editor) {
    const doc = editor.document;
    if (!doc.fileName.endsWith(".tsheet.ts")) return;
    const content = doc.getText();
    const decorations = [];
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
  clearDecorations(filePath) {
    const decorationType = this.decorationTypes.get(filePath);
    if (decorationType) {
      decorationType.dispose();
      this.decorationTypes.delete(filePath);
    }
  }
  dispose() {
    for (const d of this.disposables) {
      d.dispose();
    }
    for (const decoration of this.decorationTypes.values()) {
      decoration.dispose();
    }
    this.decorationTypes.clear();
  }
}
exports.TurboSheetRunner = TurboSheetRunner;
function parseTestResults(output) {
  const results = [];
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
//# sourceMappingURL=runner.js.map
