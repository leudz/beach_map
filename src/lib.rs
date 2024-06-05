//! A BeachMap is a SlotMap, a data structure used to store elements and access them with an id.
//!
//! # Example:
//! ```
//! use beach_map::BeachMap;
//!
//! let mut beach = BeachMap::default();
//! let id1 = beach.insert(1);
//! let id2 = beach.insert(2);
//!
//! assert_eq!(beach.len(), 2);
//! assert_eq!(beach[id1], 1);
//!
//! assert_eq!(beach.remove(id2), Some(2));
//! assert_eq!(beach.get(id2), None);
//! assert_eq!(beach.len(), 1);
//!
//! beach[id1] = 7;
//! assert_eq!(beach[id1], 7);
//!
//! beach.extend(1..4);
//!
//! assert_eq!(beach.data(), [7, 1, 2, 3]);
//! ```
//! You shouldn't assume the order of the elements as any removing operation will shuffle them.
//!
//! # Rayon
//!
//! To use rayon with beach_map, you need rayon in your dependencies and add the parallel feature to beach_map.
//!
//! ## Example:
//! ```
//! use beach_map::BeachMap;
//! # #[cfg(feature = "parallel")]
//! use rayon::prelude::*;
//!
//! let mut beach = BeachMap::default();
//! let ids = beach.extend(0..500);
//!
//! # #[cfg(feature = "parallel")]
//! beach.par_iter_mut().for_each(|x| {
//!     *x *= 2;
//! });
//!
//! # #[cfg(feature = "parallel")]
//! for i in 0..ids.len() {
//!     assert_eq!(beach[ids[i]], i * 2);
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

extern crate alloc;

mod id;

pub use id::{Id, UntypedId};

use alloc::vec::Vec;
use core::marker::PhantomData;

#[derive(Copy, Clone, Debug, PartialEq)]
struct LinkedList {
    newest: u32,
    oldest: u32,
}

impl LinkedList {
    #[inline]
    fn is_last(&self) -> bool {
        self.oldest == self.newest
    }
}

#[derive(Clone)]
pub struct BeachMap<T> {
    // Id.index is either an ids/data index or the index of the next available slot
    slots: Vec<Id<T>>,
    // Id.index is the index into the slots array
    ids: Vec<Id<T>>,
    data: Vec<T>,
    // linked list inside slots
    // the list use slots[index].index to point to the next slot
    available_ids: Option<LinkedList>,
}

impl<T> Default for BeachMap<T> {
    fn default() -> Self {
        BeachMap {
            slots: Vec::new(),
            ids: Vec::new(),
            data: Vec::new(),
            available_ids: None,
        }
    }
}

impl<T> BeachMap<T> {
    /// Constructs a new, empty [`BeachMap`].
    ///
    /// # Exemple:
    /// ```
    /// # use beach_map::BeachMap;
    /// let beach = BeachMap::new();
    /// # let _: BeachMap<u32> = beach;
    /// ```
    #[inline]
    pub fn new() -> BeachMap<T> {
        BeachMap {
            slots: Vec::new(),
            ids: Vec::new(),
            data: Vec::new(),
            available_ids: None,
        }
    }

    /// Constructs a new, empty [`BeachMap`] with the specified capacity.
    ///
    /// # Exemple:
    /// ```
    /// # use beach_map::BeachMap;
    /// let beach = BeachMap::with_capacity(5);
    /// assert_eq!(beach.len(), 0);
    /// # let _: BeachMap<u32> = beach;
    /// ```
    pub fn with_capacity(capacity: usize) -> BeachMap<T> {
        BeachMap {
            slots: Vec::with_capacity(capacity),
            ids: Vec::with_capacity(capacity),
            data: Vec::with_capacity(capacity),
            available_ids: None,
        }
    }
}

impl<T> BeachMap<T> {
    /// Returns a reference to an element without checking if the id is valid.
    ///
    /// # Safety
    /// Should only be used with a valid id while the value is present in the [`BeachMap`].
    #[inline]
    pub unsafe fn get_unchecked(&self, id: Id<T>) -> &T {
        self.data
            .get_unchecked(self.slots.get_unchecked(id.uindex()).uindex())
    }

