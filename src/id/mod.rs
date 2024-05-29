#[cfg(feature = "serde")]
mod serde;

use core::{cmp::Ordering, marker::PhantomData, num::NonZeroU32};

/// Handle to a value inside the BeachMap.
pub struct Id<T> {
    data: NonZeroU32,
    phantom: PhantomData<*const T>,
}

impl<T> core::hash::Hash for Id<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Id<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Id<T>) -> Ordering {
        match self.index().cmp(&other.index()) {
            ord @ Ordering::Less | ord @ Ordering::Greater => ord,
            Ordering::Equal => self.gen().cmp(&other.gen()),
        }
    }
}

impl<T> core::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Id({}.{})", self.index(), self.gen())
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> Id<T> {
    pub(crate) fn new(index: u32) -> Id<T> {
        Id {
            data: unsafe { NonZeroU32::new_unchecked(index + 1) },
            phantom: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_gen(index: u32, gen: u32) -> Id<T> {
        Id {
            data: unsafe { NonZeroU32::new_unchecked(index + 1 | (gen << 24)) },
            phantom: PhantomData,
        }
    }

    /// Returns an [`Id`] that can be used as a dummy value.
    /// This [`Id`] can't access any value.
    pub fn invalid() -> Self {
        Id {
            data: unsafe { NonZeroU32::new_unchecked(u32::MAX) },
            phantom: PhantomData,
        }
    }

    pub(crate) fn index(self) -> u32 {
        // 24 first bits
        (self.data.get() & !(255 << 24)) - 1
    }

    pub(crate) fn uindex(self) -> usize {
        self.index() as usize
    }

    pub(crate) fn set_index(&mut self, new_index: u32) {
        self.data = unsafe { NonZeroU32::new_unchecked(new_index + 1 | (self.gen() << 24)) };
    }

    pub(crate) fn gen(self) -> u32 {
        // 8 last bits
        self.data.get() >> 24
    }

    pub(crate) fn increment_gen(&mut self) {
        self.data =
            unsafe { NonZeroU32::new_unchecked(self.index() + 1 | ((self.gen() + 1) << 24)) };
    }

    /// Returns a handle without the generic type.
    pub fn untype(self) -> UntypedId {
        UntypedId(Id {
            data: self.data,
            phantom: PhantomData,
        })
    }
}

/// Same as [Id] but without type.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UntypedId(Id<()>);

impl core::fmt::Debug for UntypedId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UntypedId({}.{})", self.0.index(), self.0.gen())
    }
}

impl UntypedId {
    /// Adds back a type to the handle.
    pub fn as_type<T>(self) -> Id<T> {
        Id {
            data: self.0.data,
            phantom: PhantomData,
        }
    }
}
