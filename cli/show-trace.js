#!/usr/bin/env node
const { readFileSync, existsSync } = require("fs");
const { createServer } = require("http");
const { join, extname } = require("path");
const { statSync } = require("fs");

const { trace_load_and_deserialize } = require("..");

function usage() {
  console.log("Usage: tsheet show-trace <trace-file>");
  console.log("");
  console.log("Open a TurboTrace viewer for a .ttrc or .tsheet-trace file");
  console.log("");
  console.log("Options:");
  console.log("  --port <port>  Port to serve on (default: 3000)");
}

const MIME_TYPES = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".svg": "image/svg+xml",
};

async function showTrace(args) {
  let port = 3000;
  let traceFile = null;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--port" && i + 1 < args.length) {
      port = parseInt(args[++i], 10);
    } else if (!args[i].startsWith("--")) {
      traceFile = args[i];
    }
  }

  if (!traceFile) {
    usage();
    process.exit(1);
  }

  if (!existsSync(traceFile)) {
    console.error(`Error: File not found: ${traceFile}`);
    process.exit(1);
  }

  const traceData = readFileSync(traceFile);

  let traceJson = null;
  try {
    traceJson = trace_load_and_deserialize(traceData);
  } catch (e) {
    console.error("Error deserializing trace:", e.message);
    process.exit(1);
  }

  const viewerDir = join(__dirname, "..", "packages", "trace-viewer", "dist");

  if (!existsSync(viewerDir)) {
    console.error("Trace viewer is not built. Please run `pnpm build` in packages/trace-viewer.");
    process.exit(1);
  }

  const server = createServer((req, res) => {
    if (req.url === "/api/trace") {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(traceJson);
      return;
    }

    let filePath = join(viewerDir, req.url === "/" ? "index.html" : req.url);
    if (!existsSync(filePath) || statSync(filePath).isDirectory()) {
      filePath = join(viewerDir, "index.html");
    }

    const ext = extname(filePath);
    const contentType = MIME_TYPES[ext] || "application/octet-stream";

    try {
      const content = readFileSync(filePath);
      res.writeHead(200, { "Content-Type": contentType });
      res.end(content);
    } catch {
      res.writeHead(404);
      res.end("Not Found");
    }
  });

  server.listen(port, () => {
    console.log(`🚀 TurboTrace viewer opened at http://localhost:${port}`);
    console.log(`Viewing: ${traceFile}`);
  });
}

module.exports = { showTrace };

if (require.main === module) {
  showTrace(process.argv.slice(2)).catch((e) => {
    console.error("Error:", e.message);
    process.exit(1);
  });
}
