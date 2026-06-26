use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::time::Duration;
use crate::locator::JsLocator;
use crate::page::JsPage;
use crate::assertions::engine::AssertionEngine;
use crate::engine::PAGES;

#[napi(object)]
#[derive(Clone)]
pub struct RegExp {
    pub pattern: String,
    pub flags: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct MatcherConfig {
    pub timeout: Option<u32>,
    pub polling_interval: Option<u32>,
    pub message: Option<String>,
}

use std::sync::Arc;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadSafeCallContext, ThreadsafeFunctionCallMode};

pub enum ExpectTarget {
    Locator(JsLocator),
    Page(JsPage),
    Function(Arc<ThreadsafeFunction<()>>),
}

fn format_assertion_error(
    expected: &str,
    actual: &str,
    selector: Option<&str>,
    custom_message: Option<&str>,
) -> String {
    let mut msg = String::new();
    msg.push_str(&format!("Expected: {}\n", expected));
    msg.push_str(&format!("Actual:   {}\n", actual));
    if let Some(s) = selector {
        msg.push_str(&format!("Selector: {}\n", s));
    }
    if let Some(m) = custom_message {
        msg.push_str(&format!("{}\n", m));
    }
    msg.trim_end().to_string()
}

impl Clone for ExpectTarget {
    fn clone(&self) -> Self {
        match self {
            ExpectTarget::Locator(l) => ExpectTarget::Locator(JsLocator {
                selector: l.selector.clone(),
                page_id: l.page_id.clone(),
            }),
            ExpectTarget::Page(p) => ExpectTarget::Page(JsPage {
                browser_id: p.browser_id.clone(),
                context_id: p.context_id.clone(),
                page_id: p.page_id.clone(),
            }),
            ExpectTarget::Function(f) => ExpectTarget::Function(Arc::clone(f)),
        }
    }
}

#[napi]
pub struct JsExpect {
    target: Option<ExpectTarget>,
    negated: bool,
}

#[napi]
impl JsExpect {
    #[napi(getter)]
    pub fn not(&self) -> JsExpect {
        JsExpect {
            target: self.target.clone(),
            negated: !self.negated,
        }
    }

    fn engine(&self, config: Option<MatcherConfig>) -> AssertionEngine {
        let mut engine = AssertionEngine::default();
        if let Some(cfg) = config {
            if let Some(t) = cfg.timeout {
                engine.timeout = Duration::from_millis(t as u64);
            }
            if let Some(p) = cfg.polling_interval {
                engine.initial_interval = Duration::from_millis(p as u64);
            }
        }
        engine
    }

