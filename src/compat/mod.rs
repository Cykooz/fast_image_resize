//! This module is used as compatability layer to support `std` and `no_std`.

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::boxed::Box;
        pub use std::{vec, vec::Vec};
        pub use std::borrow::ToOwned;
    } else {
        pub use alloc::boxed::Box;
        pub use alloc::{vec, vec::Vec};
        pub use alloc::borrow::ToOwned;

        // `no_std` feature must be enabled
        pub use num_traits::Float;
    }
}
