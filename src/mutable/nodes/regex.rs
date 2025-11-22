use super::MutableTrieNode;
use crate::Result;
use regex::{Regex, RegexBuilder};
use std::{borrow::BorrowMut, collections::HashMap, fmt, ops::Deref};

#[derive(Clone)]
pub struct RegexTrieNode<V> {
    value: Option<V>,
    children: HashMap<String, Box<Self>>,
}

impl<V> RegexTrieNode<V> {
    fn get_child_mut<S: AsRef<str>>(&mut self, token: S) -> Option<&mut Self> {
        self.children
            .iter_mut()
            .find(|(pat, _)| pat.is_match(token.as_ref()))
            .map(|(_, node)| node.borrow_mut())
    }
}

impl<V> MutableTrieNode<V> for RegexTrieNode<V> {
    fn add<S, I>(&mut self, mut items_iter: I, value: V) -> Result<()>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let pattern = if let Some(part) = items_iter.next() {
            format!(
                "^{}$",
                part.as_ref().trim_start_matches('^').trim_end_matches('$')
            )
        } else {
            self.value = Some(value);
            return Ok(());
        };
        let mut child = if let Some(child) = self.children.entry(pattern.as_str()) {
            child
        } else {
            let child = Self::default();
            let regex = RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .unicode(true)
                .build()?;
            self.children.insert(regex, Box::new(child));
            self.children.get_mut(idx).unwrap()
        };
        let mut child = if let Some(child) = self.get_child_mut(&key) {
            child
        } else {
            let child = Self::default();
            self.children
                .entry(&key)
                .or_default()
                .insert(key.clone(), Box::new(child));
            self.get_child_mut(&key).unwrap()
        };
        child.add(items_iter, value)
    }

    fn match_child<S: AsRef<str>>(&self, token: S) -> Option<&Self> {
        self.children
            .iter()
            .find(|(pat, _)| pat.is_match(token.as_ref()))
            .map(|(_, node)| node.as_ref())
    }

    fn match_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self> {
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
            children: Default::default(),
        }
    }
}
