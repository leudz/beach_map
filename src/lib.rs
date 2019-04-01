//! A BeachMap is actually a SlotMap, a data structure used to store elements and access them with an id.
//! # Exemple:
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

pub type DefaultVersion = u32;

#[derive(Copy, Clone, Debug, PartialEq)]
struct LinkedList {
    head: usize,
    tail: usize,
}

impl LinkedList {
    #[inline]
    fn is_last(&self) -> bool {
        self.tail == self.head
    }
}

/// Handle to a value inside the BeachMap.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ID<V> {
    index: usize,
    version: V,
}

// V is the type of the version
// T is the type of the item stored
pub struct BeachMap<V, T> {
    // ID.index is either a data index or the index of the next available id in ids
    ids: Vec<ID<V>>,
    // ids index of every elements in data
    slots: Vec<usize>,
    data: Vec<T>,
    // linked list inside ids
    // the list use ids[index].index to point to the next node
    // head is older than tail
    available_ids: Option<LinkedList>,
}

impl<T> Default for BeachMap<DefaultVersion, T> {
    fn default() -> Self {
        BeachMap {
            ids: Vec::new(),
            slots: Vec::new(),
            data: Vec::new(),
            available_ids: None,
        }
    }
}

impl<T> BeachMap<DefaultVersion, T> {
    /// Constructs a new, empty BeachMap with the specified capacity and the default version type (u32).
    /// # Exemple:
    /// ```
    /// # use beach_map::BeachMap;
    /// let beach = BeachMap::<_, i32>::with_capacity(5);
    /// assert_eq!(beach.len(), 0);
    /// // or if T can be inferred
    /// let mut beach = BeachMap::with_capacity(5);
    /// beach.insert(10);
    /// ```
    pub fn with_capacity(capacity: usize) -> BeachMap<DefaultVersion, T> {
        BeachMap {
            ids: Vec::with_capacity(capacity),
            slots: Vec::with_capacity(capacity),
            data: Vec::with_capacity(capacity),
            available_ids: None,
        }
    }
    /// Constructs a new, empty BeachMap with a custom version type.
    /// # Exemple:
    /// ```
    /// # use beach_map::BeachMap;
    /// let beach = BeachMap::<_, i32>::with_version::<u8>();
    /// // or if T can be inferred
    /// let mut beach = BeachMap::with_version::<u8>();
    /// beach.insert(5);
    /// ```
    pub fn with_version<V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>>() -> BeachMap<V, T> {
        BeachMap {
            ids: Vec::new(),
            slots: Vec::new(),
            data: Vec::new(),
            available_ids: None,
        }
    }
    /// Constructs a new, empty BeachMap with the given capacity and a custom version type.
    /// # Exemple:
    /// ```
    /// # use beach_map::BeachMap;
    /// let beach = BeachMap::<_, i32>::with_version_and_capacity::<u8>(5);
    /// // or if T can be inferred
    /// let mut beach = BeachMap::with_version_and_capacity::<u8>(5);
    /// beach.insert(10);
    /// ```
    pub fn with_version_and_capacity<V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>>(capacity: usize) -> BeachMap<V, T> {
        BeachMap {
            ids: Vec::with_capacity(capacity),
            slots: Vec::with_capacity(capacity),
            data: Vec::with_capacity(capacity),
            available_ids: None,
        }
    }
}

