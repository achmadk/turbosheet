use regex::Regex;

#[derive(Clone)]
pub struct UrlPattern {
    pattern: String,
    regex: Regex,
}

impl UrlPattern {
    pub fn new(pattern: &str) -> Result<Self, String> {
        let regex_str = Self::glob_to_regex(pattern);
        let regex = Regex::new(&regex_str).map_err(|e| e.to_string())?;
        
        Ok(Self {
            pattern: pattern.to_string(),
            regex,
        })
    }

    pub fn matches(&self, url: &str) -> bool {
        self.regex.is_match(url)
    }

    fn glob_to_regex(glob: &str) -> String {
        let mut regex = String::from("^");
        let mut chars = glob.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '*' => {
                    if let Some(&'*') = chars.peek() {
                        chars.next();
                        regex.push_str(".*");
                    } else {
                        regex.push_str("[^/]*");
                    }
                }
                '?' => regex.push('.'),
                '.' | '+' | '(' | ')' | '|' | '^' | '$' | '[' | ']' | '{' | '}' | '\\' => {
                    regex.push('\\');
                    regex.push(c);
                }
                _ => regex.push(c),
            }
        }

        regex.push('$');
        regex
    }
}
