#[cfg(feature = "serde")]
mod serde;

/// Handle to a value inside the BeachMap.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ID<V = super::DefaultVersion> {
    pub(crate) index: usize,
    pub(crate) version: V,
}

impl<V: Default> ID<V> {
    /// Returns an [`ID`] that can be used as a dummy value.
    /// This [`ID`] can't access any value.
    ///
    /// [`ID`]: ID
    pub fn invalid() -> Self {
        ID {
            index: usize::MAX,
            version: V::default(),
        }
    }
}
