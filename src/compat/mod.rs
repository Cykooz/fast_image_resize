//! This module is used as compatability layer to support `std` and `no_std`.

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        mod std_impl;
        pub use std_impl::*;
    } else {
        mod no_std_impl;
        pub use no_std_impl::*;
    }
}
