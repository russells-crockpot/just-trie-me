use crate::{
    Error, Result,
    tokenization::{BoundaryTokenizer, Tokenizer, WhitespaceTokenizer},
};
use educe::Educe;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt,
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
};

pub mod nodes;
use nodes::*;

pub struct ImmutableTrieBuilder<B, V, T = WhitespaceTokenizer>
where
    B: ImmutableTrieNodeBuilder<V>,
    T: Tokenizer,
{
    tokenizer: T,
    builder: B,
    _spooky: PhantomData<V>,
}

impl<B, V, T> Default for ImmutableTrieBuilder<B, V, T>
where
    B: ImmutableTrieNodeBuilder<V> + Default,
    T: Tokenizer + Default,
{
    #[inline]
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}

impl<B, V, T> ImmutableTrieBuilder<B, V, T>
where
    B: ImmutableTrieNodeBuilder<V> + Default,
    T: Tokenizer,
{
    #[inline]
    pub fn with_tokenizer(tokenizer: T) -> Self {
        Self::new(tokenizer, Default::default())
    }
}

impl<B, V, T> ImmutableTrieBuilder<B, V, T>
where
    B: ImmutableTrieNodeBuilder<V>,
    T: Tokenizer + Default,
{
    #[inline]
    pub fn with_builder(builder: B) -> Self {
        Self::new(Default::default(), builder)
    }
}

impl<B, V, T> ImmutableTrieBuilder<B, V, T>
where
    B: ImmutableTrieNodeBuilder<V>,
    T: Tokenizer,
{
    pub fn new(tokenizer: T, builder: B) -> Self {
        Self {
            tokenizer,
            builder,
            _spooky: PhantomData,
        }
    }

    pub fn add<S: AsRef<str>>(&mut self, key: S, value: V) -> Result<()> {
        let tokens = self.tokenizer.tokenize(key);
        self.builder.add(tokens.into_iter(), value)
    }

    pub fn add_tokens<S, I>(&mut self, tokens: I, value: V) -> Result<()>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        self.builder.add(tokens.into_iter(), value)
    }

    pub fn build<TT: Tokenizer>(self, trie_tokenizer: TT) -> Result<ImmutableTrie<B::Node, V, TT>> {
        let root = self.builder.build()?;
        Ok(ImmutableTrie::new(trie_tokenizer, root))
    }

    pub fn build_default<TT: Tokenizer + Default>(self) -> Result<ImmutableTrie<B::Node, V, TT>> {
        let root = self.builder.build()?;
        Ok(ImmutableTrie::new(Default::default(), root))
    }
}

#[derive(Clone, Educe)]
#[educe(Debug)]
pub struct ImmutableTrie<N, V, T = BoundaryTokenizer>
where
    N: ImmutableTrieNode<V>,
    T: Tokenizer,
{
    #[educe(Debug(ignore))]
    tokenizer: T,
    root: N,
    #[educe(Debug(ignore))]
    _spooky: PhantomData<V>,
}

impl<N, V, T> ImmutableTrie<N, V, T>
where
    N: ImmutableTrieNode<V>,
    T: Tokenizer,
{
    pub fn new(tokenizer: T, root: N) -> Self {
        Self {
            tokenizer,
            root,
            _spooky: PhantomData,
        }
    }

    pub fn find_any<S: AsRef<str>>(&self, search_str: S) -> Option<&V> {
        let tokens = self.tokenizer.tokenize(search_str.as_ref());
        for i in 0..tokens.len() {
            if let Some(value) = self.root.get_any(&tokens[i..]) {
                return Some(value);
            }
        }
        None
    }

    pub fn find_all<S: AsRef<str>>(&self, search_str: S) -> Vec<&V> {
        let tokens = self.tokenizer.tokenize(search_str.as_ref());
        log::trace!("find_all tokens: {tokens:?}");
        let mut found = Vec::new();
        for i in 0..tokens.len() {
            found.extend(self.root.get_all(&tokens[i..]));
        }
        found
    }

    #[inline]
    pub fn root(&self) -> &N {
        &self.root
    }
}

impl<N, V, T> ImmutableTrie<N, V, T>
where
    N: ImmutableTrieNode<V>,
    V: Hash + Eq,
    T: Tokenizer,
{
    #[inline]
    pub fn find_unique<S: AsRef<str>>(&self, search_str: S) -> HashSet<&V> {
        self.find_all(search_str).into_iter().collect()
    }
}

impl<N, V, T> ImmutableTrie<N, V, T>
where
    N: ImmutableTrieNode<V>,
    V: Ord,
    T: Tokenizer,
{
    #[inline]
    pub fn find_unique_sorted<S: AsRef<str>>(&self, search_str: S) -> BTreeSet<&V> {
        self.find_all(search_str).into_iter().collect()
    }
}

impl<N, T> ImmutableTrie<N, bool, T>
where
    N: ImmutableTrieNode<bool>,
    T: Tokenizer,
{
    #[inline]
    pub fn has_match<S: AsRef<str>>(&self, value: S) -> bool {
        *self.find_any(value).unwrap_or(&false)
    }
}

impl<N, V, T> Default for ImmutableTrie<N, V, T>
where
    N: ImmutableTrieNode<V> + Default,
    T: Tokenizer + Default,
{
    fn default() -> Self {
        Self {
            tokenizer: Default::default(),
            root: Default::default(),
            _spooky: PhantomData,
        }
    }
}

