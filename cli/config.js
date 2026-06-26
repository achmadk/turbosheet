const { readFileSync, existsSync } = require("fs");
const { join } = require("path");

const DEFAULT_CONFIG = {
  testDir: "./tests",
  testMatch: ["**/*.tsheet.ts", "**/*.tsheet.spec.ts"],
  outputDir: "./test-results",
  workers: 1,
  retries: 0,
  timeout: 30000,
  reporter: "list",
  fullyParallel: false,
  forbidOnly: false,
  forbidDescribeOnly: false,
  projects: [],
  use: {},
};

class ConfigLoader {
  constructor() {
    this.config = { ...DEFAULT_CONFIG };
    this.configFile = null;
  }

  async load(cwd = process.cwd()) {
    const candidates = [
      "tsheet.config.ts",
      "tsheet.config.js",
      "tsheet.config.mjs",
      "tsheet.config.cjs",
      "package.json",
    ];

    for (const candidate of candidates) {
      const configPath = join(cwd, candidate);
      if (existsSync(configPath)) {
        this.configFile = configPath;
        await this.parseConfig(configPath);
        break;
      }
    }

    this.applyEnvVars();

    return this.config;
  }

  async parseConfig(path) {
    if (path.endsWith("package.json")) {
      const pkg = JSON.parse(readFileSync(path, "utf-8"));
      if (pkg["tsheet-config"]) {
        this.config = { ...this.config, ...pkg["tsheet-config"] };
      }
      return;
    }

    try {
      const module = require(path);
      const config = module.default || module;
      this.config = { ...this.config, ...config };
    } catch (e) {
      console.warn(`Failed to load config from ${path}:`, e.message);
    }
  }

  applyEnvVars() {
    if (process.env.TSHEET_TEST_DIR) {
      this.config.testDir = process.env.TSHEET_TEST_DIR;
    }
    if (process.env.TSHEET_WORKERS) {
      this.config.workers = parseInt(process.env.TSHEET_WORKERS, 10);
    }
    if (process.env.TSHEET_RETRIES) {
      this.config.retries = parseInt(process.env.TSHEET_RETRIES, 10);
    }
    if (process.env.TSHEET_TIMEOUT) {
      this.config.timeout = parseInt(process.env.TSHEET_TIMEOUT, 10);
    }
    if (process.env.TSHEET_REPORTER) {
      this.config.reporter = process.env.TSHEET_REPORTER;
    }
  }

  get(key) {
    return this.config[key];
  }

  set(key, value) {
    this.config[key] = value;
  }

  toJSON() {
    return { ...this.config };
  }
}

class EnvLoader {
  constructor() {
    this.env = {};
    this.load();
  }

  load() {
    const envFile = join(process.cwd(), ".env");
    if (existsSync(envFile)) {
      const content = readFileSync(envFile, "utf-8");
      this.env = this.parseEnvFile(content);
    }

    this.env = { ...this.env, ...process.env };
  }

  parseEnvFile(content) {
    const env = {};
    const lines = content.split("\n");

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;

      const eqIndex = trimmed.indexOf("=");
      if (eqIndex === -1) continue;

      const key = trimmed.slice(0, eqIndex).trim();
      let value = trimmed.slice(eqIndex + 1).trim();

      if (
        (value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))
      ) {
        value = value.slice(1, -1);
      }

      env[key] = value;
    }

    return env;
  }

  get(key, defaultValue) {
    return this.env[key] !== undefined ? this.env[key] : defaultValue;
  }

  getInt(key, defaultValue) {
    const val = this.get(key);
    if (val === undefined) return defaultValue;
    const parsed = parseInt(val, 10);
    return isNaN(parsed) ? defaultValue : parsed;
  }

  getBool(key, defaultValue) {
    const val = this.get(key);
    if (val === undefined) return defaultValue;
    return val === "true" || val === "1";
  }

  toJSON() {
    return { ...this.env };
  }
}

module.exports = { ConfigLoader, EnvLoader, DEFAULT_CONFIG };

if (require.main === module) {
  const loader = new ConfigLoader();
  const config = await loader.load();
  console.log("Loaded config:");
  console.log(JSON.stringify(config, null, 2));

  const env = new EnvLoader();
  console.log("\nEnvironment:");
  console.log(JSON.stringify(env.toJSON(), null, 2));
}
