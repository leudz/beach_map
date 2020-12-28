use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
mod serde;

/// Handle to a value inside the BeachMap.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ID<V> {
    pub(crate) index: usize,
    pub(crate) version: V,
}

impl <V: Hash> Hash for ID<V> {
    fn hash<H: Hasher>(&self, state:&mut H) {
        self.index.hash(state);
        self.version.hash(state);
    }
}
