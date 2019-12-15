// Copyright 2019 Robin Freyler
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

//! Re-exports an interface that is usable from `std` and `no_std` environments.

use cfg_if::cfg_if;

/// Used to list all re-exported `alloc` crate items from another namespace.
macro_rules! reexport_alloc_from {
    ( $from:ident ) => {
        pub use ::$from::{
            alloc,
            borrow,
            boxed,
            collections,
            fmt,
            format,
            rc,
            slice,
            str,
            string,
            sync,
            vec,
        };
    };
}

cfg_if! {
    if #[cfg(feature = "std")] {
        // Re-export only `alloc` components from `std`.
        reexport_alloc_from!(std);
    } else {
        // Re-export `alloc` environment.
        reexport_alloc_from!(alloc);
    }
}

/// The prelude shared between `std` and `alloc`.
pub mod prelude {
    pub use super::{
        borrow::ToOwned,
        boxed::Box,
        string::{String, ToString},
        vec,
        vec::Vec,
    };
}
