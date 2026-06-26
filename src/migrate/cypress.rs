use crate::migrate::report::{JsApiMapping, JsManualTodo, JsMigrationReport};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn migrate_cypress(source_path: String) -> Result<JsMigrationReport> {
    let mappings = vec![
        JsApiMapping {
            original: "cy.visit()".to_string(),
            converted: "page.goto()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "cy.get()".to_string(),
            converted: "page.locator()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "cy.contains()".to_string(),
            converted: "page.getByText()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "cy.click()".to_string(),
            converted: "await locator.click()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "cy.type()".to_string(),
            converted: "await locator.fill()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "cy.should('be.visible')".to_string(),
            converted: "expect(locator).toBeVisible()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "cy.should('have.text')".to_string(),
            converted: "expect(locator).toHaveText()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "cy.intercept()".to_string(),
            converted: "page.route()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "cy.wait()".to_string(),
            converted: "// Use expect.poll() or remove - TurboSheet auto-waits".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "cy.stub()".to_string(),
            converted: "// Use page.route() with fulfill()".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "cy.spy()".to_string(),
            converted: "// Use page.route() with fulfill()".to_string(),
            automated: false,
        },
        JsApiMapping {
            original: "cy.viewport()".to_string(),
            converted: "test.use({ viewport: ... })".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "before()".to_string(),
            converted: "test.beforeAll()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "after()".to_string(),
            converted: "test.afterAll()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "beforeEach()".to_string(),
            converted: "test.beforeEach()".to_string(),
            automated: true,
        },
        JsApiMapping {
            original: "afterEach()".to_string(),
            converted: "test.afterEach()".to_string(),
            automated: true,
        },
    ];

    let todos = vec![
        JsManualTodo {
            file: source_path.clone(),
            description:
                "Review all cy.wait() calls - most can be removed with TurboSheet's auto-waiting"
                    .to_string(),
            severity: "warning".to_string(),
        },
        JsManualTodo {
            file: source_path.clone(),
            description: "Convert cy.stub() and cy.spy() to page.route() with mock handlers"
                .to_string(),
            severity: "warning".to_string(),
        },
        JsManualTodo {
            file: source_path.clone(),
            description: "Review cy.intercept() - ensure route handlers handle all cases"
                .to_string(),
            severity: "info".to_string(),
        },
    ];

    let coverage = if !mappings.is_empty() {
        (mappings.iter().filter(|m| m.automated).count() as f64 / mappings.len() as f64) * 100.0
    } else {
        0.0
    };

    Ok(JsMigrationReport {
        source_framework: "cypress".to_string(),
        files_converted: 0,
        files_skipped: 0,
        manual_todos: todos,
        api_mappings: mappings,
        coverage_percent: coverage,
    })
}

#[napi]
pub fn migrate_cypress_file(content: String) -> String {
    let mut output = content;

    output = output.replace(
        "import cy from 'cypress';",
        "import { test, expect, page } from 'tsheet';",
    );
    output = output.replace("from 'cypress'", "from 'tsheet'");

    output = output.replace("cy.visit(", "await page.goto(");
    output = output.replace("cy.get('", "await page.locator('");
    output = output.replace("cy.get(\"", "await page.locator(\"");
    output = output.replace("cy.get(`", "await page.locator(`");
    output = output.replace("cy.get(", "await page.locator(");
    output = output.replace("cy.contains('", "await page.getByText('");
    output = output.replace("cy.contains(\"", "await page.getByText(\"");
    output = output.replace("cy.contains(`", "await page.getByText(`");
    output = output.replace("cy.contains(", "await page.getByText(");

    output = output.replace("cy.click()", ".click()");
    output = output.replace("cy.dblclick()", ".dblclick()");
    output = output.replace("cy.rightclick()", ".click({ button: 'right' })");
    output = output.replace("cy.type(", ".fill(");
    output = output.replace("cy.clear()", ".fill('')");

    output = output.replace("cy.should('be.visible')", "expect(locator).toBeVisible()");
    output = output.replace(
        "cy.should('not.be.visible')",
        "expect(locator).not.toBeVisible()",
    );
    output = output.replace("cy.should('have.text',", "expect(locator).toHaveText(");
    output = output.replace("cy.should('have.value',", "expect(locator).toHaveValue(");
    output = output.replace("cy.should('have.class',", "expect(locator).toHaveClass(");
    output = output.replace("cy.should('be.enabled')", "expect(locator).toBeEnabled()");
    output = output.replace("cy.should('be.disabled')", "expect(locator).toBeDisabled()");
    output = output.replace(
        "cy.should('exist')",
        "// TurboSheet auto-waits for element existence",
    );

    output = output.replace("cy.intercept('GET',", "await page.route('**");
    output = output.replace("cy.intercept('POST',", "await page.route('**");
    output = output.replace("cy.intercept('PUT',", "await page.route('**");
    output = output.replace("cy.intercept('DELETE',", "await page.route('**");
    output = output.replace(
        "(req, res) => {",
        "async (route) => {\n  // TODO: Replace body with route.fulfill() or route.continue()",
    );

    output = output.replace("before(() =>", "test.beforeAll(async () =>");
    output = output.replace("after(() =>", "test.afterAll(async () =>");
    output = output.replace("beforeEach(() =>", "test.beforeEach(async () =>");
    output = output.replace("afterEach(() =>", "test.afterEach(async () =>");

    output = output.replace("describe('", "test.describe('");
    output = output.replace("it('", "test('");
    output = output.replace("it.only('", "test.only('");
    output = output.replace("it.skip('", "test.skip('");
    output = output.replace("xit('", "test.skip('");
    output = output.replace("describe.only('", "test.describe.only('");
    output = output.replace("describe.skip('", "test.describe.skip('");

    output
}

