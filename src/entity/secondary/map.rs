// Copyright 2020 Robin Freyler
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::Index32;
use core::{
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Index, IndexMut},
};
use std::collections::{
    hash_map::{self, Iter as HashMapIter, IterMut as HashMapIterMut},
    HashMap,
};

/// Sparse secondary map to associated new components for existing entities.
///
/// # Efficiency
///
/// Very efficient if the component is very uncommon for the entities.
/// Might be less efficient than optimal for common operations if too many entities
/// have the component.
///
///
/// # Note
///
/// - The component map is well suited when only few entities have a component.
/// - By design all secondary component containers are meant to be easily interchangable.
#[derive(Debug)]
pub struct ComponentMap<K, V> {
    components: HashMap<u32, V>,
    key: PhantomData<fn() -> K>,
}

impl<K, V> Clone for ComponentMap<K, V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            components: self.components.clone(),
            key: Default::default(),
        }
    }
}

impl<K, V> Default for ComponentMap<K, V> {
    fn default() -> Self {
        Self {
            components: Default::default(),
            key: Default::default(),
        }
    }
}

impl<K, V> ComponentMap<K, V>
where
    K: Index32,
{
    /// Returns `true` if the key is valid for the secondary map.
    ///
    /// If the key is invalid the secondary map has to be enlarged to fit the key.
    pub fn contains_key(&self, key: K) -> bool {
        self.components.contains_key(&key.into_u32())
    }

    /// Returns the number of components in the secondary map.
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Returns `true` if there are no components in the secondary map.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Inserts the component for the key and returns the previous component if any.
    pub fn insert(&mut self, key: K, component: V) -> Option<V> {
        self.components.insert(key.into_u32(), component)
    }

    /// Removes the components for the key and returns the removed component if any.
    pub fn remove(&mut self, key: K) -> Option<V> {
        self.components.remove(&key.into_u32())
    }

    /// Returns a shared reference to the component at the given key.
    pub fn get(&self, key: K) -> Option<&V> {
        self.components.get(&key.into_u32())
    }

    /// Returns a exclusive reference to the component at the given key.
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.components.get_mut(&key.into_u32())
    }

    /// Returns an iterator over the keys and a shared reference to their associated components.
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            iter: self.components.iter(),
            key: Default::default(),
        }
    }

    /// Returns an iterator over the keys and an exclusive reference to their associated components.
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            iter: self.components.iter_mut(),
            key: Default::default(),
        }
    }

    /// Clears the component map for reusing its memory.
    pub fn clear(&mut self) {
        self.components.clear();
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        let key_index = key.into_u32();
        match self.components.entry(key_index) {
            hash_map::Entry::Occupied(occupied) => {
                Entry::Occupied(OccupiedEntry {
                    occupied,
                    key: Default::default(),
                })
            }
            hash_map::Entry::Vacant(vacant) => {
                Entry::Vacant(VacantEntry {
                    vacant,
                    key: Default::default(),
                })
            }
        }
    }
}

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This enum is constructed from the entry method on `ComponentMap`.
#[derive(Debug)]
pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Index32,
{
    /// Ensures a value is in the entry by inserting the default if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(move || default)
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(occupied) => occupied.into_mut(),
            Entry::Vacant(vacant) => vacant.insert(default()),
        }
    }

    /// Returns a reference to this entry's key.
    pub fn key(&self) -> K {
        match self {
            Entry::Occupied(occupied) => occupied.key(),
            Entry::Vacant(vacant) => vacant.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut occupied) => {
                f(occupied.get_mut());
                Entry::Occupied(occupied)
            }
            Entry::Vacant(vacant) => Entry::Vacant(vacant),
        }
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Index32,
    V: Default,
{
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V {
        match self {
            Entry::Occupied(occupied) => occupied.into_mut(),
            Entry::Vacant(vacant) => vacant.insert(Default::default()),
        }
    }
}

/// A view into an occupied entry in a `ComponentMap`. It is part of the `Entry` enum.
#[derive(Debug)]
pub struct OccupiedEntry<'a, K, V> {
    occupied: hash_map::OccupiedEntry<'a, u32, V>,
    key: PhantomData<fn() -> K>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Index32,
{
    /// Returns the key from the entry.
    pub fn key(&self) -> K {
        K::from_u32(*self.occupied.key())
    }

    /// Take the ownership of the key and value from the map.
    pub fn remove_entry(self) -> (K, V) {
        let (key, component) = self.occupied.remove_entry();
        (K::from_u32(key), component)
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        self.occupied.get()
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see `into_mut`.
    pub fn get_mut(&mut self) -> &mut V {
        self.occupied.get_mut()
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in
    /// the entry with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see `get_mut`.
    pub fn into_mut(self) -> &'a mut V {
        self.occupied.into_mut()
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        self.occupied.insert(value)
    }

    /// Takes the value out of the entry, and returns it.
    pub fn remove(self) -> V {
        self.occupied.remove()
    }
}

/// A view into a vacant entry in a `ComponentMap`. It is part of the `Entry` enum.
#[derive(Debug)]
pub struct VacantEntry<'a, K, V> {
    vacant: hash_map::VacantEntry<'a, u32, V>,
    key: PhantomData<fn() -> K>,
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Index32,
{
    /// Returns the key that would be used when inserting a value through the `VacantEntry`.
    pub fn key(&self) -> K {
        K::from_u32(*self.vacant.key())
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        self.vacant.insert(value)
    }
}

impl<K, V> Index<K> for ComponentMap<K, V>
where
    K: Index32,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(index)
            .expect("invalid key for sparsely stored component")
    }
}

impl<K, V> IndexMut<K> for ComponentMap<K, V>
where
    K: Index32,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(index)
            .expect("invalid key for sparsely stored component")
    }
}

/// Iterator yielding keys and a shared reference to their associated components.
#[derive(Debug)]
pub struct Iter<'a, K, V> {
    iter: HashMapIter<'a, u32, V>,
    key: PhantomData<fn() -> K>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Index32,
{
    type Item = (K, &'a V);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(key, component)| (K::from_u32(*key), component))
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> where K: Index32 {}
impl<'a, K, V> FusedIterator for Iter<'a, K, V> where K: Index32 {}

/// Iterator yielding keys and an exclusive reference to their associated components.
#[derive(Debug)]
pub struct IterMut<'a, K, V> {
    iter: HashMapIterMut<'a, u32, V>,
    key: PhantomData<fn() -> K>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: Index32,
{
    type Item = (K, &'a mut V);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(key, component)| (K::from_u32(*key), component))
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> where K: Index32 {}
impl<'a, K, V> FusedIterator for IterMut<'a, K, V> where K: Index32 {}
