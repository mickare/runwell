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

//! Index types to operate on primary and secondary entity data structures.
//!
//! Design inspired by https://crates.io/crates/la-arena.

use core::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU32,
};

/// The raw index of an entity.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawIdx {
    /// The raw index shifted by +1.
    ///
    /// A [`NonZeroU32`] value is used in order to space-optimize raw indices
    /// for example when used inside `Option` somewhere in a secondary data
    /// structure.
    index: NonZeroU32,
}

impl RawIdx {
    /// Constructs a raw index from an `u32` value.
    ///
    /// # Panics
    ///
    /// If the given index is equal to `u32::MAX`.
    pub fn from_u32(index: u32) -> Self {
        Self {
            index: NonZeroU32::new(index.wrapping_add(1))
                .expect("encountered invalid u32::MAX value"),
        }
    }

    /// Converts the raw index into its underlying `u32` value.
    pub(super) fn into_u32(self) -> u32 {
        self.index.get().wrapping_sub(1)
    }
}

impl fmt::Debug for RawIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.into_u32().fmt(f)
    }
}

impl fmt::Display for RawIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.into_u32().fmt(f)
    }
}

/// The index of an entity allocated in an entity arena that holds `T`s.
pub struct Idx<T> {
    raw: RawIdx,
    marker: PhantomData<fn() -> T>,
}

impl<T> Idx<T> {
    /// Creates a new index from a [`RawIdx`].
    pub fn from_raw(raw: RawIdx) -> Self {
        Idx {
            raw,
            marker: Default::default(),
        }
    }

    /// Converts this index into the underlying [`RawIdx`].
    pub fn into_raw(self) -> RawIdx {
        self.raw
    }
}

impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Idx<T> {}

impl<T> PartialEq for Idx<T> {
    fn eq(&self, other: &Idx<T>) -> bool {
        self.raw == other.raw
    }
}
impl<T> Eq for Idx<T> {}

impl<T> Hash for Idx<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}

impl<T> fmt::Debug for Idx<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut type_name = core::any::type_name::<T>();
        if let Some(idx) = type_name.rfind(':') {
            type_name = &type_name[idx + 1..]
        }
        write!(f, "Idx::<{}>({})", type_name, self.raw)
    }
}