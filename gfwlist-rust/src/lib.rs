#![doc = include_str!("../README.md")]

use aho_corasick::AhoCorasick;
use regex::Regex;
use url::Url;

mod constants {
    /// Marker byte for the beginning of a URL scheme
    pub const BEGIN_OF_SCHEME: u8 = 0x01;
    /// Marker byte for the beginning of a host
    pub const BEGIN_OF_HOST: u8 = 0x02;
    /// Marker byte for the beginning of a path
    pub const BEGIN_OF_PATH: u8 = 0x03;
    /// Delimiter byte for host components
    pub const HOST_DELIMITER: u8 = b'.';
    /// Delimiter byte for path components
    pub const PATH_DELIMITER: u8 = b'/';
}

/// Errors that can occur when building a GfwList.
#[derive(Debug)]
pub enum BuildError<'i> {
    /// Error related to syntax issues in a rule
    Syntax(&'i str, SyntaxError),
    /// Error from the Aho-Corasick algorithm during pattern compilation
    AhoCorasick(aho_corasick::BuildError),
}

/// Specific syntax errors encountered during GfwList parsing
#[derive(Debug)]
pub enum SyntaxError {
    /// General rule syntax error
    Rule(),
    /// Error in a regular expression
    Regex(regex::Error),
    /// Error parsing a URL
    Url(url::ParseError),
}

/// `GfwList` represents a compiled set of rules for matching URLs.
///
/// It uses Aho-Corasick for fast pattern matching and regular expressions
/// for more complex matching. Rules can be either positive (block) or
/// negative (allow) patterns.
#[derive(Debug)]
pub struct GfwList {
    positive_ac: AhoCorasick,
    negative_ac: AhoCorasick,
    positive_rules: Vec<String>,
    regex_patterns: Vec<(Regex, String)>,
}

fn append_host(acc: &mut Vec<u8>, host: &[u8]) {
    if !host.starts_with(&[constants::HOST_DELIMITER]) {
        acc.push(constants::HOST_DELIMITER);
    }
    acc.extend(host);
}

fn append_path(acc: &mut Vec<u8>, path: &[u8]) {
    acc.extend(path);
    if !path.ends_with(&[constants::PATH_DELIMITER]) {
        acc.push(constants::PATH_DELIMITER);
    }
}

fn append_host_path(acc: &mut Vec<u8>, input: &[u8]) {
    let pos = input
        .iter()
        .position(|&b| b == constants::PATH_DELIMITER)
        .unwrap_or(input.len());
    append_host(acc, &input[..pos]);
    acc.push(constants::BEGIN_OF_PATH);
    append_path(acc, &input[pos..]);
}

fn append_url<const FULL_MODE: bool>(acc: &mut Vec<u8>, input: &str) -> Result<(), url::ParseError> {
    let url = Url::parse(input)?;
    let host_str = url.host_str().ok_or(url::ParseError::EmptyHost)?;
    acc.push(constants::BEGIN_OF_SCHEME);
    acc.extend(url.scheme().as_bytes());
    acc.push(constants::BEGIN_OF_HOST);
    append_host(acc, host_str.as_bytes());
    let path = url.path();
    if FULL_MODE || path != "/" || input.ends_with('/') {
        acc.push(constants::BEGIN_OF_PATH);
        append_path(acc, url.path().as_bytes());
    }
    Ok(())
}

impl GfwList {
    /// Constructs a new `GfwList` from a string containing GFW list rules.
    ///
    /// The input string should follow the GFW list format, with each line containing
    /// a rule. Rules can be:
    /// - Regular expressions: `/pattern/`
    /// - Negative patterns: `@@pattern` (whitelist)
    /// - Positive patterns: `pattern` (blacklist)
    /// - Patterns with different formats: `.example.com`, `||example.com`, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gfwlist::GfwList;
    /// let list_content = "\
    ///     ||blocked-site.com\n\
    ///     @@||exception.com\n\
    ///     /regex-pattern/\n\
    /// ";
    /// let gfw_list = GfwList::from(list_content).unwrap();
    /// ```
    pub fn from(input: &str) -> Result<Self, BuildError> {
        let mut positive_rules: Vec<String> = vec![];
        let mut negative_rules: Vec<String> = vec![];
        let mut positive_patterns: Vec<Vec<u8>> = vec![];
        let mut negative_patterns: Vec<Vec<u8>> = vec![];
        let mut regex_patterns: Vec<(Regex, String)> = vec![];
        // split the source into lines
        for mut line_str in input.lines() {
            // skip empty lines and comments
            if line_str.is_empty() || line_str.starts_with('!') {
                continue;
            }
            if line_str.starts_with('/') {
                if line_str.len() == 1 || !line_str.ends_with('/') {
                    return Err(BuildError::Syntax(line_str, SyntaxError::Rule()));
                }
                let regex = Regex::new(&line_str[1..line_str.len() - 1])
                    .map_err(|e| BuildError::Syntax(line_str, SyntaxError::Regex(e)))?;
                regex_patterns.push((regex, line_str.to_string()));
                continue;
            }
            let (patterns, rules) = if line_str.starts_with("@@") {
                line_str = &line_str[2..];
                (&mut negative_patterns, &mut negative_rules)
            } else {
                (&mut positive_patterns, &mut positive_rules)
            };
            let line = line_str.as_bytes();
            let mut needle: Vec<u8> = vec![];
            if line[0] == b'.' {
                append_host_path(&mut needle, &line[1..]);
            } else if line[0] != b'|' {
                needle.push(constants::BEGIN_OF_HOST);
                append_host_path(&mut needle, line);
            } else if line.get(1) == Some(&b'|') {
                append_host_path(&mut needle, &line[2..]);
            } else {
                append_url::<false>(&mut needle, &line_str[1..])
                    .map_err(|e| BuildError::Syntax(line_str, SyntaxError::Url(e)))?;
            }
            patterns.push(needle);
            rules.push(line_str.to_string());
        }
        Ok(GfwList {
            positive_ac: AhoCorasick::new(positive_patterns).map_err(BuildError::AhoCorasick)?,
            negative_ac: AhoCorasick::new(negative_patterns).map_err(BuildError::AhoCorasick)?,
            positive_rules,
            regex_patterns,
        })
    }

