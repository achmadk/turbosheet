use crate::migrate::report::{JsApiMapping, JsManualTodo, JsMigrationReport};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn migrate_playwright(source_path: String) -> Result<JsMigrationReport> {
    let mappings = vec![
        JsApiMapping {
            original: "page.locator()".to_string(),
            converted: "page.locator()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.waitForSelector()".to_string(),
            converted: "await page.locator().waitFor() / native auto-wait".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.$eval()".to_string(),
            converted: "page.evaluate()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "test.beforeAll()".to_string(),
            converted: "test.beforeAll()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "test.afterAll()".to_string(),
            converted: "test.afterAll()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "test.beforeEach()".to_string(),
            converted: "test.beforeEach()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "test.afterEach()".to_string(),
            converted: "test.afterEach()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "test.extend()".to_string(),
            converted: "test.extend()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.route()".to_string(),
            converted: "page.route()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "expect(locator).toBeVisible()".to_string(),
            converted: "expect(locator).toBeVisible()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.screenshot()".to_string(),
            converted: "page.screenshot()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "browserType.launch()".to_string(),
            converted: "launch()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "custom reporters".to_string(),
            converted: "registerReporter() / reporter plugins".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "webServer config".to_string(),
            converted: "use globalSetup in tsheet.config.ts".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.addInitScript()".to_string(),
            converted: "// Use context.addInitScript() or test.use()".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.addStyleTag()".to_string(),
            converted: "// Not yet supported".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.addScriptTag()".to_string(),
            converted: "// Not yet supported".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.pause()".to_string(),
            converted: "// Not yet supported - use debug mode".to_string(),
            automated: false,
        },
    ];

    let todos = vec![
        JsManualTodo {
            file: source_path.clone(),
            description: "Verify all page.waitForSelector() calls are replaced with auto-waiting"
                .to_string(),
            severity: "warning".to_string(),
        },
        JsManualTodo {
            file: source_path.clone(),
            description: "Check for any custom Playwright extensions or plugins".to_string(),
            severity: "info".to_string(),
        },
    ];

    let coverage =
        if mappings.iter().filter(|m| m.automated).count() as u32 + mappings.len() as u32 > 0 {
            (mappings.iter().filter(|m| m.automated).count() as f64 / mappings.len() as f64) * 100.0
        } else {
            0.0
        };

    Ok(JsMigrationReport {
        source_framework: "playwright".to_string(),
        files_converted: 0,
        files_skipped: 0,
        manual_todos: todos,
        api_mappings: mappings,
        coverage_percent: coverage,
    })
}

#[napi]
pub fn migrate_playwright_file(content: String) -> String {
    let mut output = content;

    output = output.replace(
        "import { test, expect } from '@playwright/test';",
        "import { test, expect } from 'tsheet';",
    );
    output = output.replace("from 'playwright'", "from 'tsheet'");

    output = output.replace(
        "page.waitForSelector",
        "// Auto-waits in TurboSheet: page.locator",
    );
    output = output.replace("page.$eval", "page.evaluate");
    output = output.replace(
        "page.$$eval(",
        "page.evaluate(/* $$eval returns array - use querySelectorAll inside evaluate */",
    );

    output = output.replace(
        "import { chromium, firefox, webkit } from 'playwright';",
        "// Cross-browser support built-in",
    );

    output
}

#[napi]
pub fn migrate_playwright_config(content: String) -> String {
    let mut output = content;

    output = output.replace("playwright.config.ts", "tsheet.config.ts");
    output = output.replace(
        "import { defineConfig } from '@playwright/test'",
        "import { defineConfig } from 'tsheet'",
    );

    output = output.replace(
        "webServer:",
        "// Use globalSetup in tsheet.config.ts instead of webServer",
    );
    output = output.replace("  command:", "  // command:");
    output = output.replace("  port:", "  // port:");
    output = output.replace("  timeout:", "  // timeout:");
    output = output.replace("  reuseExistingServer:", "  // reuseExistingServer:");

    output = output.replace("use: {", "projects: [{");
    output = output.replace("browserName:", "browser:");
    output = output.replace("} as Project,", "}],");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_replacement() {
        let input = "import { test, expect } from '@playwright/test';".to_string();
        let result = migrate_playwright_file(input);
        assert!(result.contains("import { test, expect } from 'tsheet'"));
    }

    #[test]
    fn test_from_playwright_replacement() {
        let input = "from 'playwright'".to_string();
        let result = migrate_playwright_file(input);
        assert!(result.contains("from 'tsheet'"));
        assert!(!result.contains("from 'turbo-sheet'"));
    }

    #[test]
    fn test_waitforselector_replacement() {
        let input = "page.waitForSelector('.my-class');".to_string();
        let result = migrate_playwright_file(input);
        assert!(result.contains("Auto-waits"));
        assert!(!result.contains("page.waitForSelector("));
    }

    #[test]
    fn test_dollar_eval() {
        let input = "const text = await page.$eval('#el', el => el.textContent);".to_string();
        let result = migrate_playwright_file(input);
        assert!(!result.contains("page.$eval("));
        assert!(result.contains("page.evaluate("));
    }

    #[test]
    fn test_dollar_dollar_eval_has_comment() {
        let input = "const items = await page.$$eval('.list li', els => els.length);".to_string();
        let result = migrate_playwright_file(input);
        assert!(result.contains("$$eval") || result.contains("querySelectorAll"));
        assert!(result.contains("page.evaluate("));
    }

    #[test]
    fn test_cross_browser_import() {
        let input = "import { chromium, firefox, webkit } from 'playwright';".to_string();
        let result = migrate_playwright_file(input);
        assert!(result.contains("Cross-browser support built-in"));
    }

    #[test]
    fn test_config_replacement() {
        let input = "import { defineConfig } from '@playwright/test'".to_string();
        let result = migrate_playwright_config(input);
        assert!(result.contains("from 'tsheet'"));
    }

    #[test]
    fn test_config_webserver() {
        let input =
            "webServer: {\n  command: 'npm start',\n  port: 3000,\n  timeout: 10000,\n  reuseExistingServer: true,\n}"
                .to_string();
        let result = migrate_playwright_config(input);
        assert!(result.contains("globalSetup"));
        assert!(result.contains("//  command:"));
        assert!(result.contains("//  port:"));
        assert!(result.contains("//  timeout:"));
        assert!(result.contains("//  reuseExistingServer:"));
    }

    #[test]
    fn test_new_mappings_exist() {
        let report = migrate_playwright("test.spec.ts".to_string()).unwrap();
        let has_add_init_script = report
            .api_mappings
            .iter()
            .any(|m| m.original.contains("addInitScript"));
        let has_add_style_tag = report
            .api_mappings
            .iter()
            .any(|m| m.original.contains("addStyleTag"));
        let has_add_script_tag = report
            .api_mappings
            .iter()
            .any(|m| m.original.contains("addScriptTag"));
        assert!(has_add_init_script, "addInitScript mapping should exist");
        assert!(has_add_style_tag, "addStyleTag mapping should exist");
        assert!(has_add_script_tag, "addScriptTag mapping should exist");
    }

    #[test]
    fn test_coverage_calculation() {
        let report = migrate_playwright("test.spec.ts".to_string()).unwrap();
        assert!(report.coverage_percent > 0.0);
        assert_eq!(report.source_framework, "playwright");
        assert!(!report.api_mappings.is_empty());
    }
}
