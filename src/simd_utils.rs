use std::arch::x86_64::*;
use std::intrinsics::transmute;

#[inline(always)]
pub unsafe fn loadu_si128<T>(buf: &[T], index: usize) -> __m128i {
    _mm_loadu_si128(buf.get_unchecked(index..).as_ptr() as *const __m128i)
}

#[inline(always)]
pub unsafe fn loadu_si256<T>(buf: &[T], index: usize) -> __m256i {
    _mm256_loadu_si256(buf.get_unchecked(index..).as_ptr() as *const __m256i)
}

#[inline(always)]
pub unsafe fn loadl_epi64<T>(buf: &[T], index: usize) -> __m128i {
    _mm_loadl_epi64(buf.get_unchecked(index..).as_ptr() as *const __m128i)
}

#[inline(always)]
pub unsafe fn mm_cvtepu8_epi32(buf: &[u32], index: usize) -> __m128i {
    let v: i32 = transmute(*buf.get_unchecked(index));
    _mm_cvtepu8_epi32(_mm_cvtsi32_si128(v))
}

#[inline(always)]
pub unsafe fn mm_cvtepu8_epi32_from_u8(buf: &[u8], index: usize) -> __m128i {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const i32;
    _mm_cvtepu8_epi32(_mm_cvtsi32_si128(*ptr))
}

#[inline(always)]
pub unsafe fn mm_cvtsi32_si128_from_u32(buf: &[u32], index: usize) -> __m128i {
    let v: i32 = transmute(*buf.get_unchecked(index));
    _mm_cvtsi32_si128(v)
}

#[inline(always)]
pub unsafe fn mm_cvtsi32_si128_from_u8(buf: &[u8], index: usize) -> __m128i {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const i32;
    _mm_cvtsi32_si128(*ptr)
}

#[inline(always)]
pub unsafe fn ptr_i16_to_set1_epi32(buf: &[i16], index: usize) -> __m128i {
    _mm_set1_epi32(*(buf.get_unchecked(index..).as_ptr() as *const i32))
}

#[inline(always)]
pub unsafe fn ptr_i16_to_256set1_epi32(buf: &[i16], index: usize) -> __m256i {
    _mm256_set1_epi32(*(buf.get_unchecked(index..).as_ptr() as *const i32))
}
