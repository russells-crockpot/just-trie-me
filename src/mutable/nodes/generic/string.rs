use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    marker::PhantomData,
    ops::Deref,
};

#[derive(Clone)]
pub struct StringNodeKey(String);

impl fmt::Display for StringNodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for StringNodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl NodeKey for StringNodeKey {
    #[inline]
    fn new<S: AsRef<str>>(key: S) -> Result<Self> {
        Ok(Self(String::from(key.as_ref())))
    }

    #[inline]
    fn is_match<S: AsRef<str>>(&self, value: S) -> bool {
        self.0 == value.as_ref()
    }
}

impl PartialEq for StringNodeKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<str> for StringNodeKey {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl Deref for StringNodeKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