pub type StringTrie<V, T = BoundaryTokenizer> = ImmutableTrie<StringTrieNode<V>, V, T>;
pub type StringTrieBuilder<V, T = WhitespaceTokenizer> =
    ImmutableTrieBuilder<StringTrieNode<V>, V, T>;
pub type StringMatcher<T = BoundaryTokenizer> = StringTrie<bool, T>;
pub type StringMatcherBuilder<T = WhitespaceTokenizer> =
    ImmutableTrieBuilder<StringTrieNode<bool>, bool, T>;

#[feature("regex-filtered")]
pub type RegexTrie<V, T = BoundaryTokenizer> = ImmutableTrie<RegexFilteredTrieNode<V>, V, T>;
#[feature("regex-filtered")]
pub type RegexTrieBuilder<V, T = WhitespaceTokenizer> =
    ImmutableTrieBuilder<RegexFilteredTrieNodeBuilder<V>, V, T>;
#[feature("regex-filtered")]
pub type RegexMatcher<T = BoundaryTokenizer> = RegexTrie<bool, T>;
#[feature("regex-filtered")]
pub type RegexMatcherBuilder<T = WhitespaceTokenizer> = RegexTrieBuilder<bool, T>;

//#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_matcher_has_match() {
        let mut trie_builder: StringMatcherBuilder = Default::default();
        trie_builder.add("bobby", true).unwrap();
        trie_builder.add("mister bobby", true).unwrap();
        let trie: StringMatcher = trie_builder.build_default().unwrap();
        assert!(trie.has_match("bobby"));
        assert!(trie.has_match("mister bobby"));
        assert!(!trie.has_match("bobbys"));
        assert!(!trie.has_match("mister bobbys"));
        assert!(trie.has_match("the mister bobby"));
        assert!(trie.has_match("mister the bobby"));
        assert!(!trie.has_match("mister the baby"));
        assert!(!trie.has_match("mister baby"));
    }

    #[test]
    fn test_regex_trie_conflict_1() {
        let mut trie_builder: RegexTrieBuilder<&str> = Default::default();
        trie_builder
            .add("(a|the|slumber|pool) party", "val 1")
            .unwrap();
        trie_builder.add("pool", "val 2").unwrap();
        let trie: RegexTrie<&str> = trie_builder.build_default().unwrap();
        let results = trie.find_unique("pool");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"val 2"));
        let results = trie.find_unique("a pool");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"val 2"));
        let results = trie.find_unique("a party");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"val 1"));
    }

    //FIXME
    //#[test]
    //fn test_regex_matcher_add_value() {
    //    let mut builder = RegexMatcherBuilder::default();
    //    builder.add("Test Name", true).unwrap();
    //    let matcher: RegexMatcher<BoundaryTokenizer> = builder.build_default().unwrap();
    //    assert_eq!(matcher.root.children.len(), 1);
    //    let first_node = &matcher.root.children[0];
    //    assert_eq!(first_node.key.as_str(), "^test$");
    //    assert_eq!(first_node.children.len(), 1);
    //    assert!(first_node.value.is_none());
    //    let maybe_second_node = first_node.children.get("name");
    //    assert!(maybe_second_node.is_some());
    //    let second_node = maybe_second_node.unwrap();
    //    assert_eq!(second_node.key.as_str(), "^name$");
    //    assert_eq!(second_node.children.len(), 0);
    //    assert!(matches!(second_node.value, Some(true)));
    //}

    #[test]
    fn test_string_trie_find() {
        let mut trie_builder: StringTrieBuilder<&str> = StringTrieBuilder::default();
        trie_builder.add("test value", "v1").unwrap();
        trie_builder.add("another test value", "v2").unwrap();
        trie_builder.add("something else", "v3").unwrap();
        trie_builder.add("another something else", "v3").unwrap();
        let trie: StringTrie<&str> = trie_builder.build_default().unwrap();
        let all = trie.find_all("this is a test value");
        assert_eq!(all, vec![&"v1"]);
        let mut all = trie.find_all("this is a another test value");
        all.sort();
        assert_eq!(all, vec![&"v1", &"v2"]);
        let all = trie.find_all("another something else");
        assert_eq!(all, vec![&"v3", &"v3"]);
        let all = trie.find_all("nothing");
        assert!(all.is_empty());
        let all = trie.find_all("");
        assert!(all.is_empty());
    }

    #[test]
    fn test_string_trie_find_unique() {
        let mut trie_builder: StringTrieBuilder<&str> = StringTrieBuilder::default();
        trie_builder.add("test value", "v1").unwrap();
        trie_builder.add("another test value", "v2").unwrap();
        trie_builder.add("something else", "v3").unwrap();
        trie_builder.add("another something else", "v3").unwrap();
        let trie: StringTrie<&str> = trie_builder.build_default().unwrap();
        let all = trie.find_unique("this is a test value");
        assert_eq!(all.len(), 1);
        assert!(all.contains(&"v1"));
        let mut all = trie.find_unique("this is a another test value");
        assert_eq!(all.len(), 2);
        assert!(all.contains(&"v1"));
        assert!(all.contains(&"v2"));
        let all = trie.find_unique("another something else");
        assert_eq!(all.len(), 1);
        assert!(all.contains(&"v3"));
        let all = trie.find_unique("nothing");
        assert!(all.is_empty());
        let all = trie.find_unique("");
        assert!(all.is_empty());
    }
}
