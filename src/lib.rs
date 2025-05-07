use aho_corasick::AhoCorasick;
use url::Url;
use regex::Regex;

mod constants {
    pub const BEGIN_OF_SCHEME: u8 = 0x01;
    pub const BEGIN_OF_HOST: u8 = 0x02;
    pub const BEGIN_OF_PATH: u8 = 0x03;
    pub const HOST_DELIMITER: u8 = b'.';
    pub const PATH_DELIMITER: u8 = b'/';
}

#[derive(Debug)]
pub enum BuildError<'i> {
    Syntax(&'i str, SyntaxError),
    AhoCorasick(aho_corasick::BuildError),
}

#[derive(Debug)]
pub enum SyntaxError {
    Rule(),
    Regex(regex::Error),
    Url(url::ParseError),
}

#[derive(Debug)]
pub struct GfwList {
    positive_ac: AhoCorasick,
    negative_ac: AhoCorasick,
    regex_patterns: Vec<Regex>,
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
    let pos = input.iter().position(|&b| b == constants::PATH_DELIMITER).unwrap_or(input.len());
    append_host(acc, &input[..pos]);
    acc.push(constants::BEGIN_OF_PATH);
    append_path(acc, &input[pos..]);
}

fn append_url(acc: &mut Vec<u8>, input: &str) -> Result<(), url::ParseError> {
    let url = Url::parse(input)?;
    let host_str = url.host_str().ok_or(url::ParseError::EmptyHost)?;
    acc.push(constants::BEGIN_OF_SCHEME);
    acc.extend(url.scheme().as_bytes());
    acc.push(constants::BEGIN_OF_HOST);
    append_host(acc, host_str.as_bytes());
    acc.push(constants::BEGIN_OF_PATH);
    append_path(acc, url.path().as_bytes());
    Ok(())
}

impl GfwList {
    pub fn from(input: &str) -> Result<Self, BuildError> {
        let mut positive_patterns: Vec<Vec<u8>> = vec![];
        let mut negative_patterns: Vec<Vec<u8>> = vec![];
        let mut regex_patterns: Vec<Regex> = vec![];
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
                regex_patterns.push(regex);
                continue;
            }
            let patterns = if line_str.starts_with("@") {
                if line_str.len() == 1 || !line_str.starts_with("@@") {
                    return Err(BuildError::Syntax(line_str, SyntaxError::Rule()));
                }
                line_str = &line_str[2..];
                &mut negative_patterns
            } else {
                &mut positive_patterns
            };
            let line = line_str.as_bytes();
            let mut needle: Vec<u8> = vec![];
            if line[0] == b'.' {
                append_host_path(&mut needle, &line[1..]);
            } else if line[0] != b'|' {
                needle.push(constants::BEGIN_OF_HOST);
                append_host_path(&mut needle, &line);
            } else if line.get(1) == Some(&b'|') {
                append_host_path(&mut needle, &line[2..]);
            } else {
                append_url(&mut needle, line_str)
                    .map_err(|e| BuildError::Syntax(line_str, SyntaxError::Url(e)))?;
            }
            patterns.push(needle);
        }
        Ok(GfwList {
            positive_ac: AhoCorasick::new(positive_patterns).map_err(BuildError::AhoCorasick)?,
            negative_ac: AhoCorasick::new(negative_patterns).map_err(BuildError::AhoCorasick)?,
            regex_patterns,
        })
    }

    pub fn test(&self, input: &str) -> Result<bool, url::ParseError> {
        if self.regex_patterns.iter().any(|regex| regex.is_match(input)) {
            return Ok(true);
        }
        let mut haystack: Vec<u8> = vec![];
        append_url(&mut haystack, input)?;
        if self.negative_ac.find(&haystack).is_some() {
            return Ok(false);
        }
        if self.positive_ac.find(&haystack).is_some() {
            return Ok(true);
        }
        Ok(false)
    }
}
