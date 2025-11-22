use super::Tokenizer;
use convert_case::Boundary;

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

//impl Default for BoundaryTokenizer {
//    #[inline]
//    fn default() -> Self {
//        Self::new(DEFAULT_TOKEN_BOUNDARIES)
//    }
//}

impl Tokenizer for BoundaryTokenizer {
    fn tokenize<S: AsRef<str>>(&self, s: S) -> Vec<String> {
        convert_case::split(&s, &self.0)
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    }
}
