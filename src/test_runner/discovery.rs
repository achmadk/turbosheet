use glob::glob;
use std::path::PathBuf;
use tracing::info;
use crate::error::TurbosheetError;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct TestFile {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: u64,
}

pub fn discover_tests(
    test_dir: &str,
    patterns: &[String],
    grep: Option<&str>,
) -> Result<Vec<TestFile>, TurbosheetError> {
    let mut files = Vec::new();
    
    let grep_regex = match grep {
        Some(pattern) => Some(Regex::new(pattern).map_err(|e| {
            TurbosheetError::Other(format!("Invalid grep pattern {}: {}", pattern, e))
        })?),
        None => None,
    };

    for pattern in patterns {
        let full_pattern = if test_dir == "." {
            pattern.clone()
        } else {
            format!("{}/{}", test_dir, pattern)
        };

        info!("Discovering tests with pattern: {}", full_pattern);

        let entries = glob(&full_pattern).map_err(|e| {
            TurbosheetError::Other(format!("Invalid glob pattern {}: {}", full_pattern, e))
        })?;

        for entry in entries {
            match entry {
                Ok(path) => {
                    let path_str = path.to_string_lossy();
                    if path_str.contains("node_modules")
                        || path_str.contains("target")
                        || path_str.contains(".git")
                    {
                        continue;
                    }
                    
                    if let Some(ref re) = grep_regex {
                        let matches_path = re.is_match(&path_str);
                        let content = std::fs::read_to_string(&path).unwrap_or_default();
                        if !matches_path && !re.is_match(&content) {
                            continue;
                        }
                    }

                    let metadata = std::fs::metadata(&path)?;
                    let mtime = metadata
                        .modified()
                        .unwrap_or_else(|_| std::time::SystemTime::now())
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    files.push(TestFile {
                        path: path.canonicalize()?,
                        size: metadata.len(),
                        mtime,
                    });
                }
                Err(e) => {
                    tracing::warn!("Error reading glob entry: {}", e);
                }
            }
        }
    }

    info!("Discovered {} test files", files.len());
    Ok(files)
}
