const ts = require("typescript");

function applyEdits(code, edits) {
  // Sort edits by start position descending to avoid invalidating offsets
  edits.sort((a, b) => b.start - a.start);
  let res = code;
  for (const edit of edits) {
    res = res.substring(0, edit.start) + edit.text + res.substring(edit.end);
  }
  return res;
}

function transformPlaywright(code, fileName) {
  const sourceFile = ts.createSourceFile(fileName, code, ts.ScriptTarget.Latest, true);
  const edits = [];

  function visit(node) {
    // 1. Rewrite imports
    // import { test, expect } from '@playwright/test';
    if (ts.isImportDeclaration(node)) {
      if (node.moduleSpecifier.text === "@playwright/test") {
        const start = node.moduleSpecifier.getStart(sourceFile);
        const end = node.moduleSpecifier.getEnd();
        edits.push({ start, end, text: "'tsheet'" });
      }
    }

    // 2. Rewrite page.$eval('selector', ...) -> page.evaluate('selector', ...)
    if (ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression)) {
      const propAccess = node.expression;
      if (propAccess.name.text === "$eval" || propAccess.name.text === "$$eval") {
        const start = propAccess.name.getStart(sourceFile);
        const end = propAccess.name.getEnd();
        edits.push({ start, end, text: "evaluate" });
      }

      // 3. page.waitForSelector('...') -> page.locator('...').waitFor()
      if (propAccess.name.text === "waitForSelector") {
        const start = node.getStart(sourceFile);
        const end = node.getEnd();

        const args = node.arguments;
        if (args.length > 0) {
          const selector = args[0].getText(sourceFile);
          const obj = propAccess.expression.getText(sourceFile); // e.g. "page"
          edits.push({
            start,
            end,
            text: `${obj}.locator(${selector}).waitFor() /* TODO: verify auto-wait */`,
          });
        }
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return applyEdits(code, edits);
}

function transformCypress(code, fileName) {
  const sourceFile = ts.createSourceFile(fileName, code, ts.ScriptTarget.Latest, true);
  const edits = [];

  function visit(node) {
    // Cypress migrations: cy.visit(url) -> await page.goto(url)
    // cy.get(selector).click() -> await page.locator(selector).click()

    if (ts.isCallExpression(node)) {
      const expr = node.expression;

      // Handle cy.visit('url')
      if (ts.isPropertyAccessExpression(expr) && expr.expression.getText(sourceFile) === "cy") {
        if (expr.name.text === "visit") {
          const start = node.getStart(sourceFile);
          const end = node.getEnd();
          const args = node.arguments.map((a) => a.getText(sourceFile)).join(", ");
          edits.push({ start, end, text: `await page.goto(${args})` });
        }
      }

      // Handle chained cy.get(selector).action()
      if (ts.isPropertyAccessExpression(expr)) {
        const innerCall = expr.expression;
        if (ts.isCallExpression(innerCall) && ts.isPropertyAccessExpression(innerCall.expression)) {
          const innerPropAccess = innerCall.expression;
          if (
            innerPropAccess.expression.getText(sourceFile) === "cy" &&
            innerPropAccess.name.text === "get"
          ) {
            const selector = innerCall.arguments[0]?.getText(sourceFile) || "";
            const action = expr.name.text;
            const actionArgs = node.arguments.map((a) => a.getText(sourceFile)).join(", ");

            const start = node.getStart(sourceFile);
            const end = node.getEnd();

            edits.push({
              start,
              end,
              text: `await page.locator(${selector}).${action}(${actionArgs})`,
            });
          }
        }
      }
    }

    // Rewrite global describe/it to import { test } from 'tsheet'
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      if (node.expression.text === "it") {
        const start = node.expression.getStart(sourceFile);
        const end = node.expression.getEnd();
        edits.push({ start, end, text: "test" });

        // Ensure function is async
        const args = node.arguments;
        if (args.length > 1) {
          const fnNode = args[1];
          if (ts.isArrowFunction(fnNode) || ts.isFunctionExpression(fnNode)) {
            // Need to insert async if not present
            const isAsync = fnNode.modifiers?.some((m) => m.kind === ts.SyntaxKind.AsyncKeyword);
            if (!isAsync) {
              edits.push({
                start: fnNode.getStart(sourceFile),
                end: fnNode.getStart(sourceFile),
                text: "async ",
              });

              // And inject { page } argument
              if (fnNode.parameters.length === 0) {
                const openParen = fnNode
                  .getChildren(sourceFile)
                  .find((c) => c.kind === ts.SyntaxKind.OpenParenToken);
                if (openParen) {
                  edits.push({
                    start: openParen.getEnd(),
                    end: openParen.getEnd(),
                    text: "{ page }",
                  });
                }
              }
            }
          }
        }
      }

      if (node.expression.text === "describe") {
        const start = node.expression.getStart(sourceFile);
        const end = node.expression.getEnd();
        edits.push({ start, end, text: "test.describe" });
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);

  let newCode = applyEdits(code, edits);

  // Add tsheet imports at the top if we modified 'it'/'describe' and they aren't imported
  if (edits.some((e) => e.text === "test" || e.text === "test.describe")) {
    if (!newCode.includes("from 'tsheet'")) {
      newCode = `import { test, expect } from 'tsheet';\n` + newCode;
    }
  }

  return newCode;
}

module.exports = {
  transformPlaywright,
  transformCypress,
};
