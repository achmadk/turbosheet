import { defineConfig, configDefaults } from "vite-plus";

export default defineConfig({
  staged: {
    "*": "vp check --fix",
  },
  fmt: {},
  lint: {
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    rules: { "vite-plus/prefer-vite-plus-imports": "error" },
    options: { typeAware: true, typeCheck: true },
  },
  test: {
    include: [...configDefaults.include, "__test__/**/*.spec.ts"],
    exclude: [...configDefaults.exclude, "test-migration/**", "__test__/component.test.js"],
    testTimeout: 60_000,
    hookTimeout: 60_000,
    retry: 0,
  },
  pack: {
    dts: true,
    format: ["esm", "cjs"],
    entry: {
      core: "js/src/injected-core.ts",
      actions: "js/src/injected-actions.ts",
    },
    minify: true,
    sourcemap: false,
    outDir: "js/dist",
    platform: "browser",
  },
});
