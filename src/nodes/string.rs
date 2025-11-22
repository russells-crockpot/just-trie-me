use super::{TrieNode, TrieNodeBuilder};
use crate::Result;
use std::{borrow::BorrowMut as _, collections::HashMap, fmt};

#[derive(Clone)]
pub struct StringTrieNode<V> {
    value: Option<V>,
    children: HashMap<String, Box<Self>>,
}

impl<V> StringTrieNode<V> {
    fn get_child_mut<S: AsRef<str>>(&mut self, token: S) -> Option<&mut Self> {
        self.children
            .get_mut(token.as_ref())
            .map(|n| n.borrow_mut())
    }
}

impl<V> Default for StringTrieNode<V> {
    fn default() -> Self {
        Self {
            value: None,
            children: HashMap::new(),
        }
    }
}

impl<V> TrieNode<V> for StringTrieNode<V> {
    fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    fn get_children<S: AsRef<str>>(&self, token: S) -> Vec<&Self> {
        self.children
            .get(token.as_ref())
            .into_iter()
            .map(|n| n.as_ref())
            .collect()
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

impl<V> TrieNodeBuilder<V> for StringTrieNode<V> {
    type Node = StringTrieNode<V>;

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
        let mut child = if let Some(child) = self.get_child_mut(&pattern) {
            child
        } else {
            let child = Self::default();
            self.children.insert(pattern.clone(), Box::new(child));
            self.get_child_mut(&pattern).unwrap()
        };
        let _ = child.add(items_iter, value);
        Ok(())
    }

    fn build(self) -> Result<Self::Node> {
        Ok(self)
    }
}
impl<V> fmt::Debug for StringTrieNode<V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringTrieNode")
            .field("value", &self.value)
            .field("children", &self.children)
            .finish()
    }
}

//#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::{TrieNode, TrieNodeBuilder};

    #[test]
    fn test_string_trie_shape() {
        let mut node = StringTrieNode::default();
        node.add(["bobby"].into_iter(), true).unwrap();
        node.add(["mister", "bobby"].into_iter(), true).unwrap();
        node.add(["mister", "mark"].into_iter(), true).unwrap();
        assert_eq!(node.children.len(), 2);
        assert!(node.children.contains_key("bobby"));
        assert!(node.children.contains_key("mister"));
        assert!(!node.children.contains_key("mark"));
        assert!(node.value.is_none());
        let children = node.get_children("bobby");
        assert_eq!(children.len(), 1);
        let child = children[0];
        assert!(child.children.is_empty());
        assert!(child.value.unwrap());
        let children = node.get_children("mister");
        assert_eq!(children.len(), 1);
        let child = children[0];
        assert_eq!(child.children.len(), 2);
        assert!(child.children.contains_key("bobby"));
        assert!(child.children.contains_key("mark"));
        assert!(child.value.is_none());
    }

    #[test]
    fn test_string_trie_get_any() {
        let mut node = StringTrieNode::default();
        node.add(["bobby"].into_iter(), true).unwrap();
        node.add(["mister", "bobby"].into_iter(), true).unwrap();
        node.add(["mister", "mark"].into_iter(), true).unwrap();
        assert!(matches!(node.get_any(&["bobby"]), Some(true)));
        assert!(matches!(node.get_any(&["mister", "bobby"]), Some(true)));
        assert!(matches!(node.get_any(&["mister", "mark"]), Some(true)));
        assert!(node.get_any(&["mister", "the", "bobby"]).is_none());
        assert!(node.get_any(&["mister", "the", "mark"]).is_none());
        assert!(node.get_any(&["mark"]).is_none());
        assert!(node.get_any(&["mister"]).is_none());
        assert!(node.get_any(&["mister", "joe"]).is_none());
    }
}
