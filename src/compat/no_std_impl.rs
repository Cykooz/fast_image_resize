pub use alloc::borrow::ToOwned;
pub use alloc::boxed::Box;
pub use alloc::vec;
pub use alloc::vec::Vec;

// `no_std` feature must be enabled
pub use num_traits::Float;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        // `no_std` feature must be enabled
        cpufeatures::new!(cpuid_sse41, "sse4.1");
        cpufeatures::new!(cpuid_avx2, "avx2");

        pub fn has_sse41() -> bool {
            cpuid_sse41::get()
        }

        pub fn has_avx2() -> bool {
            cpuid_avx2::get()
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub fn has_neon() -> bool {
    true
}

#[cfg(target_arch = "wasm32")]
pub fn has_simd128() -> bool {
    true
}
