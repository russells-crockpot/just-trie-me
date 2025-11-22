use regex::{Regex, RegexBuilder};
use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    marker::PhantomData,
    ops::Deref,
};

#[derive(Clone)]
pub struct RegexNodeKey(Regex);

impl Deref for RegexNodeKey {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for RegexNodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for RegexNodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl NodeKey for RegexNodeKey {
    #[inline]
    fn new<S: AsRef<str>>(key: S) -> Result<Self> {
        Ok(Self(
            RegexBuilder::new(&format!("^{}$", key.as_ref()))
                .unicode(true)
                .case_insensitive(true)
                .build()?,
        ))
    }

    #[inline]
    fn is_match<S: AsRef<str>>(&self, value: S) -> bool {
        self.0.is_match(value.as_ref())
    }
}

impl PartialEq for RegexNodeKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl PartialEq<str> for RegexNodeKey {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for RegexNodeKey {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.as_str() == *other
    }
}
