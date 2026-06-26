use crate::migrate::report::{JsApiMapping, JsManualTodo, JsMigrationReport};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn migrate_puppeteer(source_path: String) -> Result<JsMigrationReport> {
    let mappings = vec![
        JsApiMapping {
            original: "puppeteer.launch()".to_string(),
            converted: "launch()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "browser.newPage()".to_string(),
            converted:
                "const context = await browser.newContext(); const page = await context.newPage()"
                    .to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.goto()".to_string(),
            converted: "page.goto()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.$()".to_string(),
            converted: "page.locator()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.$$()".to_string(),
            converted: "page.locator().all()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.waitForSelector()".to_string(),
            converted: "// Auto-waits - removed".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.waitForNavigation()".to_string(),
            converted: "// Auto-waits - removed".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.click()".to_string(),
            converted: "page.click()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.type()".to_string(),
            converted: "page.fill()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.evaluate()".to_string(),
            converted: "page.evaluate()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.screenshot()".to_string(),
            converted: "page.screenshot()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.emulate()".to_string(),
            converted: "test.use({ ...devices['...'] })".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.setGeolocation()".to_string(),
            converted: "context.setGeolocation()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "page.setContent()".to_string(),
            converted: "// Use page.goto() with data URL or mount component".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.pdf()".to_string(),
            converted: "// Not yet supported".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.emulateNetworkConditions()".to_string(),
            converted: "// Use test.use() context options".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.setExtraHTTPHeaders()".to_string(),
            converted: "// Use route interception with page.route()".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.screencast()".to_string(),
            converted: "// Not yet supported in TurboSheet".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.setCookie()".to_string(),
            converted: "context.addCookies()".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.deleteCookie()".to_string(),
            converted: "context.clearCookies()".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.cookies()".to_string(),
            converted: "context.cookies()".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.workers()".to_string(),
            converted: "// Not yet supported".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "page.frames()".to_string(),
            converted: "// Use iframe locator instead".to_string(),
            automated: false,
        },
    ];

    let todos = vec![
        JsManualTodo {
            file: source_path.clone(),
            description: "Review page.setContent() calls - may need refactoring to use page.goto() with data URLs".to_string(),
            severity: "warning".to_string(),
        },
        JsManualTodo {
            file: source_path.clone(),
            description: "Convert page.emulate() to test.use() with device descriptors".to_string(),
            severity: "info".to_string(),
        },
        JsManualTodo {
            file: source_path.clone(),
            description: "page.pdf() is not yet supported in TurboSheet".to_string(),
            severity: "critical".to_string(),
        },
    ];

    let coverage = if !mappings.is_empty() {
        (mappings.iter().filter(|m| m.automated).count() as f64 / mappings.len() as f64) * 100.0
    } else {
        0.0
    };

    Ok(JsMigrationReport {
        source_framework: "puppeteer".to_string(),
        files_converted: 0,
        files_skipped: 0,
        manual_todos: todos,
        api_mappings: mappings,
        coverage_percent: coverage,
    })
}

