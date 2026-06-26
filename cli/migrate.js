#!/usr/bin/env node
const { statSync, readFileSync, writeFileSync } = require("fs");
const { join } = require("path");

// We'll put AST transformation logic in another file
const { transformPlaywright, transformCypress } = require("./ast-transformer");

function usage() {
  console.log("Usage: tsheet migrate <framework> [options]");
  console.log("");
  console.log("Migrate existing tests to TurboSheet format.");
  console.log("");
  console.log("Frameworks:");
  console.log("  playwright   Migrate Playwright tests (.spec.ts)");
  console.log("  cypress      Migrate Cypress tests (.cy.js/.cy.ts)");
  console.log("");
  console.log("Options:");
  console.log("  --dry-run    Run migration without modifying files and print changes to stdout");
  console.log("  --dir <dir>  Directory to search for tests (default: .)");
}

function findFiles(dir, matchPattern) {
  let results = [];
  const files = require("fs").readdirSync(dir);

  for (const file of files) {
    if (file === "node_modules" || file === "dist" || file.startsWith(".")) continue;

    const fullPath = join(dir, file);
    if (statSync(fullPath).isDirectory()) {
      results = results.concat(findFiles(fullPath, matchPattern));
    } else if (matchPattern.test(file)) {
      results.push(fullPath);
    }
  }

  return results;
}

async function runMigrate(args) {
  const framework = args[0];
  if (!framework || framework.startsWith("--")) {
    usage();
    process.exit(1);
  }

  if (framework !== "playwright" && framework !== "cypress") {
    console.error(
      `Error: Unsupported framework '${framework}'. Only 'playwright' and 'cypress' are supported.`,
    );
    process.exit(1);
  }

  let isDryRun = false;
  let targetDir = ".";

  for (let i = 1; i < args.length; i++) {
    if (args[i] === "--dry-run") {
      isDryRun = true;
    } else if (args[i] === "--dir" && i + 1 < args.length) {
      targetDir = args[++i];
    }
  }

  const pattern = framework === "playwright" ? /\.spec\.(ts|js)$/ : /\.cy\.(ts|js)$/;
  console.log(`🔍 Searching for ${framework} tests in ${targetDir}...`);
  const files = findFiles(targetDir, pattern);

  if (files.length === 0) {
    console.log(`No files found matching ${pattern}.`);
    return;
  }

  console.log(`Found ${files.length} file(s) to migrate.`);

  let successCount = 0;
  let skipCount = 0;
  let errorCount = 0;

  for (const file of files) {
    console.log(`\n📄 Processing: ${file}`);
    try {
      const code = readFileSync(file, "utf8");

      let newCode;
      if (framework === "playwright") {
        newCode = transformPlaywright(code, file);
      } else {
        newCode = transformCypress(code, file);
      }

      if (code === newCode) {
        console.log("   ⏭️ No changes needed.");
        skipCount++;
        continue;
      }

      if (isDryRun) {
        console.log("   👀 [DRY RUN] Would write:");
        console.log("--- BEGIN ---");
        console.log(newCode);
        console.log("--- END ---");
      } else {
        writeFileSync(file, newCode, "utf8");
        console.log("   ✅ Migrated successfully.");
      }
      successCount++;
    } catch (e) {
      console.error(`   ❌ Error migrating ${file}:`, e.message);
      errorCount++;
    }
  }

  console.log("\n📊 Migration Summary:");
  console.log(`  Processed: ${files.length}`);
  console.log(`  Migrated:  ${successCount}`);
  console.log(`  Skipped:   ${skipCount}`);
  console.log(`  Errors:    ${errorCount}`);

  if (isDryRun) {
    console.log("\n⚠️ This was a dry run. No files were modified.");
  } else {
    console.log(
      "\n✨ Migration complete! Please review the changes manually and fix any // TODO: migration warnings.",
    );
  }
}

module.exports = { runMigrate };
