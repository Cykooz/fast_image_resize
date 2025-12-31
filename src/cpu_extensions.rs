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
            Self::Avx2 => Self::has_avx2(),
            #[cfg(target_arch = "x86_64")]
            Self::Sse4_1 => Self::has_sse41(),
            #[cfg(target_arch = "aarch64")]
            // FIXME: Not sure how, I don't know ARM assembly. AFAIK, all ARM compliant
            // processors must implement NEON
            Self::Neon => true,
            #[cfg(target_arch = "wasm32")]
            Self::Simd128 => true,
            Self::None => true,
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[inline]
    fn has_avx2() -> bool {
        use core::arch::x86_64::{__cpuid, __cpuid_count, _xgetbv};

        unsafe {
            let cpuid1 = __cpuid(1);

            if (cpuid1.ecx & (1 << 27)) == 0 || (cpuid1.ecx & (1 << 28)) == 0 {
                // OSXSAVE + AVX
                return false;
            }

            let xcr0 = _xgetbv(0);
            if (xcr0 & 0b110) != 0b110 {
                // XCR0: XMM (bit 1) and YMM (bit 2) enabled
                return false;
            }

            // CPUID leaf 7 subleaf 0: AVX2
            let cpuid7 = __cpuid_count(7, 0);
            (cpuid7.ebx & (1 << 5)) != 0
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[inline]
    fn has_sse41() -> bool {
        use core::arch::x86_64::__cpuid;

        // CPUID leaf 1: feature bits
        let res = unsafe { __cpuid(1) };

        // ECX bit 19 = SSE4.1
        (res.ecx & (1 << 19)) != 0
    }
}

impl Default for CpuExtensions {
    #[cfg(target_arch = "x86_64")]
    fn default() -> Self {
        if Self::has_avx2() {
            Self::Avx2
        } else if Self::has_sse41() {
            Self::Sse4_1
        } else {
            Self::None
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn default() -> Self {
        // FIXME: Maybe we detect?
        Self::Neon
    }
    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Self::Simd128
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