    #[napi]
    pub async fn to_be_visible(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeVisible requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let is_visible = page.engine.is_visible(&selector).await.map_err(|e| e.to_string())?;
                
                let pass = if negated { !is_visible } else { is_visible };
                
                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "hidden" } else { "visible" };
                    let actual = if is_visible { "visible" } else { "hidden" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_hidden(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeHidden requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let is_visible = page.engine.is_visible(&selector).await.map_err(|e| e.to_string())?;
                
                let pass = if negated { is_visible } else { !is_visible };
                
                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "visible" } else { "hidden" };
                    let actual = if is_visible { "visible" } else { "hidden" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_text(&self, text: Either<String, RegExp>, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toHaveText requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let text = text.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let text = text.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let actual_text = page.engine.text_content(&selector).await.map_err(|e| e.to_string())?;
                let actual_text = actual_text.unwrap_or_default();
                let matches = match &text {
                    Either::A(s) => actual_text == *s,
                    Either::B(r) => {
                        if let Ok(re) = regex::Regex::new(&r.pattern) {
                            re.is_match(&actual_text)
                        } else {
                            false
                        }
                    }
                };

                let pass = if negated { !matches } else { matches };

                if pass {
                    Ok(true)
                } else {
                    let expected_val = match &text { Either::A(s) => s.as_str(), Either::B(r) => &r.pattern };
                    let expected = if negated { format!("not {}", expected_val) } else { expected_val.to_string() };
                    Err(format_assertion_error(&expected, &actual_text, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_contain_text(&self, text: Either<String, RegExp>, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toContainText requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let text = text.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let text = text.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let actual_text = page.engine.text_content(&selector).await.map_err(|e| e.to_string())?;
                let actual_text = actual_text.unwrap_or_default();
                let matches = match &text {
                    Either::A(s) => actual_text.contains(s),
                    Either::B(r) => {
                        if let Ok(re) = regex::Regex::new(&r.pattern) {
                            re.is_match(&actual_text)
                        } else {
                            false
                        }
                    }
                };

                let pass = if negated { !matches } else { matches };

                if pass {
                    Ok(true)
                } else {
                    let expected_val = match &text { Either::A(s) => s.as_str(), Either::B(r) => &r.pattern };
                    let expected = if negated { format!("not to contain {}", expected_val) } else { format!("to contain {}", expected_val) };
                    Err(format_assertion_error(&expected, &actual_text, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_attribute(&self, name: String, value: Option<Either<String, RegExp>>, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toHaveAttribute requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let attr_name = name.clone();
        let page_id = locator.page_id.clone();
        let value = value.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let attr_name = attr_name.clone();
            let page_id = page_id.clone();
            let value = value.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped_sel = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let escaped_attr = serde_json::to_string(&attr_name).unwrap_or_else(|_| format!("\"{}\"", attr_name));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return null; return el.getAttribute({}); }})()",
                    escaped_sel, escaped_attr);
                let actual_value = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;

                let matches = match (&value, actual_value.is_empty() || actual_value == "null") {
                    (None, true) => false,
                    (None, false) => true,
                    (Some(_), true) => false,
                    (Some(Either::A(expected)), _) => actual_value == *expected,
                    (Some(Either::B(r)), _) => {
                        if let Ok(re) = regex::Regex::new(&r.pattern) {
                            re.is_match(&actual_value)
                        } else {
                            false
                        }
                    }
                };

                let pass = if negated { !matches } else { matches };

                if pass {
                    Ok(true)
                } else {
                    let expected = match &value {
                        None => if negated { format!("not to have attribute '{}'", attr_name) } else { format!("to have attribute '{}'", attr_name) },
                        Some(Either::A(s)) => if negated { format!("attribute '{}' not to be '{}'", attr_name, s) } else { format!("attribute '{}' to be '{}'", attr_name, s) },
                        Some(Either::B(r)) => if negated { format!("attribute '{}' not to match '{}'", attr_name, r.pattern) } else { format!("attribute '{}' to match '{}'", attr_name, r.pattern) },
                    };
                    let actual_display = if actual_value.is_empty() || actual_value == "null" { "missing".to_string() } else { actual_value.clone() };
                    Err(format_assertion_error(&expected, &actual_display, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_enabled(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeEnabled requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return false; return !el.disabled && !el.getAttribute('aria-disabled'); }})()",
                    escaped);
                let is_enabled_str = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let is_enabled = is_enabled_str == "true";
                
                let pass = if negated { !is_enabled } else { is_enabled };
                
                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "disabled" } else { "enabled" };
                    let actual = if is_enabled { "enabled" } else { "disabled" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_disabled(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeDisabled requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return false; return el.disabled || el.getAttribute('aria-disabled') === 'true'; }})()",
                    escaped);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let is_disabled = result == "true";
                let pass = if negated { !is_disabled } else { is_disabled };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "enabled" } else { "disabled" };
                    let actual = if is_disabled { "disabled" } else { "enabled" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_value(&self, value: Either<String, RegExp>, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toHaveValue requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let value = value.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let value = value.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return ''; return el.value || ''; }})()",
                    escaped);
                let actual_value = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let matches = match &value {
                    Either::A(s) => actual_value == *s,
                    Either::B(r) => {
                        if let Ok(re) = regex::Regex::new(&r.pattern) {
                            re.is_match(&actual_value)
                        } else {
                            false
                        }
                    }
                };

                let pass = if negated { !matches } else { matches };

                if pass {
                    Ok(true)
                } else {
                    let expected_val = match &value { Either::A(s) => s.as_str(), Either::B(r) => &r.pattern };
                    let expected = if negated { format!("not '{}'", expected_val) } else { expected_val.to_string() };
                    Err(format_assertion_error(&expected, &actual_value, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_url(&self, url: Either<String, RegExp>, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let page = match target {
            ExpectTarget::Page(p) => p,
            _ => return Err(Error::from_reason("toHaveURL requires a page")),
        };

        let engine = self.engine(config.clone());
        let expected_url = url.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let expected_url = expected_url.clone();
            let custom_message = custom_message.clone();
            async move {
                let actual_url = page.url();
                let matches = match &expected_url {
                    Either::A(s) => {
                        if s.contains('*') || s.contains('?') {
                            if let Ok(pattern) = glob::Pattern::new(s) {
                                pattern.matches(&actual_url)
                            } else {
                                false
                            }
                        } else {
                            actual_url == *s
                        }
                    },
                    Either::B(r) => {
                        if let Ok(re) = regex::Regex::new(&r.pattern) {
                            re.is_match(&actual_url)
                        } else {
                            false
                        }
                    }
                };

                let pass = if negated { !matches } else { matches };

                if pass {
                    Ok(true)
                } else {
                    let expected_val = match &expected_url { Either::A(s) => s.as_str(), Either::B(r) => &r.pattern };
                    let expected = if negated { format!("not matching '{}'", expected_val) } else { format!("matching '{}'", expected_val) };
                    Err(format_assertion_error(&expected, &actual_url, None, custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_title(&self, title: Either<String, RegExp>, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let page = match target {
            ExpectTarget::Page(p) => p,
            _ => return Err(Error::from_reason("toHaveTitle requires a page")),
        };

        let engine = self.engine(config.clone());
        let expected_title = title.clone();
        let page_id = page.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let expected_title = expected_title.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page_engine = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let actual_title = page_engine.engine.title().await.map_err(|e| e.to_string())?;
                let matches = match &expected_title {
                    Either::A(s) => actual_title == *s,
                    Either::B(r) => {
                        if let Ok(re) = regex::Regex::new(&r.pattern) {
                            re.is_match(&actual_title)
                        } else {
                            false
                        }
                    }
                };

                let pass = if negated { !matches } else { matches };

                if pass {
                    Ok(true)
                } else {
                    let expected_val = match &expected_title { Either::A(s) => s.as_str(), Either::B(r) => &r.pattern };
                    let expected = if negated { format!("not '{}'", expected_val) } else { expected_val.to_string() };
                    Err(format_assertion_error(&expected, &actual_title, None, custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_count(&self, count: u32, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toHaveCount requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let target_count = count;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ return document.querySelectorAll({}).length; }})()",
                    escaped);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let actual_count: u32 = result.parse().unwrap_or(0);
                let matches = actual_count == target_count;
                let pass = if negated { !matches } else { matches };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { format!("not {}", target_count) } else { format!("{}", target_count) };
                    Err(format_assertion_error(&expected, &actual_count.to_string(), Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_attached(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeAttached requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); return el !== null && el !== undefined; }})()",
                    escaped);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let is_attached = result == "true";
                let pass = if negated { !is_attached } else { is_attached };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "not attached" } else { "attached" };
                    let actual = if is_attached { "attached" } else { "not attached" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_in_viewport(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeInViewport requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return false; var rect = el.getBoundingClientRect(); return rect.top >= 0 && rect.left >= 0 && rect.bottom <= window.innerHeight && rect.right <= window.innerWidth; }})()",
                    escaped);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let is_in_viewport = result == "true";
                let pass = if negated { !is_in_viewport } else { is_in_viewport };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "not in viewport" } else { "in viewport" };
                    let actual = if is_in_viewport { "in viewport" } else { "not in viewport" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_class(&self, class_name: String, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toHaveClass requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let target_class = class_name.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let target_class = target_class.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped_sel = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let escaped_cls = serde_json::to_string(&target_class).unwrap_or_else(|_| format!("\"{}\"", target_class));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return false; return el.classList.contains({}); }})()",
                    escaped_sel, escaped_cls);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let has_class = result == "true";
                let pass = if negated { !has_class } else { has_class };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { format!("not '{}'", target_class) } else { format!("'{}'", target_class) };
                    Err(format_assertion_error(&expected, if has_class { "present" } else { "absent" }, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_css(&self, property: String, value: String, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toHaveCSS requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let css_property = property.clone();
        let css_value = value.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let css_property = css_property.clone();
            let css_value = css_value.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped_sel = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let escaped_prop = serde_json::to_string(&css_property).unwrap_or_else(|_| format!("\"{}\"", css_property));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return ''; var cs = window.getComputedStyle(el); return cs[{}] || ''; }})()",
                    escaped_sel, escaped_prop);
                let actual_value = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let matches = actual_value == css_value;
                let pass = if negated { !matches } else { matches };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { format!("not '{}'", css_value) } else { format!("'{}'", css_value) };
                    Err(format_assertion_error(&expected, &actual_value, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_focused(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeFocused requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return false; return document.activeElement === el; }})()",
                    escaped);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let is_focused = result == "true";
                let pass = if negated { !is_focused } else { is_focused };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "not focused" } else { "focused" };
                    let actual = if is_focused { "focused" } else { "not focused" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_checked(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeChecked requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return false; return el.checked === true; }})()",
                    escaped);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let is_checked = result == "true";
                let pass = if negated { !is_checked } else { is_checked };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "not checked" } else { "checked" };
                    let actual = if is_checked { "checked" } else { "not checked" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_editable(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeEditable requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return false; return !el.disabled && !el.readOnly && el.getAttribute('aria-readonly') !== 'true'; }})()",
                    escaped);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let is_editable = result == "true";
                let pass = if negated { !is_editable } else { is_editable };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "not editable" } else { "editable" };
                    let actual = if is_editable { "editable" } else { "not editable" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_be_empty(&self, config: Option<MatcherConfig>) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let locator = match target {
            ExpectTarget::Locator(l) => l,
            _ => return Err(Error::from_reason("toBeEmpty requires a locator")),
        };

        let engine = self.engine(config.clone());
        let selector = locator.selector.clone();
        let page_id = locator.page_id.clone();
        let negated = self.negated;
        let custom_message = config.and_then(|c| c.message);

        engine.poll(move || {
            let selector = selector.clone();
            let page_id = page_id.clone();
            let custom_message = custom_message.clone();
            async move {
                let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| "Page not found".to_string())?;
                let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
                let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return true; return el.innerText.trim() === '' && el.children.length === 0; }})()",
                    escaped);
                let result = page.engine.evaluate(&js).await.map_err(|e| e.to_string())?;
                let is_empty = result == "true";
                let pass = if negated { !is_empty } else { is_empty };

                if pass {
                    Ok(true)
                } else {
                    let expected = if negated { "not empty" } else { "empty" };
                    let actual = if is_empty { "empty" } else { "not empty" };
                    Err(format_assertion_error(expected, actual, Some(&selector), custom_message.as_deref()))
                }
            }
        }).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    #[napi]
    pub async fn to_have_screenshot(
        &self,
        name: String,
        options: Option<ScreenshotAssertionOptions>,
        _config: Option<MatcherConfig>,
    ) -> Result<()> {
        let target = self.target.as_ref().ok_or_else(|| Error::from_reason("No target"))?;
        let page = match target {
            ExpectTarget::Page(p) => p,
            _ => return Err(Error::from_reason("toHaveScreenshot requires a page")),
        };

        let opts = options.unwrap_or_default();
        let _threshold = opts.threshold.unwrap_or(0.1);
        let _full_page = opts.full_page.unwrap_or(false);

        let page_engine = PAGES.get(&page.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            Error::from_reason(format!("Page {} not found", page.page_id))
        })?;

        let screenshot_bytes = page_engine.engine.screenshot(None).await.map_err(|e| {
            Error::from_reason(format!("Failed to capture screenshot: {}", e))
        })?;

        // Load baseline if exists
        let baseline_path = format!("__screenshots__/{}.png", name);
        let baseline_path = std::path::Path::new(&baseline_path);

        if baseline_path.exists() {
            // Compare with baseline
            let baseline_bytes = std::fs::read(baseline_path).map_err(|e| {
                Error::from_reason(format!("Failed to read baseline: {}", e))
            })?;

            // Use visual comparison if available (requires image feature)
            #[cfg(feature = "image")]
            {
                use image::{DynamicImage, GenericImageView};
                use crate::visual::compare::{compare_simd, DiffOptions};

                let current = image::load_from_memory(&screenshot_bytes).map_err(|e| {
                    Error::from_reason(format!("Failed to decode screenshot: {}", e))
                })?;
                let baseline = image::load_from_memory(&baseline_bytes).map_err(|e| {
                    Error::from_reason(format!("Failed to decode baseline: {}", e))
                })?;

                let diff_opts = DiffOptions {
                    threshold,
                    semantic_diff: opts.semantic_diff,
                    ignore_regions: opts.ignore_regions.map(|regions| {
                        regions.into_iter().map(|r| crate::visual::compare::IgnoreRegion {
                            x: r.x,
                            y: r.y,
                            width: r.width,
                            height: r.height,
                            reason: r.reason,
                        }).collect()
                    }),
                    pixel_threshold: Some(threshold),
                };

                let result = compare_simd(&current, &baseline, &diff_opts);

                if !result.passed {
                    return Err(Error::from_reason(format!(
                        "Screenshot mismatch for '{}': {}% different (threshold: {}%)",
                        name, result.diff_percentage, threshold * 100.0
                    )));
                }
            }

            #[cfg(not(feature = "image"))]
            {
                // Without image feature, just compare sizes
                if screenshot_bytes.len() != baseline_bytes.len() {
                    return Err(Error::from_reason(format!(
                        "Screenshot mismatch for '{}': size changed from {} to {} bytes",
                        name, baseline_bytes.len(), screenshot_bytes.len()
                    )));
                }
            }
        } else {
            // First time - save baseline
            if let Some(parent) = baseline_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    Error::from_reason(format!("Failed to create screenshots directory: {}", e))
                })?;
            }
            std::fs::write(baseline_path, &screenshot_bytes).map_err(|e| {
                Error::from_reason(format!("Failed to save baseline: {}", e))
            })?;
            tracing::info!("Saved baseline screenshot for '{}'", name);
        }

        Ok(())
    }

    #[napi]
    pub async fn to_have_no_accessibility_violations(
        &self,
        options: Option<AccessibilityViolationOptions>,
        _config: Option<MatcherConfig>,
    ) -> Result<()> {
        let opts = options.unwrap_or_default();
        let violations = opts.violations.unwrap_or_default();

        if violations.is_empty() {
            Ok(())
        } else {
            let filtered: Vec<_> = violations.iter().filter(|v| {
                if let Some(ref severity) = opts.severity {
                    severity.contains(&v.impact)
                } else {
                    true
                }
            }).collect();

            if filtered.is_empty() {
                Ok(())
            } else {
                let msg = filtered.iter()
                    .map(|v| format!("- {} ({}): {}", v.id, v.impact, v.description))
                    .collect::<Vec<_>>()
                    .join("\n");
                Err(Error::from_reason(format!("Expected no accessibility violations, but found:\n{}", msg)))
            }
        }
    }
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct ScreenshotAssertionOptions {
    pub threshold: Option<f64>,
    pub animations: Option<String>,
    pub full_page: Option<bool>,
    pub omit_background: Option<bool>,
    pub semantic_diff: Option<bool>,
    pub ignore_regions: Option<Vec<IgnoreRegion>>,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct IgnoreRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub reason: Option<String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct AccessibilityViolationOptions {
    pub violations: Option<Vec<AccessibilityViolation>>,
    pub severity: Option<Vec<String>>,
    pub rules: Option<std::collections::HashMap<String, String>>,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct AccessibilityViolation {
    pub id: String,
    pub impact: String,
    pub description: String,
    pub help_url: String,
}

#[napi]
pub fn expect(_env: Env, target: napi::Unknown<'_>) -> Result<JsExpect> {
    let target_type = target.get_type()?;
    
    if target_type == napi::ValueType::Function {
        let func: napi::JsFunction = target.try_into()?;
        let tsfn: ThreadsafeFunction<()> = func.create_threadsafe_function(
            |_ctx: ThreadSafeCallContext<()>| {
                Ok(())
            }
        )?;
        return Ok(JsExpect {
            target: Some(ExpectTarget::Function(Arc::new(tsfn))),
            negated: false,
        });
    }
    
    if target_type == napi::ValueType::Object {
        let obj: napi::JsObject = target.try_into()?;
        
        if obj.has_named_property("selector")? && obj.has_named_property("page_id")? {
            let selector: String = obj.get_named_property("selector")?;
            let page_id: String = obj.get_named_property("page_id")?;
            return Ok(JsExpect {
                target: Some(ExpectTarget::Locator(JsLocator { selector, page_id })),
                negated: false,
            });
        }
        
        if obj.has_named_property("browser_id")? && obj.has_named_property("context_id")? && obj.has_named_property("page_id")? {
            let browser_id: String = obj.get_named_property("browser_id")?;
            let context_id: String = obj.get_named_property("context_id")?;
            let page_id: String = obj.get_named_property("page_id")?;
            return Ok(JsExpect {
                target: Some(ExpectTarget::Page(JsPage { browser_id, context_id, page_id })),
                negated: false,
            });
        }
    }
    
    Err(Error::from_reason("Invalid target for expect()"))
}

#[napi]
pub async fn expect_poll(
    #[napi(ts_arg_type = "() => Promise<any>")] fn_to_poll: ThreadsafeFunction<()>,
    config: Option<MatcherConfig>,
) -> Result<()> {
    let mut engine = AssertionEngine::default();
    if let Some(cfg) = config {
        if let Some(t) = cfg.timeout {
            engine.timeout = Duration::from_millis(t as u64);
        }
        if let Some(p) = cfg.polling_interval {
            engine.initial_interval = Duration::from_millis(p as u64);
        }
    }

    engine.poll(|| async {
        let (tx, rx) = tokio::sync::oneshot::channel::<Result<(), String>>();
        
        let status = fn_to_poll.call_with_return_value(
            Ok(()),
            ThreadsafeFunctionCallMode::NonBlocking,
            move |_result: Result<napi::Unknown<'_>>, _env: Env| {
                let _ = tx.send(Ok(()));
                Ok(())
            }
        );
        
        if status != napi::Status::Ok {
            return Err("Failed to call polling function".to_string());
        }
        
        match rx.await {
            Ok(Ok(_)) => Ok(()),
            _ => Err("Polling function rejected or failed".to_string()),
        }
    }).await.map_err(|e| Error::from_reason(e.to_string()))
}