    /// Tests whether a URL matches any rule in the GfwList.
    ///
    /// The test follows these steps:
    /// 1. Check if the URL matches any regex pattern
    /// 2. Check if the URL matches any negative (whitelist) pattern
    /// 3. Check if the URL matches any positive (blacklist) pattern
    ///
    /// Returns `Some(rule)` if the URL matches a rule, `None` if it doesn't match any rules.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gfwlist::GfwList;
    /// let list_content = "||blocked-site.com\n@@||exception.com";
    /// let gfw_list = GfwList::from(list_content).unwrap();
    /// assert_eq!(gfw_list.test("http://blocked-site.com/page").unwrap(), Some("||blocked-site.com"));
    /// assert_eq!(gfw_list.test("http://exception.com/page").unwrap(), None);
    /// assert_eq!(gfw_list.test("http://allowed-site.com/page").unwrap(), None);
    /// ```
    pub fn test(&self, input: &str) -> Result<Option<&str>, url::ParseError> {
        for (regex, rule) in &self.regex_patterns {
            if regex.is_match(input) {
                return Ok(Some(rule));
            }
        }
        let mut haystack: Vec<u8> = vec![];
        append_url::<true>(&mut haystack, input)?;
        if self.negative_ac.find(&haystack).is_some() {
            return Ok(None);
        }
        if let Some(match_) = self.positive_ac.find(&haystack) {
            return Ok(Some(&self.positive_rules[match_.pattern().as_usize()]));
        }
        Ok(None)
    }

    /// Returns the number of rules in the GfwList.
    ///
    /// This includes the number of positive patterns, negative patterns,
    /// and regex patterns.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gfwlist::GfwList;
    /// let list_content = "||blocked-site.com\n@@||exception.com";
    /// let gfw_list = GfwList::from(list_content).unwrap();
    /// assert_eq!(gfw_list.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.positive_ac.patterns_len() + self.negative_ac.patterns_len() + self.regex_patterns.len()
    }

    /// Checks if the GfwList is empty.
    ///
    /// This includes the number of positive patterns, negative patterns,
    /// and regex patterns.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gfwlist::GfwList;
    /// let list_content = "\n\n";
    /// let gfw_list = GfwList::from(list_content).unwrap();
    /// assert_eq!(gfw_list.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// https://github.com/gfwlist/gfwlist/wiki/Syntax
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1() {
        let gfw = GfwList::from("|http://example.com").unwrap();

        assert!(gfw.test("http://example.com").unwrap().is_some());
        assert!(gfw.test("http://example.com/page").unwrap().is_some());
        assert!(gfw.test("http://example.com.co").unwrap().is_some());

        assert!(gfw.test("http://www.example.com").unwrap().is_none());
        assert!(gfw.test("https://example.com/page").unwrap().is_none());
        assert!(gfw.test("https://example.com").unwrap().is_none());
        assert!(gfw.test("https://www.example.com").unwrap().is_none());
        assert!(gfw.test("https://example.com.co").unwrap().is_none());
    }

    #[test]
    fn test_2() {
        let gfw = GfwList::from("||example.com").unwrap();

        assert!(gfw.test("http://example.com").unwrap().is_some());
        assert!(gfw.test("http://www.example.com").unwrap().is_some());
        assert!(gfw.test("https://example.com").unwrap().is_some());
        assert!(gfw.test("https://www.example.com").unwrap().is_some());

        assert!(gfw.test("http://anotherexample.com").unwrap().is_none());
        assert!(gfw.test("https://anotherexample.com").unwrap().is_none());
        assert!(gfw.test("http://example.com.co").unwrap().is_none());
        assert!(gfw.test("https://example.com.co").unwrap().is_none());
    }

    #[test]
    fn test_3() {
        let gfw = GfwList::from(".example.com\n@@|http://sub.example.com").unwrap();

        assert!(gfw.test("http://example.com").unwrap().is_some());
        assert!(gfw.test("http://www.sub.example.com").unwrap().is_some());
        assert!(gfw.test("https://sub.example.com").unwrap().is_some());
        assert!(gfw.test("https://www.example.com").unwrap().is_some());

        assert!(gfw.test("http://sub.example.com").unwrap().is_none());
        assert!(gfw.test("http://sub.example.com/page").unwrap().is_none());
    }
}