    /// Returns a mutable reference to an element without checking if the versions match.
    ///
    /// # Safety
    /// Should only be used with a valid id while the value is present in the [`BeachMap`].
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, id: Id<T>) -> &mut T {
        self.data
            .get_unchecked_mut(self.slots.get_unchecked(id.uindex()).uindex())
    }

    /// Returns an iterator visiting all values in the [`BeachMap`].
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<T> {
        self.data.iter()
    }

    /// Returns an iterator visiting all values in the [`BeachMap`] and let you mutate them.
    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<T> {
        self.data.iter_mut()
    }

    /// Returns an iterator visiting all values in the [`BeachMap`] and their [`Id`].
    #[inline]
    pub fn iter_with_id(&self) -> IterId<T> {
        IterId {
            beach: self,
            index: 0,
        }
    }

    /// Returns an iterator visiting all values in the [`BeachMap`] and their [`Id`], values being mutable.
    #[inline]
    pub fn iter_mut_with_id(&mut self) -> IterMutId<T> {
        IterMutId {
            index: 0,
            slots: &self.ids,
            data: self.data.as_mut_ptr(),
            _phantom: PhantomData,
        }
    }

    /// Returns the number of elements the [`BeachMap`] can hold without reallocating ids or data.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Returns the number of elements in the [`BeachMap`].
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` is the [`BeachMap`] contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Reserves capacity for at least `additional` more elements.\
    /// The function will reserve space regardless of vacant slots.\
    /// Meaning some times ids and data may not allocate new space while slots does.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.slots.reserve(additional);
        self.ids.reserve(additional);
        self.data.reserve(additional);
    }

    /// Reserves the minimum capacity for exactly `additional` more elements to be inserted in the given [`BeachMap`].\
    /// The function will reserve space regardless of vacant slots.\
    /// Meaning some times ids and data may not allocate new space while slots does.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.slots.reserve_exact(additional);
        self.ids.reserve_exact(additional);
        self.data.reserve_exact(additional);
    }

    /// Shrinks the capacity of the [`BeachMap`] as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.slots.shrink_to_fit();
        self.ids.shrink_to_fit();
        self.data.shrink_to_fit();
    }

    /// Returns a slice of the underlying data vector.
    #[inline]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Returns a mutable slice of the underlying data vector.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T> BeachMap<T> {
    /// Returns the next id, from the linked list if it isn't empty or a brand new one
    #[inline]
    fn next_id(&mut self) -> Id<T> {
        let Some(available_ids) = &mut self.available_ids else {
            // if there is no available id, push a new one at the end of ids
            self.slots.push(Id::new(self.data.len() as u32));

            return Id::new((self.slots.len() - 1) as u32);
        };

        let index = available_ids.oldest;

        // SAFE available_ids are always valid indices
        let slot = unsafe { self.slots.get_unchecked_mut(index as usize) };

        if available_ids.is_last() {
            self.available_ids = None
        } else {
            // assign the oldest available id to the next one
            available_ids.oldest = slot.index();
        }

        // reassign id.index to the future index in the data array
        slot.set_index(self.data.len() as u32);

        let mut id = *slot;

        id.set_index(index);

        id
    }

    /// Adds an item to the [`BeachMap`] returning an [`Id`].\
    /// If you want to insert a self-referencing value, use [`BeachMap::insert_with_id`].
    #[inline]
    pub fn insert(&mut self, value: T) -> Id<T> {
        let id = self.next_id();

        if id.index() >= !(255 << 24) {
            panic!("Maximum number of elements reached")
        }

        self.ids.push(id);
        self.data.push(value);

        id
    }

    /// Pass the element's future [`Id`] to `f` and inserts its return value in the [`BeachMap`].
    ///
    /// # Example
    /// ```
    /// # use beach_map::{BeachMap, Id};
    /// struct Node {
    ///     content: String,
    ///     id: Id<Node>,
    /// }
    ///
    /// let mut beach = BeachMap::new();
    /// let id = beach.insert_with_id(|id| Node {
    ///     id,
    ///     content: String::new(),
    /// });
    ///
    /// let node = &beach[id];
    /// assert_eq!(node.id, id);
    /// ```
    #[track_caller]
    pub fn insert_with_id<F: FnOnce(Id<T>) -> T>(&mut self, f: F) -> Id<T> {
        let id = self.next_id();

        self.ids.push(id);
        self.data.push(f(id));

        id
    }

    /// Adds an item to the [`BeachMap`] with the given [`Id`] instead of creating a new one.
    ///
    /// # Example:
    /// ```
    /// # #[cfg(feature = "serde")]
    /// # {
    /// # use beach_map::{BeachMap, Id};
    /// let mut beach = BeachMap::new();
    /// let id = beach.insert(1u32);
    ///
    /// let values = beach.iter_with_id().collect::<Vec<_>>();
    /// let json = serde_json::to_string(&values).unwrap();
    /// drop(beach);
    ///
    /// let mut new_beach = BeachMap::new();
    /// let new_values: Vec<(_, u32)> = serde_json::from_str(&json).unwrap();
    ///
    /// for (id, value) in new_values {
    ///     new_beach.spawn(id, value);
    /// }
    ///
    /// assert_eq!(*new_beach.get(id).unwrap(), 1);
    /// # }
    /// ```
    #[track_caller]
    pub fn spawn(&mut self, id: Id<T>, value: T) {
        if id.uindex() < self.slots.len() {
            // SAFE we checked that it's present
            let slot = unsafe { self.slots.get_unchecked_mut(id.uindex()) };
            let index = slot.index();

            if slot.gen() >= id.gen() + 1 {
                let is_slot_occupied = if slot.gen() >= id.gen() {
                    self.ids
                        .get(slot.uindex())
                        .map(|id| id.index() == id.index())
                        .unwrap_or(false)
                } else {
                    true
                };

                if is_slot_occupied {
                    panic!("Trying to override an Id with a greater generation");
                }
            }

            *slot = id;
            slot.set_index(self.ids.len() as u32);

            if let Some(available_ids) = &mut self.available_ids {
                if available_ids.oldest == id.index() && available_ids.is_last() {
                    // if slot where spawn is the only one in the available list
                    self.available_ids = None;
                } else if available_ids.newest == id.index() {
                    // if slot where spawn is at the tail of the available list
                    let mut index_before = available_ids.oldest as usize;
                    let mut slot_before = unsafe { self.slots.get_unchecked_mut(index_before) };

                    while slot_before.index() != id.index() {
                        index_before = slot_before.uindex();
                        slot_before = unsafe { self.slots.get_unchecked_mut(index_before) };
                    }

                    available_ids.newest = index_before as u32;
                } else if available_ids.oldest == id.index() {
                    // if slot where spawn is at the front of the available list
                    available_ids.oldest = index;
                } else {
                    // if slot where spawn is in the middle of the available list
                    let mut slot_before =
                        unsafe { self.slots.get_unchecked_mut(available_ids.oldest as usize) };

                    while slot_before.index() != id.index() {
                        let next_index = slot_before.uindex();
                        slot_before = unsafe { self.slots.get_unchecked_mut(next_index) };
                    }

                    slot_before.set_index(index);
                }
            }

            if let Some(current_id) = self.ids.get(index as usize) {
                if current_id.index() == id.index() {
                    // change ids[slots.last()].index to index
                    // SAFE slots are always valid index
                    unsafe {
                        self.slots
                            .get_unchecked_mut(self.ids.last().unwrap_unchecked().uindex())
                            .set_index(index);
                    }

                    self.ids.swap_remove(index as usize);
                    self.data.swap_remove(index as usize);
                }
            }
        } else {
            let range = (self.slots.len() as u32)..id.index();
            self.slots.extend(range.clone().map(|i| Id::new(i + 1)));

            let mut id = id;
            id.set_index(self.ids.len() as u32);
            self.slots.push(id);

            if !range.is_empty() {
                if let Some(available_ids) = &mut self.available_ids {
                    let slot =
                        unsafe { self.slots.get_unchecked_mut(available_ids.newest as usize) };

                    slot.set_index(range.start);

                    available_ids.newest = range.end - 1;
                } else {
                    self.available_ids = Some(LinkedList {
                        oldest: range.start,
                        newest: range.end - 1,
                    });
                }
            }
        }

        self.ids.push(id);
        self.data.push(value);
    }

    /// Inserts all elements of `iterator` into the [`BeachMap`] and returns an [`Id`] for each element.
    pub fn extend<'beach>(
        &'beach mut self,
        iterator: impl IntoIterator<Item = T> + 'beach,
    ) -> Vec<Id<T>> {
        iterator
            .into_iter()
            .map(|value| self.insert(value))
            .collect()
    }

    /// Returns a reference to the element corresponding to `id`.
    #[inline]
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        let index = id.uindex();

        if let Some(slot) = self.slots.get(index) {
            if slot.gen() == id.gen() {
                // SAFE index is valid and the versions match
                Some(unsafe { self.data.get_unchecked(slot.uindex()) })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns a mutable reference to an element corresponding to `id`.
    #[inline]
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        let index = id.uindex();

        if let Some(slot) = self.slots.get(index) {
            if slot.gen() == id.gen() {
                // SAFE index is valid and the versions match
                Some(unsafe { self.data.get_unchecked_mut(slot.uindex()) })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns true if the [`BeachMap`] contains a value for the specified [`Id`].
    pub fn contains(&self, id: Id<T>) -> bool {
        let index = id.uindex();

        if let Some(slot) = self.slots.get(index) {
            slot.gen() == id.gen()
        } else {
            false
        }
    }

    /// Removes an element from the [`BeachMap`] and returns it.
    pub fn remove(&mut self, id: Id<T>) -> Option<T> {
        let slot_index = id.uindex();

        if slot_index >= self.slots.len() {
            return None;
        }

        // SAFE all index lower than len() exist
        let slot = unsafe { self.slots.get_unchecked_mut(slot_index) };

        if slot.gen() != id.gen() {
            return None;
        }

        // increment the version of ids[index]
        slot.increment_gen();
        let gen = slot.gen();

        let data_index = slot.uindex();
        // change ids[slots.last()].index to data_index
        // SAFE slots are always valid index
        unsafe {
            self.slots
                .get_unchecked_mut(self.ids.last().unwrap_unchecked().uindex())
                .set_index(data_index as u32);
        }

        self.ids.swap_remove(data_index);
        let value = Some(self.data.swap_remove(data_index));

        // this slot won't be reused
        if gen == 255 {
            return value;
        }

        if let Some(available_ids) = &mut self.available_ids {
            // if the linked list isn't empty, add slot_index to it
            unsafe {
                self.slots
                    .get_unchecked_mut(available_ids.newest as usize)
                    .set_index(slot_index as u32);
            }

            available_ids.newest = slot_index as u32;
        } else {
            self.available_ids = Some(LinkedList {
                oldest: slot_index as u32,
                newest: slot_index as u32,
            });
        }

        value
    }

    /// Clears the [`BeachMap`], removing all values.\
    /// It does not affet the capacity.
    pub fn clear(&mut self) {
        drop(self.drain());
    }

    /// Keeps in the [`BeachMap`] the elements for which `f` returns true.
    pub fn retain<F: FnMut(Id<T>, &mut T) -> bool>(&mut self, mut f: F) {
        let BeachMap {
            slots,
            ids,
            data,
            available_ids,
        } = self;

        let mut i = 0;
        while i < ids.len() {
            // SAFE ids and data have the same length
            let id = unsafe { *ids.get_unchecked(i) };
            let value = unsafe { data.get_unchecked_mut(i) };

            let should_keep = f(id, value);
            if should_keep {
                i += 1;

                continue;
            }

            ids.swap_remove(i);
            data.swap_remove(i);

            let available_ids = available_ids.get_or_insert_with(|| LinkedList {
                oldest: id.index(),
                newest: id.index(),
            });

            // SAFE ids are always valid slot indices
            let slot = unsafe { slots.get_unchecked_mut(id.uindex()) };

            slot.increment_gen();

            if slot.gen() < 255 {
                slot.set_index(available_ids.newest);

                available_ids.newest = id.index();
            }
        }
    }

    /// Creates a draining iterator that removes all elements in the [`BeachMap`] and yields the removed items.\
    /// The elements are removed even if the iterator is only partially consumed or not consumed at all.
    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        if self.is_empty() {
            return self.data.drain(..);
        }

        let mut list = {
            // SAFE ids is not empty
            let index = unsafe { self.ids.last().unwrap_unchecked().index() };

            LinkedList {
                newest: index,
                oldest: index,
            }
        };

        // add all elements of the BeachMap to the available id linked list
        for id in self.ids.drain(..).rev() {
            // SAFE ids are always valid slot indices
            let slot = unsafe { self.slots.get_unchecked_mut(id.uindex()) };

            slot.increment_gen();

            if slot.gen() < 255 {
                slot.set_index(list.oldest);

                list.oldest = id.index();
            }
        }

        if let Some(available_ids) = &mut self.available_ids {
            // we merge the two linked list

            // SAFE ids are always valid slot indices
            let slot = unsafe { self.slots.get_unchecked_mut(available_ids.newest as usize) };
            slot.set_index(list.oldest);

            available_ids.newest = list.newest;
        } else {
            self.available_ids = Some(list);
        }

        self.data.drain(..)
    }

    /// Creates a draining iterator that removes all elements in the [`BeachMap`] and yields the removed items along with their [`Id`].\
    /// Only the elements consumed by the iterator are removed.
    pub fn drain_with_id(&mut self) -> impl Iterator<Item = (Id<T>, T)> + '_ {
        self.ids.drain(..).zip(self.data.drain(..))
    }
}

impl<T> core::ops::Index<Id<T>> for BeachMap<T> {
    type Output = T;

    #[track_caller]
    fn index(&self, id: Id<T>) -> &T {
        self.get(id).unwrap()
    }
}

impl<T> core::ops::IndexMut<Id<T>> for BeachMap<T> {
    #[track_caller]
    fn index_mut(&mut self, id: Id<T>) -> &mut T {
        self.get_mut(id).unwrap()
    }
}

impl<'beach, T> IntoIterator for &'beach BeachMap<T> {
    type Item = &'beach T;
    type IntoIter = core::slice::Iter<'beach, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'beach, T> IntoIterator for &'beach mut BeachMap<T> {
    type Item = &'beach mut T;
    type IntoIter = core::slice::IterMut<'beach, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(feature = "parallel")]
impl<'beach, T: Sync + 'beach> rayon::prelude::IntoParallelIterator for &'beach BeachMap<T> {
    type Item = &'beach T;
    type Iter = rayon::slice::Iter<'beach, T>;

    fn into_par_iter(self) -> Self::Iter {
        self.data.into_par_iter()
    }
}

#[cfg(feature = "parallel")]
impl<'beach, T: Send + Sync + 'beach> rayon::prelude::IntoParallelIterator
    for &'beach mut BeachMap<T>
{
    type Item = &'beach mut T;
    type Iter = rayon::slice::IterMut<'beach, T>;

    fn into_par_iter(self) -> Self::Iter {
        use rayon::prelude::IntoParallelRefMutIterator;

        self.data.par_iter_mut()
    }
}

pub struct IterId<'beach, T> {
    beach: &'beach BeachMap<T>,
    index: usize,
}