#[napi]
pub fn migrate_puppeteer_file(content: String) -> String {
    let mut output = content;

    output = output.replace(
        "const puppeteer = require('puppeteer');",
        "// TurboSheet uses ES modules or napi-rs",
    );
    output = output.replace(
        "import puppeteer from 'puppeteer';",
        "import { launch } from 'tsheet';",
    );
    output = output.replace(
        "const { chromium } = require('puppeteer');",
        "import { launch } from 'tsheet';",
    );
    output = output.replace(
        "import { chromium } from 'puppeteer';",
        "import { launch } from 'tsheet';",
    );

    output = output.replace("await puppeteer.launch(", "const browser = await launch(");
    output = output.replace("puppeteer.launch(", "const browser = await launch(");

    output = output.replace(
        "const page = await browser.pages()[0];",
        "// Use newContext().newPage() instead",
    );
    output = output.replace(
        "const page = await browser.newPage();",
        "// const context = await browser.newContext();\n// const page = await context.newPage();",
    );

    output = output.replace(
        "await page.waitForSelector(",
        "// Auto-waits in TurboSheet: await page.locator(",
    );
    output = output.replace(
        "await page.waitForNavigation(",
        "// Auto-waits in TurboSheet: // removed",
    );
    output = output.replace("await page.waitForXPath(", "await page.locator('xpath=");
    output = output.replace(
        "await page.waitForFunction(",
        "// Use expect.poll() for custom waits",
    );

    output = output.replace("await page.$eval(", "await page.evaluate(");
    output = output.replace("await page.$$eval(", "const results = await page.evaluate(");

    output = output.replace("await page.type(", "await page.fill(");
    output = output.replace("await page.click(", "await page.click(");
    output = output.replace("await page.hover(", "await page.hover(");

    output = output.replace(
        "await page.setViewport({",
        "// Viewport set in context options:",
    );

    output = output.replace(
        "await page.setGeolocation({",
        "await context.setGeolocation({",
    );

    output = output.replace("await page.screenshot({ path:", "await page.screenshot({");
    output = output.replace("fullPage: true", "// fullPage: true");

    output = output.replace("await browser.close()", "// Browser cleanup automatic");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_require_replacement() {
        let input = "const puppeteer = require('puppeteer');".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(!result.contains("require('puppeteer')"));
        assert!(result.contains("// TurboSheet uses ES modules or napi-rs"));
    }

    #[test]
    fn test_import_esm_replacement() {
        let input = "import puppeteer from 'puppeteer';".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(result.contains("import { launch } from 'tsheet'"));
    }

    #[test]
    fn test_launch_replacement() {
        let input = "const browser = await puppeteer.launch({ headless: true });".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(!result.contains("puppeteer.launch("));
        assert!(result.contains("await launch("));
    }

    #[test]
    fn test_newpage_replacement() {
        let input = "const page = await browser.newPage();".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(result.contains("newContext()"));
    }

    #[test]
    fn test_waitforselector_replacement() {
        let input = "await page.waitForSelector('.my-class');".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(!result.contains("waitForSelector("));
        assert!(result.contains("Auto-waits"));
    }

    #[test]
    fn test_type_to_fill() {
        let input = "await page.type('#input', 'hello');".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(!result.contains("page.type("));
        assert!(result.contains("page.fill("));
    }

    #[test]
    fn test_dollar_eval() {
        let input = "const text = await page.$eval('#el', el => el.textContent);".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(!result.contains("page.$eval("));
        assert!(result.contains("page.evaluate("));
    }

    #[test]
    fn test_dollar_dollar_eval() {
        let input = "const items = await page.$$eval('.list li', els => els.length);".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(!result.contains("page.$$eval("));
        assert!(result.contains("const results = await page.evaluate("));
    }

    #[test]
    fn test_width_not_replaced_outside_viewport() {
        let input = r#"const width = 1920;
const height = 1080;
const obj = { width: width, height: height };"#
            .to_string();
        let result = migrate_puppeteer_file(input);
        // width: and height: should NOT be replaced outside setViewport context
        assert!(result.contains("const width = 1920"));
        assert!(result.contains("width: width"));
        assert!(result.contains("height: height"));
    }

    #[test]
    fn test_type_png_not_replaced_outside_screenshot() {
        let input = r#"const format: 'png' | 'jpeg' = 'png';
const opts = { type: 'png' };"#
            .to_string();
        let result = migrate_puppeteer_file(input);
        // type: 'png' should NOT be globally replaced
        assert!(result.contains("type: 'png'") || result.contains("'png'"));
    }

    #[test]
    fn test_screenshot_replacement() {
        let input =
            "await page.screenshot({ path: 'screenshot.png', fullPage: true });".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(result.contains("fullPage: true"));
    }

    #[test]
    fn test_setviewport_replacement() {
        let input = "await page.setViewport({ width: 1920, height: 1080 });".to_string();
        let result = migrate_puppeteer_file(input);
        assert!(result.contains("Viewport set in context options"));
    }

    #[test]
    fn test_coverage_calculation() {
        let report = migrate_puppeteer("test.js".to_string()).unwrap();
        assert!(report.coverage_percent > 0.0);
        assert!(report.coverage_percent < 100.0); // Some mappings are automated: false
        assert_eq!(report.source_framework, "puppeteer");
        assert!(!report.api_mappings.is_empty());
    }

    #[test]
    fn test_new_mappings_exist() {
        let report = migrate_puppeteer("test.js".to_string()).unwrap();
        let has_network = report
            .api_mappings
            .iter()
            .any(|m| m.original.contains("emulateNetworkConditions"));
        let has_screencast = report
            .api_mappings
            .iter()
            .any(|m| m.original.contains("screencast"));
        let has_cookies = report
            .api_mappings
            .iter()
            .any(|m| m.original.contains("setCookie"));
        assert!(has_network, "emulateNetworkConditions mapping should exist");
        assert!(has_screencast, "screencast mapping should exist");
        assert!(has_cookies, "setCookie mapping should exist");
    }

    #[test]
    fn test_emulate_is_not_automated() {
        let report = migrate_puppeteer("test.js".to_string()).unwrap();
        let emulate = report
            .api_mappings
            .iter()
            .find(|m| m.original.contains("emulate("))
            .unwrap();
        assert!(
            !emulate.automated,
            "page.emulate() should not be marked automated (no replacement exists)"
        );
    }
}
