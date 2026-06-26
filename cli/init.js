import { promises as fs } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

export async function initCI(provider) {
  switch (provider) {
    case "github":
      await generateGitHubWorkflow();
      break;
    case "gitlab":
      await generateGitLabCI();
      break;
    default:
      throw new Error(`Unknown CI provider: ${provider}. Use 'github' or 'gitlab'.`);
  }

  console.log(`✓ Generated ${provider} CI configuration`);
}

async function generateGitHubWorkflow() {
  const workflowDir = ".github/workflows";
  const workflowPath = join(workflowDir, "tsheet.yml");

  await fs.mkdir(workflowDir, { recursive: true });

  const template = `name: TurboSheet Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    timeout-minutes: 60
    strategy:
      fail-fast: false
      matrix:
        shard: [1, 2, 3, 4]
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Cache browser binaries
        uses: actions/cache@v4
        id: browser-cache
        with:
          path: ~/.cache/turbosheet
          key: browsers-$\{{ runner.os }}-$\{{ hashFiles('**/package-lock.json') }}
          restore-keys: |
            browsers-$\{{ runner.os }}-

      - name: Install dependencies
        run: npm ci

      - name: Install browsers
        run: npx tsheet install
        env:
          TURBOSHEET_CACHE_DIR: ~/.cache/turbosheet

      - name: Run tests (shard \${{ matrix.shard }}/4)
        run: npx tsheet test --shard=\${{ matrix.shard }}/4 --reporter=html --reporter=junit
        env:
          TURBOSHEET_CACHE_DIR: ~/.cache/turbosheet

      - name: Upload HTML report
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: test-results-shard-\${{ matrix.shard }}
          path: test-results/

      - name: Upload JUnit report
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: junit-results-shard-\${{ matrix.shard }}
          path: junit.xml
`;

  await fs.writeFile(workflowPath, template);
  console.log(`  Created ${workflowPath}`);
}

async function generateGitLabCI() {
  const gitlabCIPath = ".gitlab-ci.yml";

  const template = `stages:
  - test

variables:
  TURBOSHEET_CACHE_DIR: ~/.cache/turbosheet

cache:
  key: browsers-$CI_COMMIT_REF_SLUG
  paths:
    - ~/.cache/turbosheet

.install_browsers:
  before_script:
    - npm ci
    - npx tsheet install || true

test:
  stage: test
  image: tsheet/tsheet:chromium-only
  extends: .install_browsers
  parallel:
    matrix:
      - SHARD: [1/4, 2/4, 3/4, 4/4]
  script:
    - npx tsheet test --shard=$SHARD --reporter=html --reporter=junit
  artifacts:
    when: always
    paths:
      - test-results/
      - junit.xml
    reports:
      junit: junit.xml
`;

  await fs.writeFile(gitlabCIPath, template);
  console.log(`  Created ${gitlabCIPath}`);
}

export async function initProject() {
  const files = [{ path: "tsheet.config.ts", content: getDefaultConfig() }];

  for (const file of files) {
    await fs.writeFile(file.path, file.content);
    console.log(`  Created ${file.path}`);
  }
}

function getDefaultConfig() {
  return `import { defineConfig } from 'tsheet';

export default defineConfig({
  testDir: './tests',
  workers: 4,
  retries: 1,
  timeout: 30000,
  reporter: ['html', 'junit'],
  projects: [
    {
      name: 'chromium',
      use: {
        browser: 'chromium',
      },
    },
  ],
});
`;
}
