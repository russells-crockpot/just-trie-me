use std::{ops::Deref as _, sync::LazyLock};

#[cfg(feature = ("boundary-tokenizer"))]
mod boundary;
#[cfg(feature = ("boundary-tokenizer"))]
pub use boundary::*;

const WHITESPACE_CHARS: &[char] = &[' ', '\t', '\n', '\r'];

pub trait Tokenizer {
    fn tokenize<S: AsRef<str>>(&self, s: S) -> Vec<String>;
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
pub struct NoOpTokenizer;

impl NoOpTokenizer {
    pub fn tokenize<S: AsRef<str>>(s: S) -> Vec<String> {
        vec![s.as_ref().into()]
    }
}

impl Tokenizer for NoOpTokenizer {
    #[inline]
    fn tokenize<S: AsRef<str>>(&self, s: S) -> Vec<String> {
        Self::tokenize(s)
    }
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
