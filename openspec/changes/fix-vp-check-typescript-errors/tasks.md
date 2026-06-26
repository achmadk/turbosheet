## 1. Add missing exports to `index.d.ts` (launch, Browser, version)

- [x] 1.1 Add `launch()` function declaration with `LaunchOptions` parameter type and `Promise<Browser>` return type
- [x] 1.2 Add `Browser` interface with `newContext()` and `close()` methods
- [x] 1.3 Add `version()` function declaration
- [x] 1.4 Export all new declarations from the module (they must appear at the top level, not just as type augmentations)
- [x] 1.5 Verify `vp check --fix` on `__test__/smoke.spec.ts` — `tsheet.launch`, `tsheet.version` errors should clear

## 2. Complete `Page` interface with core browser automation methods

- [x] 2.1 Add `goto(url: string): Promise<void>` to the `Page` interface
- [x] 2.2 Add `locator(selector: string): Locator` to `Page` (+ `Locator` interface with click, fill, textContent, isVisible, etc.)
- [x] 2.3 Add `close(): Promise<void>` to `Page`
- [x] 2.4 Add `content(): Promise<string>` to `Page`
- [x] 2.5 Add `title(): Promise<string>` to `Page`
- [x] 2.6 Add `url(): string` to `Page`
- [x] 2.7 Add `evaluate(js: string): Promise<any>` to `Page`
- [x] 2.8 Verify `vp check --fix` on `__test__/benchmark.spec.ts` and `__test__/integration.spec.ts` — all Page method errors should clear

## 3. Add `src/runtime/*.ts` to tsconfig `include`

- [x] 3.1 Update root `tsconfig.json` `include` to add `"src/runtime/**/*.ts"`
- [x] 3.2 Verify `vp check --fix` — `fs`, `path`, `process`, `readline`, `NodeJS`, `require` errors in `src/runtime/` should clear

## 4. Add `ES2024` to tsconfig `lib`

- [x] 4.1 Update root `tsconfig.json` `compilerOptions.lib` to include `"ES2024"` alongside existing values
- [x] 4.2 Verify `vp check --fix` — `Promise.withResolvers` error in `js/src/injected-actions.ts` should clear

## 5. Fix trace-viewer `vite.config.ts` type mismatch

- [x] 5.1 Change `import { defineConfig } from "vite-plus"` to `import { defineConfig } from "vite"` in `packages/trace-viewer/vite.config.ts`
- [x] 5.2 Verify `vp check --fix` — `defineConfig` / `react()` type errors in `packages/trace-viewer/vite.config.ts` should clear

## 6. Final verification

- [x] 6.1 Run `vp check --fix` and confirm zero TypeScript errors (exit code 0)
- [ ] 6.2 Run `vp test` to confirm no test regressions (pre-existing: vitest binary missing from @voidzero-dev/vite-plus-test)
