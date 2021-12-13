use std::arch::x86_64::*;
use std::intrinsics::transmute;

use crate::pixels::{U8x3, U8x4, U8};

#[inline(always)]
pub unsafe fn loadu_si128<T>(buf: &[T], index: usize) -> __m128i {
    _mm_loadu_si128(buf.get_unchecked(index..).as_ptr() as *const __m128i)
}

#[inline(always)]
pub unsafe fn loadu_si256<T>(buf: &[T], index: usize) -> __m256i {
    _mm256_loadu_si256(buf.get_unchecked(index..).as_ptr() as *const __m256i)
}

#[inline(always)]
pub unsafe fn loadu_si256_raw<T>(buf: &[T], offset: usize) -> __m256i {
    let ptr = buf.as_ptr() as *const u8;
    _mm256_loadu_si256(ptr.add(offset) as *const __m256i)
}

#[inline(always)]
pub unsafe fn loadl_epi64<T>(buf: &[T], index: usize) -> __m128i {
    _mm_loadl_epi64(buf.get_unchecked(index..).as_ptr() as *const __m128i)
}

#[inline(always)]
pub unsafe fn loadl_epi64_raw<T>(buf: &[T], offset: usize) -> __m128i {
    let ptr = buf.as_ptr() as *const u8;
    _mm_loadl_epi64(ptr.add(offset) as *const __m128i)
}

#[inline(always)]
pub unsafe fn mm_cvtepu8_epi32(buf: &[U8x4], index: usize) -> __m128i {
    let v: i32 = transmute(buf.get_unchecked(index).0);
    _mm_cvtepu8_epi32(_mm_cvtsi32_si128(v))
}

#[inline(always)]
pub unsafe fn mm_cvtepu8_epi32_u8x3(buf: &[U8x3], index: usize) -> __m128i {
    let pixel = buf.get_unchecked(index).0;
    let v: i32 = i32::from_le_bytes([pixel[0], pixel[1], pixel[2], 0]);
    _mm_cvtepu8_epi32(_mm_cvtsi32_si128(v))
}

#[inline(always)]
pub unsafe fn mm_cvtepu8_epi32_from_u8(buf: &[U8], index: usize) -> __m128i {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const i32;
    _mm_cvtepu8_epi32(_mm_cvtsi32_si128(*ptr))
}

#[inline(always)]
pub unsafe fn mm_cvtepu8_epi32_from_raw<T>(buf: &[T], offset: usize) -> __m128i {
    let ptr = (buf.as_ptr() as *const u8).add(offset) as *const i32;
    _mm_cvtepu8_epi32(_mm_cvtsi32_si128(*ptr))
}

#[inline(always)]
pub unsafe fn mm_cvtsi32_si128_from_u32(buf: &[U8x4], index: usize) -> __m128i {
    let v: i32 = transmute(*buf.get_unchecked(index));
    _mm_cvtsi32_si128(v)
}

#[inline(always)]
pub unsafe fn mm_cvtsi32_si128_from_u8(buf: &[U8], index: usize) -> __m128i {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const i32;
    _mm_cvtsi32_si128(*ptr)
}

#[inline(always)]
pub unsafe fn mm_cvtsi32_si128_from_raw<T>(buf: &[T], offset: usize) -> __m128i {
    let ptr = (buf.as_ptr() as *const u8).add(offset) as *const i32;
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
