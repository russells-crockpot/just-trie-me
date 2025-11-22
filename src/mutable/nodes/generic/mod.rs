use super::{MutableTrieNode, MutableTrieNodeBuilder};
use crate::{
    Result,
    tokenization::{BoundaryTokenizer, Tokenizer},
};
use educe::Educe;
use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    marker::PhantomData,
    ops::Deref,
};

mod char;
#[feature("regex")]
mod regex;
mod string;

pub use char::*;
#[feature("regex")]
pub use regex::*;
pub use string::*;

#[derive(Clone)]
pub struct GenericTrieNode<K, V>
where
    K: NodeKey,
{
    // public in the crate for testing purposes
    pub(crate) key: K,
    pub(crate) value: Option<V>,
    pub(crate) children: HashMap<String, Box<GenericTrieNode<K, V>>>,
}

impl<K, V> GenericTrieNode<K, V>
where
    K: NodeKey,
{
    pub fn new<S: AsRef<str>>(key: S) -> Result<Self> {
        Ok(Self {
            key: K::new(key)?,
            value: None,
            children: HashMap::new(),
        })
    }

    pub fn get_all<S: AsRef<str>>(&self, tokens: &[S]) -> Vec<&V> {
        let mut items = Vec::new();
        if tokens.is_empty() {
            return items;
        }
        let token = tokens[0].as_ref();
        if !self.token_is_match(token) {
            return items;
        }
        // first check end/exit conditions against this token.
        if self.value.is_some() {
            items.push(self.value.as_ref().unwrap());
        }
        let remaining_tokens = &tokens[1..];
        for child in self.children.values() {
            items.extend(child.get_all(remaining_tokens));
        }
        items
    }

    pub fn get_any<S: AsRef<str>>(&self, tokens: &[S]) -> Option<&V> {
        let token = tokens[0].as_ref();
        // first check end/exit conditions against this token.
        if !self.token_is_match(token) {
            return None;
        // the token is a match, so we check to see if it's the last token or if there's a value
        } else if tokens.len() == 1 || self.value.is_some() {
            return self.value.as_ref();
        }
        let remaining_tokens = &tokens[1..];
        for child in self.children.values() {
            let value = child.get_any(remaining_tokens);
            if value.is_some() {
                return value;
            }
        }
        None
    }

    #[inline]
    pub fn token_is_match<S: AsRef<str>>(&self, value: S) -> bool {
        self.key.is_match(value.as_ref())
    }

    pub fn is_match<S: AsRef<str>>(&self, tokens: &[S]) -> bool {
        self.get_any(tokens).is_some()
    }
}

impl<K, V> MutableTrieNode<V> for GenericTrieNode<K, V>
where
    K: NodeKey,
{
    fn get_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self> {
        self.children
            .values()
            .filter(|c| c.token_is_match(token.as_ref()))
            .map(|b| b.as_ref())
            .collect()
    }

    fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    #[inline]
    fn len(&self) -> usize {
        self.children.len()
    }

    fn len_recursive(&self) -> usize {
        self.len()
            + self
                .children
                .values()
                .map(|n| n.len_recursive())
                .sum::<usize>()
    }
}

impl<K, V> MutableTrieNodeBuilder<V> for GenericTrieNode<K, V>
where
    K: NodeKey,
{
    type Node = Self;

    fn build(self) -> Result<Self::Node> {
        Ok(self)
    }

    fn add<S, I>(&mut self, mut items_iter: I, value: V) -> Result<()>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let pattern = if let Some(part) = items_iter.next() {
            String::from(part.as_ref())
        } else {
            self.value = Some(value);
            return Ok(());
        };
        if !self.children.contains_key(&pattern) {
            let child = GenericTrieNode::new(&pattern)?;
            self.children.insert(pattern.clone(), Box::new(child));
        }
        self.children
            .get_mut(&pattern)
            .unwrap()
            .add(items_iter, value)
    }
}

impl<K, V> fmt::Debug for GenericTrieNode<K, V>
where
    K: NodeKey,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TieNode")
            .field("key", &self.key)
            .field("value", &self.value)
            .field("children", &self.children)
            .finish()
    }
}

pub trait NodeKey: Sized + Clone + fmt::Debug + PartialEq<str> {
    fn new<S: AsRef<str>>(key: S) -> Result<Self>;
    fn is_match<S: AsRef<str>>(&self, value: S) -> bool;
}
