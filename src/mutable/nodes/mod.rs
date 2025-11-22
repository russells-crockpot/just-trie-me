use crate::Result;

mod char;
mod generic;
#[cfg(feature = ("regex"))]
mod regex;
mod string;

pub use char::*;
pub use generic::*;
#[cfg(feature = ("regex"))]
pub use regex::*;
pub use string::*;

pub trait MutableTrieNode<V> {
    fn add<S, I>(&mut self, items_iter: I, value: V) -> Result<()>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>;

    fn value(&self) -> Option<&V>;

    fn len(&self) -> usize;
    fn len_recursive(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the first child that matches the given token.
    #[inline]
    fn match_child<S: AsRef<str>>(&self, token: S) -> Option<&Self> {
        self.match_children(token).into_iter().next()
    }

    fn match_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self>;

    /*
    /// Gets the child node with the given key. This is different from [`match_child`] because this
    /// one is meant to get the child that matches EXACTLY. For example, if you're using a regex
    /// and a child node has the key `^t.*t$` then `test` would _match_ but since the key
    fn get_child<S: AsRef<str>>(&self, token: S) -> Option<&Self> {
        self.get_children(token).into_iter().next()
    }

    fn get_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self>;
    */

    fn match_any<S: AsRef<str>>(&self, tokens: &[S]) -> Option<&V> {
        let mut child = self;
        for token in tokens {
            if let Some(value) = child.value() {
                return Some(value);
            //FIXME should use match children?
            } else if let Some(next_child) = child.match_child(token) {
                child = next_child;
            } else {
                return None;
            }
        }
        child.value()
    }

    fn match_all<S: AsRef<str>>(&self, tokens: &[S]) -> Vec<&V> {
        let mut values = Vec::new();
        if let Some(token) = tokens.first() {
            for child in self.match_children(token) {
                if let Some(value) = child.value() {
                    values.push(value)
                }
                values.extend(child.match_all(&tokens[1..]));
            }
        }
        values
    }
}
