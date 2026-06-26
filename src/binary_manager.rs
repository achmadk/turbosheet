use crate::error::TurbosheetError;
use flate2::read::GzDecoder;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use tar::Archive;
use zip::ZipArchive;

pub struct BrowserBinaryManager;

impl BrowserBinaryManager {
    fn base_dir() -> Result<PathBuf, TurbosheetError> {
        let home = dirs::home_dir().ok_or_else(|| TurbosheetError::Other("Could not find home directory".into()))?;
        let dir = home.join(".tsheet").join("browsers");
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    pub async fn install(browser_type: &str) -> Result<PathBuf, TurbosheetError> {
        let base_dir = Self::base_dir()?;
        let browser_dir = base_dir.join(browser_type);
        
        if browser_dir.exists() {
            return Self::get_path(browser_type);
        }

        fs::create_dir_all(&browser_dir)?;

        let (url, is_zip) = match browser_type {
            "firefox" => {
                // Download geckodriver
                let os = std::env::consts::OS;
                let arch = std::env::consts::ARCH;
                let url = match (os, arch) {
                    ("linux", "x86_64") => "https://github.com/mozilla/geckodriver/releases/download/v0.34.0/geckodriver-v0.34.0-linux64.tar.gz",
                    ("linux", "aarch64") => "https://github.com/mozilla/geckodriver/releases/download/v0.34.0/geckodriver-v0.34.0-linux-aarch64.tar.gz",
                    ("macos", "x86_64") => "https://github.com/mozilla/geckodriver/releases/download/v0.34.0/geckodriver-v0.34.0-macos.tar.gz",
                    ("macos", "aarch64") => "https://github.com/mozilla/geckodriver/releases/download/v0.34.0/geckodriver-v0.34.0-macos-aarch64.tar.gz",
                    ("windows", "x86_64") => "https://github.com/mozilla/geckodriver/releases/download/v0.34.0/geckodriver-v0.34.0-win64.zip",
                    _ => return Err(TurbosheetError::Other(format!("Unsupported OS/Arch for firefox: {}/{}", os, arch))),
                };
                (url, url.ends_with(".zip"))
            }
            "webkit" => {
                // For webkit, we rely on system safaridriver on macOS and webkit2gtk on Linux.
                // We don't download a binary, just verify it exists.
                let os = std::env::consts::OS;
                if os == "macos" {
                    let path = PathBuf::from("/usr/bin/safaridriver");
                    if path.exists() {
                        return Ok(path);
                    } else {
                        return Err(TurbosheetError::Other("safaridriver not found".into()));
                    }
                } else if os == "linux" {
                    // webkit2gtk WebDriver is usually named WebKitWebDriver
                    let path = PathBuf::from("/usr/bin/WebKitWebDriver");
                    if path.exists() {
                        return Ok(path);
                    } else {
                        return Err(TurbosheetError::Other("WebKitWebDriver not found. Please install webkit2gtk-4.1".into()));
                    }
                } else {
                    return Err(TurbosheetError::Other("WebKit is only supported on macOS and Linux".into()));
                }
            }
            "chromium" => {
                // Download Chrome for Testing via CfT JSON API
                let os = std::env::consts::OS;
                let arch = std::env::consts::ARCH;
                let platform = match (os, arch) {
                    ("linux", "x86_64") => "linux64",
                    ("linux", "aarch64") => "linux-arm64",
                    ("macos", "x86_64") => "mac-x64",
                    ("macos", "aarch64") => "mac-arm64",
                    ("windows", "x86_64") => "win64",
                    ("windows", "aarch64") => "win-arm64",
                    _ => return Err(TurbosheetError::Other(format!("Unsupported OS/Arch for chromium: {}/{}", os, arch))),
                };

                // Fetch the latest stable version from CfT
                let manifest_url = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json";
                let manifest: serde_json::Value = reqwest::get(manifest_url)
                    .await
                    .map_err(|e| TurbosheetError::Other(format!("Failed to fetch CfT manifest: {}", e)))?
                    .json()
                    .await
                    .map_err(|e| TurbosheetError::Other(format!("Failed to parse CfT manifest: {}", e)))?;

                let version = manifest["channels"]["Stable"]["version"]
                    .as_str()
                    .ok_or_else(|| TurbosheetError::Other("Missing version in CfT manifest".into()))?;

                // Build the download URL for Chromium itself (not chromedriver)
                let url = format!(
                    "https://storage.googleapis.com/chrome-for-testing-public/{}/{}/chrome-{}.zip",
                    version, platform, platform
                );

                tracing::info!("Downloading Chromium {} ({}): {}", version, platform, url);
                let response = reqwest::get(&url).await.map_err(|e| {
                    TurbosheetError::Other(format!("Failed to download Chromium: {}", e))
                })?;
                let bytes = response.bytes().await.map_err(|e| {
                    TurbosheetError::Other(format!("Failed to read Chromium download: {}", e))
                })?;

                // Extract zip — CfT archives have a {browser_type}-{platform}/ top-level dir
                // so we extract into a temp dir then move.
                let reader = Cursor::new(bytes);
                let mut archive = ZipArchive::new(reader)?;
                archive.extract(&browser_dir)?;

                // Return the path to the chrome binary
                let chrome_path = browser_dir.join(format!("chrome-{}", platform)).join("chrome");
                if chrome_path.exists() {
                    return Ok(chrome_path);
                }
                // Fallback: check directly in browser_dir (some CfT archives flatten differently)
                let chrome_fallback = browser_dir.join("chrome");
                if chrome_fallback.exists() {
                    return Ok(chrome_fallback);
                }
                // If we can't find it, return the browser_dir so BinaryResolver can search
                tracing::warn!("Extracted Chromium but chrome binary not found at expected paths; returning directory");
                return Ok(browser_dir);
            }
            _ => return Err(TurbosheetError::Other(format!("Unknown browser type: {}", browser_type))),
        };

        tracing::info!("Downloading {} from {}", browser_type, url);
        let response = reqwest::get(url).await.map_err(|e| TurbosheetError::Other(e.to_string()))?;
        let bytes = response.bytes().await.map_err(|e| TurbosheetError::Other(e.to_string()))?;

        if is_zip {
            let reader = Cursor::new(bytes);
            let mut archive = ZipArchive::new(reader).map_err(|e| TurbosheetError::Other(e.to_string()))?;
            archive.extract(&browser_dir).map_err(|e| TurbosheetError::Other(e.to_string()))?;
        } else {
            let tar = GzDecoder::new(Cursor::new(bytes));
            let mut archive = Archive::new(tar);
            archive.unpack(&browser_dir)?;
        }

        Self::get_path(browser_type)
    }

    pub fn get_path(browser_type: &str) -> Result<PathBuf, TurbosheetError> {
        if browser_type == "webkit" {
            let os = std::env::consts::OS;
            if os == "macos" {
                return Ok(PathBuf::from("/usr/bin/safaridriver"));
            } else if os == "linux" {
                return Ok(PathBuf::from("/usr/bin/WebKitWebDriver"));
            }
        }

        let base_dir = Self::base_dir()?;
        let browser_dir = base_dir.join(browser_type);
        
        if !browser_dir.exists() {
            return Err(TurbosheetError::Other(format!("Browser {} not installed", browser_type)));
        }

        let executable_name = match browser_type {
            "firefox" => if std::env::consts::OS == "windows" { "geckodriver.exe" } else { "geckodriver" },
            "chromium" => if std::env::consts::OS == "windows" { "chrome.exe" } else { "chrome" },
            _ => return Err(TurbosheetError::Other(format!("Unknown browser type: {}", browser_type))),
        };

        let path = browser_dir.join(executable_name);
        if path.exists() {
            Ok(path)
        } else {
            Err(TurbosheetError::Other(format!("Executable not found at {:?}", path)))
        }
    }

    pub fn list_installed() -> Vec<String> {
        let mut installed = Vec::new();
        if let Ok(base_dir) = Self::base_dir() {
            if let Ok(entries) = fs::read_dir(base_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            if let Ok(name) = entry.file_name().into_string() {
                                installed.push(name);
                            }
                        }
                    }
                }
            }
        }
        
        // Check system webkit
        let os = std::env::consts::OS;
        if os == "macos" && PathBuf::from("/usr/bin/safaridriver").exists() {
            installed.push("webkit".to_string());
        } else if os == "linux" && PathBuf::from("/usr/bin/WebKitWebDriver").exists() {
            installed.push("webkit".to_string());
        }

        installed
    }
}