impl<'beach, T> Iterator for IterId<'beach, T> {
    type Item = (Id<T>, &'beach T);

    fn next(&mut self) -> Option<(Id<T>, &'beach T)> {
        if self.index >= self.beach.len() {
            return None;
        }

        let index = self.index;
        self.index += 1;

        // SAFE slots and data have always the same len
        let id = unsafe { *self.beach.ids.get_unchecked(index) };
        let value = unsafe { self.beach.data.get_unchecked(index) };

        Some((id, value))
    }
}

pub struct IterMutId<'beach, T> {
    slots: &'beach [Id<T>],
    data: *mut T,
    index: usize,
    _phantom: PhantomData<&'beach mut T>,
}

impl<'beach, T> Iterator for IterMutId<'beach, T> {
    type Item = (Id<T>, &'beach mut T);

    fn next(&mut self) -> Option<(Id<T>, &'beach mut T)> {
        if self.index >= self.slots.len() {
            return None;
        }

        let index = self.index;
        self.index += 1;

        // SAFE slots and data have always the same len
        let id = unsafe { *self.slots.get_unchecked(index) };
        // SAFE slots are always valid ids indices
        // SAFE we are making multiple &mut ref but it is always to different indices
        let value = unsafe { &mut *self.data.add(index) };

        Some((id, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut beach = BeachMap::<u32>::default();
        let id = beach.insert(5);
        assert_eq!(id.index(), 0);
        assert_eq!(id.gen(), 0);
        assert_eq!(beach.slots[id.uindex()].gen(), id.gen());
        assert_eq!(beach.ids[beach.ids.len() - 1].index(), id.index());
        assert_eq!(beach.data[beach.slots[id.uindex()].uindex()], 5);
        assert!(beach.slots.len() == 1 && beach.ids.len() == 1 && beach.data.len() == 1);
        let id2 = beach.insert(10);
        assert_eq!(id2.index(), 1);
        assert_eq!(id.gen(), 0);
        assert_eq!(beach.slots[id2.uindex()].gen(), id2.gen());
        assert_eq!(beach.ids[beach.ids.len() - 1].index(), id2.index());
        assert_eq!(beach.data[beach.slots[id2.uindex()].uindex()], 10);
        assert!(beach.slots.len() == 2 && beach.ids.len() == 2 && beach.data.len() == 2);
    }

    #[test]
    fn insert_with_id() {
        struct SelfRef {
            self_ref: Id<SelfRef>,
        }
        let mut beach = BeachMap::<SelfRef>::default();
        let id = beach.insert_with_id(|id| SelfRef { self_ref: id });
        assert_eq!(beach[id].self_ref, id);
    }

    #[test]
    fn extend() {
        let mut beach = BeachMap::default();

        let ids = beach.extend([0, 1, 2, 3]);

        assert_eq!(beach.slots.len(), 4);
        assert_eq!(beach.ids.len(), 4);
        assert_eq!(beach.data.len(), 4);

        for (i, id) in ids.into_iter().enumerate() {
            assert_eq!(id.uindex(), i);
            assert_eq!(id.gen(), 0);
            assert_eq!(beach.slots[id.uindex()].gen(), id.gen());
            assert_eq!(beach.ids[i].index(), id.index());
            assert_eq!(beach[id], i);
        }
    }

    #[test]
    fn extend_and_remove() {
        let mut beach = BeachMap::<u32>::default();

        let mut old_ids = beach.extend([0, 1, 2, 3]);
        let ids = old_ids.drain(2..).collect::<Vec<_>>();
        for id in &old_ids {
            beach.remove(*id);
        }
        let new_ids = beach.extend([4, 5, 6]);
        assert_eq!(beach.slots.len(), 5);
        assert_eq!(beach.ids.len(), 5);
        assert_eq!(beach.data.len(), 5);
        assert_eq!(new_ids[0].index(), old_ids[0].index());
        assert_eq!(new_ids[0].gen(), old_ids[0].gen() + 1);
        assert_eq!(beach[new_ids[0]], 4);
        assert_eq!(new_ids[1].index(), old_ids[1].index());
        assert_eq!(new_ids[1].gen(), old_ids[1].gen() + 1);
        assert_eq!(beach[new_ids[1]], 5);
        assert_eq!(new_ids[2].index(), 4);
        assert_eq!(new_ids[2].gen(), 0);
        assert_eq!(beach[new_ids[2]], 6);
        assert_eq!(beach[ids[0]], 2);
        assert_eq!(beach[ids[1]], 3);
    }

    #[test]
    fn get_and_get_mut() {
        let mut beach = BeachMap::<u32>::default();
        let id = beach.insert(5);
        assert_eq!(beach.get(id), Some(&5));
        assert_eq!(beach.get_mut(id), Some(&mut 5));
        let id2 = beach.insert(10);
        assert_eq!(beach.get(id2), Some(&10));
        assert_eq!(beach.get_mut(id2), Some(&mut 10));
    }

    #[test]
    fn get_and_get_mut_non_existant() {
        let mut beach = BeachMap::default();
        beach.insert(5);
        let id = Id::new(1);
        assert_eq!(beach.get(id), None);
        assert_eq!(beach.get_mut(id), None);
        let id2 = Id::with_gen(0, 1);
        assert_eq!(beach.get(id2), None);
        assert_eq!(beach.get_mut(id2), None);
    }

    #[test]
    fn contains() {
        let mut beach = BeachMap::default();
        let id = beach.insert(5);
        assert_eq!(beach.contains(id), true);
        beach.remove(id);
        assert_eq!(beach.contains(id), false);
    }

    #[test]
    fn remove() {
        let mut beach = BeachMap::<u32>::default();
        let id = beach.insert(5);
        let id2 = beach.insert(10);
        assert_eq!(beach.remove(id), Some(5));
        assert_eq!(beach.remove(id), None);
        assert_eq!(beach.slots[id.uindex()].gen(), id.gen() + 1);
        assert_eq!(
            beach.available_ids,
            Some(LinkedList {
                oldest: id.index(),
                newest: id.index()
            })
        );
        assert_eq!(beach[id2], 10);
        assert_eq!(beach.slots.len(), 2);
        assert_eq!(beach.ids.len(), 1);
        assert_eq!(beach.data.len(), 1);
        assert_eq!(beach.remove(id2), Some(10));
        assert_eq!(
            beach.available_ids,
            Some(LinkedList {
                newest: id2.index(),
                oldest: id.index()
            })
        );
        assert_eq!(beach.slots[id2.uindex()].gen(), id2.gen() + 1);
        assert_eq!(beach.remove(id2), None);
        assert_eq!(beach.slots[id2.uindex()].gen(), id2.gen() + 1);
        assert_eq!(
            beach.available_ids,
            Some(LinkedList {
                newest: id2.index(),
                oldest: id.index()
            })
        );
        assert_eq!(beach.slots.len(), 2);
        assert_eq!(beach.ids.len(), 0);
        assert_eq!(beach.data.len(), 0);
        assert_eq!(beach.len(), 0);
        assert!(beach.capacity() >= 2);
        let id3 = beach.insert(15);
        assert_eq!(id3.index(), id.index());
        assert_eq!(id3.gen(), id.gen() + 1);
        assert_eq!(beach.slots[id3.uindex()].gen(), id3.gen());
        assert_eq!(
            beach.available_ids,
            Some(LinkedList {
                oldest: id2.index(),
                newest: id2.index()
            })
        );
        assert_eq!(beach[id3], 15);
        assert_eq!(beach.slots.len(), 2);
        assert_eq!(beach.ids.len(), 1);
        assert_eq!(beach.data.len(), 1);
        let id4 = beach.insert(20);
        assert_eq!(id4.index(), id2.index());
        assert_eq!(id4.gen(), id2.gen() + 1);
        assert_eq!(beach.slots[id4.uindex()].gen(), id4.gen());
        assert_eq!(beach.available_ids, None);
        assert_eq!(beach[id4], 20);
        assert_eq!(beach.slots.len(), 2);
        assert_eq!(beach.ids.len(), 2);
        assert_eq!(beach.data.len(), 2);
    }

    #[test]
    fn iterators() {
        let mut beach = BeachMap::default();
        for i in 0..5 {
            beach.insert(i);
        }
        for (value_iter, value_data) in beach.iter().zip(beach.data.iter()) {
            assert_eq!(value_iter, value_data);
        }
        for value in &mut beach {
            *value += 1;
        }
        for (i, (value_iter, value_data)) in beach.iter().zip(beach.data.iter()).enumerate() {
            assert_eq!(value_iter, value_data);
            assert_eq!(*value_iter, i + 1);
        }
    }

    #[test]
    fn iterator_with_id() {
        let mut beach = BeachMap::default();
        let ids = beach.extend(0..4);
        let mut iter = beach.iter_with_id();
        assert_eq!(iter.next(), Some((ids[0], &0)));
        assert_eq!(iter.next(), Some((ids[1], &1)));
        assert_eq!(iter.next(), Some((ids[2], &2)));
        assert_eq!(iter.next(), Some((ids[3], &3)));
        assert_eq!(iter.next(), None);
    }

    // // Should not compile or pass
    // // You should not be able to have at the same time a mut ref to some variable inside the BeachMap and a mut ref to the BeachMap
    // #[test]
    // fn iter_mut_with_id() {
    //     let mut beach = BeachMap::<u32>::default();
    //     beach.insert(5);
    //     let test;
    //     {
    //         let mut iter = beach.iter_mut_with_id();
    //         test = iter.next();
    //     }
    //     beach.reserve(100);
    //     println!("{:?}", test);
    // }

    #[test]
    fn len() {
        let mut beach = BeachMap::default();
        let id = beach.insert(5);
        assert_eq!(beach.len(), beach.ids.len());
        assert_eq!(beach.len(), beach.data.len());
        beach.remove(id);
        assert_eq!(beach.len(), beach.ids.len());
        assert_eq!(beach.len(), beach.data.len());
        let _ = beach.extend(0..100);
        assert_eq!(beach.len(), beach.ids.len());
        assert_eq!(beach.len(), beach.data.len());
    }

    #[test]
    fn reserve() {
        let mut beach = BeachMap::<u32>::default();
        beach.reserve(100);
        assert!(beach.slots.capacity() >= 100);
        assert!(beach.ids.capacity() >= 100);
        assert!(beach.data.capacity() >= 100);
    }

    #[test]
    fn clear() {
        let mut beach = BeachMap::default();
        let _ = beach.extend(0..4);
        beach.clear();
        assert!(beach.capacity() >= 4);
        assert_eq!(beach.len(), 0);
        let ids = beach.extend(5..7);
        assert_eq!(beach[ids[0]], 5);
        assert_eq!(beach[ids[1]], 6);
    }

    #[test]
    fn drain() {
        let mut beach = BeachMap::default();
        let ids = beach.extend(0..4);
        // make sure we handle draining with some available ids already present
        beach.remove(ids[0]);

        let mut drain = beach.drain();
        assert_eq!(drain.next(), Some(3));
        assert_eq!(drain.next(), Some(1));
        assert_eq!(drain.next(), Some(2));
        assert_eq!(drain.next(), None);
        drop(drain);

        let ids = beach.extend(4..8);
        assert_eq!(beach[ids[0]], 4);
        assert_eq!(beach[ids[1]], 5);
        assert_eq!(beach[ids[2]], 6);
        assert_eq!(beach[ids[3]], 7);
        // check that we are re-using the slots
        assert_eq!(beach.slots.len(), 4);
    }

    #[test]
    fn spawn_start_of_available_ids() {
        let mut beach = BeachMap::default();
        let ids = beach.extend(0..4);

        for &id in &ids {
            beach.remove(id);
        }

        beach.spawn(ids[0], 10);
        let id2 = beach.insert(20);
        let id3 = beach.insert(30);
        let id4 = beach.insert(40);
        let id5 = beach.insert(5);

        assert_eq!(beach[ids[0]], 10);
        assert_eq!(beach[id2], 20);
        assert_eq!(beach[id3], 30);
        assert_eq!(beach[id4], 40);
        assert_eq!(beach[id5], 5);
    }

    #[test]
    fn spawn_end_of_available_ids() {
        let mut beach = BeachMap::default();
        let ids = beach.extend(0..4);

        for &id in &ids {
            beach.remove(id);
        }

        beach.spawn(ids[3], 10);
        let id2 = beach.insert(20);
        let id3 = beach.insert(30);
        let id4 = beach.insert(40);
        let id5 = beach.insert(5);

        assert_eq!(beach[ids[3]], 10);
        assert_eq!(beach[id2], 20);
        assert_eq!(beach[id3], 30);
        assert_eq!(beach[id4], 40);
        assert_eq!(beach[id5], 5);
    }

    #[test]
    fn spawn_middle_of_available_ids() {
        let mut beach = BeachMap::default();
        let ids = beach.extend(0..4);

        for &id in &ids {
            beach.remove(id);
        }

        beach.spawn(ids[1], 10);
        let id2 = beach.insert(20);
        let id3 = beach.insert(30);
        let id4 = beach.insert(40);
        let id5 = beach.insert(5);

        assert_eq!(beach[ids[1]], 10);
        assert_eq!(beach[id2], 20);
        assert_eq!(beach[id3], 30);
        assert_eq!(beach[id4], 40);
        assert_eq!(beach[id5], 5);
    }

    #[test]
    fn spawn_only_available_ids() {
        let mut beach = BeachMap::default();
        let id = beach.insert(0);
        beach.remove(id);

        beach.spawn(id, 10);
        let id2 = beach.insert(20);

        assert_eq!(beach[id], 10);
        assert_eq!(beach[id2], 20);
    }

    #[test]
    fn spawn_past_last_id_no_available_ids() {
        let mut beach = BeachMap::default();
        let ids = beach.extend(0..4);
        drop(beach);

        let mut beach = BeachMap::new();
        beach.spawn(ids[2], 30);
        let id1 = beach.insert(10);
        let id2 = beach.insert(20);
        let id4 = beach.insert(40);
        assert_eq!(beach[ids[2]], 30);
        assert_eq!(beach[id1], 10);
        assert_eq!(beach[id2], 20);
        assert_eq!(beach[id4], 40);
    }

    #[test]
    fn spawn_past_last_id_available_ids() {
        let mut beach = BeachMap::default();
        let ids = beach.extend(0..4);
        drop(beach);

        let mut beach = BeachMap::new();
        let id1 = beach.insert(1);
        let id2 = beach.insert(2);
        beach.remove(id1);
        beach.remove(id2);
        beach.spawn(ids[3], 30);
        let id1 = beach.insert(10);
        let id2 = beach.insert(20);
        let id4 = beach.insert(40);
        assert_eq!(beach[ids[3]], 30);
        assert_eq!(beach[id1], 10);
        assert_eq!(beach[id2], 20);
        assert_eq!(beach[id4], 40);
    }
}