impl<V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>, T> BeachMap<V, T> {
    /// Construct a new, empty BeachMap with a custom version type.
    /// # Exemple:
    /// ```
    /// # use beach_map::BeachMap;
    /// let beach = BeachMap::<u8, i32>::new();
    /// ```
    #[allow(clippy::new_ret_no_self)]
    #[inline]
    pub fn new() -> BeachMap<V, T> {
        BeachMap {
            ids: Vec::new(),
            slots: Vec::new(),
            data: Vec::new(),
            available_ids: None,
        }
    }
    /// Returns the next id, from the linked list if it isn't empty or a brand new one
    fn next_id(&mut self) -> ID<V> {
        match &mut self.available_ids {
            Some(available_ids) => {
                let head = available_ids.head;
                if available_ids.is_last() {
                    self.available_ids = None
                } else {
                    // assign the first available id to the next one
                    // SAFE available_ids are always valid ids indices
                    available_ids.head = unsafe { self.ids.get_unchecked(head).index };
                }
                // reassign id.index to the future index of the data
                // SAFE first_available_id is always a valid index
                unsafe { self.ids.get_unchecked_mut(head) }.index = self.data.len();
                ID {
                    index: head,
                    version: unsafe { self.ids.get_unchecked_mut(head) }.version,
                }
            }
            None => {
                // if there is no available id, push a new one at the end of ids
                self.ids.push(ID {
                    index: self.data.len(),
                    version: V::default(),
                });
                ID {
                    index: self.ids.len() - 1,
                    version: V::default(),
                }
            }
        }
    }
    /// Adds an item to the BeachMap returning an ID.\
    /// If you want to insert a self-referencing value, use `insert_with_id`.
    pub fn insert(&mut self, value: T) -> ID<V> {
        let id = self.next_id();
        self.slots.push(id.index);
        self.data.push(value);
        id
    }
    /// Gives to `f` the future id of the element and insert the result of `f` to the BeachMap.
    pub fn insert_with_id<F: FnOnce(ID<V>) -> T>(&mut self, f: F) -> ID<V> {
        let id = self.next_id();
        self.slots.push(id.index);
        self.data.push(f(id));
        id
    }
    /// Consumes as many available ids as possible and map slots to the new ids
    fn map_indices(&mut self, additional: usize) {
        let start = self.slots.len();
        if let Some(available_ids) = &mut self.available_ids {
            for i in 0..additional {
                let head = available_ids.head;
                self.slots.push(available_ids.head);
                // assign first_available_id to the next available index
                // or None if it's the last one
                // SAFE while in the linked list, ids[index].index is always a valid ids index
                if available_ids.is_last() {
                    self.available_ids = None;
                    // assign ids[index] to the index where the data will be
                    // SAFE slots are always valid ids index
                    unsafe {self.ids.get_unchecked_mut(head)}.index = start + i;
                    break;
                } else {
                    available_ids.head = unsafe {self.ids.get_unchecked(available_ids.head)}.index;
                    // assign ids[index] to the index where the data will be
                    // SAFE slots are always valid ids index
                    unsafe {self.ids.get_unchecked_mut(head)}.index = start + i;
                }
            }
        }
        self.ids
            .reserve(additional - (self.slots.len() - start));
        for i in (self.slots.len() - start)..additional {
            self.slots.push(start + i);
            self.ids.push(ID {
                index: start + i,
                version: V::default(),
            });
        }
    }
    /// Moves all `vec`'s elements into the BeachMap leaving it empty.
    pub fn append(&mut self, vec: &mut Vec<T>) -> Vec<ID<V>> {
        let elements_count = vec.len();
        // anchor to the first new index
        let start = self.slots.len();

        self.slots.reserve(elements_count);
        self.map_indices(elements_count);
        self.data.append(vec);
        // SAFE slots are always valid ids index
        unsafe {
            self.slots[start..]
                .iter()
                .map(|&index| ID {
                    index,
                    version: self.ids.get_unchecked(index).version,
                })
                .collect()
        }
    }
    // Moves all `vec`'s elements into the BeachMap leaving it empty.\
    // The IDs are stored in container.
    pub fn append_in(&mut self, vec: &mut Vec<T>, container: &mut Vec<ID<V>>) {
        let elements_count = vec.len();
        // anchor to the first new index
        let start = self.slots.len();

        self.slots.reserve(elements_count);
        self.map_indices(elements_count);
        self.data.append(vec);
        container.reserve(self.slots.len() - start);
        self.slots[start..].iter().for_each(|&index| {
            // SAFE slots are always valid ids index
            unsafe {
                container.push(ID {
                    index,
                    version: self.ids.get_unchecked(index).version,
                });
            }
        });
    }
    /// Inserts all elements of `iterator` into the BeachMap and return an ID for each element.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iterator: I) -> Vec<ID<V>> {
        let iter = iterator.into_iter();
        let mut ids = Vec::with_capacity(iter.size_hint().0);
        for item in iter {
            ids.push(self.insert(item));
        }
        ids
    }
    /// Returns a reference to the element corresponding to `id`.
    #[inline]
    pub fn get(&self, id: ID<V>) -> Option<&T> {
        if let Some(&ID { version, .. }) = self.ids.get(id.index) {
            if version == id.version {
                // SAFE index is valid and the versions match
                Some(unsafe { self.get_unchecked(id.index) })
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Returns a mutable reference to an element corresponding to `id`.
    #[inline]
    pub fn get_mut(&mut self, id: ID<V>) -> Option<&mut T> {
        if let Some(&ID { version, .. }) = self.ids.get(id.index) {
            if version == id.version {
                // SAFE index is valid and the versions match
                Some(unsafe { self.get_unchecked_mut(id.index) })
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Returns a reference to an element without checking if the versions match.
    /// # Safety
    /// Should only be used if index is less that ids.len() and the versions match.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.data.get_unchecked(self.ids.get_unchecked(index).index)
    }
    /// Returns a mutable reference to an element without checking if the versions match.
    /// # Safety
    /// Should only be used if index is less than ids.len() and the versions match.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.data
            .get_unchecked_mut(self.ids.get_unchecked(index).index)
    }
    /// Returns true if the BeachMap contains a value for the specified ID.
    pub fn contains(&self, id: ID<V>) -> bool {
        if id.index < self.ids.len() {
            // SAFE all indices lower than len() exist
            id.version == unsafe {self.ids.get_unchecked(id.index)}.version
        } else {
            false
        }
    }
    /// Removes an element from the BeachMap and returns it.
    pub fn remove(&mut self, id: ID<V>) -> Option<T> {
        if id.index < self.ids.len() {
            //SAFE all index lower than len() exist
            if unsafe { self.ids.get_unchecked(id.index) }.version == id.version {
                // SAFE ids[index].index is always a valid data index
                let data_index = unsafe { self.ids.get_unchecked(id.index) }.index;
                // change ids[slots.last()].index to data_index
                // SAFE slots are always valid index
                unsafe {
                    self.ids
                        .get_unchecked_mut(*self.slots.get_unchecked(self.slots.len() - 1))
                }
                .index = data_index;
                // increment the version of ids[index]
                unsafe { self.ids.get_unchecked_mut(id.index) }.version += 1.into();
                if let Some(available_ids) = &mut self.available_ids {
                    // if the linked list isn't empty make ids[index] point to the old tail
                    unsafe { self.ids.get_unchecked_mut(available_ids.tail) }.index = id.index;
                    available_ids.tail = id.index;
                } else {
                    self.available_ids = Some(LinkedList {tail: id.index, head: id.index});
                }
                self.slots.swap_remove(data_index);
                Some(self.data.swap_remove(data_index))
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Returns an iterator visiting all values in the BeachMap.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }
    /// Returns an iterator visiting all values in the BeachMap and let you mutate them.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.data.iter_mut()
    }
    /// Returns an iterator visiting all values in the BeachMap and their ID.
    pub fn iter_with_id(&self) -> IterID<V, T> {
        IterID {
            beach: self,
            index: 0,
        }
    }
    /// Returns an iterator visiting all values in the BeachMap and their ID, values being mutable.
    pub fn iter_mut_with_id(&mut self) -> IterMutID<V, T> {
        IterMutID {
            beach: self,
            index: 0,
        }
    }
    /// Returns the number of elements the BeachMap can hold without reallocating slots or data.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    /// Returns the number of elements in the BeachMap.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Returns true is the BeachMap contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// Reserves capacity for at least `additional` more elements.\
    /// The function will reserve space regardless of vacant slots.\
    /// Meaning some times slots and data may not allocate new space while ids does.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.ids.reserve(additional);
        self.slots.reserve(additional);
        self.data.reserve(additional);
    }
    /// Reserves the minimum capacity for exactly `additional` more elements to be inserted in the given BeachMap.\
    /// The function will reserve space regardless of vacant slots.\
    /// Meaning some times slots and data may not allocate new space while ids does.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.ids.reserve_exact(additional);
        self.slots.reserve_exact(additional);
        self.data.reserve_exact(additional);
    }
    /// Shrinks the capacity of the BeachMap as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.ids.shrink_to_fit();
        self.slots.shrink_to_fit();
        self.data.shrink_to_fit();
    }
    /// Clears the BeachMap, removing all values.\
    /// It does not affet the capacity.
    pub fn clear(&mut self) {
        // make sure available_ids is Some to be able to use unreachable_unchecked
        if self.available_ids.is_none() && !self.slots.is_empty() {
            self.available_ids = Some(LinkedList {tail: unsafe {*self.slots.get_unchecked(0)}, head: unsafe {*self.slots.get_unchecked(0)}});
        }
        // add all elements of the BeachMap to the available id linked list
        for index in self.slots.drain(..) {
            match &mut self.available_ids {
                Some(available_ids) => {
                    // SAFE slots are always valid ids indices
                    unsafe {self.ids.get_unchecked_mut(index)}.version += 1.into();
                    // SAFE available_ids are always valid ids indices
                    unsafe {self.ids.get_unchecked_mut(available_ids.tail)}.index = index;
                    available_ids.tail = index;
                },
                // SAFE we made sure it is Some unless slots.is_empty() and in this case the loop is ignored
                None => unsafe {std::hint::unreachable_unchecked()},
            }
        }
        self.data.clear();
    }
    /// Creates an iterator which uses a closure to determine if an element should be removed.\
    /// If the closure returns true, then the element is removed and yielded along with its ID.
    pub fn filter_out<F: FnMut(ID<V>, &mut T) -> bool>(&mut self, f: F) -> FilterOut<V, T, F> {
        FilterOut::new(self, f)
    }
    /// Keep in the BeachMap the elements for which `f` returns true.
    pub fn retain<F: FnMut(ID<V>, &mut T) -> bool>(&mut self, mut f: F) {
        FilterOut::new(self, |id, x| !f(id, x)).for_each(|_| {})
    }
    /// Creates a draining iterator that removes all elements in the BeachMap and yields the removed items.\
    /// The elements are removed even if the iterator is only partially consumed or not consumed at all.
    pub fn drain(&mut self) -> std::vec::Drain<T> {
        // make sure available_ids is Some to be able to use unreachable_unchecked
        if self.available_ids.is_none() && !self.slots.is_empty() {
            self.available_ids = Some(LinkedList {tail: unsafe {*self.slots.get_unchecked(0)}, head: unsafe {*self.slots.get_unchecked(0)}});
        }
        // add all elements of the BeachMap to the available id linked list
        for index in self.slots.drain(..) {
            match &mut self.available_ids {
                Some(available_ids) => {
                    // SAFE slots are always valid ids indices
                    unsafe {self.ids.get_unchecked_mut(index)}.version += 1.into();
                    // SAFE available_ids are always valid ids indices
                    unsafe {self.ids.get_unchecked_mut(available_ids.tail)}.index = index;
                    available_ids.tail = index;
                },
                // SAFE we made sure it is Some unless slots.is_empty() and in this case the loop is ignored
                None => unsafe {std::hint::unreachable_unchecked()},
            }
        }
        self.data.drain(..)
    }
    /// Creates a draining iterator that removes all elements in the BeachMap and yields the removed items along with their ID.\
    /// Only the elements consumed by the iterator are removed.
    pub fn drain_with_id(&mut self) -> FilterOut<V, T, impl FnMut(ID<V>, &mut T) -> bool> {
        FilterOut::new(self, |_, _| true)
    }
    /// Reserves `additional` ids. Call `use_ids` to add elements.
    /// # Safety
    /// `reverve_ids` and `use_ids` can be called in any order and can be called multiple times.\
    /// But until you have use `additional` ids, adding or removing elements from the BeachMap will break it and other methods may panic.
    pub unsafe fn reserve_ids(&mut self, additional: usize) -> Vec<ID<V>> {
        let start = self.slots.len();

        self.slots.reserve(additional);
        self.map_indices(additional);
        self.data.reserve(additional);
        // SAFE slots are always valid ids index
        self.slots[start..]
            .iter()
            .map(|&index| ID {
                index,
                version: self.ids.get_unchecked(index).version,
            })
            .collect()
    }
    /// Just adds the elements without making ids nor slots for it, use `reserve_ids` to add ids and slots.\
    /// See `reserve_ids` to know in which situation it is safe.
    #[inline]
    pub unsafe fn use_ids(&mut self, elements: &mut Vec<T>) {
        self.data.append(elements);
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

impl<V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>, T>
    std::ops::Index<ID<V>> for BeachMap<V, T>
{
    type Output = T;

    fn index(&self, id: ID<V>) -> &T {
        self.get(id).unwrap()
    }
}

impl<V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>, T>
    std::ops::IndexMut<ID<V>> for BeachMap<V, T>
{
    fn index_mut(&mut self, id: ID<V>) -> &mut T {
        self.get_mut(id).unwrap()
    }
}

impl<'beach, V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>, T> IntoIterator
    for &'beach BeachMap<V, T>
{
    type Item = &'beach T;
    type IntoIter = std::slice::Iter<'beach, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'beach, V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>, T> IntoIterator
    for &'beach mut BeachMap<V, T>
{
    type Item = &'beach mut T;
    type IntoIter = std::slice::IterMut<'beach, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(feature = "parallel")]
impl<
        'beach,
        V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>,
        T: Send + Sync + 'beach,
    > rayon::prelude::IntoParallelRefIterator<'beach> for BeachMap<V, T>
{
    type Item = &'beach T;
    type Iter = rayon::slice::Iter<'beach, T>;

    fn par_iter(&'beach self) -> Self::Iter {
        self.data.par_iter()
    }
}

#[cfg(feature = "parallel")]
impl<
        'beach,
        V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>,
        T: Send + 'beach,
    > rayon::prelude::IntoParallelRefMutIterator<'beach> for BeachMap<V, T>
{
    type Item = &'beach mut T;
    type Iter = rayon::slice::IterMut<'beach, T>;

    fn par_iter_mut(&'beach mut self) -> Self::Iter {
        self.data.par_iter_mut()
    }
}

pub struct FilterOut<'beach, V, T, F>
where V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>,
F: FnMut(ID<V>, &mut T) -> bool {
    beach: &'beach mut BeachMap<V, T>,
    index: usize,
    pred: F,
}

impl<'beach, V, T, F> FilterOut<'beach, V, T, F>
where V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>,
F: FnMut(ID<V>, &mut T) -> bool {
    fn new(beach: &'beach mut BeachMap<V, T>, f: F) -> FilterOut<'beach, V, T, F> {
        // make sure first_available_id is Some to be able to use unreachable_unchecked
        if beach.available_ids.is_none() && !beach.slots.is_empty() {
            beach.available_ids = Some(LinkedList {tail: unsafe {*beach.slots.get_unchecked(0)}, head: unsafe {*beach.slots.get_unchecked(0)}});
        }
        FilterOut {
            beach,
            index: 0,
            pred: f,
        }
    }
}

impl<'beach, V, T, F> Iterator for FilterOut<'beach, V, T, F> 
where V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>,
F: FnMut(ID<V>, &mut T) -> bool {
    type Item = (ID<V>, T);

    fn next(&mut self) -> Option<(ID<V>, T)> {
        let data = &mut self.beach.data;
        while self.index < data.len() {
            // SAFE slots and data have the same length
            let index = unsafe {*self.beach.slots.get_unchecked(self.index)};
            // SAFE slots are always valid ids indices
            let version = unsafe {self.beach.ids.get_unchecked(index)}.version;
            let id = ID {
                index,
                version,
            };
            if (self.pred)(id, unsafe {data.get_unchecked_mut(self.index)}) {
                unsafe {self.beach.ids.get_unchecked_mut(index)}.version += 1.into();
                match &mut self.beach.available_ids {
                    Some(available_ids) => {
                        // SAFE available_ids are always valid ids indices
                        unsafe {self.beach.ids.get_unchecked_mut(available_ids.tail)}.index = index;
                        available_ids.tail = index;
                    },
                    None => unsafe {std::hint::unreachable_unchecked()},
                };
                let result = Some((id, data.swap_remove(self.index)));
                self.beach.slots.swap_remove(self.index);
                return result;
            } else {
                self.index += 1;
            }
        }
        None
    }
}

pub struct IterID<'beach, V, T> {
    beach: &'beach BeachMap<V, T>,
    index: usize,
}

impl<'beach, V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>, T> Iterator for IterID<'beach, V, T> {
    type Item = (ID<V>, &'beach T);

    fn next(&mut self) -> Option<(ID<V>, &'beach T)> {
        if self.index < self.beach.len() {
            let data_index = self.index;
            self.index += 1;
            // SAFE slots and data have always the same len
            let index = unsafe {*self.beach.slots.get_unchecked(data_index)};
            // SAFE slots are always valid ids indices
            Some((ID {index, version: unsafe {self.beach.ids.get_unchecked(index)}.version }, unsafe {self.beach.data.get_unchecked(data_index)}))
        } else {
            None
        }
    }
}

pub struct IterMutID<'beach, V, T> {
    beach: &'beach mut BeachMap<V, T>,
    index: usize,
}

impl<'beach, V: Copy + Clone + Default + PartialEq + std::ops::AddAssign + From<u8>, T: 'beach> Iterator for IterMutID<'beach, V, T> {
    type Item = (ID<V>, &'beach mut T);

    fn next(&mut self) -> Option<(ID<V>, &'beach mut T)> {
        if self.index < self.beach.len() {
            let data_index = self.index;
            self.index += 1;
            // SAFE slots and data have always the same len
            let index = unsafe {*self.beach.slots.get_unchecked(data_index)};
            // SAFE slots are always valid ids indices
            // SAFEISH we are making multiple &mut ref to BeachMap but it is always to different indices
            // and since we have the only &mut ref to BeachMap, it can't reallocate
            Some((ID {index, version: unsafe {self.beach.ids.get_unchecked(index)}.version }, unsafe {&mut *self.beach.data.as_mut_ptr().add(data_index)}))
        } else {
            None
        }
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insert() {
        let mut beach = BeachMap::<u32, _>::default();
        let id = beach.insert(5);
        assert_eq!(id.index, 0);
        assert_eq!(id.version, 0);
        assert_eq!(beach.ids[id.index].version, id.version);
        assert_eq!(beach.slots[beach.slots.len() - 1], id.index);
        assert_eq!(beach.data[beach.ids[id.index].index], 5);
        assert!(beach.ids.len() == 1 && beach.slots.len() == 1 && beach.data.len() == 1);
        let id2 = beach.insert(10);
        assert_eq!(id2.index, 1);
        assert_eq!(id.version, 0);
        assert_eq!(beach.ids[id2.index].version, id2.version);
        assert_eq!(beach.slots[beach.slots.len() - 1], id2.index);
        assert_eq!(beach.data[beach.ids[id2.index].index], 10);
        assert!(beach.ids.len() == 2 && beach.slots.len() == 2 && beach.data.len() == 2);
    }
    #[test]
    fn insert_with_id() {
        struct SelfRef {
            self_ref: ID<u32>
        }
        let mut beach = BeachMap::<u32, _>::default();
        let id = beach.insert_with_id(|id| SelfRef {self_ref: id});
        assert_eq!(beach[id].self_ref, id);
    }
    #[test]
    fn append() {
        let mut beach = BeachMap::<u32, _>::default();
        let ids = beach.append(&mut vec![0, 1, 2, 3]);
        assert_eq!(beach.ids.len(), 4);
        assert_eq!(beach.slots.len(), 4);
        assert_eq!(beach.data.len(), 4);
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(id.index, i);
            assert_eq!(id.version, 0);
            assert_eq!(beach.ids[id.index].version, id.version);
            assert_eq!(beach.slots[i], id.index);
            assert_eq!(beach[id], i);
        }
    }
    #[test]
    fn append_and_remove() {
        let mut beach = BeachMap::<u32, _>::default();
        let mut values = vec![0, 1, 2, 3];
        let mut new_values = vec![4, 5, 6];

        let mut old_ids = beach.append(&mut values);
        let ids = old_ids.drain(2..).collect::<Vec<_>>();
        for id in &old_ids {
            beach.remove(*id);
        }
        let new_ids = beach.append(&mut new_values);
        assert_eq!(beach.ids.len(), 5);
        assert_eq!(beach.slots.len(), 5);
        assert_eq!(beach.data.len(), 5);
        assert_eq!(new_ids[0].index, old_ids[0].index);
        assert_eq!(new_ids[0].version, old_ids[0].version + 1);
        assert_eq!(beach[new_ids[0]], 4);
        assert_eq!(new_ids[1].index, old_ids[1].index);
        assert_eq!(new_ids[1].version, old_ids[1].version + 1);
        assert_eq!(beach[new_ids[1]], 5);
        assert_eq!(new_ids[2].index, 4);
        assert_eq!(new_ids[2].version, 0);
        assert_eq!(beach[new_ids[2]], 6);
        assert_eq!(beach[ids[0]], 2);
        assert_eq!(beach[ids[1]], 3);
    }
    #[test]
    fn append_in() {
        let mut beach = BeachMap::<u32, _>::default();
        let mut values = vec![0, 1, 2, 3];

        let mut ids = Vec::new();
        beach.append_in(&mut values, &mut ids);
        assert_eq!(beach.ids.len(), 4);
        assert_eq!(beach.slots.len(), 4);
        assert_eq!(beach.data.len(), 4);
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(id.index, i);
            assert_eq!(id.version, 0);
            assert_eq!(beach.ids[id.index].version, id.version);
            assert_eq!(beach.slots[i], id.index);
            assert_eq!(beach.data[beach.ids[id.index].index], i);
        }
    }
    #[test]
    fn append_in_and_remove() {
        let mut beach = BeachMap::<u32, _>::default();
        let mut values = vec![0, 1, 2, 3];
        let mut new_values = vec![4, 5, 6];

        let mut old_ids = Vec::new();
        beach.append_in(&mut values, &mut old_ids);
        let ids = old_ids.drain(2..).collect::<Vec<_>>();
        for id in &old_ids {
            beach.remove(*id);
        }
        let new_ids = beach.append(&mut new_values);
        assert_eq!(beach.ids.len(), 5);
        assert_eq!(beach.slots.len(), 5);
        assert_eq!(beach.data.len(), 5);
        assert_eq!(new_ids[0].index, old_ids[0].index);
        assert_eq!(new_ids[0].version, old_ids[0].version + 1);
        assert_eq!(beach[new_ids[0]], 4);
        assert_eq!(new_ids[1].index, old_ids[1].index);
        assert_eq!(new_ids[1].version, old_ids[1].version + 1);
        assert_eq!(beach[new_ids[1]], 5);
        assert_eq!(new_ids[2].index, 4);
        assert_eq!(new_ids[2].version, 0);
        assert_eq!(beach[new_ids[2]], 6);
        assert_eq!(beach[ids[0]], 2);
        assert_eq!(beach[ids[1]], 3);
    }
    #[test]
    fn expend() {
        let mut beach = BeachMap::default();
        beach.extend(0..4);
        assert_eq!(beach.data(), [0, 1, 2, 3]);
    }
    #[test]
    fn get_and_get_mut() {
        let mut beach = BeachMap::<u32, _>::default();
        let id = beach.insert(5);
        assert_eq!(beach.get(id), Some(&5));
        assert_eq!(beach.get_mut(id), Some(&mut 5));
        let id2 = beach.insert(10);
        assert_eq!(beach.get(id2), Some(&10));
        assert_eq!(beach.get_mut(id2), Some(&mut 10));
    }
    #[test]
    fn get_and_get_mut_non_existant() {
        let mut beach = BeachMap::<u32, _>::default();
        beach.insert(5);
        let id = ID {
            index: 1,
            version: 0,
        };
        assert_eq!(beach.get(id), None);
        assert_eq!(beach.get_mut(id), None);
        let id2 = ID {
            index: 0,
            version: 1,
        };
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
        let mut beach = BeachMap::<u32, _>::default();
        let id = beach.insert(5);
        let id2 = beach.insert(10);
        assert_eq!(beach.remove(id), Some(5));
        assert_eq!(beach.remove(id), None);
        assert_eq!(beach.ids[id.index].version, id.version + 1);
        assert_eq!(beach.available_ids, Some(LinkedList {tail: id.index, head: id.index}));
        assert_eq!(beach[id2], 10);
        assert_eq!(beach.ids.len(), 2);
        assert_eq!(beach.slots.len(), 1);
        assert_eq!(beach.data.len(), 1);
        assert_eq!(beach.len(), 1);
        assert!(beach.capacity() >= 2);
        assert_eq!(beach.remove(id2), Some(10));
        assert_eq!(beach.remove(id2), None);
        assert_eq!(beach.ids[id2.index].version, id2.version + 1);
        assert_eq!(beach.available_ids, Some(LinkedList {tail: id2.index, head: id.index}));
        assert_eq!(beach.ids.len(), 2);
        assert_eq!(beach.slots.len(), 0);
        assert_eq!(beach.data.len(), 0);
        assert_eq!(beach.len(), 0);
        assert!(beach.capacity() >= 2);
        let id3 = beach.insert(15);
        assert_eq!(id3.index, id.index);
        assert_eq!(id3.version, id.version + 1);
        assert_eq!(beach.ids[id3.index].version, id3.version);
        assert_eq!(beach.available_ids, Some(LinkedList {tail: id2.index, head: id2.index}));
        assert_eq!(beach[id3], 15);
        assert_eq!(beach.ids.len(), 2);
        assert_eq!(beach.slots.len(), 1);
        assert_eq!(beach.data.len(), 1);
        let id4 = beach.insert(20);
        assert_eq!(id4.index, id2.index);
        assert_eq!(id4.version, id2.version + 1);
        assert_eq!(beach.ids[id4.index].version, id4.version);
        assert_eq!(beach.available_ids, None);
        assert_eq!(beach[id4], 20);
        assert_eq!(beach.ids.len(), 2);
        assert_eq!(beach.slots.len(), 2);
        assert_eq!(beach.data.len(), 2);
    }
    #[test]
    fn iterators() {
        let mut beach = BeachMap::<u32, _>::default();
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
        let mut beach = BeachMap::<u32, _>::default();
        let ids = beach.extend(0..4);
        let mut iter = beach.iter_with_id();
        assert_eq!(iter.next(), Some((ids[0], &0)));
        assert_eq!(iter.next(), Some((ids[1], &1)));
        assert_eq!(iter.next(), Some((ids[2], &2)));
        assert_eq!(iter.next(), Some((ids[3], &3)));
        assert_eq!(iter.next(), None);
    }
    /*// Should not compile or pass
    // You should not be able to have at the same time a mut ref to some variable inside the BeachMap and a mut ref to the BeachMap
    #[test]
    fn iter_mut_with_id() {
        let mut beach = BeachMap::<u32, _>::default();
        beach.insert(5);
        let test;
        {
            let mut iter = beach.iter_mut_with_id();
            test = iter.next();
        }
        beach.reserve(100);
        println!("{:?}", test);
    }*/
    #[test]
    fn len() {
        let mut beach = BeachMap::<u32, _>::default();
        let id = beach.insert(5);
        assert_eq!(beach.len(), beach.slots.len());
        assert_eq!(beach.len(), beach.data.len());
        beach.remove(id);
        assert_eq!(beach.len(), beach.slots.len());
        assert_eq!(beach.len(), beach.data.len());
        let mut values = (0..100).collect::<Vec<_>>();
        beach.append(&mut values);
        assert_eq!(beach.len(), beach.slots.len());
        assert_eq!(beach.len(), beach.data.len());
    }
    #[test]
    fn reserve() {
        let mut beach = BeachMap::<u32, u32>::default();
        beach.reserve(100);
        assert!(beach.ids.capacity() >= 100);
        assert!(beach.slots.capacity() >= 100);
        assert!(beach.data.capacity() >= 100);
    }
    #[test]
    fn reserve_ids() {
        let mut beach = BeachMap::<u32, _>::default();
        let mut values = (0..100).collect::<Vec<_>>();

        let ids = unsafe { beach.reserve_ids(values.len()) };
        assert_eq!(beach.ids.len(), 100);
        assert_eq!(beach.slots.len(), 100);
        assert_eq!(beach.data.len(), 0);
        assert!(beach.data.capacity() >= 100);
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(id.index, i);
            assert_eq!(id.version, 0);
            assert_eq!(beach.ids[id.index].version, id.version);
            assert_eq!(beach.slots[i], id.index);
        }
        unsafe { beach.use_ids(&mut values) };
        assert_eq!(beach.ids.len(), 100);
        assert_eq!(beach.slots.len(), 100);
        assert_eq!(beach.data.len(), 100);
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(beach.ids[id.index].version, id.version);
            assert_eq!(beach.slots[i], id.index);
            assert_eq!(beach.data[beach.ids[id.index].index], i);
        }
    }
    #[test]
    fn clear() {
        let mut beach = BeachMap::<u32, _>::default();
        beach.extend(0..4);
        beach.clear();
        assert!(beach.capacity() >= 4);
        assert_eq!(beach.len(), 0);
        let ids = beach.extend(5..7);
        assert_eq!(beach[ids[0]], 5);
        assert_eq!(beach[ids[1]], 6);
    }
    #[test]
    fn filter_out() {
        let mut beach = BeachMap::<u32, _>::default();
        beach.extend(0..5);
        let mut filter_out = beach.filter_out(|_, _| true);
        assert_eq!(filter_out.next(), Some((ID {index: 0, version: 0}, 0)));
        assert_eq!(filter_out.next(), Some((ID {index: 4, version: 0}, 4)));
        assert_eq!(filter_out.next(), Some((ID {index: 3, version: 0}, 3)));
        assert_eq!(filter_out.next(), Some((ID {index: 2, version: 0}, 2)));
        assert_eq!(filter_out.next(), Some((ID {index: 1, version: 0}, 1)));
        assert_eq!(filter_out.next(), None);
        let ids = beach.extend(5..10);
        assert_eq!(beach[ids[0]], 5);
        assert_eq!(beach[ids[1]], 6);
        assert_eq!(beach[ids[2]], 7);
        assert_eq!(beach[ids[3]], 8);
        assert_eq!(beach[ids[4]], 9);
        
        let mut beach2 = BeachMap::<u32, _>::default();
        (0u32..6).map(|x| beach2.insert(x)).for_each(|_| {});
        let mut filter_out = beach2.filter_out(|_, &mut x| x % 2 == 0);
        assert_eq!(filter_out.next(), Some((ID {index: 0, version: 0}, 0)));
        assert_eq!(filter_out.next(), Some((ID {index: 2, version: 0}, 2)));
        assert_eq!(filter_out.next(), Some((ID {index: 4, version: 0}, 4)));
        assert_eq!(filter_out.next(), None);
        assert_eq!(beach2.data[0], 5);
        assert_eq!(beach2.data[1], 1);
        assert_eq!(beach2.data[2], 3);

        let mut beach3 = BeachMap::<u32, _>::default();
        (0u32..6).map(|x| beach3.insert(x)).for_each(|_| {});
        let mut filter_out = beach3.filter_out(|_, &mut x| x % 2 != 0);
        assert_eq!(filter_out.next(), Some((ID {index: 1, version: 0}, 1)));
        assert_eq!(filter_out.next(), Some((ID {index: 5, version: 0}, 5)));
        assert_eq!(filter_out.next(), Some((ID {index: 3, version: 0}, 3)));
        assert_eq!(filter_out.next(), None);
        assert_eq!(beach3.data[0], 0);
        assert_eq!(beach3.data[1], 4);
        assert_eq!(beach3.data[2], 2);
    }
    #[test]
    fn drain() {
        let mut beach = BeachMap::<u32, _>::default();
        beach.extend(0..4);
        let mut drain = beach.drain();
        assert_eq!(drain.next(), Some(0));
        assert_eq!(drain.next(), Some(1));
        assert_eq!(drain.next(), Some(2));
        assert_eq!(drain.next(), Some(3));
        assert_eq!(drain.next(), None);
        drop(drain);
        let ids = beach.extend(4..8);
        assert_eq!(beach[ids[0]], 4);
        assert_eq!(beach[ids[1]], 5);
        assert_eq!(beach[ids[2]], 6);
        assert_eq!(beach[ids[3]], 7);
    }
}