use crate::Result;

mod char;
mod generic;
#[feature("regex")]
mod regex;
mod string;

pub use char::*;
pub use generic::*;
#[feature("regex")]
pub use regex::*;
pub use string::*;

pub trait MutableTrieNodeBuilder<V> {
    type Node: MutableTrieNode<V>;

    fn add<S, I>(&mut self, items_iter: I, value: V) -> Result<()>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>;

    fn build(self) -> Result<Self::Node>;
}

pub trait MutableTrieNode<V> {
    fn value(&self) -> Option<&V>;

    fn get_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self>;

    fn len(&self) -> usize;
    fn len_recursive(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the first child that matches the given token.
    fn get_child<S: AsRef<str>>(&self, token: S) -> Option<&Self> {
        self.get_children(token).into_iter().next()
    }

    fn get_any<S: AsRef<str>>(&self, tokens: &[S]) -> Option<&V> {
        let mut child = self;
        for token in tokens {
            if let Some(value) = child.value() {
                return Some(value);
            //FIXME should use get children?
            } else if let Some(next_child) = child.get_child(token) {
                child = next_child;
            } else {
                return None;
            }
        }
        child.value()
    }

    fn get_all<S: AsRef<str>>(&self, tokens: &[S]) -> Vec<&V> {
        let mut values = Vec::new();
        if let Some(token) = tokens.first() {
            for child in self.get_children(token) {
                if let Some(value) = child.value() {
                    values.push(value)
                }
                values.extend(child.get_all(&tokens[1..]));
            }
        }
        values
    }
}
