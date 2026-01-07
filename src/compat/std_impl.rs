pub use std::borrow::ToOwned;
pub use std::boxed::Box;
pub use std::vec;
pub use std::vec::Vec;

#[cfg(target_arch = "x86_64")]
pub fn has_avx2() -> bool {
    std::is_x86_feature_detected!("avx2")
}

#[cfg(target_arch = "x86_64")]
pub fn has_sse41() -> bool {
    std::is_x86_feature_detected!("sse4.1")
}

#[cfg(target_arch = "aarch64")]
pub fn has_neon() -> bool {
    std::arch::is_aarch64_feature_detected!("neon")
}

#[cfg(target_arch = "wasm32")]
pub fn has_simd128() -> bool {
    true
}
