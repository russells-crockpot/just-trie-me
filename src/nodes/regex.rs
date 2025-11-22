use super::{TrieNode, TrieNodeBuilder};
use crate::Result;
use educe::Educe;
use regex::{Regex, RegexBuilder, RegexSet, RegexSetBuilder};
use regex_filtered::{Builder as RegexesBuilder, Options as RegexesOptions, Regexes};
use std::{
    borrow::BorrowMut,
    collections::{BTreeSet, HashMap},
    fmt,
    marker::PhantomData,
    ops::Deref,
};
use triomphe::Arc;

lazy_static::lazy_static! {
    static ref OPTIMIZED_REGEX_OPTS: RegexesOptions = {
        let mut opts = RegexesOptions::new();
        opts.case_insensitive(true);
        opts
    };
}

#[derive(Clone)]
pub struct RegexFilteredTrieNode<V> {
    value: Option<V>,
    patterns: Arc<Regexes>,
    pub(crate) children: Vec<Box<RegexFilteredTrieNode<V>>>,
}

impl<V: fmt::Debug> fmt::Debug for RegexFilteredTrieNode<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let children: HashMap<&str, &Box<Self>> = self
            .patterns
            .regexes()
            .iter()
            .map(|r| r.as_str())
            .zip(&self.children)
            .collect();
        f.debug_struct("RegexFilteredTrieNode")
            .field("value", &self.value)
            .field("children", &children)
            .finish()
    }
}

impl<V> TrieNode<V> for RegexFilteredTrieNode<V> {
    fn get_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self> {
        let mut matches: Vec<_> = self.patterns.matching(token.as_ref()).collect();
        if matches.is_empty() {
            Vec::default()
        } else {
            matches.sort_by(|(v1, _), (v2, _)| v1.cmp(v2));
            matches
                .into_iter()
                .map(|(idx, _)| self.children[idx].as_ref())
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

pub struct RegexFilteredTrieNodeBuilder<V> {
    value: Option<V>,
    children: HashMap<String, Box<RegexFilteredTrieNodeBuilder<V>>>,
}

impl<V> Default for RegexFilteredTrieNodeBuilder<V> {
    fn default() -> Self {
        Self {
            value: None,
            children: HashMap::new(),
        }
    }
}

impl<V> TrieNodeBuilder<V> for RegexFilteredTrieNodeBuilder<V> {
    type Node = RegexFilteredTrieNode<V>;

    fn add<S, I>(&mut self, mut items_iter: I, value: V) -> Result<()>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let pattern = if let Some(part) = items_iter.next() {
            //String::from(part.as_ref())
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
        let mut regexes_builder = RegexesBuilder::new();
        for (pattern, child) in self.children.into_iter() {
            regexes_builder = regexes_builder.push_opt(pattern.as_str(), &OPTIMIZED_REGEX_OPTS)?;
            let child = child.build()?;
            children.push(Box::new(child));
        }
        Ok(RegexFilteredTrieNode {
            value: self.value,
            patterns: Arc::new(regexes_builder.build()?),
            children,
        })
    }
}

#[derive(Clone)]
pub struct RegexSetTrieNode<V> {
    value: Option<V>,
    patterns: RegexSet,
    children: Vec<Box<RegexSetTrieNode<V>>>,
}

impl<V> TrieNode<V> for RegexSetTrieNode<V> {
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

impl<V> TrieNodeBuilder<V> for RegexSetTrieNodeBuilder<V> {
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

#[derive(Clone)]
pub struct RegexTrieNode<V> {
    value: Option<V>,
    children: Vec<(Regex, Box<Self>)>,
}

impl<V> RegexTrieNode<V> {
    fn get_child_mut<S: AsRef<str>>(&mut self, token: S) -> Option<&mut Self> {
        self.children
            .iter_mut()
            .find(|(pat, _)| pat.is_match(token.as_ref()))
            .map(|(_, node)| node.borrow_mut())
    }
}

impl<V> TrieNode<V> for RegexTrieNode<V> {
    fn get_child<S: AsRef<str>>(&self, token: S) -> Option<&Self> {
        self.children
            .iter()
            .find(|(pat, _)| pat.is_match(token.as_ref()))
            .map(|(_, node)| node.as_ref())
    }

    fn get_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self> {
        self.children
            .iter()
            .filter(|(pat, _)| pat.is_match(token.as_ref()))
            .map(|(_, node)| node.as_ref())
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
                .iter()
                .map(|(_, n)| n.len_recursive())
                .sum::<usize>()
    }
}

impl<V> Default for RegexTrieNode<V> {
    fn default() -> Self {
        Self {
            value: None,
            children: Vec::new(),
        }
    }
}

impl<V> TrieNodeBuilder<V> for RegexTrieNode<V> {
    type Node = RegexTrieNode<V>;

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
        let mut child = if let Some(child) = self.get_child_mut(&pattern) {
            child
        } else {
            let child = Self::default();
            let regex = RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .unicode(true)
                .build()?;
            self.children.push((regex, Box::new(child)));
            let idx = self.children.len() - 1;
            self.children
                .get_mut(idx)
                .map(|(_, node)| node.borrow_mut())
                .unwrap()
        };
        child.add(items_iter, value)
    }

    fn build(self) -> Result<Self::Node> {
        Ok(self)
    }
}
