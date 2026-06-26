use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub source_framework: String,
    pub files_converted: u32,
    pub files_skipped: u32,
    pub manual_todos: Vec<ManualTodo>,
    pub api_mappings: Vec<ApiMapping>,
    pub coverage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualTodo {
    pub file: String,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMapping {
    pub original: String,
    pub converted: String,
    pub automated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMigrationResult {
    pub input_file: String,
    pub output_file: String,
    pub success: bool,
    pub lines_converted: u32,
    pub lines_skipped: u32,
}

#[napi(object)]
pub struct JsMigrationReport {
    pub source_framework: String,
    pub files_converted: u32,
    pub files_skipped: u32,
    pub manual_todos: Vec<JsManualTodo>,
    pub api_mappings: Vec<JsApiMapping>,
    pub coverage_percent: f64,
}

#[napi(object)]
pub struct JsManualTodo {
    pub file: String,
    pub description: String,
    pub severity: String,
}

#[napi(object)]
pub struct JsApiMapping {
    pub original: String,
    pub converted: String,
    pub automated: bool,
}

impl From<MigrationReport> for JsMigrationReport {
    fn from(r: MigrationReport) -> Self {
        JsMigrationReport {
            source_framework: r.source_framework,
            files_converted: r.files_converted,
            files_skipped: r.files_skipped,
            manual_todos: r.manual_todos.into_iter().map(|t| JsManualTodo {
                file: t.file,
                description: t.description,
                severity: t.severity,
            }).collect(),
            api_mappings: r.api_mappings.into_iter().map(|m| JsApiMapping {
                original: m.original,
                converted: m.converted,
                automated: m.automated,
            }).collect(),
            coverage_percent: r.coverage_percent,
        }
    }
}

impl MigrationReport {
    pub fn new(source: &str) -> Self {
        Self {
            source_framework: source.to_string(),
            files_converted: 0,
            files_skipped: 0,
            manual_todos: Vec::new(),
            api_mappings: Vec::new(),
            coverage_percent: 0.0,
        }
    }

    pub fn add_todo(&mut self, file: &str, description: &str, severity: &str) {
        self.manual_todos.push(ManualTodo {
            file: file.to_string(),
            description: description.to_string(),
            severity: severity.to_string(),
        });
    }

    pub fn calculate_coverage(&mut self) {
        let total = self.files_converted + self.files_skipped;
        if total > 0 {
            self.coverage_percent = (self.files_converted as f64 / total as f64) * 100.0;
        }
    }
}

#[napi]
pub fn create_migration_report(
    source_framework: String,
    files_converted: u32,
    files_skipped: u32,
    manual_todos: Vec<JsManualTodo>,
    api_mappings: Vec<JsApiMapping>,
) -> JsMigrationReport {
    let coverage_percent = if files_converted + files_skipped > 0 {
        (files_converted as f64 / (files_converted + files_skipped) as f64) * 100.0
    } else {
        0.0
    };

    JsMigrationReport {
        source_framework,
        files_converted,
        files_skipped,
        manual_todos,
        api_mappings,
        coverage_percent,
    }
}

#[napi]
pub fn generate_migration_summary(
    report: JsMigrationReport,
) -> String {
    let mut summary = String::new();
    summary.push_str(&format!("=== {} Migration Report ===\n\n", report.source_framework));
    summary.push_str(&format!("Files converted: {}\n", report.files_converted));
    summary.push_str(&format!("Files skipped: {}\n", report.files_skipped));
    summary.push_str(&format!("Coverage: {:.1}%\n\n", report.coverage_percent));

    if !report.manual_todos.is_empty() {
        summary.push_str("Manual TODOs:\n");
        for todo in &report.manual_todos {
            summary.push_str(&format!(
                "  [{}] {}: {}\n",
                todo.severity.to_uppercase(),
                todo.file,
                todo.description
            ));
        }
        summary.push('\n');
    }

    summary.push_str("API Mappings:\n");
    for mapping in &report.api_mappings {
        let status = if mapping.automated { "[AUTO]" } else { "[MANUAL]" };
        summary.push_str(&format!(
            "  {} {} -> {}\n",
            status, mapping.original, mapping.converted
        ));
    }

    summary
}
