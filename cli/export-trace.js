#!/usr/bin/env node
const { readFileSync, writeFileSync, existsSync } = require("fs");

const { trace_load_and_deserialize } = require("..");

function usage() {
  console.log("Usage: tsheet export-trace <trace-file> [output-file.json]");
  console.log("");
  console.log("Exports a TurboTrace (.ttrc) file into readable JSON format");
}

async function exportTrace(args) {
  let traceFile = null;
  let outputFile = null;

  for (let i = 0; i < args.length; i++) {
    if (!args[i].startsWith("--")) {
      if (!traceFile) {
        traceFile = args[i];
      } else if (!outputFile) {
        outputFile = args[i];
      }
    }
  }

  if (!traceFile) {
    usage();
    process.exit(1);
  }

  if (!outputFile) {
    outputFile = traceFile + ".json";
  }

  if (!existsSync(traceFile)) {
    console.error(`Error: File not found: ${traceFile}`);
    process.exit(1);
  }

  const traceData = readFileSync(traceFile);

  try {
    const traceJson = trace_load_and_deserialize(traceData);
    writeFileSync(outputFile, traceJson);
    console.log(`✅ Successfully exported trace to ${outputFile}`);
  } catch (e) {
    console.error("❌ Error exporting trace:", e.message);
    process.exit(1);
  }
}

module.exports = { exportTrace };

if (require.main === module) {
  exportTrace(process.argv.slice(2)).catch((e) => {
    console.error("Error:", e.message);
    process.exit(1);
  });
}
