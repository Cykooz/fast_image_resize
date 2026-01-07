use crate::compat::*;

/// SIMD extension of CPU.
/// Specific variants depend on target architecture.
/// Look at source code to see all available variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuExtensions {
    None,
    #[cfg(target_arch = "x86_64")]
    /// SIMD extension of x86_64 architecture
    Sse4_1,
    #[cfg(target_arch = "x86_64")]
    /// SIMD extension of x86_64 architecture
    Avx2,
    #[cfg(target_arch = "aarch64")]
    /// SIMD extension of Arm64 architecture
    Neon,
    #[cfg(target_arch = "wasm32")]
    /// SIMD extension of Wasm32 architecture
    Simd128,
}

impl CpuExtensions {
    /// Returns `true` if your CPU support the extension.
    pub fn is_supported(&self) -> bool {
        match self {
            #[cfg(target_arch = "x86_64")]
            Self::Avx2 => has_avx2(),
            #[cfg(target_arch = "x86_64")]
            Self::Sse4_1 => has_sse41(),
            #[cfg(target_arch = "aarch64")]
            Self::Neon => has_neon(),
            #[cfg(target_arch = "wasm32")]
            Self::Simd128 => has_simd128(),
            Self::None => true,
        }
    }
}

impl Default for CpuExtensions {
    #[cfg(target_arch = "x86_64")]
    fn default() -> Self {
        if has_avx2() {
            Self::Avx2
        } else if has_sse41() {
            Self::Sse4_1
        } else {
            Self::None
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn default() -> Self {
        if has_neon() {
            Self::Neon
        } else {
            Self::None
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        if has_simd128() {
            Self::Simd128
        } else {
            Self::None
        }
    }

    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "wasm32"
    )))]
    fn default() -> Self {
        Self::None
    }
}
