#[cfg(feature = "serde")]
mod serde;

/// Handle to a value inside the BeachMap.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ID<V> {
    pub(crate) index: usize,
    pub(crate) version: V,
}