#[napi]
pub fn migrate_cypress_config(content: String) -> String {
    let mut output = content;

    output = output.replace("cypress.config.ts", "tsheet.config.ts");
    output = output.replace(
        "import { defineConfig } from 'cypress'",
        "import { defineConfig } from 'tsheet'",
    );

    output = output.replace("e2e:", "// Projects for cross-browser testing:");
    output = output.replace("specPattern:", "// testMatch:");
    output = output.replace("video:", "// video:");
    output = output.replace("screenshotOnRunFailure:", "// screenshotOnRunFailure:");
    output = output.replace("viewportWidth:", "viewport:");
    output = output.replace("viewportHeight:", "viewport:");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_replacement() {
        let input = "import cy from 'cypress';".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("import { test, expect, page } from 'tsheet'"));
    }

    #[test]
    fn test_cy_get_single_quotes() {
        let input = "cy.get('.my-class').click();".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("await page.locator('.my-class'"));
        assert!(!result.contains("await page.locator(\".my-class'"));
    }

    #[test]
    fn test_cy_get_double_quotes() {
        let input = r#"cy.get(".my-class").click();"#.to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("await page.locator(\".my-class\""));
    }

    #[test]
    fn test_cy_get_backtick_quotes() {
        let input = "cy.get(`.my-class`).click();".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("await page.locator(`.my-class`"));
    }

    #[test]
    fn test_cy_contains_single_quotes() {
        let input = "cy.contains('Submit').click();".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("await page.getByText('Submit'"));
        assert!(!result.contains("await page.getByText(\"Submit'"));
    }

    #[test]
    fn test_cy_contains_double_quotes() {
        let input = r#"cy.contains("Submit").click();"#.to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("await page.getByText(\"Submit\""));
    }

    #[test]
    fn test_cy_visit_replacement() {
        let input = "cy.visit('/dashboard');".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("await page.goto('/dashboard')"));
    }

    #[test]
    fn test_cy_type_replacement() {
        let input = "cy.get('input').type('hello');".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains(".fill('hello')"));
    }

    #[test]
    fn test_cy_clear_replacement() {
        let input = "cy.get('input').clear();".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains(".fill('')"));
    }

    #[test]
    fn test_describe_it_replacement() {
        let input = "describe('suite', () => { it('test', () => { }); });".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("test.describe('suite'"));
        assert!(result.contains("test('test'"));
    }

    #[test]
    fn test_before_each_replacement() {
        let input = "beforeEach(() => { cy.visit('/'); });".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("test.beforeEach(async () =>"));
    }

    #[test]
    fn test_intercept_callback_conversion() {
        let input =
            "cy.intercept('GET', '/api/data', { times: 1 }, (req, res) => { res.json({ ok: true }); });"
                .to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("async (route) =>"));
        assert!(result.contains("TODO: Replace body"));
    }

    #[test]
    fn test_coverage_calculation() {
        let report = migrate_cypress("test.cy.ts".to_string()).unwrap();
        assert!(report.coverage_percent > 0.0);
        assert_eq!(report.source_framework, "cypress");
        assert!(!report.api_mappings.is_empty());
    }

    #[test]
    fn test_should_replacement() {
        let input = "cy.get('.btn').should('be.visible');".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("expect(locator).toBeVisible()"));
    }

    #[test]
    fn test_intercept_http_methods() {
        let input = "cy.intercept('POST', '/api/data', (req) => { req.reply({}); });".to_string();
        let result = migrate_cypress_file(input);
        assert!(result.contains("await page.route('**"));
    }

    #[test]
    fn test_cypress_config_replacement() {
        let input =
            "import { defineConfig } from 'cypress'\nexport default defineConfig({ e2e: { specPattern: 'cypress/**/*.cy.ts' } })"
                .to_string();
        let result = migrate_cypress_config(input);
        assert!(result.contains("from 'tsheet'"));
        assert!(result.contains("// Projects for cross-browser testing"));
    }

    #[test]
    fn test_cypress_config_viewport() {
        let input = "viewportWidth: 1280,\nviewportHeight: 720,".to_string();
        let result = migrate_cypress_config(input);
        assert!(result.contains("viewport:"));
    }
}
