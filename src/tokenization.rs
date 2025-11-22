use convert_case::Boundary;
use regex::Regex;
use std::{ops::Deref as _, sync::LazyLock};

pub static DEFAULT_TOKENIZER: LazyLock<BoundaryTokenizer> = LazyLock::new(Default::default);

const WHITESPACE_CHARS: &[char] = &[' ', '\t', '\n', '\r'];

pub trait Tokenizer {
    fn tokenize<S: AsRef<str>>(&self, s: S) -> Vec<String>;
}

pub fn get_default_tokenizer() -> &'static BoundaryTokenizer {
    DEFAULT_TOKENIZER.deref()
}

const DEFAULT_TOKEN_BOUNDARIES: [Boundary; 32] = [
    Boundary::SPACE,
    Boundary::HYPHEN,
    Boundary::UNDERSCORE,
    Boundary::LOWER_UPPER,
    Boundary::ACRONYM,
    Boundary::LOWER_DIGIT,
    Boundary::UPPER_DIGIT,
    Boundary::DIGIT_LOWER,
    Boundary::DIGIT_UPPER,
    Boundary::from_delim("."),
    Boundary::from_delim(","),
    Boundary::from_delim("<"),
    Boundary::from_delim(">"),
    Boundary::from_delim("|"),
    Boundary::from_delim("["),
    Boundary::from_delim("]"),
    Boundary::from_delim("{"),
    Boundary::from_delim("}"),
    Boundary::from_delim("\t"),
    Boundary::from_delim(";"),
    Boundary::from_delim(":"),
    Boundary::from_delim("@"),
    Boundary::from_delim("&"),
    Boundary::from_delim("^"),
    Boundary::from_delim("$"),
    Boundary::from_delim("("),
    Boundary::from_delim(")"),
    Boundary::from_delim("+"),
    Boundary::from_delim("!"),
    Boundary::from_delim("?"),
    Boundary::from_delim("*"),
    Boundary::from_delim("#"),
];

#[derive(Debug, Clone)]
pub struct BoundaryTokenizer(Vec<Boundary>);

impl BoundaryTokenizer {
    #[inline]
    pub fn new<I>(boundaries: I) -> Self
    where
        I: IntoIterator<Item = Boundary>,
    {
        Self(boundaries.into_iter().collect())
    }
}

impl Default for BoundaryTokenizer {
    #[inline]
    fn default() -> Self {
        Self::new(DEFAULT_TOKEN_BOUNDARIES)
    }
}

impl Tokenizer for BoundaryTokenizer {
    fn tokenize<S: AsRef<str>>(&self, s: S) -> Vec<String> {
        convert_case::split(&s, &self.0)
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub struct WhitespaceTokenizer;

impl WhitespaceTokenizer {
    pub fn tokenize<S: AsRef<str>>(s: S) -> Vec<String> {
        s.as_ref()
            .trim()
            .split(WHITESPACE_CHARS)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    }
}

impl Tokenizer for WhitespaceTokenizer {
    #[inline]
    fn tokenize<S: AsRef<str>>(&self, s: S) -> Vec<String> {
        Self::tokenize(s)
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub struct DummyTokenizer;

impl DummyTokenizer {
    pub fn tokenize<S: AsRef<str>>(s: S) -> Vec<String> {
        vec![s.as_ref().into()]
    }
}

impl Tokenizer for DummyTokenizer {
    #[inline]
    fn tokenize<S: AsRef<str>>(&self, s: S) -> Vec<String> {
        Self::tokenize(s)
    }
}

#[inline]
pub fn tokenize<S: AsRef<str>>(s: S) -> Vec<String> {
    DEFAULT_TOKENIZER.tokenize(s)
}

//#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_whitespace_tokenizer() {
        assert_eq!(WhitespaceTokenizer::tokenize("test"), vec!["test"]);
        assert_eq!(WhitespaceTokenizer::tokenize("Test"), vec!["test"]);
        assert_eq!(WhitespaceTokenizer::tokenize("Test "), vec!["test"]);
        assert_eq!(
            WhitespaceTokenizer::tokenize("Test test"),
            vec!["test", "test"]
        );
        assert_eq!(
            WhitespaceTokenizer::tokenize("Test  test"),
            vec!["test", "test"]
        );
    }
}
