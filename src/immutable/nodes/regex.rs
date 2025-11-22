use super::{ImmutableTrieNode, ImmutableTrieNodeBuilder};
use crate::Result;
use educe::Educe;
use regex::{Regex, RegexBuilder, RegexSet, RegexSetBuilder};
use std::{
    borrow::BorrowMut,
    collections::{BTreeSet, HashMap},
    fmt,
    marker::PhantomData,
    ops::Deref,
};
use triomphe::Arc;

#[derive(Clone)]
pub struct RegexSetTrieNode<V> {
    value: Option<V>,
    patterns: RegexSet,
    children: Vec<Box<RegexSetTrieNode<V>>>,
}

impl<V> ImmutableTrieNode<V> for RegexSetTrieNode<V> {
    fn get_child<S: AsRef<str>>(&self, token: S) -> Option<&Self> {
        self.patterns
            .matches(token.as_ref())
            .iter()
            .next()
            .map(|idx| self.children[idx].as_ref())
    }

    fn get_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self> {
        let mut matches: Vec<_> = self.patterns.matches(token.as_ref()).iter().collect();
        if matches.is_empty() {
            Vec::default()
        } else {
            matches.sort();
            matches
                .into_iter()
                .map(|idx| self.children[idx].as_ref())
                .collect()
        }
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
                .iter()
                .map(|n| n.len_recursive())
                .sum::<usize>()
    }
}

pub struct RegexSetTrieNodeBuilder<V> {
    value: Option<V>,
    children: HashMap<String, Box<RegexSetTrieNodeBuilder<V>>>,
}

impl<V> Default for RegexSetTrieNodeBuilder<V> {
    fn default() -> Self {
        Self {
            value: None,
            children: HashMap::new(),
        }
    }
}

impl<V> ImmutableTrieNodeBuilder<V> for RegexSetTrieNodeBuilder<V> {
    type Node = RegexSetTrieNode<V>;

    fn add<S, I>(&mut self, mut items_iter: I, value: V) -> Result<()>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let pattern = if let Some(part) = items_iter.next() {
            format!("^{}$", part.as_ref())
        } else {
            self.value = Some(value);
            return Ok(());
        };
        if !self.children.contains_key(&pattern) {
            let child = Self::default();
            self.children.insert(pattern.clone(), Box::new(child));
        }
        self.children
            .get_mut(&pattern)
            .unwrap()
            .add(items_iter, value)
    }

    fn build(self) -> Result<Self::Node> {
        let mut children = Vec::with_capacity(self.children.len());
        let mut patterns = Vec::with_capacity(self.children.len());
        for (pattern, child) in self.children.into_iter() {
            patterns.push(pattern);
            let child = child.build()?;
            children.push(Box::new(child));
        }
        let regexes = RegexSetBuilder::new(patterns)
            .unicode(true)
            .case_insensitive(true)
            .build()?;
        Ok(RegexSetTrieNode {
            value: self.value,
            patterns: regexes,
            children,
        })
    }
}
